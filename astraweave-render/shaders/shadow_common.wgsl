// CSM shadow sampling — shared concatenation fragment (L.3).
//
// Hoisted verbatim-in-behavior from the static-mesh SHADER_SRC's inline PCF
// block so the terrain-forward path (pbr_terrain_forward.wgsl) consumes the
// SAME cascade-select + filter implementation instead of growing a parallel
// one (the duplicate-pipeline trap the Fix-27 campaign unwound).
//
// Contract (same as ibl_common.wgsl): this file is NOT a standalone module —
// it is prepended into consumer shaders by string concatenation. It defines
// only the `MainLightUbo` struct and the `csm_shadow_factor` function; each
// consumer declares its own uniform/texture/sampler globals at whatever bind
// group it owns (static mesh: group(2); terrain forward: group(4)) and passes
// them in. Consumers must NOT redeclare `MainLightUbo`.
//
// The shadows-off sentinel gate (`extras.x >= 0.0`) deliberately stays at the
// CALL SITE, outside this function: extras.x is uniform across the draw, so
// the branch lets the GPU skip all 9 comparison samples for every warp when
// shadows are disabled (and the textual gate is contract-tested in
// renderer.rs::test_shader_has_conditional_shadow_not_hardcoded).

struct MainLightUbo {
    view_proj0: mat4x4<f32>,
    view_proj1: mat4x4<f32>,
    // L.3.A: the survey span. L.3.C split it in two (500→1400, 1400→3000)
    // because a single 500→3000 cascade rendered 3.74 m texels, and the
    // resulting 6.4x step at the 500 m seam made >50% of this dune world's
    // relief unable to cast beyond it (docs/audits/L3C_OUTCOME.md).
    view_proj2: mat4x4<f32>,
    view_proj3: mat4x4<f32>,
    // x: split0 (c0|c1), y: split1 (c1|c2), z: shadow_far (coverage end),
    // w: fade_start (the wide far fade begins here — L.3.A widened it from
    // the last 20% of 500 m to the last 30% of shadow_far, because a narrow
    // fade at plainly-visible range reads as a moving edge).
    splits: vec4<f32>,
    // x: pcf_radius_px or -1.0 shadows-off sentinel, y: depth_bias,
    // z: split2 (c2|c3 — L.3.C put it in a lane that was already padding so
    // splits' lanes and extras.x/.y keep their offsets), w: reserved.
    extras: vec4<f32>,
};

// L.3.C receiver-bias caps for the cached far cascades, as multipliers on the
// shipped NDC bias. Each is (c1_world_bias / this_cascade_depth_range) /
// shipped_ndc_bias, i.e. "same world-space slack c1 has, expressed in this
// cascade's NDC" — derived from the measured fits in L3C_OUTCOME: c1 depth
// 1762 m, c2 4714 m, c3 10309 m. Constants rather than uniforms because the
// cascade slice geometry is fixed; if the split distances change, recompute.
const C2_BIAS_SCALE: f32 = 0.374;   // 1762 / 4714
const C3_BIAS_SCALE: f32 = 0.171;   // 1762 / 10309

// Cascade-selected 3x3 PCF shadow factor in [0, 1] (1.0 = fully lit).
//
// Cascade select is a hard 3-way switch on view distance at splits.x /
// splits.y (the engine's idiom); the visible seam is softened by two fades:
// a WIDE distance fade over splits.w → splits.z (L.3.A — the last 30% of
// coverage), and a UV edge fade at each cascade's ortho boundary. Depth
// bias: constant receiver bias from extras.y here, plus the caster
// pipelines' hardware slope-scaled bias (DepthBiasState) — the
// acne/peter-pan tuning pair.
fn csm_shadow_factor(
    light: MainLightUbo,
    shadow_map: texture_depth_2d_array,
    shadow_comparison: sampler_comparison,
    world_pos: vec3<f32>,
    frag_dist: f32,
) -> f32 {
    var shadow: f32 = 1.0;
    let shadow_far = light.splits.z;
    // Cascade select: hard switch on view distance (the engine's idiom),
    // 4-way since L.3.C. c0: 0..split0, c1: split0..split1,
    // c2: split1..split2, c3: split2..shadow_far.
    var lvp: mat4x4<f32>;
    var layer: i32;
    // L.3.C: per-cascade receiver bias. `extras.y` is an NDC bias, so one
    // value meant wildly different WORLD slack per cascade — their ortho depth
    // ranges span 30x (355 m at c0, 10.3 km at c3), which put 5.4 m of the
    // survey cascade's 13.4 m total slack in the receiver term alone and was
    // half the reason dune relief could not cast out there. The far cascades
    // are CAPPED at c1's world-space equivalent; c0/c1 keep exactly the bias
    // they shipped with (never raised), which is what preserves the pinned
    // station frames.
    var bias_scale: f32 = 1.0;
    if (frag_dist < light.splits.x) {
        lvp = light.view_proj0;
        layer = 0;
    } else if (frag_dist < light.splits.y) {
        lvp = light.view_proj1;
        layer = 1;
    } else if (frag_dist < light.extras.z) {
        lvp = light.view_proj2;
        layer = 2;
        bias_scale = C2_BIAS_SCALE;
    } else {
        lvp = light.view_proj3;
        layer = 3;
        bias_scale = C3_BIAS_SCALE;
    }
    let lp = lvp * vec4<f32>(world_pos, 1.0);
    let ndc_shadow = lp.xyz / lp.w;
    let uv = ndc_shadow.xy * 0.5 + vec2<f32>(0.5, 0.5);
    let depth = ndc_shadow.z;
    let base_bias = light.extras.y * bias_scale;
    let bias = max(base_bias, 0.00001);

    if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0 && frag_dist < shadow_far) {
        // PCF 3x3 (scaled by pcf radius in texels from extras.x)
        let dims = vec2<f32>(textureDimensions(shadow_map).xy);
        let texel = 1.0 / dims;
        let r = max(0.0, light.extras.x);
        var sum = 0.0;
        for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
            for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
                let o = vec2<f32>(f32(dx), f32(dy)) * texel * r;
                sum = sum + textureSampleCompare(shadow_map, shadow_comparison, uv + o, layer, depth - bias);
            }
        }
        shadow = sum / 9.0;

        // Fade shadow to 1.0 toward the coverage end. L.3.A: the band is
        // splits.w → splits.z (the last 30% of shadow_far — ~900 m), wide
        // enough that no edge is perceptible from a moving camera; the L.3
        // 100 m band at 400→500 m was the render-gate rejection's arc.
        let fade_start = light.splits.w;
        if (frag_dist > fade_start) {
            let fade = (frag_dist - fade_start) / max(shadow_far - fade_start, 1.0);
            shadow = mix(shadow, 1.0, clamp(fade, 0.0, 1.0));
        }

        // Also fade at UV edges to soften the ortho projection boundary
        let edge_fade_x = min(uv.x, 1.0 - uv.x) * 10.0;
        let edge_fade_y = min(uv.y, 1.0 - uv.y) * 10.0;
        let edge_fade = clamp(min(edge_fade_x, edge_fade_y), 0.0, 1.0);
        shadow = mix(1.0, shadow, edge_fade);
    }
    return shadow;
}
