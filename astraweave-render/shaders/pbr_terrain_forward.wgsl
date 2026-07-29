// pbr_terrain_forward.wgsl — Forward-lit splat terrain shader (Phase 1).
//
// Part of the Terrain Material System Campaign (Option D). Renders terrain
// chunks with per-fragment 8-biome splat-map blending AND lighting in a
// single pass that writes directly to the engine's forward HDR target
// (Rgba16Float). Unlike the companion `pbr_terrain.wgsl` (deferred-style,
// 3 color targets, unlit), this shader produces a single lit HDR color
// compatible with `astraweave_render::Renderer`'s forward pass.
//
// Reference: `astraweave-render/src/renderer.rs:18-328` (SHADER_SRC) — the
// engine's main PBR shader whose lighting model this file mirrors for the
// terrain pass. `TerrainMaterialManager::set_material` uploads 8 layer
// texture sets into the arrays bound at group(1) bindings 3-5. Per-chunk
// splat textures are bound at group(2).
//
// Lighting scope (L.3, 2026-07-28 — supersedes the L.2 "no shadows" line):
//   - Sun direct lighting: Cook-Torrance + Burley via evaluate_brdf,
//     multiplied by the CSM shadow factor (shared csm_shadow_factor from
//     shadow_common.wgsl at group(4); extras.x < 0 sentinel disables)
//   - Environment: full split-sum IBL via the shared compute_ibl
//     (ibl_common.wgsl) — diffuse irradiance + prefiltered specular + BRDF
//     LUT, bound at group(3), multiplied by material AO. This REPLACES the
//     Phase-1 flat 0.35×ambient fill (no double-count). The shadow term does
//     NOT touch these indirect terms — shadowed ground stays skylit.
//   - Distance fog: SHADER_SRC's apply_scene_fog linear+exp blend
//   - Still absent: SSGI, cloud shadows, screen tint.
//
// The PI, INV_PI constants, all BRDF helpers, compute_ibl, and
// csm_shadow_factor are prepended at pipeline build time via
// `concat!(include_str!("constants.wgsl"), include_str!("brdf_common.wgsl"),
// include_str!("ibl_common.wgsl"), include_str!("shadow_common.wgsl"),
// include_str!("stochastic_tiling.wgsl"),
// include_str!("pbr_terrain_forward.wgsl"))`.

// ============================================================================
// Uniforms
// ============================================================================

// Camera UBO — mirrors `Camera` in SHADER_SRC (renderer.rs:48-54) byte-for-byte.
struct CameraTerrain {
    view_proj: mat4x4<f32>,
    light_dir: vec3<f32>,
    _pad0: f32,
    camera_pos: vec3<f32>,
    _pad1: f32,
}

// Per-layer material parameters. Matches `TerrainLayerGpu` in
// `astraweave-render/src/terrain_material.rs:16-36` (64 bytes).
struct TerrainLayer {
    texture_indices: vec4<u32>,     // [albedo, normal, orm, height]
    uv_scale: vec2<f32>,
    height_range: vec2<f32>,
    blend_sharpness: f32,
    triplanar_power: f32,
    material_factors: vec2<f32>,    // [metallic, roughness]
    _pad: vec4<u32>,
}

// Terrain material UBO — mirrors `TerrainMaterialGpu` in
// `astraweave-render/src/terrain_material.rs` (Real-Fix.D 2026-05-08:
// 2112 bytes, was 576). Reused from the deferred pipeline; most fields
// (triplanar, normal_blend_method, height_blend_enabled) are ignored by
// Phase 1's simplified blending path.
struct TerrainParams {
    // Real-Fix.D 2026-05-08: 32 layers (was 8). Must match
    // `astraweave_render::MAX_TERRAIN_LAYERS`.
    layers: array<TerrainLayer, 32>,
    splat_map_index_0: u32,   // dead field; preserved for byte layout
    splat_map_index_1: u32,   // dead field; preserved for byte layout
    splat_uv_scale: f32,
    triplanar_enabled: u32,
    normal_blend_method: u32,
    triplanar_slope_threshold: f32,
    height_blend_enabled: u32,
    active_layer_count: u32,
    // `array<u32, 8>` stride is 4 B; WGSL uniform address-space requires >=16 B
    // stride. Matches Rust-side `_pad: [u32; 8]` via two vec4<u32> entries.
    _pad: array<vec4<u32>, 2>,
}

