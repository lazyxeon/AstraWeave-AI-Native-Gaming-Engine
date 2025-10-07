// Phase PBR-E: Advanced Materials Extension
// Adds clearcoat, anisotropy, subsurface scattering, sheen, and transmission to base PBR

// ============================================================================
// MATERIAL DEFINITION (Extended from PBR-D)
// ============================================================================

struct MaterialGpuExtended {
    // Base PBR (from Phase PBR-D)
    albedo_index: u32,
    normal_index: u32,
    orm_index: u32,           // Occlusion, Roughness, Metallic
    flags: u32,               // Feature enable bits
    
    base_color_factor: vec4<f32>,
    metallic_factor: f32,
    roughness_factor: f32,
    occlusion_strength: f32,
    _pad0: f32,
    
    emissive_factor: vec3<f32>,
    _pad1: f32,
    
    // Clearcoat (car paint, lacquer)
    clearcoat_strength: f32,
    clearcoat_roughness: f32,
    clearcoat_normal_index: u32,
    _pad2: f32,
    
    // Anisotropy (brushed metal, hair)
    anisotropy_strength: f32,    // [-1, 1]
    anisotropy_rotation: f32,    // [0, 2π] radians
    _pad3: vec2<f32>,
    
    // Subsurface Scattering (skin, wax)
    subsurface_color: vec3<f32>,
    subsurface_scale: f32,
    subsurface_radius: f32,
    thickness_index: u32,
    _pad4: vec2<f32>,
    
    // Sheen (velvet, fabric)
    sheen_color: vec3<f32>,
    sheen_roughness: f32,
    
    // Transmission (glass, water)
    transmission_factor: f32,
    ior: f32,                    // Index of refraction
    _pad5: vec2<f32>,
    
    attenuation_color: vec3<f32>,
    attenuation_distance: f32,
}

// Feature flags (bitfield in MaterialGpuExtended.flags)
const MATERIAL_FLAG_CLEARCOAT: u32 = 0x01u;
const MATERIAL_FLAG_ANISOTROPY: u32 = 0x02u;
const MATERIAL_FLAG_SUBSURFACE: u32 = 0x04u;
const MATERIAL_FLAG_SHEEN: u32 = 0x08u;
const MATERIAL_FLAG_TRANSMISSION: u32 = 0x10u;

// Helper to check feature flags
fn has_feature(material: MaterialGpuExtended, flag: u32) -> bool {
    return (material.flags & flag) != 0u;
}

// ============================================================================
// TANGENT BASIS (for anisotropy)
// ============================================================================

struct TangentBasis {
    T: vec3<f32>,  // Tangent
    B: vec3<f32>,  // Bitangent
    N: vec3<f32>,  // Normal
}

// Compute tangent basis from normal and UV derivatives
// dpdu = dPosition/du (tangent direction from UV gradient)
fn compute_tangent_basis(N: vec3<f32>, dpdu: vec3<f32>) -> TangentBasis {
    // Gram-Schmidt orthogonalization
    let T = normalize(dpdu - dot(dpdu, N) * N);
    let B = cross(N, T);
    
    return TangentBasis(T, B, N);
}

// Rotate tangent basis by angle (for anisotropy_rotation)
fn rotate_tangent_basis(basis: TangentBasis, rotation: f32) -> TangentBasis {
    let cos_r = cos(rotation);
    let sin_r = sin(rotation);
    
    let T_rotated = cos_r * basis.T + sin_r * basis.B;
    let B_rotated = -sin_r * basis.T + cos_r * basis.B;
    
    return TangentBasis(T_rotated, B_rotated, basis.N);
}

// ============================================================================
// CLEARCOAT (2nd Specular Lobe)
// ============================================================================

// Clearcoat uses fixed F0 = 0.04 (IOR = 1.5 for polyurethane/acrylic)
const CLEARCOAT_F0: f32 = 0.04;

