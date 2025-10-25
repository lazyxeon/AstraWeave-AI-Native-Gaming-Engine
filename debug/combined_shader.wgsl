// Shared PBR utilities and data structures for AstraWeave examples

// ============================================================================
// Material GPU Representation
// ============================================================================

struct MaterialGpu {
    albedo_index: u32,
    normal_index: u32,
    orm_index: u32,
    flags: u32,
    base_color_factor: vec4<f32>,
    emissive_factor: vec4<f32>,
    orm_factors: vec4<f32>,  // occlusion, roughness, metallic, unused
    tiling_triplanar: vec4<f32>,  // tiling.x, tiling.y, triplanar_scale, unused
};

const MATERIAL_FLAG_HAS_ALBEDO: u32 = 1u << 0u;
const MATERIAL_FLAG_HAS_NORMAL: u32 = 1u << 1u;
const MATERIAL_FLAG_HAS_ORM: u32 = 1u << 2u;
const MATERIAL_FLAG_TRIPLANAR: u32 = 1u << 3u;

fn has_flag(flags: u32, flag: u32) -> bool {
    return (flags & flag) != 0u;
}

// ============================================================================
// PBR BRDF Functions (Cook-Torrance)
// ============================================================================

const PI: f32 = 3.14159265359;

/// GGX/Trowbridge-Reitz Normal Distribution Function
/// Determines microfacet distribution for given roughness
fn distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    
    let denom = NdotH2 * (a2 - 1.0) + 1.0;
    return a2 / (PI * denom * denom);
}

/// Smith Geometry Function (Schlick-GGX)
/// Accounts for microfacet shadowing and masking
fn geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0; // Direct lighting
    
    return NdotV / (NdotV * (1.0 - k) + k);
}

fn geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx2 = geometry_schlick_ggx(NdotV, roughness);
    let ggx1 = geometry_schlick_ggx(NdotL, roughness);
    
    return ggx1 * ggx2;
}

/// Fresnel-Schlick Approximation
/// Determines ratio of reflected vs refracted light
fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    let m = clamp(1.0 - cos_theta, 0.0, 1.0);
    let factor = m * m * m * m * m;  // (1-cos)^5
    return f0 + (vec3<f32>(1.0) - f0) * factor;
}

/// Fresnel-Schlick with roughness for IBL
fn fresnel_schlick_roughness(cos_theta: f32, f0: vec3<f32>, roughness: f32) -> vec3<f32> {
    let m = clamp(1.0 - cos_theta, 0.0, 1.0);
    let factor = m * m * m * m * m;
    return f0 + (max(vec3<f32>(1.0 - roughness), f0) - f0) * factor;
}

/// Cook-Torrance BRDF (Specular Term)
/// Full physically-based specular reflection
fn cook_torrance_brdf(
    N: vec3<f32>,
    V: vec3<f32>,
    L: vec3<f32>,
    H: vec3<f32>,
    roughness: f32,
    F0: vec3<f32>
) -> vec3<f32> {
    let D = distribution_ggx(N, H, roughness);
    let G = geometry_smith(N, V, L, roughness);
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    
    let NdotL = max(dot(N, L), 0.0);
    let NdotV = max(dot(N, V), 0.0);
    let denom = 4.0 * NdotV * NdotL + 0.0001; // Avoid divide by zero
    
    return (D * G * F) / denom;
}

/// Complete PBR Lighting Calculation (Direct Lighting)
/// Combines diffuse (Lambertian) and specular (Cook-Torrance) terms
fn pbr_direct_lighting(
    N: vec3<f32>,
    V: vec3<f32>,
    L: vec3<f32>,
    light_color: vec3<f32>,
    albedo: vec3<f32>,
    roughness: f32,
    metallic: f32
) -> vec3<f32> {
    let H = normalize(V + L);
    let NdotL = max(dot(N, L), 0.0);
    
    // Calculate F0 (base reflectivity at normal incidence)
    let dielectric_f0 = vec3<f32>(0.04); // Non-metals reflect ~4%
    let F0 = mix(dielectric_f0, albedo, metallic);
    
    // Specular (Cook-Torrance BRDF)
    let specular = cook_torrance_brdf(N, V, L, H, roughness, F0);
    
    // Diffuse (Lambertian with energy conservation)
    let F = fresnel_schlick(max(dot(H, V), 0.0), F0);
    let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic); // Metals have no diffuse
    let diffuse = kD * albedo / PI;
    
    // Combine diffuse + specular
    return (diffuse + specular) * light_color * NdotL;
}

// ============================================================================
// IBL (Image-Based Lighting) Functions
// ============================================================================

fn sample_brdf_lut(
    lut: texture_2d<f32>,
    lut_sampler: sampler,
    ndotv: f32,
    roughness: f32,
) -> vec2<f32> {
    let coord = vec2<f32>(clamp(ndotv, 0.0, 1.0), clamp(roughness, 0.0, 1.0));
    return textureSample(lut, lut_sampler, coord).rg;
}

/// Sample diffuse irradiance from preconvolved irradiance cubemap
fn sample_ibl_diffuse(
    irradiance_cube: texture_cube<f32>,
    ibl_sampler: sampler,
    N: vec3<f32>
) -> vec3<f32> {
    return textureSample(irradiance_cube, ibl_sampler, N).rgb;
}

/// Sample specular reflection from prefiltered environment map
/// Uses roughness to select appropriate mip level
fn sample_ibl_specular(
    specular_cube: texture_cube<f32>,
    ibl_sampler: sampler,
    R: vec3<f32>,
    roughness: f32,
    max_mip_level: f32
) -> vec3<f32> {
    let mip = roughness * max_mip_level;
    return textureSampleLevel(specular_cube, ibl_sampler, R, mip).rgb;
}

/// Complete IBL contribution (diffuse + specular with BRDF integration)
/// V = view direction, N = surface normal, F0 = base reflectance, roughness, occlusion
fn evaluate_ibl(
    irradiance_cube: texture_cube<f32>,
    specular_cube: texture_cube<f32>,
    brdf_lut: texture_2d<f32>,
    ibl_sampler: sampler,
    N: vec3<f32>,
    V: vec3<f32>,
    F0: vec3<f32>,
    roughness: f32,
    occlusion: f32,
    albedo: vec3<f32>,
    metallic: f32,
    max_mip_level: f32
) -> vec3<f32> {
    let NdotV = max(dot(N, V), 0.0);
    let R = reflect(-V, N);
    
    // Fresnel with roughness for environment lighting
    let F = fresnel_schlick_roughness(NdotV, F0, roughness);
    
    // Diffuse IBL (irradiance) with energy conservation
    let irradiance = sample_ibl_diffuse(irradiance_cube, ibl_sampler, N);
    let kD = (vec3<f32>(1.0) - F) * (1.0 - metallic); // Metals have no diffuse
    let diffuse = kD * irradiance * albedo;
    
    // Specular IBL (prefiltered environment + BRDF LUT)
    let prefilteredColor = sample_ibl_specular(specular_cube, ibl_sampler, R, roughness, max_mip_level);
    let brdf = sample_brdf_lut(brdf_lut, ibl_sampler, NdotV, roughness);
    let specular = prefilteredColor * (F * brdf.x + brdf.y);
    
    // Combine with ambient occlusion
    return (diffuse + specular) * occlusion;
}

// ============================================================================
// Material Sampling Functions
// ============================================================================

// Sample material from GPU storage buffer and apply to base values
struct MaterialSample {
    albedo: vec3<f32>,
    normal: vec3<f32>,
    orm: vec3<f32>, // occlusion, roughness, metallic
    emissive: vec3<f32>,
    has_textures: bool,
};