// Scene env UBO — mirrors `SceneEnv` in SHADER_SRC (renderer.rs:86-100)
// **byte-for-byte** per Phase 1.E handoff §5 "Option 1". `tint_color`,
// `tint_alpha`, `blend_factor`, and the `_pad1` triplet are declared here
// even though Phase 1's fragment shader doesn't read them — preserving the
// layout keeps this UBO in sync with SHADER_SRC so future shader revisions
// adding screen tint won't need a silent byte-offset update.
// 96 bytes total, align 16.
struct TerrainSceneEnv {
    fog_color: vec3<f32>,
    fog_density: f32,
    fog_start: f32,
    fog_end: f32,
    _pad0: vec2<f32>,
    ambient_color: vec3<f32>,
    ambient_intensity: f32,
    tint_color: vec3<f32>,
    tint_alpha: f32,
    blend_factor: f32,
    // ED-3: 0=lit, 1=unlit albedo, 2=world-space normals, 3=UVs (former pad).
    debug_mode: f32,
    // L.1: tonemap exposure (post-pass concern; never read here — declared
    // for byte-layout parity with SceneEnvironmentUBO).
    exposure: f32,
    _pad1z: f32,
    sun_color: vec3<f32>,
    sun_intensity: f32,
}

@group(0) @binding(0) var<uniform> uCamera: CameraTerrain;

@group(1) @binding(0) var<uniform> uTerrain: TerrainParams;
@group(1) @binding(1) var<uniform> uScene: TerrainSceneEnv;
@group(1) @binding(2) var terrain_sampler: sampler;
@group(1) @binding(3) var layer_albedo: texture_2d_array<f32>;
@group(1) @binding(4) var layer_normal: texture_2d_array<f32>;
@group(1) @binding(5) var layer_orm:    texture_2d_array<f32>;

// Real-Fix.D 2026-05-08: 8 splat textures (was 2) for 32-channel weights.
// splat_map_i carries layers (i*4)..(i*4+3) in channels R..A.
@group(2) @binding(0) var splat_map_0: texture_2d<f32>;   // layers 0..3
@group(2) @binding(1) var splat_map_1: texture_2d<f32>;   // layers 4..7
@group(2) @binding(2) var splat_map_2: texture_2d<f32>;   // layers 8..11
@group(2) @binding(3) var splat_map_3: texture_2d<f32>;   // layers 12..15
@group(2) @binding(4) var splat_map_4: texture_2d<f32>;   // layers 16..19
@group(2) @binding(5) var splat_map_5: texture_2d<f32>;   // layers 20..23
@group(2) @binding(6) var splat_map_6: texture_2d<f32>;   // layers 24..27
@group(2) @binding(7) var splat_map_7: texture_2d<f32>;   // layers 28..31
// Phase 1 re-cleanup Issue 1 fix: dedicated ClampToEdge sampler for per-
// chunk splat textures. `terrain_sampler` (group 1 binding 2) uses Repeat
// addressing which tiles layer textures correctly but wraps splat UVs
// at `uv == 1.0`, producing a linear-blend of the chunk's rightmost and
// leftmost biome weights at every chunk boundary — the root cause of the
// visible chunk seam grid.
@group(2) @binding(8) var splat_sampler: sampler;

// L.2: IBL resources (same names ibl_common.wgsl's compute_ibl expects; the
// static PBR shader binds the identical set at group(5)). The bind group is
// renderer-owned (`terrain_ibl_bind_group`) and shares the engine's baked
// views, params buffer, and sampler — rebuilt by rebuild_ibl_bind_group.
@group(3) @binding(0) var ibl_specular: texture_cube<f32>;
@group(3) @binding(1) var ibl_irradiance: texture_cube<f32>;
@group(3) @binding(2) var ibl_brdf_lut: texture_2d<f32>;
@group(3) @binding(3) var ibl_sampler: sampler;
@group(3) @binding(4) var<uniform> uIbl: IblParams;