// Clearcoat Fresnel (simplified, fixed F0)
fn clearcoat_fresnel(cos_theta: f32) -> f32 {
    let f = 1.0 - cos_theta;
    let f2 = f * f;
    let f5 = f2 * f2 * f;
    return CLEARCOAT_F0 + (1.0 - CLEARCOAT_F0) * f5;
}

// Clearcoat GGX distribution (same as base, but with clearcoat_roughness)
fn clearcoat_distribution_ggx(N: vec3<f32>, H: vec3<f32>, roughness: f32) -> f32 {
    let alpha = roughness * roughness;
    let alpha2 = alpha * alpha;
    let NdotH = max(dot(N, H), 0.0);
    let NdotH2 = NdotH * NdotH;
    
    let denom = NdotH2 * (alpha2 - 1.0) + 1.0;
    let denom2 = denom * denom;
    
    return alpha2 / (3.14159265359 * denom2 + 0.0001);
}

// Clearcoat Smith geometry (simplified, no height correlation for performance)
fn clearcoat_geometry_schlick_ggx(NdotV: f32, roughness: f32) -> f32 {
    let r = roughness + 1.0;
    let k = (r * r) / 8.0;
    return NdotV / (NdotV * (1.0 - k) + k + 0.0001);
}

fn clearcoat_geometry_smith(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, roughness: f32) -> f32 {
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    let ggx_v = clearcoat_geometry_schlick_ggx(NdotV, roughness);
    let ggx_l = clearcoat_geometry_schlick_ggx(NdotL, roughness);
    return ggx_v * ggx_l;
}

// Evaluate clearcoat BRDF lobe
fn evaluate_clearcoat(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, H: vec3<f32>, material: MaterialGpuExtended) -> vec3<f32> {
    if (!has_feature(material, MATERIAL_FLAG_CLEARCOAT)) {
        return vec3<f32>(0.0);
    }
    
    let D = clearcoat_distribution_ggx(N, H, material.clearcoat_roughness);
    let G = clearcoat_geometry_smith(N, V, L, material.clearcoat_roughness);
    let F = clearcoat_fresnel(max(dot(H, V), 0.0));
    
    let NdotV = max(dot(N, V), 0.0);
    let NdotL = max(dot(N, L), 0.0);
    
    let specular = (D * G * F) / (4.0 * NdotV * NdotL + 0.0001);
    
    return vec3<f32>(specular * material.clearcoat_strength);
}

// ============================================================================
// ANISOTROPIC REFLECTIONS
// ============================================================================

// Anisotropic GGX distribution (Burley 2012)
fn distribution_ggx_anisotropic(
    T: vec3<f32>,
    B: vec3<f32>,
    N: vec3<f32>,
    H: vec3<f32>,
    alpha_t: f32,
    alpha_b: f32
) -> f32 {
    let TdotH = dot(T, H);
    let BdotH = dot(B, H);
    let NdotH = dot(N, H);
    
    let a2_t = alpha_t * alpha_t;
    let a2_b = alpha_b * alpha_b;
    
    let denom = (TdotH * TdotH) / a2_t + (BdotH * BdotH) / a2_b + NdotH * NdotH;
    
    return 1.0 / (3.14159265359 * alpha_t * alpha_b * denom * denom + 0.0001);
}

// Anisotropic Smith geometry (simplified, approximates separable masking-shadowing)
fn geometry_smith_anisotropic(
    T: vec3<f32>,
    B: vec3<f32>,
    N: vec3<f32>,
    V: vec3<f32>,
    L: vec3<f32>,
    alpha_t: f32,
    alpha_b: f32
) -> f32 {
    let TdotV = dot(T, V);
    let BdotV = dot(B, V);
    let NdotV = dot(N, V);
    
    let TdotL = dot(T, L);
    let BdotL = dot(B, L);
    let NdotL = dot(N, L);
    
    let lambda_v = 0.5 * (-1.0 + sqrt(1.0 + (alpha_t * alpha_t * TdotV * TdotV + alpha_b * alpha_b * BdotV * BdotV) / (NdotV * NdotV)));
    let lambda_l = 0.5 * (-1.0 + sqrt(1.0 + (alpha_t * alpha_t * TdotL * TdotL + alpha_b * alpha_b * BdotL * BdotL) / (NdotL * NdotL)));
    
    return 1.0 / (1.0 + lambda_v + lambda_l + 0.0001);
}