/// Sample material textures and apply factors
/// Returns MaterialSample with albedo, normal (tangent-space), orm, and emissive
fn sample_material(
    mat: MaterialGpu,
    uv: vec2<f32>,
    albedo_array: texture_2d_array<f32>,
    albedo_samp: sampler,
    normal_array: texture_2d_array<f32>,
    normal_samp: sampler,
    orm_array: texture_2d_array<f32>,
    orm_samp: sampler
) -> MaterialSample {
    var result: MaterialSample;
    
    // Apply tiling to UV coordinates
    let tiled_uv = uv * mat.tiling_triplanar.xy;
    
    // Sample albedo with sRGBâ†’Linear conversion (if texture format is sRGB, it's automatic)
    if (has_flag(mat.flags, MATERIAL_FLAG_HAS_ALBEDO)) {
        let albedo_sample = textureSample(albedo_array, albedo_samp, tiled_uv, i32(mat.albedo_index));
        result.albedo = albedo_sample.rgb * mat.base_color_factor.rgb;
        result.has_textures = true;
    } else {
        result.albedo = mat.base_color_factor.rgb;
        result.has_textures = false;
    }
    
    // Sample normal map (stored as RG, reconstruct Z)
    if (has_flag(mat.flags, MATERIAL_FLAG_HAS_NORMAL)) {
        let normal_sample = textureSample(normal_array, normal_samp, tiled_uv, i32(mat.normal_index));
        // Convert from [0,1] to [-1,1] and reconstruct Z
        let nxy = normal_sample.rg * 2.0 - 1.0;
        let nz = sqrt(max(0.0, 1.0 - dot(nxy, nxy)));
        result.normal = normalize(vec3<f32>(nxy.x, nxy.y, nz));
    } else {
        result.normal = vec3<f32>(0.0, 0.0, 1.0); // Flat normal in tangent space
    }
    
    // Sample ORM (Occlusion, Roughness, Metallic) - all linear
    if (has_flag(mat.flags, MATERIAL_FLAG_HAS_ORM)) {
        let orm_sample = textureSample(orm_array, orm_samp, tiled_uv, i32(mat.orm_index));
        result.orm = orm_sample.rgb * mat.orm_factors.rgb;
    } else {
        result.orm = mat.orm_factors.rgb;
    }
    
    // Emissive (additive, no lighting applied)
    result.emissive = mat.emissive_factor.rgb;
    
    return result;
}

/// Transform tangent-space normal to world space using TBN matrix
fn apply_normal_map(tangent_normal: vec3<f32>, N: vec3<f32>, T: vec3<f32>, B: vec3<f32>) -> vec3<f32> {
    let tbn = mat3x3<f32>(
        normalize(T),
        normalize(B),
        normalize(N)
    );
    return normalize(tbn * tangent_normal);
}

// ============================================================================
// Utility Functions
// ============================================================================

/// Calculate TBN (Tangent-Bitangent-Normal) matrix
/// For surfaces without explicit tangents
fn compute_tangent_basis(normal: vec3<f32>) -> vec3<f32> {
    // Find least significant component to avoid numerical issues
    let c = select(
        vec3<f32>(1.0, 0.0, 0.0),
        vec3<f32>(0.0, 0.0, 1.0),
        abs(normal.x) > abs(normal.z)
    );
    
    // Compute tangent using cross product
    return normalize(cross(c, normal));
}

/// Tone mapping (Reinhard)
fn tonemap_reinhard(hdr: vec3<f32>) -> vec3<f32> {
    return hdr / (hdr + vec3<f32>(1.0));
}