// L.3: CSM shadow resources (same names the static PBR shader binds at its
// group(2); MainLightUbo + csm_shadow_factor come from shadow_common.wgsl,
// prepended). The bind group is the renderer's EXISTING `light_bg` — the same
// light UBO, 2-layer depth array, and comparison sampler the static path
// samples; terrain binds it at group(4). One owner, no parallel resource
// (§7.7). Sampled-texture budget: this is the fragment stage's 15th of 16.
@group(4) @binding(0) var<uniform> uLight: MainLightUbo;
@group(4) @binding(1) var shadow_tex: texture_depth_2d_array;
@group(4) @binding(2) var shadow_sampler: sampler_comparison;

// ============================================================================
// Vertex stage
// ============================================================================

struct VSIn {
    @location(0) position: vec3<f32>,
    @location(1) normal:   vec3<f32>,
    @location(2) uv:       vec2<f32>,
}

struct VSOut {
    @builtin(position) clip_pos: vec4<f32>,
    @location(0) world_pos:  vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VSIn) -> VSOut {
    var out: VSOut;
    out.clip_pos    = uCamera.view_proj * vec4<f32>(in.position, 1.0);
    out.world_pos   = in.position;
    out.world_normal = in.normal;
    out.uv          = in.uv;
    return out;
}

// ============================================================================
// Helpers — fog (copied from SHADER_SRC to stay in sync with apply_scene_fog)
// ============================================================================

fn apply_terrain_fog(color: vec3<f32>, dist: f32) -> vec3<f32> {
    let linear_fog = clamp(
        (dist - uScene.fog_start) / max(uScene.fog_end - uScene.fog_start, 0.001),
        0.0, 1.0,
    );
    let exp_fog = 1.0 - exp(-uScene.fog_density * dist);
    // Cap at 0.92 to avoid the white-void horizon effect (mirrors SHADER_SRC).
    let fog_factor = clamp(max(linear_fog, exp_fog), 0.0, 0.92);
    return mix(color, uScene.fog_color, fog_factor);
}

// ============================================================================
// Fragment stage
// ============================================================================