// Evaluate anisotropic specular BRDF
fn evaluate_anisotropic_specular(
    basis: TangentBasis,
    V: vec3<f32>,
    L: vec3<f32>,
    H: vec3<f32>,
    F0: vec3<f32>,
    material: MaterialGpuExtended
) -> vec3<f32> {
    if (!has_feature(material, MATERIAL_FLAG_ANISOTROPY)) {
        return vec3<f32>(0.0);  // Fall back to isotropic in caller
    }
    
    // Compute anisotropic roughness
    let aspect = sqrt(1.0 - 0.9 * abs(material.anisotropy_strength));
    let alpha_t = max(material.roughness_factor * material.roughness_factor / aspect, 0.001);
    let alpha_b = max(material.roughness_factor * material.roughness_factor * aspect, 0.001);
    
    // Rotate tangent basis if needed
    let rotated_basis = rotate_tangent_basis(basis, material.anisotropy_rotation);
    
    let D = distribution_ggx_anisotropic(rotated_basis.T, rotated_basis.B, rotated_basis.N, H, alpha_t, alpha_b);
    let G = geometry_smith_anisotropic(rotated_basis.T, rotated_basis.B, rotated_basis.N, V, L, alpha_t, alpha_b);
    
    // Fresnel (same as base PBR)
    let cos_theta = max(dot(H, V), 0.0);
    let f = 1.0 - cos_theta;
    let f2 = f * f;
    let F = F0 + (vec3<f32>(1.0) - F0) * f2 * f2 * f;
    
    let NdotV = max(dot(rotated_basis.N, V), 0.0);
    let NdotL = max(dot(rotated_basis.N, L), 0.0);
    
    return (D * G * F) / (4.0 * NdotV * NdotL + 0.0001);
}

// ============================================================================
// SUBSURFACE SCATTERING (Burley Diffusion Approximation)
// ============================================================================

// Wrapped diffuse for subsurface approximation
fn wrap_diffuse(n_dot_l: f32, wrap: f32) -> f32 {
    return max((n_dot_l + wrap) / (1.0 + wrap), 0.0);
}

// Burley diffusion profile (simplified for realtime)
fn burley_diffusion_profile(n_dot_l: f32, subsurface_color: vec3<f32>) -> vec3<f32> {
    // Two-lobe approximation: forward + back scattering
    let wrap_forward = wrap_diffuse(n_dot_l, 0.5);
    let wrap_back = wrap_diffuse(n_dot_l, -0.5);
    
    // Weights from Burley 2015 paper
    let A = 0.7;
    let B = 0.3;
    
    let profile = A * wrap_forward + B * wrap_back;
    
    return subsurface_color * profile / 3.14159265359;
}

// Evaluate subsurface scattering (replaces Lambertian diffuse when enabled)
fn evaluate_subsurface(N: vec3<f32>, L: vec3<f32>, material: MaterialGpuExtended) -> vec3<f32> {
    if (!has_feature(material, MATERIAL_FLAG_SUBSURFACE)) {
        // Fall back to Lambertian
        let n_dot_l = max(dot(N, L), 0.0);
        return vec3<f32>(n_dot_l / 3.14159265359);
    }
    
    let n_dot_l = dot(N, L);
    let sss_profile = burley_diffusion_profile(n_dot_l, material.subsurface_color);
    
    // Blend between Lambertian and SSS
    let lambertian = max(n_dot_l, 0.0) / 3.14159265359;
    return mix(vec3<f32>(lambertian), sss_profile, material.subsurface_scale);
}

// ============================================================================
// SHEEN (Fabric Retroreflection - Estevez & Kulla 2017)
// ============================================================================