/// Tone mapping (ACES filmic)
fn tonemap_aces(hdr: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((hdr * (a * hdr + b)) / (hdr * (c * hdr + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

/// Gamma correction (sRGB)
fn gamma_correct(linear: vec3<f32>) -> vec3<f32> {
    return pow(linear, vec3<f32>(1.0 / 2.2));
}

struct Camera { view_proj: mat4x4<f32> };
struct DebugParams { debug_tint: u32, _pad: vec3<f32> };
struct PostParams { exposure: f32, _pad0: vec3<f32> };
struct SceneParams { time: f32, camera_height: f32, _pad: vec2<f32> };
struct MaterialUniform {
    albedo: vec4<f32>,
    emissive: vec4<f32>,
    roughness_metallic: vec4<f32>,
    flags: u32,
    _pad0: u32,
    _pad1: u32,
    _pad2: u32,
};

// Align to WGSL std140-like rules: use vec4 for cascade splits to ensure 16-byte alignment
struct ShadowParams {
    resolution: f32,
    cascade_count: u32,
    softness: f32,
    bias: f32,
    cascade_splits: vec4<f32>,
};

@group(0) @binding(0) var<uniform> u_camera: Camera;
@group(0) @binding(1) var<uniform> u_post: PostParams;
@group(0) @binding(2) var<uniform> u_scene: SceneParams;
@group(0) @binding(4) var<uniform> u_debug: DebugParams;
const MATERIAL_LAYER_TOTAL: i32 = 12;
const MATERIAL_GRASS: i32 = 0;
const MATERIAL_DIRT: i32 = 1;
const MATERIAL_STONE: i32 = 2;
const MATERIAL_SAND: i32 = 3;
const MATERIAL_FOREST_FLOOR: i32 = 4;
const MATERIAL_TREE_BARK: i32 = 5;
const MATERIAL_TREE_LEAVES: i32 = 6;
const MATERIAL_ROCK_LICHEN: i32 = 7;
const MATERIAL_ROCK_SLATE: i32 = 8;
const MATERIAL_PLASTER: i32 = 9;
const MATERIAL_ROOF_TILE: i32 = 10;
const MATERIAL_CLOTH: i32 = 11;

@group(1) @binding(0) var material_albedo: texture_2d_array<f32>;
@group(1) @binding(1) var material_albedo_sampler: sampler;
@group(1) @binding(2) var material_normal: texture_2d_array<f32>;
@group(1) @binding(3) var material_normal_sampler: sampler;
@group(1) @binding(4) var material_mra: texture_2d_array<f32>;
@group(1) @binding(5) var<storage, read> materials: array<MaterialGpu>;

@group(4) @binding(0) var<uniform> u_material: MaterialUniform;

// IBL bindings: prefiltered specular, irradiance, BRDF LUT, sampler
@group(5) @binding(0) var ibl_specular: texture_cube<f32>;
@group(5) @binding(1) var ibl_irradiance: texture_cube<f32>;
@group(5) @binding(2) var brdf_lut: texture_2d<f32>;
@group(5) @binding(3) var ibl_sampler: sampler;

// Shadows
@group(2) @binding(0) var shadow_map: texture_depth_2d;
@group(2) @binding(1) var shadow_sampler: sampler_comparison;
@group(3) @binding(0) var<uniform> u_light: Camera;
@group(3) @binding(1) var<uniform> u_shadow_params: ShadowParams;

// Helper struct and functions for triplanar sampling across texture sets
struct SampleSet { c: vec3<f32>, n: vec3<f32>, m: vec4<f32> };

// -----------------------------------------------------------------------------
// Noise utilities (hash, value noise, FBM) to avoid grid-aligned artifacts
// -----------------------------------------------------------------------------

fn hash21(p: vec2<f32>) -> f32 {
    let h = dot(p, vec2<f32>(127.1, 311.7));
    return fract(sin(h) * 43758.5453);
}

fn smooth2(t: vec2<f32>) -> vec2<f32> {
    // Quintic smoothing for C2 continuity
    return t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
}

fn value_noise2(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = smooth2(f);

    let a = hash21(i + vec2<f32>(0.0, 0.0));
    let b = hash21(i + vec2<f32>(1.0, 0.0));
    let c = hash21(i + vec2<f32>(0.0, 1.0));
    let d = hash21(i + vec2<f32>(1.0, 1.0));

    let x1 = mix(a, b, u.x);
    let x2 = mix(c, d, u.x);
    return mix(x1, x2, u.y);
}

fn fbm2(p: vec2<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    var f = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    var i = 0;
    loop {
        if (i >= octaves) { break; }
        f = f + value_noise2(p * freq) * amp;
        freq = freq * lacunarity;
        amp = amp * gain;
        i = i + 1;
    }
    return f;
}

fn fbm_ridge2(p: vec2<f32>, octaves: i32, lacunarity: f32, gain: f32) -> f32 {
    // Ridged multifractal using absolute value and inversion
    var f = 0.0;
    var amp = 0.5;
    var freq = 1.0;
    var i = 0;
    loop {
        if (i >= octaves) { break; }
        let n = value_noise2(p * freq);
        let r = 1.0 - abs(n * 2.0 - 1.0);
        f = f + r * amp;
        freq = freq * lacunarity;
        amp = amp * gain;
        i = i + 1;
    }
    return f;
}

fn noise_grad2(p: vec2<f32>) -> vec2<f32> {
    // Central differences gradient of value noise
    let e = 0.001;
    let dx = value_noise2(p + vec2<f32>(e, 0.0)) - value_noise2(p - vec2<f32>(e, 0.0));
    let dy = value_noise2(p + vec2<f32>(0.0, e)) - value_noise2(p - vec2<f32>(0.0, e));
    return vec2<f32>(dx, dy) / (2.0 * e);
}

fn reconstruct_normal_from_rg(nrg: vec2<f32>) -> vec3<f32> {
    let nxy = nrg * 2.0 - 1.0;
    let nz = sqrt(max(0.0, 1.0 - dot(nxy, nxy)));
    return vec3<f32>(nxy.x, nxy.y, nz);
}

// Wrapper functions for pbr_lib.wgsl IBL helpers
// These provide simplified signatures using global bindings (ibl_irradiance, ibl_specular, brdf_lut, ibl_sampler)
fn sample_ibl_diffuse_wrapper(N: vec3<f32>) -> vec3<f32> {
    return sample_ibl_diffuse_wrapper(ibl_irradiance, ibl_sampler, N);
}

fn sample_ibl_specular_wrapper(R: vec3<f32>, roughness: f32, NdotV: f32) -> vec3<f32> {
    let max_mips = f32(textureNumLevels(ibl_specular));
    let max_mip_level = max(0.0, max_mips - 1.0);
    let prefiltered = sample_ibl_specular_wrapper(ibl_specular, ibl_sampler, R, roughness, max_mip_level);
    let brdf = textureSample(brdf_lut, ibl_sampler, vec2<f32>(clamp(NdotV, 0.0, 1.0), clamp(roughness, 0.0, 1.0))).rg;
    // Split-sum: F approx is applied outside; here we return scaled specular
    return prefiltered * vec3<f32>(brdf.x, brdf.x, brdf.x) + vec3<f32>(brdf.y, brdf.y, brdf.y);
}

fn sample_material_layer(layer: i32, uv: vec2<f32>) -> SampleSet {
    let idx = clamp(layer, 0, MATERIAL_LAYER_TOTAL - 1);
    let c = textureSample(material_albedo, material_albedo_sampler, uv, idx).rgb;
    var n_samp = textureSample(material_normal, material_normal_sampler, uv, idx);
    // If normals stored as RG, reconstruct Z; otherwise use RGB
    let n = vec3<f32>(n_samp.rg, sqrt(max(0.0, 1.0 - dot(n_samp.rg * 2.0 - 1.0, n_samp.rg * 2.0 - 1.0))));
    let m = textureSample(material_mra, material_normal_sampler, uv, idx);
    return SampleSet(c, n, m);
}

// by biome (legacy mappings)
fn sample_set(biome: i32, uv: vec2<f32>) -> SampleSet {
    if (biome == 0) { return sample_material_layer(MATERIAL_GRASS, uv); }
    if (biome == 1) { return sample_material_layer(MATERIAL_SAND, uv); }
    return sample_material_layer(MATERIAL_FOREST_FLOOR, uv);
}

// Percentage-closer filtering shadow test
fn sample_shadow(world_pos: vec3<f32>, N: vec3<f32>, L: vec3<f32>) -> f32 {
    let pos_light = u_light.view_proj * vec4<f32>(world_pos, 1.0);
    let proj = pos_light.xyz / pos_light.w;
    // Transform from NDC (-1..1) to 0..1
    let uv = proj.xy * 0.5 + vec2<f32>(0.5, 0.5);
    let depth = proj.z * 0.5 + 0.5;
    let cascade_count = max(u_shadow_params.cascade_count, 1u);
    var cascade_idx: u32 = 0u;
    for (var i: u32 = 0u; i < cascade_count; i = i + 1u) {
        if (depth <= u_shadow_params.cascade_splits[i]) {
            cascade_idx = i;
            break;
        }
        cascade_idx = i;
    }
    cascade_idx = min(cascade_idx, cascade_count - 1u);

    let cascade_factor = 1.0 + f32(cascade_idx) * 0.35;
    let texel = 1.0 / max(u_shadow_params.resolution, 1.0);
    let base_bias = max(u_shadow_params.bias, 1e-5);
    let bias = max(base_bias * (1.0 - dot(N, L)), base_bias * 0.5);
    // Early out if outside light frustum
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0) { return 0.0; }

    // Rotated Poisson-disk PCF (16 taps)
    let softness = u_shadow_params.softness;

    // Random rotation per-fragment derived from UV to reduce banding
    let rnd = fract(sin(dot(uv, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    let angle = rnd * 6.2831853; // 2*pi
    let c = cos(angle);
    let s = sin(angle);
    let rot = mat2x2<f32>(vec2<f32>(c, s), vec2<f32>(-s, c)); // [ [c,-s],[s,c] ] in column form

    // Filter radius: slightly wider at grazing angles to hide stair-steps
    let radius = (softness * cascade_factor + (1.0 - clamp(dot(N, L), 0.0, 1.0)) * 2.0) * texel;

    // Unrolled 16 Poisson taps to avoid dynamic array indexing restrictions
    var occl: f32 = 0.0;
    let p0 = rot * vec2<f32>(-0.94201624, -0.39906216);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p0 * radius * 2.5, depth - bias));
    let p1 = rot * vec2<f32>(0.94558609, -0.76890725);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p1 * radius * 2.5, depth - bias));
    let p2 = rot * vec2<f32>(-0.09418410, -0.92938870);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p2 * radius * 2.5, depth - bias));
    let p3 = rot * vec2<f32>(0.34495938, 0.29387760);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p3 * radius * 2.5, depth - bias));
    let p4 = rot * vec2<f32>(-0.91588581, 0.45771432);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p4 * radius * 2.5, depth - bias));
    let p5 = rot * vec2<f32>(-0.81544232, -0.87912464);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p5 * radius * 2.5, depth - bias));
    let p6 = rot * vec2<f32>(-0.38277543, 0.27676845);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p6 * radius * 2.5, depth - bias));
    let p7 = rot * vec2<f32>(0.97484398, 0.75648379);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p7 * radius * 2.5, depth - bias));
    let p8 = rot * vec2<f32>(0.44323325, -0.97511554);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p8 * radius * 2.5, depth - bias));
    let p9 = rot * vec2<f32>(0.53742981, -0.47373420);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p9 * radius * 2.5, depth - bias));
    let p10 = rot * vec2<f32>(-0.26496911, -0.41893023);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p10 * radius * 2.5, depth - bias));
    let p11 = rot * vec2<f32>(0.79197514, 0.19090188);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p11 * radius * 2.5, depth - bias));
    let p12 = rot * vec2<f32>(-0.24188840, 0.99706507);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p12 * radius * 2.5, depth - bias));
    let p13 = rot * vec2<f32>(-0.81409955, 0.91437590);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p13 * radius * 2.5, depth - bias));
    let p14 = rot * vec2<f32>(0.19984126, 0.78641367);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p14 * radius * 2.5, depth - bias));
    let p15 = rot * vec2<f32>(0.14383161, -0.14100790);
    occl = occl + (1.0 - textureSampleCompare(shadow_map, shadow_sampler, uv + p15 * radius * 2.5, depth - bias));
    return occl / 16.0;
}

// by material: 0=grass,1=dirt,2=stone,3=sand,4=forest
// Note: sample_material is provided by pbr_lib.wgsl
// Local helper for backward compatibility with simple signature
// by material: 0=grass,1=dirt,2=stone,3=sand,4=forest - calls sample_material_layer
fn get_material(which: i32, uv: vec2<f32>) -> SampleSet {
    return sample_material_layer(which, uv);
}

