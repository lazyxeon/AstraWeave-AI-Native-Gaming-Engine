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
    splits: vec2<f32>,
    extras: vec2<f32>, // x: pcf_radius_px, y: depth_bias; z: slope_scale in skinned path extras.x reused; keep 2 vec2s for alignment
};

// Cascade-selected 3x3 PCF shadow factor in [0, 1] (1.0 = fully lit).
//
// Cascade select is a hard switch on view distance at splits.x (the engine's
// idiom); the visible seam is softened by two fades ported unchanged from the
// static path: a distance fade over the outer 20% of shadow range, and a UV
// edge fade at each cascade's ortho boundary. Depth bias: constant receiver
// bias from extras.y here, plus the caster pipelines' hardware slope-scaled
// bias (DepthBiasState) — the acne/peter-pan tuning pair.
fn csm_shadow_factor(
    light: MainLightUbo,
    shadow_map: texture_depth_2d_array,
    shadow_comparison: sampler_comparison,
    world_pos: vec3<f32>,
    frag_dist: f32,
) -> f32 {
    var shadow: f32 = 1.0;
    let shadow_far = light.splits.y;
    let use_c0 = frag_dist < light.splits.x;
    var lvp: mat4x4<f32>;
    if (use_c0) { lvp = light.view_proj0; } else { lvp = light.view_proj1; }
    let lp = lvp * vec4<f32>(world_pos, 1.0);
    let ndc_shadow = lp.xyz / lp.w;
    let uv = ndc_shadow.xy * 0.5 + vec2<f32>(0.5, 0.5);
    let depth = ndc_shadow.z;
    let base_bias = light.extras.y;
    let bias = max(base_bias, 0.00001);

    if (uv.x >= 0.0 && uv.x <= 1.0 && uv.y >= 0.0 && uv.y <= 1.0 && frag_dist < shadow_far) {
        var layer: i32;
        if (use_c0) { layer = 0; } else { layer = 1; }
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

        // Fade shadow to 1.0 at cascade boundary edges to eliminate
        // the hard square cutoff. Fade in the outer 20% of shadow range.
        let fade_start = shadow_far * 0.8;
        if (frag_dist > fade_start) {
            let fade = (frag_dist - fade_start) / (shadow_far - fade_start);
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