@fragment
fn fs_main(in: VSOut) -> @location(0) vec4<f32> {
    // 1. Sample all 8 splat textures at the fragment UV (Real-Fix.D
    //    2026-05-08; was 2). Use the dedicated ClampToEdge `splat_sampler`,
    //    not the tiling `terrain_sampler` — see the binding's comment for
    //    the rationale.
    let splat_uv = in.uv * uTerrain.splat_uv_scale;
    let splat0 = textureSample(splat_map_0, splat_sampler, splat_uv);
    let splat1 = textureSample(splat_map_1, splat_sampler, splat_uv);
    let splat2 = textureSample(splat_map_2, splat_sampler, splat_uv);
    let splat3 = textureSample(splat_map_3, splat_sampler, splat_uv);
    let splat4 = textureSample(splat_map_4, splat_sampler, splat_uv);
    let splat5 = textureSample(splat_map_5, splat_sampler, splat_uv);
    let splat6 = textureSample(splat_map_6, splat_sampler, splat_uv);
    let splat7 = textureSample(splat_map_7, splat_sampler, splat_uv);

    var raw_weights: array<f32, 32>;
    raw_weights[0]  = splat0.r;  raw_weights[1]  = splat0.g;
    raw_weights[2]  = splat0.b;  raw_weights[3]  = splat0.a;
    raw_weights[4]  = splat1.r;  raw_weights[5]  = splat1.g;
    raw_weights[6]  = splat1.b;  raw_weights[7]  = splat1.a;
    raw_weights[8]  = splat2.r;  raw_weights[9]  = splat2.g;
    raw_weights[10] = splat2.b;  raw_weights[11] = splat2.a;
    raw_weights[12] = splat3.r;  raw_weights[13] = splat3.g;
    raw_weights[14] = splat3.b;  raw_weights[15] = splat3.a;
    raw_weights[16] = splat4.r;  raw_weights[17] = splat4.g;
    raw_weights[18] = splat4.b;  raw_weights[19] = splat4.a;
    raw_weights[20] = splat5.r;  raw_weights[21] = splat5.g;
    raw_weights[22] = splat5.b;  raw_weights[23] = splat5.a;
    raw_weights[24] = splat6.r;  raw_weights[25] = splat6.g;
    raw_weights[26] = splat6.b;  raw_weights[27] = splat6.a;
    raw_weights[28] = splat7.r;  raw_weights[29] = splat7.g;
    raw_weights[30] = splat7.b;  raw_weights[31] = splat7.a;

    // 2. Normalize weights over the active layer count.
    let count = uTerrain.active_layer_count;
    var total_w: f32 = 0.0;
    for (var i: u32 = 0u; i < count; i = i + 1u) {
        total_w = total_w + raw_weights[i];
    }
    if (total_w > 0.0001) {
        for (var i: u32 = 0u; i < count; i = i + 1u) {
            raw_weights[i] = raw_weights[i] / total_w;
        }
    } else {
        // Fallback: if no biome authored at this fragment, pin to layer 0.
        raw_weights[0] = 1.0;
    }

    // 3. Pre-compute screen-space derivatives in uniform control flow so
    //    textureSampleGrad inside the per-layer loop is well-defined on
    //    every backend (FXC/DXC don't unroll gradient-dependent dynamic
    //    loops cleanly; see the pattern in pbr_terrain.wgsl).
    let ddx_uv = dpdx(in.uv);
    let ddy_uv = dpdy(in.uv);

    // 4. Accumulate PBR inputs from each contributing layer using planar
    //    projection. Phase 1 skips triplanar — terrain is generally close
    //    to horizontal in the editor's test scenes, and triplanar adds
    //    ~30 ALU per fragment for marginal gain at this stage.
    var final_albedo = vec3<f32>(0.0);
    var final_normal_ts = vec3<f32>(0.0, 0.0, 0.0);
    var final_ao: f32 = 0.0;
    var final_roughness: f32 = 0.0;
    var final_metallic: f32 = 0.0;

    for (var i: u32 = 0u; i < count; i = i + 1u) {
        let w = raw_weights[i];
        if (w < 0.001) {
            continue;
        }

        let layer = uTerrain.layers[i];
        let a_idx = layer.texture_indices.x;
        let n_idx = layer.texture_indices.y;
        let o_idx = layer.texture_indices.z;
        let scaled_uv = in.uv * layer.uv_scale;
        let scaled_ddx = ddx_uv * layer.uv_scale;
        let scaled_ddy = ddy_uv * layer.uv_scale;

        // E3-terrain texturing (2026-07-03, rev 2): hex-tile stochastic
        // sampling with per-cell ROTATION + translation (hex_cells from
        // stochastic_tiling.wgsl). Rev 1 used translations only, which cannot
        // break a periodic texture (a shifted tiling is the same tiling) —
        // the repetition grid survived. Rotation re-orients the tile lattice
        // per hex cell; cells span ~1 texture repeat (density 1.0) so no full
        // repeat ever completes within one cell. Weights are sharpened (pow 4)
        // to keep contrast; textureSampleGrad keeps the loop's non-uniform
        // control flow well-defined; gradients are rotated with the UVs so
        // mip/aniso selection stays correct.
        let hex = hex_cells(scaled_uv);
        var hw0 = pow(hex.w0, 2.0);
        var hw1 = pow(hex.w1, 2.0);
        var hw2 = pow(hex.w2, 2.0);
        let hw_sum = hw0 + hw1 + hw2;
        hw0 = hw0 / hw_sum;
        hw1 = hw1 / hw_sum;
        hw2 = hw2 / hw_sum;
        let uv0 = rotate_dir(scaled_uv, hex.rot0) + hex.off0;
        let uv1 = rotate_dir(scaled_uv, hex.rot1) + hex.off1;
        let uv2 = rotate_dir(scaled_uv, hex.rot2) + hex.off2;
        let dx0 = rotate_dir(scaled_ddx, hex.rot0);
        let dy0 = rotate_dir(scaled_ddy, hex.rot0);
        let dx1 = rotate_dir(scaled_ddx, hex.rot1);
        let dy1 = rotate_dir(scaled_ddy, hex.rot1);
        let dx2 = rotate_dir(scaled_ddx, hex.rot2);
        let dy2 = rotate_dir(scaled_ddy, hex.rot2);

        let albedo_s = textureSampleGrad(
                layer_albedo, terrain_sampler, uv0, a_idx, dx0, dy0,
            ) * hw0
            + textureSampleGrad(
                layer_albedo, terrain_sampler, uv1, a_idx, dx1, dy1,
            ) * hw1
            + textureSampleGrad(
                layer_albedo, terrain_sampler, uv2, a_idx, dx2, dy2,
            ) * hw2;
        let orm_s = textureSampleGrad(
                layer_orm, terrain_sampler, uv0, o_idx, dx0, dy0,
            ) * hw0
            + textureSampleGrad(
                layer_orm, terrain_sampler, uv1, o_idx, dx1, dy1,
            ) * hw1
            + textureSampleGrad(
                layer_orm, terrain_sampler, uv2, o_idx, dx2, dy2,
            ) * hw2;

        // Normals: decode each tap and COUNTER-rotate its XY by the cell's
        // angle (conjugate cos/sin) — sampling at a rotated UV pulls the
        // texture content back by the inverse rotation, so the tangent-space
        // vectors must follow, or per-cell lighting shears. Blend decoded.
        let nr0 = textureSampleGrad(
            layer_normal, terrain_sampler, uv0, n_idx, dx0, dy0,
        ).rgb * 2.0 - 1.0;
        let nr1 = textureSampleGrad(
            layer_normal, terrain_sampler, uv1, n_idx, dx1, dy1,
        ).rgb * 2.0 - 1.0;
        let nr2 = textureSampleGrad(
            layer_normal, terrain_sampler, uv2, n_idx, dx2, dy2,
        ).rgb * 2.0 - 1.0;
        let inv0 = vec2<f32>(hex.rot0.x, -hex.rot0.y);
        let inv1 = vec2<f32>(hex.rot1.x, -hex.rot1.y);
        let inv2 = vec2<f32>(hex.rot2.x, -hex.rot2.y);
        let n_ts = vec3<f32>(
            rotate_dir(nr0.xy, inv0) * hw0
                + rotate_dir(nr1.xy, inv1) * hw1
                + rotate_dir(nr2.xy, inv2) * hw2,
            nr0.z * hw0 + nr1.z * hw1 + nr2.z * hw2,
        );

        final_albedo = final_albedo + albedo_s.rgb * w;

        // Tangent-space normal: linear-blend XY, keep Z coherent (UDN-style;
        // full RNM is a Phase 3 setting, §2.6 "Normal blend formulas").
        // NORMAL_XY_STRENGTH compensates the relief flattening from the
        // 512² aux arrays + box-filtered mip chain (authored 2K normals lose
        // ~2 octaves of gradient detail by the first visible mip; without
        // the boost the ground reads flat, "like a plain .png").
        let NORMAL_XY_STRENGTH: f32 = 1.4;
        final_normal_ts =
            final_normal_ts + vec3<f32>(n_ts.xy * NORMAL_XY_STRENGTH * w, n_ts.z * w);

        // ORM: R=AO, G=Roughness, B=Metallic (standard packing).
        final_ao = final_ao + orm_s.r * w;
        final_roughness = final_roughness + (orm_s.g * layer.material_factors.y) * w;
        final_metallic = final_metallic + (orm_s.b * layer.material_factors.x) * w;
    }

    // Assemble tangent-space normal. The world-space surface normal is
    // in.world_normal; for a terrain mesh we construct a TBN using the
    // geometric normal + arbitrary tangent (the XZ plane's +X direction),
    // then project the accumulated tangent-space perturbation into world.
    let N_geom = normalize(in.world_normal);
    // Derive tangent as the horizontal direction perpendicular to N_geom.
    // For a near-vertical normal (y ≈ 1), this picks +X; for steep slopes
    // it remains well-defined via the cross-product.
    var T_world: vec3<f32>;
    if (abs(N_geom.y) < 0.99) {
        T_world = normalize(cross(vec3<f32>(0.0, 1.0, 0.0), N_geom));
    } else {
        T_world = vec3<f32>(1.0, 0.0, 0.0);
    }
    let B_world = normalize(cross(N_geom, T_world));
    let n_ts_norm = normalize(final_normal_ts + vec3<f32>(0.0, 0.0, 0.001));
    let N = normalize(T_world * n_ts_norm.x + B_world * n_ts_norm.y + N_geom * n_ts_norm.z);

    let V = normalize(uCamera.camera_pos - in.world_pos);
    let L = normalize(-uCamera.light_dir);
    // View distance — consumed by the CSM cascade select (block 6) and the
    // distance fog (block 8). Computed once.
    let frag_dist = length(in.world_pos - uCamera.camera_pos);
    let base_color = final_albedo;
    // Clamp to the range SHADER_SRC uses (metallic full, roughness >= 0.04
    // to avoid singular GGX at perfect mirrors).
    let metallic = clamp(final_metallic, 0.0, 1.0);
    let roughness = clamp(final_roughness, 0.04, 1.0);
    let F0 = mix(vec3<f32>(0.04), base_color, metallic);

    // ED-3 debug shading (uniform branch — same pipeline, same pass). The UV
    // view shows the CHUNK parameterization (fract(in.uv)); per-layer tiling
    // multiplies this by uv_scale uniformly, so chunk-level wrap/scale
    // problems are what this view exposes.
    if (uScene.debug_mode > 0.5) {
        if (uScene.debug_mode < 1.5) {
            return vec4<f32>(base_color, 1.0);               // 1: unlit albedo
        }
        if (uScene.debug_mode < 2.5) {
            return vec4<f32>(N * 0.5 + vec3<f32>(0.5), 1.0); // 2: world-space normals
        }
        return vec4<f32>(fract(in.uv), 0.0, 1.0);            // 3: UVs
    }

    // 5. Unified BRDF (Cook-Torrance + Burley + multiscatter) — same helper
    //    SHADER_SRC uses. The material-LOD tiers were retired in T.2d.F: the
    //    LOD1|2 footprint threshold was the director-observed camera-anchored
    //    boundary on this very terrain (docs/audits/T2D_CAMERA_LIGHT.md §10).
    let brdf_result = evaluate_brdf(N, V, L, base_color, metallic, roughness, F0);

    // 6. Direct sun lighting × CSM shadow (L.3). The shadow factor multiplies
    //    the DIRECT term only — never the IBL below — so shadowed ground
    //    stays skylit instead of going black (same term discipline as AO on
    //    indirect-only, per the L.2 ratified directive). extras.x < 0 is the
    //    shadows-off sentinel (uniform branch, same gate as SHADER_SRC); in
    //    that state shadow stays 1.0 and this block is arithmetically
    //    identical to the pre-L.3 shader.
    var shadow: f32 = 1.0;
    if (uLight.extras.x >= 0.0) {
        shadow = csm_shadow_factor(uLight, shadow_tex, shadow_sampler, in.world_pos, frag_dist);
    }
    let radiance = uScene.sun_color * uScene.sun_intensity;
    var lit_color = brdf_result * radiance * shadow;

    // 7. Environment light — full split-sum IBL (L.2), REPLACING the Phase-1
    //    flat ambient fill (`ambient_color * ambient_intensity * 0.35`); the
    //    scene must not double-count. AO multiplies the indirect terms only
    //    (never direct sun) — this is where the T.2a-repaired material AO
    //    finally becomes visible (T2F §4 measured it at 0.00% of pixels).
    //    NOTE: uScene.ambient_* is no longer read here; it still drives the
    //    static-mesh ambient floor (SHADER_SRC).
    let ibl_color = compute_ibl(N, V, base_color, metallic, roughness, F0);
    lit_color = lit_color + ibl_color * final_ao;

    // 8. Distance fog — matches SHADER_SRC's formula.
    lit_color = apply_terrain_fog(lit_color, frag_dist);

    return vec4<f32>(lit_color, 1.0);
}
