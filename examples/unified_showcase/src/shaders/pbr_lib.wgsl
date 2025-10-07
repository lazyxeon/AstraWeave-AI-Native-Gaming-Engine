// Shared PBR utilities and data structures for AstraWeave examples

struct MaterialGpu {
    albedo_index: u32,
    normal_index: u32,
    orm_index: u32,
    flags: u32,
    base_color_factor: vec4<f32>,
    emissive_factor: vec4<f32>,
    orm_factors: vec4<f32>,
    tiling_triplanar: vec4<f32>,
};

const MATERIAL_FLAG_HAS_ALBEDO: u32 = 1u << 0u;
const MATERIAL_FLAG_HAS_NORMAL: u32 = 1u << 1u;
const MATERIAL_FLAG_HAS_ORM: u32 = 1u << 2u;
const MATERIAL_FLAG_TRIPLANAR: u32 = 1u << 3u;

fn has_flag(flags: u32, flag: u32) -> bool {
    return (flags & flag) != 0u;
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    let m = clamp(1.0 - cos_theta, 0.0, 1.0);
    let factor = m * m * m * m * m;
    return f0 + (vec3<f32>(1.0, 1.0, 1.0) - f0) * factor;
}

fn sample_brdf_lut(
    lut: texture_2d<f32>,
    lut_sampler: sampler,
    ndotv: f32,
    roughness: f32,
) -> vec2<f32> {
    let coord = vec2<f32>(clamp(ndotv, 0.0, 1.0), clamp(roughness, 0.0, 1.0));
    return textureSample(lut, lut_sampler, coord).rg;
}

// IBL Sampling Functions

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
    
    // Diffuse IBL (irradiance)
    let irradiance = sample_ibl_diffuse(irradiance_cube, ibl_sampler, N);
    let kD = (1.0 - metallic) * (vec3<f32>(1.0) - F0); // Energy conservation
    let diffuse = kD * irradiance * albedo;
    
    // Specular IBL (prefiltered environment + BRDF LUT)
    let prefilteredColor = sample_ibl_specular(specular_cube, ibl_sampler, R, roughness, max_mip_level);
    let brdf = sample_brdf_lut(brdf_lut, ibl_sampler, NdotV, roughness);
    let specular = prefilteredColor * (F0 * brdf.x + brdf.y);
    
    // Combine with occlusion
    return (diffuse + specular) * occlusion;
}

// Sample material from GPU storage buffer and apply to base values
struct MaterialSample {
    albedo: vec3<f32>,
    normal: vec3<f32>,
    orm: vec3<f32>, // occlusion, roughness, metallic
    has_textures: bool,
};

fn sample_material_gpu(
    mat: MaterialGpu,
    uv: vec2<f32>,
    albedo_tex: texture_2d_array<f32>,
    albedo_samp: sampler,
    normal_tex: texture_2d_array<f32>,
    normal_samp: sampler,
    orm_tex: texture_2d_array<f32>,
) -> MaterialSample {
    var result: MaterialSample;
    
    // Sample albedo
    if (has_flag(mat.flags, MATERIAL_FLAG_HAS_ALBEDO)) {
        let albedo_sample = textureSample(albedo_tex, albedo_samp, uv, i32(mat.albedo_index));
        result.albedo = albedo_sample.rgb * mat.base_color_factor.rgb;
        result.has_textures = true;
    } else {
        result.albedo = mat.base_color_factor.rgb;
        result.has_textures = false;
    }
    
    // Sample normal
    if (has_flag(mat.flags, MATERIAL_FLAG_HAS_NORMAL)) {
        let normal_sample = textureSample(normal_tex, normal_samp, uv, i32(mat.normal_index));
        // Reconstruct Z component from RG
        let nxy = normal_sample.rg * 2.0 - 1.0;
        let nz = sqrt(max(0.0, 1.0 - dot(nxy, nxy)));
        result.normal = vec3<f32>(nxy.x, nxy.y, nz);
    } else {
        result.normal = vec3<f32>(0.0, 0.0, 1.0);
    }
    
    // Sample ORM (occlusion, roughness, metallic)
    if (has_flag(mat.flags, MATERIAL_FLAG_HAS_ORM)) {
        let orm_sample = textureSample(orm_tex, albedo_samp, uv, i32(mat.orm_index));
        result.orm = orm_sample.rgb * mat.orm_factors.rgb;
    } else {
        result.orm = mat.orm_factors.rgb;
    }
    
    return result;
}