// Blend three sample sets with enhanced triplanar weights and material transitions
fn blend3(a: SampleSet, b: SampleSet, c: SampleSet, w: vec3<f32>, transition_width: f32) -> SampleSet {
    // Enhanced weight calculation with smooth transitions
    let w_smooth = pow(w, vec3<f32>(transition_width));
    let wsum = max(w_smooth.x + w_smooth.y + w_smooth.z, 1e-4);
    let wnorm = w_smooth / wsum;

    // Colors are already sampled in linear space (sRGB textures are linearized by the sampler),
    // so blend directly in linear space without manual gamma conversions
    var col = a.c * wnorm.x + b.c * wnorm.y + c.c * wnorm.z;

    // Enhanced normal blending with proper tangent space interpolation
    let na = normalize(a.n * 2.0 - 1.0);
    let nb = normalize(b.n * 2.0 - 1.0);
    let nc = normalize(c.n * 2.0 - 1.0);
    
    // Use weighted average with angle-based weighting for better normal interpolation
    let n_blend = na * wnorm.x + nb * wnorm.y + nc * wnorm.z;
    let nr = normalize(n_blend) * 0.5 + 0.5;
    
    // MRA blending with material property preservation
    let mr = a.m * wnorm.x + b.m * wnorm.y + c.m * wnorm.z;
    
    // Optional debug tint to validate material sampling bindings
    if (u_debug.debug_tint == 1u) {
        let tint_a = vec3<f32>(1.0, 0.0, 0.0);
        let tint_b = vec3<f32>(0.0, 1.0, 0.0);
        let tint_c = vec3<f32>(0.0, 0.0, 1.0);
        col = col * 0.5 + (tint_a * wnorm.x + tint_b * wnorm.y + tint_c * wnorm.z) * 0.5;
    }
    return SampleSet(col, nr, mr);
}

// Enhanced triplanar sampling with detail mapping and micro-variation
fn sample_triplanar_enhanced(material_idx: i32, world_pos: vec3<f32>, normal: vec3<f32>, 
                           scale: f32, detail_scale: f32) -> SampleSet {
    // Base UV coordinates for triplanar mapping
    let uv_x = world_pos.zy * scale;
    let uv_y = world_pos.xz * scale; 
    let uv_z = world_pos.xy * scale;
    // Add micro-variation using FBM noise to break up tiling without grid artifacts
    let n_xy = fbm2(world_pos.xy * detail_scale * 0.15, 4, 2.0, 0.5);
    let n_yz = fbm2(world_pos.yz * detail_scale * 0.15, 4, 2.0, 0.5);
    let n_zx = fbm2(world_pos.zx * detail_scale * 0.15, 4, 2.0, 0.5);
    let v = 0.08; // variation strength
    let uv_x_varied = uv_x + vec2<f32>(n_yz, n_zx) * v;
    let uv_y_varied = uv_y + vec2<f32>(n_zx, n_xy) * v;
    let uv_z_varied = uv_z + vec2<f32>(n_xy, n_yz) * v;
    
    // Sample each axis with variation
    let sample_x = get_material(material_idx, uv_x_varied);
    let sample_y = get_material(material_idx, uv_y_varied);
    let sample_z = get_material(material_idx, uv_z_varied);
    
    // Enhanced weight calculation based on normal with smooth transitions
    let n_abs = abs(normal);
    let weights = pow(n_abs, vec3<f32>(2.0)); // Slightly softer to reduce seams
    let weight_sum = max(weights.x + weights.y + weights.z, 1e-4);
    let wnorm = weights / weight_sum;
    
    // Use enhanced blending with smooth transitions
    return blend3(sample_x, sample_y, sample_z, wnorm, 2.0);
}

// Detail mapping function for micro-surface variation
fn apply_detail_mapping(base_color: vec3<f32>, world_pos: vec3<f32>, detail_scale: f32, detail_strength: f32) -> vec3<f32> {
    // FBM-based subtle albedo variation
    let p = world_pos.xz * detail_scale;
    let n = fbm2(p, 4, 2.0, 0.5);
    let variation = (n - 0.5) * 2.0; // [-1,1]
    let detail_color = base_color * (1.0 + variation * 0.06 * detail_strength);
    return detail_color;
}

// Perturb the surface normal using a simple high-frequency procedural pattern
fn perturb_normal(normal: vec3<f32>, world_pos: vec3<f32>, detail_scale: f32, strength: f32) -> vec3<f32> {
    // Use gradient of FBM to perturb normal in a stable, non-grid way
    let p = world_pos.xz * detail_scale;
    let g = noise_grad2(p);
    // Build tangent basis with up approximation; for terrain N is already close to up
    let T = normalize(cross(normal, vec3<f32>(0.0, 1.0, 0.0)) + vec3<f32>(1e-4, 0.0, 0.0));
    let B = normalize(cross(normal, T));
    let n_perturb = (T * g.x + B * g.y) * strength;
    return normalize(normal + n_perturb);
}

// Calculate tessellation factor based on distance and terrain features
fn calculate_tessellation_factor(world_pos: vec3<f32>, camera_pos: vec3<f32>, slope: f32) -> f32 {
    let distance = length(world_pos - camera_pos);
    
    // Base tessellation decreases with distance
    let distance_factor = clamp(1.0 - distance / 100.0, 0.1, 1.0);
    
    // Increase tessellation on steep slopes for better detail
    let slope_factor = 1.0 + slope * 2.0;
    
    // Combine factors with some noise for natural variation
    let noise_factor = sin(world_pos.x * 0.01) * cos(world_pos.z * 0.01) * 0.1 + 0.9;
    
    return distance_factor * slope_factor * noise_factor;
}

// Vertex inputs aligned with Rust vertex buffers:
// - location(0): position (vec3)
// - locations(1..4): model matrix columns (vec4)
// - location(5): color (vec4)
// - location(6): material_id (u32)
// - location(7): mesh_type (u32)
struct VsIn {
    @location(0) pos: vec3<f32>,
    @location(1) m0: vec4<f32>,
    @location(2) m1: vec4<f32>,
    @location(3) m2: vec4<f32>,
    @location(4) m3: vec4<f32>,
    @location(5) color: vec4<f32>,
    @location(6) material_id: u32,
    @location(7) mesh_type: u32,
};

// Full-vertex path for glTF overrides (P/N/T/UV + instance data)
struct VsInFull {
    @location(0) pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) uv: vec2<f32>,
    @location(4) m0: vec4<f32>,
    @location(5) m1: vec4<f32>,
    @location(6) m2: vec4<f32>,
    @location(7) m3: vec4<f32>,
    @location(8) color: vec4<f32>,
    @location(9) material_id: u32,
    @location(10) mesh_type: u32,
};

struct VsOut {
  @builtin(position) pos: vec4<f32>,
  @location(0) color: vec4<f32>,
  @location(1) world_pos: vec3<f32>,
  @location(2) view_dir: vec3<f32>,
    @location(3) normal: vec3<f32>,
    @location(4) uv: vec2<f32>,
  @location(5) mesh_type: u32,
    @location(6) local_pos: vec3<f32>,
    @location(7) material_id: u32,
};

@vertex
fn vs_main(in: VsIn) -> VsOut {
    let model = mat4x4<f32>(in.m0, in.m1, in.m2, in.m3);
  var out: VsOut;
  let world = model * vec4<f32>(in.pos, 1.0);
  
// Special handling for skybox (mesh_type 5) - position at far plane
    if (in.mesh_type == 5u) {
    // For skybox, apply model transform then scale down the large vertices
    let scaled_vertex = vec4<f32>(in.pos * 0.5, 1.0); // Larger skybox for better coverage  
    let world_skybox = model * scaled_vertex; // Apply model transform (camera translation)
    out.pos = u_camera.view_proj * world_skybox;
    // Ensure skybox is always at far plane
    out.pos.z = out.pos.w * 0.999; // At far plane
    out.world_pos = world_skybox.xyz;
    out.local_pos = in.pos;
    
    // For skybox, we use position as texture coordinates
    let pos_normalized = normalize(in.pos);
    out.uv = vec2<f32>(
      0.5 + atan2(pos_normalized.z, pos_normalized.x) / (2.0 * 3.14159),
      0.5 - asin(pos_normalized.y) / 3.14159
    );
  } else {
    out.pos = u_camera.view_proj * world;
    out.world_pos = world.xyz;
        // Derive UVs procedurally from world position (no per-vertex UVs provided)
        let scale = 10.0;
        out.uv = vec2<f32>(world.x / scale, world.z / scale);
      out.local_pos = in.pos;
  }
  
    // Default normal (up). Real meshes should provide normals; this keeps lighting reasonable.
    out.normal = vec3<f32>(0.0, 1.0, 0.0);
  
  out.color = in.color;
  out.mesh_type = in.mesh_type;
  out.material_id = in.material_id;
  
  // Calculate view direction for sky effects
  let camera_pos = vec3<f32>(0.0, 5.0, 0.0); // Better approximation for typical camera height
  out.view_dir = normalize(world.xyz - camera_pos);
  
  return out;
}

