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
    // Per-cascade receiver-bias multipliers, one lane each (x=c0 … w=c3).
    //
    // `extras.y` is a single NDC bias, so its WORLD magnitude scales with each
    // cascade's ortho depth range — 30x across the set (359 m at c0, 10 km at
    // c3), which put 5.4 m of the survey cascade's slack in the receiver term
    // alone and was half the reason dune relief could not cast out there. Each
    // lane caps that cascade at c1's world-space slack: min(1, c1_depth_range /
    // this_depth_range). It is a CAP, never a raise, so c0 and c1 always get
    // exactly 1.0 and the pinned close/mid station frames are preserved.
    //
    // L.3.C hard-coded these as constants (0.374, 0.171) derived from a MODEL
    // of the fits — and got c2's wrong by 8.5%, because the model priced a
    // cached cascade with the every-frame ortho pad. The renderer now derives
    // them from the fits it just computed, so they cannot drift when a split,
    // a pad, or the far-cascade refresh policy changes.
    bias_scales: vec4<f32>,
    // L.3.D rung 2: c3's OUTGOING view-projection. Appended last so every
    // earlier offset is untouched. Read only while `extras.w < 1.0`, i.e. while
    // a c3 window replacement is being blended in; under every other policy the
    // weight is pinned at 1.0 and this matrix is never sampled against.
    view_proj_prev: mat4x4<f32>,
};

// THE cascade-select boundary decision — 4-way hard switch on view distance at
// splits.x / splits.y / extras.z (the engine's idiom). Returns the layer index
// 0..3; it does NOT test `shadow_far`, because coverage-end is a sampling
// concern (handled in csm_shadow_factor's UV/range guard), not a selection one.
//
// L.3.D factored this out so the debug cascade-index view (debug_mode 4) reads
// the SAME boundaries the sampler uses. Invariant 20(a) is "one CSM sampling
// implementation"; a debug overlay that re-derived the split comparisons would
// be a second one, and would drift silently the first time a split moved —
// which would make the overlay lie in exactly the situation it exists to
// diagnose.
fn csm_cascade_index(light: MainLightUbo, frag_dist: f32) -> i32 {
    if (frag_dist < light.splits.x) {
        return 0;
    }
    if (frag_dist < light.splits.y) {
        return 1;
    }
    if (frag_dist < light.extras.z) {
        return 2;
    }
    return 3;
}

// L.3.D cascade-index overlay colour (editor shading mode 4), SHARED by both
// consumer shaders so the static and terrain views cannot disagree about which
// band is which.
//
// `refresh_bits`: bit 0 = c2 re-fitted this frame, bit 1 = c3 re-fitted. The
// cascade that re-fitted is brightened for that one frame — which is what turns
// a TEMPORAL event into something an eye can catch in motion. The whole L.3.C→D
// defect is a multi-frame freeze followed by an atomic replacement; without the
// flash you have to frame-step to see the cause, and the director's gate is a
// flight, not a frame-stepper.
fn csm_cascade_debug_color(light: MainLightUbo, frag_dist: f32, refresh_bits: u32) -> vec3<f32> {
    // Beyond coverage: no cascade owns this fragment. Distinct from c3 on
    // purpose — "the shadow ends here" and "c3 is stale here" look identical
    // in a lit frame and are completely different defects.
    if (frag_dist >= light.splits.z) {
        return vec3<f32>(0.16, 0.16, 0.18);
    }
    let layer = csm_cascade_index(light, frag_dist);
    var c: vec3<f32>;
    var flashing: bool = false;
    if (layer == 0) {
        c = vec3<f32>(0.85, 0.22, 0.22);
    } else if (layer == 1) {
        c = vec3<f32>(0.22, 0.75, 0.28);
    } else if (layer == 2) {
        c = vec3<f32>(0.22, 0.42, 0.95);
        flashing = (refresh_bits & 1u) != 0u;
    } else {
        c = vec3<f32>(0.95, 0.80, 0.18);
        flashing = (refresh_bits & 2u) != 0u;
    }
    // The refresh pulse DARKENS the band rather than mixing it toward white.
    // Both read equally well in motion, but white does not survive analysis:
    // mixing 65% to white desaturates the swatch past any hue classifier AND
    // lands it on top of the sky colour, so a flashed c3 band was
    // indistinguishable from "no c3 in frame" — which is exactly how the first
    // version of the validation harness saw it (10 refresh frames reported
    // zero c3 pixels). Scaling preserves hue, so the band stays classifiable
    // and the luma delta is unambiguous in both directions.
    if (flashing) {
        c = c * 0.35;
    }
    return c;
}

// The spare array layer holding c3's OUTGOING window during a transition
// (L.3.D rung 2). Must equal the Rust-side `C3_PREV_LAYER` = `CASCADE_COUNT`;
// the pairing is contract-tested in renderer.rs so the two cannot drift.
const C3_PREV_LAYER: i32 = 4;