// Charlie distribution for sheen (inverted Gaussian)
fn distribution_charlie(roughness: f32, n_dot_h: f32) -> f32 {
    let alpha = max(roughness * roughness, 0.001);
    let inv_alpha = 1.0 / alpha;
    
    let cos2_theta = n_dot_h * n_dot_h;
    let sin2_theta = 1.0 - cos2_theta;
    
    return (2.0 + inv_alpha) * pow(sin2_theta, inv_alpha * 0.5) / (2.0 * 3.14159265359);
}

// Ashikhmin visibility for sheen
fn visibility_ashikhmin(n_dot_v: f32, n_dot_l: f32) -> f32 {
    return 1.0 / (4.0 * (n_dot_l + n_dot_v - n_dot_l * n_dot_v) + 0.0001);
}

// Evaluate sheen BRDF lobe
fn evaluate_sheen(N: vec3<f32>, V: vec3<f32>, L: vec3<f32>, H: vec3<f32>, material: MaterialGpuExtended) -> vec3<f32> {
    if (!has_feature(material, MATERIAL_FLAG_SHEEN)) {
        return vec3<f32>(0.0);
    }
    
    let n_dot_h = max(dot(N, H), 0.0);
    let n_dot_v = max(dot(N, V), 0.0);
    let n_dot_l = max(dot(N, L), 0.0);
    
    let D = distribution_charlie(material.sheen_roughness, n_dot_h);
    let V_vis = visibility_ashikhmin(n_dot_v, n_dot_l);
    
    return material.sheen_color * D * V_vis;
}

// ============================================================================
// TRANSMISSION (Glass/Refraction)
// ============================================================================

// Exact Fresnel for dielectrics (Schlick approximation not accurate enough for transmission)
fn fresnel_dielectric(cos_theta_i: f32, eta: f32) -> f32 {
    let sin_theta_t_sq = eta * eta * (1.0 - cos_theta_i * cos_theta_i);
    
    // Total internal reflection
    if (sin_theta_t_sq > 1.0) {
        return 1.0;
    }
    
    let cos_theta_t = sqrt(1.0 - sin_theta_t_sq);
    
    let r_parallel = (eta * cos_theta_i - cos_theta_t) / (eta * cos_theta_i + cos_theta_t);
    let r_perpendicular = (cos_theta_i - eta * cos_theta_t) / (cos_theta_i + eta * cos_theta_t);
    
    return 0.5 * (r_parallel * r_parallel + r_perpendicular * r_perpendicular);
}

// Refract ray using Snell's law
fn refract_ray(I: vec3<f32>, N: vec3<f32>, eta: f32) -> vec3<f32> {
    let cos_i = dot(N, I);
    let sin_t_sq = eta * eta * (1.0 - cos_i * cos_i);
    
    if (sin_t_sq > 1.0) {
        // Total internal reflection
        return reflect(-I, N);
    }
    
    let cos_t = sqrt(1.0 - sin_t_sq);
    return eta * I - (eta * cos_i + cos_t) * N;
}

// Beer-Lambert attenuation
fn beer_lambert_attenuation(distance: f32, attenuation_color: vec3<f32>, attenuation_distance: f32) -> vec3<f32> {
    if (attenuation_distance <= 0.0) {
        return vec3<f32>(1.0);
    }
    
    // Simplified Beer-Lambert: I(d) = I0 * color^(d / distance)
    let exponent = distance / attenuation_distance;
    return pow(attenuation_color, vec3<f32>(exponent));
}

// Evaluate transmission (requires environment sample and thickness)
fn evaluate_transmission(
    N: vec3<f32>,
    V: vec3<f32>,
    material: MaterialGpuExtended,
    env_sample: vec3<f32>,
    thickness: f32
) -> vec3<f32> {
    if (!has_feature(material, MATERIAL_FLAG_TRANSMISSION)) {
        return vec3<f32>(0.0);
    }
    
    let eta = 1.0 / material.ior;  // Air to material
    let cos_theta = max(dot(N, V), 0.0);
    let F = fresnel_dielectric(cos_theta, eta);
    
    // Transmitted light = (1 - F) * transmission_factor
    let transmitted_energy = (1.0 - F) * material.transmission_factor;
    
    // Apply Beer-Lambert attenuation based on thickness
    let attenuation = beer_lambert_attenuation(thickness, material.attenuation_color, material.attenuation_distance);
    
    return env_sample * transmitted_energy * attenuation;
}

