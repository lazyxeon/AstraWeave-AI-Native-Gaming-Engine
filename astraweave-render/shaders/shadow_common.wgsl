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
    // L.3.A: third cascade — the survey span (500 → shadow_far). Added after
    // the L.3 render gate caught the 500 m coverage cap as a camera-anchored
    // shadow boundary from a moving survey camera.
    view_proj2: mat4x4<f32>,
    // x: split0 (c0|c1), y: split1 (c1|c2), z: shadow_far (coverage end),
    // w: fade_start (the wide far fade begins here — L.3.A widened it from
    // the last 20% of 500 m to the last 30% of shadow_far, because a narrow
    // fade at plainly-visible range reads as a moving edge).
    splits: vec4<f32>,
    extras: vec2<f32>, // x: pcf_radius_px or -1.0 shadows-off sentinel, y: depth_bias
};

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
    // 3-way since L.3.A. c0: 0..split0, c1: split0..split1, c2: split1..far.
    var lvp: mat4x4<f32>;
    var layer: i32;
    if (frag_dist < light.splits.x) {
        lvp = light.view_proj0;
        layer = 0;
    } else if (frag_dist < light.splits.y) {
        lvp = light.view_proj1;
        layer = 1;
    } else {
        lvp = light.view_proj2;
        layer = 2;
    }
    let lp = lvp * vec4<f32>(world_pos, 1.0);
    let ndc_shadow = lp.xyz / lp.w;
    let uv = ndc_shadow.xy * 0.5 + vec2<f32>(0.5, 0.5);
    let depth = ndc_shadow.z;
    let base_bias = light.extras.y;
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