@vertex
fn vs_main_full(in: VsInFull) -> VsOut {
    let model = mat4x4<f32>(in.m0, in.m1, in.m2, in.m3);
    var out: VsOut;
    let world = model * vec4<f32>(in.pos, 1.0);
    if (in.mesh_type == 5u) {
        // Skybox vertex handling similar to vs_main
        let scaled_vertex = vec4<f32>(in.pos * 0.5, 1.0);
        let world_skybox = model * scaled_vertex;
        out.pos = u_camera.view_proj * world_skybox;
        out.pos.z = out.pos.w * 0.999;
        out.world_pos = world_skybox.xyz;
        out.local_pos = in.pos;
        let pos_normalized = normalize(in.pos);
        out.uv = vec2<f32>(
            0.5 + atan2(pos_normalized.z, pos_normalized.x) / (2.0 * 3.14159),
            0.5 - asin(pos_normalized.y) / 3.14159
        );
    } else {
        out.pos = u_camera.view_proj * world;
        out.world_pos = world.xyz;
        out.uv = in.uv;
        out.local_pos = in.pos;
    }
    let normal_matrix = mat3x3<f32>(
        normalize(model[0].xyz),
        normalize(model[1].xyz),
        normalize(model[2].xyz)
    );
    out.normal = normalize(normal_matrix * in.normal);
    out.color = in.color;
    out.mesh_type = in.mesh_type;
    out.material_id = in.material_id;
    let camera_pos = vec3<f32>(0.0, 5.0, 0.0);
    out.view_dir = normalize(world.xyz - camera_pos);
    return out;
}

// Enhanced sky color function - generates biome-appropriate sky with atmospheric effects
fn sky_color(direction: vec3<f32>, time: f32) -> vec3<f32> {
    let dir = normalize(direction);
    let y = clamp(dir.y, -1.0, 1.0);
    // Stable daytime blue gradient
    let horizon = vec3<f32>(0.75, 0.85, 1.0);
    let zenith = vec3<f32>(0.15, 0.45, 0.9);
    let t = pow(clamp((y + 1.0) * 0.5, 0.0, 1.0), 0.6);
    let base = mix(horizon, zenith, t);
    // Soft moving clouds
    let n = sin(dir.x * 5.0 + time * 0.2) * cos(dir.z * 4.0 + time * 0.15);
    let clouds = clamp(n, 0.0, 1.0) * 0.12;
    return base + vec3<f32>(clouds, clouds, clouds);
}


// Enhanced water rendering for rivers and lakes
fn get_water_level(world_pos: vec2<f32>, time: f32) -> f32 {
  let wave_scale = 0.5;
  let wave_time = time * 2.0;
  let wave_pos = world_pos * wave_scale + vec2<f32>(wave_time * 0.3, wave_time * 0.5);
  
  // Water surface animation
  let wave1 = sin(wave_pos.x * 4.0) * cos(wave_pos.y * 3.0) * 0.1;
  let wave2 = sin(wave_pos.x * 8.0 + 1.0) * cos(wave_pos.y * 6.0 + 1.5) * 0.05;
  
  return -1.8 + wave1 + wave2; // Base water level with waves
}

// Determine biome type based on world position - FIXED: More consistent biome regions
fn get_biome_type(world_pos: vec2<f32>) -> i32 {
    let biome_scale = 0.01;
    let biome_pos = world_pos * biome_scale;
    // Smooth FBM determines regions; thresholds split into 3 biomes
    let n = fbm2(biome_pos, 5, 2.0, 0.5) * 2.0 - 1.0; // [-1,1]
    if (n > 0.25) {
        return 1; // Desert
    } else if (n < -0.25) {
        return 2; // Dense Forest
    } else {
        return 0; // Grassland
    }
}

// Generate biome-specific terrain height with enhanced variation
fn get_biome_terrain_height(world_pos: vec2<f32>, biome_type: i32) -> f32 {
    // Base FBM terrain: smooth features with octave detail
    let p = world_pos * 0.01;
    let base = (fbm2(p, 5, 2.0, 0.5) * 2.0 - 1.0) * 3.5; // [-3.5,3.5]
    let detail = (fbm2(p * 3.0, 3, 2.2, 0.5) - 0.5) * 1.5;
    var h = base + detail;

    if (biome_type == 0) { // Grassland - rolling hills with river valleys
        // River mask via ridge FBM to carve shallow valleys
        let river = fbm_ridge2(p * 0.9, 3, 2.0, 0.6);
        let valleys = (river - 0.6) * 6.0; // negative mostly
        h = h + clamp(valleys, -3.5, 0.0);
        return h;

    } else if (biome_type == 1) { // Desert - dunes and mesas
        // Dunes via directional ridged noise along wind axis
        let wind_dir = normalize(vec2<f32>(0.8, 0.6));
        let d = dot(p * 6.0, wind_dir);
        let dunes = (fbm_ridge2(vec2<f32>(d, d * 0.3), 4, 2.0, 0.55) - 0.5) * 3.0;
        // Mesas via thresholded ridge noise
        let mesas = step(0.7, fbm_ridge2(p * 0.7, 4, 2.0, 0.5)) * 6.0;
        h = h + dunes + mesas;
        return h;

    } else if (biome_type == 2) { // Dense Forest - softer undulations with mounds
        let soft = fbm2(p * 0.7, 4, 2.0, 0.5) * 2.0 - 1.0;
        let mounds = step(0.65, fbm2(p * 1.3 + 12.3, 4, 2.0, 0.5)) * 1.6;
        h = h * 0.9 + soft * 2.2 + mounds;
        return h;

    } else {
        return h;
    }
}