// ============================================================================
// COMBINED ADVANCED PBR EVALUATION
// ============================================================================

// Evaluate all PBR lobes with proper energy conservation
fn evaluate_pbr_advanced(
    N: vec3<f32>,
    V: vec3<f32>,
    L: vec3<f32>,
    basis: TangentBasis,
    material: MaterialGpuExtended,
    light_color: vec3<f32>,
    env_sample: vec3<f32>,
    thickness: f32,
    base_color: vec3<f32>,
    F0: vec3<f32>
) -> vec3<f32> {
    let H = normalize(V + L);
    let n_dot_l = max(dot(N, L), 0.0);
    let n_dot_v = max(dot(N, V), 0.0);
    
    // 1. Transmission splits light into reflected/transmitted
    let transmission_lobe = evaluate_transmission(N, V, material, env_sample, thickness);
    let F_transmission = fresnel_dielectric(n_dot_v, 1.0 / material.ior);
    let reflected_energy = 1.0 - material.transmission_factor * (1.0 - F_transmission);
    
    // 2. Clearcoat attenuates base layer
    let clearcoat_lobe = evaluate_clearcoat(N, V, L, H, material);
    let F_coat = clearcoat_fresnel(n_dot_v) * material.clearcoat_strength;
    let base_energy = reflected_energy * (1.0 - F_coat);
    
    // 3. Base specular (anisotropic if enabled, else isotropic Cook-Torrance)
    var base_specular: vec3<f32>;
    if (has_feature(material, MATERIAL_FLAG_ANISOTROPY)) {
        base_specular = evaluate_anisotropic_specular(basis, V, L, H, F0, material);
    } else {
        // Fall back to isotropic GGX from PBR-D (assume imported from pbr_lib.wgsl)
        // This would call the original cook_torrance_brdf() from Phase PBR-D
        // For now, placeholder:
        base_specular = vec3<f32>(0.0);  // TODO: Import from pbr_lib.wgsl
    }
    base_specular *= base_energy;
    
    // 4. Sheen reduces diffuse energy
    let sheen_lobe = evaluate_sheen(N, V, L, H, material);
    let sheen_max = max(max(material.sheen_color.r, material.sheen_color.g), material.sheen_color.b);
    
    // Fresnel for base layer (for diffuse energy conservation)
    let cos_theta = max(dot(H, V), 0.0);
    let f = 1.0 - cos_theta;
    let f2 = f * f;
    let F_base = F0 + (vec3<f32>(1.0) - F0) * f2 * f2 * f;
    let F_base_avg = (F_base.r + F_base.g + F_base.b) / 3.0;
    
    let diffuse_energy = base_energy * (1.0 - F_base_avg) * (1.0 - material.metallic_factor) * (1.0 - sheen_max);
    
    // 5. Diffuse/SSS
    let diffuse_lobe = evaluate_subsurface(N, L, material) * base_color;
    
    // Combine all lobes
    let direct_lighting = (base_specular + clearcoat_lobe + sheen_lobe + diffuse_lobe * diffuse_energy) * light_color * n_dot_l;
    
    return transmission_lobe + direct_lighting;
}

// ============================================================================
// HELPER: Sample Material with Extended Features
// ============================================================================

// This would integrate with the material sampling from pbr_lib.wgsl (Phase PBR-D)
// Extended to handle clearcoat normals and thickness maps
fn sample_material_extended(
    material_id: u32,
    uv: vec2<f32>,
    materials: ptr<storage, array<MaterialGpuExtended>>,
    textures: texture_2d_array<f32>,
    sampler_linear: sampler
) -> MaterialGpuExtended {
    // TODO: Implement material array lookup and texture sampling
    // Returns populated MaterialGpuExtended with sampled textures
    return (*materials)[material_id];
}