// Sample ONE cascade map: PCF 3x3 plus the ortho-boundary UV edge fade,
// returning 1.0 (fully lit) outside that map's UV range.
//
// Control flow inside is deliberately BRANCH-FREE: the taps are taken
// unconditionally and the out-of-range case is folded in with `select`
// afterwards (the comparison sampler clamps, so an out-of-range tap is
// harmless). Wrapping the taps in an early return would make them conditional,
// and this function has to be callable TWICE per fragment for the c3
// transition blend.
fn csm_sample_map(
    light: MainLightUbo,
    shadow_map: texture_depth_2d_array,
    shadow_comparison: sampler_comparison,
    lvp: mat4x4<f32>,
    layer: i32,
    world_pos: vec3<f32>,
    bias: f32,
) -> f32 {
    let lp = lvp * vec4<f32>(world_pos, 1.0);
    let ndc = lp.xyz / lp.w;
    let uv = ndc.xy * 0.5 + vec2<f32>(0.5, 0.5);
    // PCF 3x3 (scaled by pcf radius in texels from extras.x)
    let dims = vec2<f32>(textureDimensions(shadow_map).xy);
    let texel = 1.0 / dims;
    let r = max(0.0, light.extras.x);
    var sum = 0.0;
    for (var dx: i32 = -1; dx <= 1; dx = dx + 1) {
        for (var dy: i32 = -1; dy <= 1; dy = dy + 1) {
            let o = vec2<f32>(f32(dx), f32(dy)) * texel * r;
            sum = sum + textureSampleCompare(shadow_map, shadow_comparison, uv + o, layer, ndc.z - bias);
        }
    }
    var s = sum / 9.0;
    // Soften the ortho projection boundary.
    let edge_fade_x = min(uv.x, 1.0 - uv.x) * 10.0;
    let edge_fade_y = min(uv.y, 1.0 - uv.y) * 10.0;
    s = mix(1.0, s, clamp(min(edge_fade_x, edge_fade_y), 0.0, 1.0));
    let inside = uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0;
    return select(1.0, s, inside);
}

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
    // c0: 0..split0, c1: split0..split1, c2: split1..split2, c3: split2..far.
    let layer = csm_cascade_index(light, frag_dist);
    var lvp: mat4x4<f32>;
    // L.3.C: per-cascade receiver bias. `extras.y` is an NDC bias, so one
    // value meant wildly different WORLD slack per cascade — their ortho depth
    // ranges span 30x (355 m at c0, 10.3 km at c3), which put 5.4 m of the
    // survey cascade's 13.4 m total slack in the receiver term alone and was
    // half the reason dune relief could not cast out there. The far cascades
    // are CAPPED at c1's world-space equivalent; c0/c1 keep exactly the bias
    // they shipped with (never raised), which is what preserves the pinned
    // station frames.
    var bias_scale: f32;
    if (layer == 0) {
        lvp = light.view_proj0;
        bias_scale = light.bias_scales.x;
    } else if (layer == 1) {
        lvp = light.view_proj1;
        bias_scale = light.bias_scales.y;
    } else if (layer == 2) {
        lvp = light.view_proj2;
        bias_scale = light.bias_scales.z;
    } else {
        lvp = light.view_proj3;
        bias_scale = light.bias_scales.w;
    }
    let bias = max(light.extras.y * bias_scale, 0.00001);

    if (frag_dist < shadow_far) {
        shadow = csm_sample_map(light, shadow_map, shadow_comparison, lvp, layer, world_pos, bias);

        // L.3.D rung 2: c3's window replacement is INCREMENTAL. `extras.w` is
        // the blend weight — 1.0 means no transition in flight, and this whole
        // block costs one comparison, so every other policy pays nothing. While
        // a transition runs, the outgoing window (retained in the spare layer,
        // addressed by `view_proj_prev`) is blended out over C3_FADE_FRAMES, so
        // the per-frame change is a fraction of the replacement instead of all
        // of it. Both maps are valid at their own time; this is a temporal
        // blend, not a coverage compromise.
        let c3_fade = light.extras.w;
        if (layer == 3 && c3_fade < 1.0) {
            let prev = csm_sample_map(
                light, shadow_map, shadow_comparison,
                light.view_proj_prev, C3_PREV_LAYER, world_pos, bias
            );
            shadow = mix(prev, shadow, clamp(c3_fade, 0.0, 1.0));
        }

        // Fade shadow to 1.0 toward the coverage end. L.3.A: the band is
        // splits.w → splits.z (the last 30% of shadow_far — ~900 m), wide
        // enough that no edge is perceptible from a moving camera; the L.3
        // 100 m band at 400→500 m was the render-gate rejection's arc.
        let fade_start = light.splits.w;
        if (frag_dist > fade_start) {
            let fade = (frag_dist - fade_start) / max(shadow_far - fade_start, 1.0);
            shadow = mix(shadow, 1.0, clamp(fade, 0.0, 1.0));
        }
    }
    return shadow;
}
