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
    
    // Sample albedo with sRGB→Linear conversion (if texture format is sRGB, it's automatic)
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