@fragment
fn fs_main(in: VsOut) -> @location(0) vec4<f32> {
  var col = in.color.rgb;
    let time = u_scene.time;
  
  // Mesh-specific rendering based on mesh type
    if (in.mesh_type == 1u) { // Tree (separate bark and foliage materials)
        let V = normalize(-in.view_dir);
        let sun_angle = time * 0.1;
        let L = normalize(vec3<f32>(cos(sun_angle) * 0.8, max(sin(sun_angle) * 0.8, 0.2), sin(sun_angle * 0.3) * 0.4));

        // Use local Y to separate trunk vs canopy; sample by mesh UVs
        let is_trunk = in.local_pos.y < 0.6;
        var base_sample: SampleSet;
        if (is_trunk) {
            base_sample = sample_material_layer(MATERIAL_TREE_BARK, in.uv);
        } else {
            base_sample = sample_material_layer(MATERIAL_TREE_LEAVES, in.uv);
        }
                // PBR lighting using vertex normal
                let N = normalize(in.normal);
                let H = normalize(L + V);
                let ao = base_sample.m.r;
                let roughness = clamp(base_sample.m.g, 0.08, 0.98);
                let metallic = clamp(base_sample.m.b * 0.2, 0.0, 0.2);
                let base_color = base_sample.c;
                let ior = mix(1.3, 2.0, metallic);
                let F0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
                let F0_vec = mix(vec3<f32>(F0, F0, F0), base_color, metallic);
                let F = F0_vec + (vec3<f32>(1.0) - F0_vec) * pow(1.0 - clamp(dot(H, V), 0.0, 1.0), 5.0);
                let a = roughness * roughness; let a2 = a * a;
                let NdotH = max(dot(N, H), 0.0); let NdotH2 = NdotH * NdotH;
                let denom = (NdotH2 * (a2 - 1.0) + 1.0);
                let D = a2 / (3.14159 * denom * denom + 1e-5);
                let NdotV = max(dot(N, V), 0.0); let NdotL = max(dot(N, L), 0.0);
                let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
                let Gv = NdotV / (NdotV * (1.0 - k) + k + 1e-5);
                let Gl = NdotL / (NdotL * (1.0 - k) + k + 1e-5);
                let G_geom = Gv * Gl;
                let kd = (vec3<f32>(1.0) - F) * (1.0 - metallic);
                let diffuse = kd * base_color / 3.14159;
                let specular = (F * D * G_geom) / max(4.0 * NdotV * NdotL + 1e-5, 1e-5);
                let ibl_on = f32((u_debug.debug_tint & 2u) >> 1u);
                let sky_ambient = sky_color(normalize(in.world_pos), time) * 0.2 + sample_ibl_diffuse_wrapper(N) * 0.15 * ibl_on;
                let enhanced_ao = pow(ao, 0.85);
                let ambient = base_color * enhanced_ao * 0.25 + sky_ambient * enhanced_ao;
                let shadow = sample_shadow(in.world_pos, N, L);
                // Specular IBL term
                let R = reflect(-V, N);
                let spec_ibl = sample_ibl_specular_wrapper(R, roughness, NdotV) * ibl_on;
                col = ambient + (diffuse + specular) * NdotL * (1.0 - shadow) + spec_ibl * 0.5;
                // Output linear color; surface is sRGB so conversion happens on write
                return vec4<f32>(col, 1.0);
    } else if (in.mesh_type == 2u) { // House/Structure (walls vs roof)
                let V = normalize(-in.view_dir);
                let sun_angle = time * 0.1;
                let L = normalize(vec3<f32>(cos(sun_angle) * 0.8, max(sin(sun_angle) * 0.8, 0.2), sin(sun_angle * 0.3) * 0.4));

                // Separate walls (lower) and roof (upper) by local Y
                let is_roof = in.local_pos.y > 0.6;
                var base_sample: SampleSet;
                if (is_roof) {
            base_sample = sample_material_layer(MATERIAL_ROOF_TILE, in.uv);
            base_sample.c = base_sample.c * vec3<f32>(1.05, 0.98, 0.92);
                } else {
            let plaster = sample_material_layer(MATERIAL_PLASTER, in.uv);
            let stone = sample_material_layer(MATERIAL_STONE, in.uv);
            base_sample = plaster;
            base_sample.c = mix(plaster.c, stone.c, 0.2);
            base_sample.m = mix(plaster.m, stone.m, vec4<f32>(0.25, 0.25, 0.25, 0.0));
                }
                // PBR shading using vertex normal
                let N = normalize(in.normal);
                let H = normalize(L + V);
                let ao = base_sample.m.r;
                let roughness = clamp(base_sample.m.g, 0.12, 0.98);
                let metallic = clamp(base_sample.m.b * 0.2, 0.0, 0.2);
                let base_color = base_sample.c;
                let ior = mix(1.3, 2.0, metallic);
                let F0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
                let F0_vec = mix(vec3<f32>(F0, F0, F0), base_color, metallic);
                let F = F0_vec + (vec3<f32>(1.0) - F0_vec) * pow(1.0 - clamp(dot(H, V), 0.0, 1.0), 5.0);
                let a = roughness * roughness; let a2 = a * a;
                let NdotH = max(dot(N, H), 0.0); let NdotH2 = NdotH * NdotH;
                let denom = (NdotH2 * (a2 - 1.0) + 1.0);
                let D = a2 / (3.14159 * denom * denom + 1e-5);
                let NdotV = max(dot(N, V), 0.0); let NdotL = max(dot(N, L), 0.0);
                let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
                let Gv = NdotV / (NdotV * (1.0 - k) + k + 1e-5);
                let Gl = NdotL / (NdotL * (1.0 - k) + k + 1e-5);
                let G_geom = Gv * Gl;
                let kd = (vec3<f32>(1.0) - F) * (1.0 - metallic);
                let diffuse = kd * base_color / 3.14159;
                let specular = (F * D * G_geom) / max(4.0 * NdotV * NdotL + 1e-5, 1e-5);
                let ibl_on = f32((u_debug.debug_tint & 2u) >> 1u);
                let sky_ambient = sky_color(normalize(in.world_pos), time) * 0.18 + sample_ibl_diffuse_wrapper(N) * 0.12 * ibl_on;
                let enhanced_ao = pow(ao, 0.85);
                let ambient = base_color * enhanced_ao * 0.25 + sky_ambient * enhanced_ao;
                let shadow = sample_shadow(in.world_pos, N, L);
                let R = reflect(-V, N);
                let spec_ibl = sample_ibl_specular_wrapper(R, roughness, NdotV) * ibl_on;
                col = ambient + (diffuse + specular) * NdotL * (1.0 - shadow) + spec_ibl * 0.4;
                return vec4<f32>(col, 1.0);
    } else if (in.mesh_type == 3u) { // Rock formations and boulders
        let V = normalize(-in.view_dir);
        let sun_angle = time * 0.1;
        let L = normalize(vec3<f32>(cos(sun_angle) * 0.8, max(sin(sun_angle) * 0.8, 0.2), sin(sun_angle * 0.3) * 0.4));
        let lichen = sample_material_layer(MATERIAL_ROCK_LICHEN, in.uv);
        let slate = sample_material_layer(MATERIAL_ROCK_SLATE, in.uv);
        var base_sample: SampleSet = lichen;
        base_sample.c = mix(lichen.c, slate.c, 0.45);
        base_sample.m = mix(lichen.m, slate.m, vec4<f32>(0.4, 0.5, 0.4, 0.0));
        let N = normalize(in.normal);
        let H = normalize(L + V);
        let ao = base_sample.m.r;
        let roughness = clamp(base_sample.m.g, 0.35, 0.95);
        let metallic = clamp(base_sample.m.b * 0.35, 0.05, 0.45);
        let base_color = base_sample.c;
        let ior = mix(1.35, 2.3, metallic);
        let F0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
        let F0_vec = mix(vec3<f32>(F0, F0, F0), base_color, metallic);
        let F = F0_vec + (vec3<f32>(1.0) - F0_vec) * pow(1.0 - clamp(dot(H, V), 0.0, 1.0), 5.0);
        let a = roughness * roughness; let a2 = a * a;
        let NdotH = max(dot(N, H), 0.0); let NdotH2 = NdotH * NdotH;
        let denom = (NdotH2 * (a2 - 1.0) + 1.0);
        let D = a2 / (3.14159 * denom * denom + 1e-5);
        let NdotV = max(dot(N, V), 0.0); let NdotL = max(dot(N, L), 0.0);
        let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
        let Gv = NdotV / (NdotV * (1.0 - k) + k + 1e-5);
        let Gl = NdotL / (NdotL * (1.0 - k) + k + 1e-5);
        let G_geom = Gv * Gl;
        let kd = (vec3<f32>(1.0) - F) * (1.0 - metallic);
        let diffuse = kd * base_color / 3.14159;
        let specular = (F * D * G_geom) / max(4.0 * NdotV * NdotL + 1e-5, 1e-5);
    let ibl_on = f32((u_debug.debug_tint & 2u) >> 1u);
    let sky_ambient = sky_color(normalize(in.world_pos), time) * 0.14 + sample_ibl_diffuse_wrapper(N) * 0.08 * ibl_on;
        let enhanced_ao = pow(ao, 0.9);
        let ambient = base_color * enhanced_ao * 0.28 + sky_ambient * enhanced_ao;
        let shadow = sample_shadow(in.world_pos, N, L);
    let R = reflect(-V, N);
    let spec_ibl = sample_ibl_specular_wrapper(R, roughness, NdotV) * ibl_on;
    col = ambient + (diffuse + specular) * NdotL * (1.0 - shadow) + spec_ibl * 0.35;
        return vec4<f32>(col, 1.0);
    } else if (in.mesh_type == 4u) { // Character - cloth-centric shading tinted per instance
        let V = normalize(-in.view_dir);
        let sun_angle = time * 0.1;
        let L = normalize(vec3<f32>(cos(sun_angle) * 0.8, max(sin(sun_angle) * 0.8, 0.2), sin(sun_angle * 0.3) * 0.4));
        var base_sample: SampleSet = sample_material_layer(MATERIAL_CLOTH, in.uv);
        base_sample.c = clamp(base_sample.c * in.color.rgb, vec3<f32>(0.05), vec3<f32>(1.0));
        let N = normalize(in.normal);
        let H = normalize(L + V);
        let ao = base_sample.m.r;
        let roughness = clamp(base_sample.m.g * 1.15, 0.35, 0.9);
        let metallic = clamp(base_sample.m.b * 0.08, 0.0, 0.18);
        let base_color = base_sample.c;
        let ior = mix(1.28, 1.9, metallic);
        let F0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
        let F0_vec = mix(vec3<f32>(F0, F0, F0), base_color, metallic);
        let F = F0_vec + (vec3<f32>(1.0) - F0_vec) * pow(1.0 - clamp(dot(H, V), 0.0, 1.0), 5.0);
        let a = roughness * roughness; let a2 = a * a;
        let NdotH = max(dot(N, H), 0.0); let NdotH2 = NdotH * NdotH;
        let denom = (NdotH2 * (a2 - 1.0) + 1.0);
        let D = a2 / (3.14159 * denom * denom + 1e-5);
        let NdotV = max(dot(N, V), 0.0); let NdotL = max(dot(N, L), 0.0);
        let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
        let Gv = NdotV / (NdotV * (1.0 - k) + k + 1e-5);
        let Gl = NdotL / (NdotL * (1.0 - k) + k + 1e-5);
        let G_geom = Gv * Gl;
        let kd = (vec3<f32>(1.0) - F) * (1.0 - metallic);
        let diffuse = kd * base_color / 3.14159;
        let specular = (F * D * G_geom) / max(4.0 * NdotV * NdotL + 1e-5, 1e-5);
    let ibl_on = f32((u_debug.debug_tint & 2u) >> 1u);
    let sky_ambient = sky_color(normalize(in.world_pos), time) * 0.16 + sample_ibl_diffuse_wrapper(N) * 0.11 * ibl_on;
        let enhanced_ao = pow(ao, 0.8);
        let ambient = base_color * enhanced_ao * 0.3 + sky_ambient * enhanced_ao;
        let shadow = sample_shadow(in.world_pos, N, L);
    let R = reflect(-V, N);
    let spec_ibl = sample_ibl_specular_wrapper(R, roughness, NdotV) * ibl_on;
    col = ambient + (diffuse + specular) * NdotL * (1.0 - shadow) + spec_ibl * 0.3;
        return vec4<f32>(col, 1.0);
    }
  else if (in.mesh_type == 5u) { // Skybox
    // Use procedural sky color based on view direction
    col = sky_color(in.view_dir, time);
    return vec4<f32>(col, 1.0);
  }
  
    // For ground or other unspecified mesh types, use PBR-ish terrain rendering
  // Determine biome type for this world position
  let biome_type = get_biome_type(in.world_pos.xz);
  
  // Enhanced ground rendering with biome-specific terrain
  let ground_y = -2.0;
  let terrain_height = get_biome_terrain_height(in.world_pos.xz, biome_type);
  let water_level = get_water_level(in.world_pos.xz, time);
  let terrain_surface = ground_y + terrain_height;
  let dist_to_terrain = abs(in.world_pos.y - terrain_surface);
  let dist_to_water = abs(in.world_pos.y - water_level);
  
  // Check if we're rendering the terrain surface
  if (dist_to_terrain < 0.8 || (in.mesh_type == 0u && in.world_pos.y < ground_y + 1.0)) {
        // Enhanced triplanar sampling with biome-specific scales and detail mapping
        let ws_pos = in.world_pos;
        let base_scale = 2.0;
        let detail_scale = 8.0;
        
        // Biome-specific texture scaling for more realistic material distribution
        var biome_scale = base_scale;
        if (biome_type == 1) { biome_scale = base_scale * 1.2; } // Desert - slightly larger scale
        else if (biome_type == 2) { biome_scale = base_scale * 0.9; } // Forest - slightly smaller scale
        
        // Enhanced surface normal calculation with better height gradients
        let eps = 0.3;
        let hC = terrain_height;
        let hL = get_biome_terrain_height((ws_pos.xz + vec2<f32>(-eps, 0.0)), biome_type);
        let hR = get_biome_terrain_height((ws_pos.xz + vec2<f32>( eps, 0.0)), biome_type);
        let hD = get_biome_terrain_height((ws_pos.xz + vec2<f32>( 0.0,-eps)), biome_type);
        let hU = get_biome_terrain_height((ws_pos.xz + vec2<f32>( 0.0, eps)), biome_type);
        let dx = (hR - hL) / (2.0 * eps);
        let dz = (hU - hD) / (2.0 * eps);
        let n_world = normalize(vec3<f32>(-dx, 1.0, -dz));

        // Advanced material weight calculation with multiple influencing factors
        let slope = clamp(1.0 - n_world.y, 0.0, 1.0);
        let height_factor = clamp((hC + 2.0) / 4.0, 0.0, 1.0); // Normalized height factor
        let moisture_factor = sin(ws_pos.x * 0.01) * cos(ws_pos.z * 0.008) * 0.5 + 0.5; // Pseudo-moisture
        
        // Base material weights with biome-specific logic
        var w_grass = 0.0;
        var w_dirt = 0.0;
        var w_stone = 0.0;
        var w_sand = 0.0;
        var w_forest = 0.0;
        
        if (biome_type == 0) { // Grassland
            // Grass dominates low slopes and mid elevations
            w_grass = (1.0 - slope) * smoothstep(0.0, 0.6, height_factor) * (1.0 - moisture_factor * 0.3);
            // Dirt appears on steeper slopes and drier areas
            w_dirt = slope * 0.4 + moisture_factor * 0.2;
            // Stone for rocky outcrops
            w_stone = smoothstep(0.3, 0.8, slope) * 0.6;
            
        } else if (biome_type == 1) { // Desert
            // Sand dominates with some variation
            w_sand = 0.7 * (1.0 - smoothstep(0.4, 0.9, slope));
            // Stone for rocky areas and steep slopes
            w_stone = smoothstep(0.2, 0.7, slope) * 0.8;
            // Dirt in transitional areas
            w_dirt = 0.2 * (1.0 - slope) * moisture_factor;
            
        } else if (biome_type == 2) { // Forest
            // Forest floor dominates with organic materials
            w_forest = 0.6 * (1.0 - slope) * smoothstep(0.0, 0.7, height_factor);
            // Grass in clearings
            w_grass = 0.3 * (1.0 - slope) * (1.0 - smoothstep(0.3, 0.8, height_factor));
            // Dirt on slopes
            w_dirt = slope * 0.4;
            // Stone for rocky areas
            w_stone = smoothstep(0.5, 1.0, slope) * 0.5;
        }
        
        // Normalize weights to ensure they sum to 1
        var total_weight = w_grass + w_dirt + w_stone + w_sand + w_forest;
        if (total_weight < 0.1) {
            // Fallback to ensure we always have some material
            w_grass = 0.5;
            w_dirt = 0.3;
            w_stone = 0.2;
            total_weight = 1.0;
        }
        w_grass /= total_weight;
        w_dirt /= total_weight;
        w_stone /= total_weight;
        w_sand /= total_weight;
        w_forest /= total_weight;

        // Enhanced triplanar sampling for each material with detail mapping
    let grass_sample = sample_triplanar_enhanced(MATERIAL_GRASS, ws_pos, n_world, biome_scale, detail_scale);
    let dirt_sample = sample_triplanar_enhanced(MATERIAL_DIRT, ws_pos, n_world, biome_scale, detail_scale);
    let stone_sample = sample_triplanar_enhanced(MATERIAL_STONE, ws_pos, n_world, biome_scale, detail_scale);
    let sand_sample = sample_triplanar_enhanced(MATERIAL_SAND, ws_pos, n_world, biome_scale, detail_scale);
    let forest_sample = sample_triplanar_enhanced(MATERIAL_FOREST_FLOOR, ws_pos, n_world, biome_scale, detail_scale);

        // Combine materials with enhanced blending
        var base_color = grass_sample.c * w_grass + 
                        dirt_sample.c * w_dirt + 
                        stone_sample.c * w_stone + 
                        sand_sample.c * w_sand + 
                        forest_sample.c * w_forest;
    base_color = base_color * mix(vec3<f32>(1.0), u_material.albedo.rgb, 0.2);
        
        var mra = grass_sample.m * w_grass + 
                 dirt_sample.m * w_dirt + 
                 stone_sample.m * w_stone + 
                 sand_sample.m * w_sand + 
                 forest_sample.m * w_forest;
        
        // Enhanced normal blending with proper interpolation
        var normal_blend = (grass_sample.n * 2.0 - 1.0) * w_grass +
                          (dirt_sample.n * 2.0 - 1.0) * w_dirt +
                          (stone_sample.n * 2.0 - 1.0) * w_stone +
                          (sand_sample.n * 2.0 - 1.0) * w_sand +
                          (forest_sample.n * 2.0 - 1.0) * w_forest;
        var normal = normalize(normal_blend);

    // Calculate tessellation factor for geometry detail
        let tessellation_factor = calculate_tessellation_factor(ws_pos, vec3<f32>(0.0, 5.0, 0.0), slope);
        
        // Use tessellation factor to modulate detail mapping strength
    let detail_strength = 0.6; // Base strength for micro detail mapping
    let adaptive_detail_strength = detail_strength * tessellation_factor;
        base_color = apply_detail_mapping(base_color, ws_pos, detail_scale, adaptive_detail_strength);
        
        // Apply normal perturbation with adaptive strength
        let adaptive_normal_strength = 0.1 * tessellation_factor;
        normal = perturb_normal(normal, ws_pos, detail_scale * 2.0, adaptive_normal_strength);

    let ao = mra.r; // ambient occlusion
    var roughness = clamp(mra.g, 0.08, 0.95);
    var metallic = clamp(mra.b, 0.0, 0.2);
    roughness = mix(roughness, clamp(u_material.roughness_metallic.x, 0.05, 1.0), 0.25);
    metallic = mix(metallic, clamp(u_material.roughness_metallic.y, 0.0, 1.0), 0.25);
    let emissive = u_material.emissive.rgb * u_material.emissive.w;
    
    // Biome-specific terrain texturing
        // Note: Removed large flat-color biome overlays; rely on layered PBR materials above
        // Enhanced PBR lighting with improved Cook-Torrance BRDF
        let V = normalize(-in.view_dir);
        let N = normalize(normal);
        let sun_angle = time * 0.1;
        let L = normalize(vec3<f32>(cos(sun_angle) * 0.8, max(sin(sun_angle) * 0.8, 0.2), sin(sun_angle * 0.3) * 0.4));
        let H = normalize(L + V);

        // Enhanced Fresnel with realistic IOR values based on material
        let ior = mix(1.3, 2.5, metallic); // Vary IOR based on metallic content
        let F0 = pow((ior - 1.0) / (ior + 1.0), 2.0);
        let F0_vec = mix(vec3<f32>(F0, F0, F0), base_color, metallic);
        let F = F0_vec + (vec3<f32>(1.0, 1.0, 1.0) - F0_vec) * pow(1.0 - clamp(dot(H, V), 0.0, 1.0), 5.0);
        
        // Improved Normal Distribution Function (GGX)
        let a = roughness * roughness;
        let a2 = a * a;
        let NdotH = max(dot(N, H), 0.0);
        let NdotH2 = NdotH * NdotH;
        let denom = (NdotH2 * (a2 - 1.0) + 1.0);
        let D = a2 / (3.14159 * denom * denom + 1e-5);
        
        // Enhanced Geometry Smith with correlated masking-shadowing
        let NdotV = max(dot(N, V), 0.0);
        let NdotL = max(dot(N, L), 0.0);
        let k = (roughness + 1.0) * (roughness + 1.0) / 8.0;
        let Gv = NdotV / (NdotV * (1.0 - k) + k + 1e-5);
        let Gl = NdotL / (NdotL * (1.0 - k) + k + 1e-5);
        let G_geom = Gv * Gl;

        // Enhanced diffuse with subsurface scattering for organic materials
        let kd = (vec3<f32>(1.0, 1.0, 1.0) - F) * (1.0 - metallic);
        
        // Add subsurface scattering approximation for materials like grass and soil
        let subsurface_factor = 0.15 * (1.0 - roughness) * (1.0 - metallic) * ao;
        let subsurface = subsurface_factor * base_color * max(dot(N, L), 0.0) * 0.5;
        
        let diffuse = kd * base_color / 3.14159 + subsurface;
        let specular = (F * D * G_geom) / max(4.0 * NdotV * NdotL + 1e-5, 1e-5);
        
        // Enhanced ambient lighting with sky contribution and improved AO
    let ibl_on = f32((u_debug.debug_tint & 2u) >> 1u);
    let sky_ambient = sky_color(normalize(in.world_pos), time) * 0.22 + sample_ibl_diffuse_wrapper(N) * 0.18 * ibl_on;
        let enhanced_ao = pow(ao, 0.8); // Enhance AO contrast
        let ambient = base_color * enhanced_ao * 0.3 + sky_ambient * enhanced_ao;
        
        let shadow = sample_shadow(in.world_pos, N, L);
    let R = reflect(-V, N);
    let spec_ibl = sample_ibl_specular_wrapper(R, roughness, NdotV) * ibl_on;
    var color = ambient + (diffuse + specular) * NdotL * (1.0 - shadow) + spec_ibl * 0.5 + emissive;

    // Remove crosshatch-like overlay modulation; rely on material detail mapping above

        // Exponential height fog with simple Rayleigh/Mie approximation (apply pre-tone-map)
    let cam_height = u_scene.camera_height;
        let height_falloff = 0.02;
        let base_density = 0.02;
        let view_dir = normalize(in.view_dir);
        let distance_view = length(in.world_pos - vec3<f32>(0.0, cam_height, 0.0));
        let altitude = clamp(in.world_pos.y, -50.0, 500.0);
        let height_term = exp(-height_falloff * (altitude - cam_height));
        let fog_amount = 1.0 - exp(-base_density * height_term * distance_view);
        let cosTheta = clamp(dot(-view_dir, L), -1.0, 1.0);
        let rayleigh_phase = 0.75 * (1.0 + cosTheta * cosTheta);
        let mie_g = 0.5;
        let mie_phase = 1.0 / pow(1.0 + mie_g * mie_g - 2.0 * mie_g * cosTheta, 1.5);
        let fog_sky = sky_color(normalize(in.world_pos), time);
        let fog_color = fog_sky * 0.6 * rayleigh_phase + vec3<f32>(0.9, 0.9, 0.85) * 0.05 * mie_phase;
        color = mix(color, fog_color, clamp(fog_amount, 0.0, 1.0));

        // Leave color in linear HDR here; post pass will apply exposure + tone mapping
        col = color;
        // Atmospheric perspective now handled by height fog above (pre-tonemap)
    
  } else if (dist_to_water < 0.3 && terrain_height < -0.5) {
    // Water rendering for rivers and lakes
    let water_color = vec3<f32>(0.1, 0.3, 0.6);
    let wave_distortion = sin(in.world_pos.x * 2.0 + time) * cos(in.world_pos.z * 1.8 + time * 1.2) * 0.05;
    let water_surface = water_color + vec3<f32>(wave_distortion, wave_distortion * 0.5, -wave_distortion * 0.3);
    
    // Reflection and transparency effects
    let view_angle = abs(dot(normalize(in.view_dir), vec3<f32>(0.0, 1.0, 0.0)));
    let reflection_factor = 1.0 - view_angle;
    let sky = sky_color(reflect(in.view_dir, vec3<f32>(0.0, 1.0, 0.0)), time);
    
    col = mix(water_surface, sky, reflection_factor * 0.6);
    
  } else {
    // Sky rendering for non-terrain objects or background
        if (in.mesh_type == 5u) { // Skybox
      // Full procedural sky rendering
      col = sky_color(in.view_dir, time);
      
    } else if (in.mesh_type == 1u) { // Trees
      // Enhanced tree rendering with seasonal variation
      let tree_base_color = in.color.rgb;
      let seasonal_factor = (sin(time * 0.05) + 1.0) * 0.5;
      let autumn_color = vec3<f32>(0.8, 0.4, 0.1);
      let summer_color = vec3<f32>(0.2, 0.8, 0.3);
      col = mix(tree_base_color, mix(summer_color, autumn_color, seasonal_factor), 0.3);
      
        } else if (in.mesh_type == 2u) { // Houses/Structures
      // Enhanced building rendering with weathering
      col = in.color.rgb;
      let weathering = sin(in.world_pos.x * 0.5) * cos(in.world_pos.z * 0.3) * 0.1;
      col = col * (0.95 + weathering);
      
        } else if (in.mesh_type == 3u) { // Rocks fallback tint
            let rock_tint = vec3<f32>(0.55, 0.56, 0.52);
            col = mix(in.color.rgb, rock_tint, 0.4);
      
    } else {
      // Other objects get sky ambient lighting
      let sky = sky_color(in.view_dir, time);
      col = in.color.rgb * 0.8 + sky * 0.2;
    }
  }
  
    return vec4<f32>(col, 1.0);
}

