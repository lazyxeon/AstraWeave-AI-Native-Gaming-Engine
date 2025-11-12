// PBR Shader for Unified Showcase
// Simple physically-based rendering with albedo, normal, and roughness maps

struct Uniforms {
    view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    model: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _pad1: f32,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

// PBR textures
@group(1) @binding(0)
var albedo_texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var normal_texture: texture_2d<f32>;

@group(1) @binding(3)
var roughness_texture: texture_2d<f32>;

// Terrain multi-material textures (group 2) - TEXTURE ARRAY
@group(2) @binding(0)
var terrain_albedo_array: texture_2d_array<f32>;

@group(2) @binding(1)
var terrain_sampler: sampler;

// TASK 2.3: Terrain normal and roughness arrays
@group(2) @binding(2)
var terrain_normal_array: texture_2d_array<f32>;

@group(2) @binding(3)
var terrain_roughness_array: texture_2d_array<f32>;

// Atlas region data for material atlas UV remapping (group 3)
struct AtlasRegion {
    uv_offset: vec2<f32>,  // UV offset in atlas (e.g., 0.0, 0.25, 0.5, 0.75)
    uv_scale: vec2<f32>,   // UV scale (e.g., 0.25, 0.25 for 4x2 grid)
}

@group(3) @binding(0)
var<uniform> atlas_regions: array<AtlasRegion, 8>;  // 8 material slots in 4×2 atlas grid

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) material_blend: vec4<f32>,
    @location(4) material_id: u32,  // Material index in atlas (0-6)
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_tangent: vec3<f32>,
    @location(4) material_blend: vec4<f32>,
    @location(5) material_id: u32,  // Material index in atlas (0-6)
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_pos = uniforms.model * vec4<f32>(in.position, 1.0);
    out.clip_position = uniforms.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = normalize((uniforms.model * vec4<f32>(in.normal, 0.0)).xyz);
    out.uv = in.uv;
    
    // Calculate tangent (simple approximation)
    let c1 = cross(in.normal, vec3<f32>(0.0, 0.0, 1.0));
    let c2 = cross(in.normal, vec3<f32>(0.0, 1.0, 0.0));
    var tangent: vec3<f32>;
    if length(c1) > length(c2) {
        tangent = c1;
    } else {
        tangent = c2;
    }
    out.world_tangent = normalize((uniforms.model * vec4<f32>(tangent, 0.0)).xyz);
    
    // Pass through material blend weights
    out.material_blend = in.material_blend;
    
    // Pass through material ID for atlas lookup
    out.material_id = in.material_id;
    
    return out;
}

// Simple PBR lighting (no IBL yet, just directional light)
fn simple_pbr(
    base_color: vec3<f32>,
    normal: vec3<f32>,
    roughness: f32,
    view_dir: vec3<f32>,
    light_dir: vec3<f32>,
) -> vec3<f32> {
    // Directional light (sun)
    let light_color = vec3<f32>(1.0, 0.95, 0.9) * 2.0;  // Increased from 1.5 to 2.0
    
    // Lambertian diffuse
    let ndotl = max(dot(normal, light_dir), 0.0);
    let diffuse = base_color * light_color * ndotl;
    
    // Blinn-Phong specular (simplified for now)
    let half_dir = normalize(view_dir + light_dir);
    let ndoth = max(dot(normal, half_dir), 0.0);
    let shininess = mix(256.0, 16.0, roughness);
    let specular = pow(ndoth, shininess) * light_color * (1.0 - roughness) * 0.5;
    
    // CRITICAL FIX: Ambient was 0.03 (3% brightness) - WAY too dark!
    // Increased to 0.35 (35% ambient) so objects are visible even in shadow
    // This simulates global illumination / sky lighting
    let ambient = base_color * vec3<f32>(0.35, 0.38, 0.40);  // Slightly blue-tinted ambient
    
    return ambient + diffuse + specular;
}

// Apply normal map
fn apply_normal_map(
    world_normal: vec3<f32>,
    world_tangent: vec3<f32>,
    normal_sample: vec3<f32>,
) -> vec3<f32> {
    // Convert from tangent space [0,1] to [-1,1]
    let tangent_normal = normal_sample * 2.0 - 1.0;
    
    // Build TBN matrix
    let N = normalize(world_normal);
    let T = normalize(world_tangent - dot(world_tangent, N) * N);
    let B = cross(N, T);
    let TBN = mat3x3<f32>(T, B, N);
    
    return normalize(TBN * tangent_normal);
}

// Remap UV coordinates to atlas region based on material ID
fn remap_atlas_uv(base_uv: vec2<f32>, material_id: u32) -> vec2<f32> {
    // Get the atlas region for this material
    let region = atlas_regions[material_id];
    
    // CRITICAL FIX: Use fract() to wrap UVs to [0,1] range for tiling
    // Kenney models often have UVs > 1.0 for texture tiling
    // fract() gives us the fractional part (e.g., fract(2.3) = 0.3)
    let wrapped_uv = fract(base_uv);
    
    // Remap: atlas_uv = (wrapped_uv * scale) + offset
    // This maps the [0,1] UV space to the material's region in the atlas
    // CRITICAL: Scale THEN offset (order matters!)
    let scaled = wrapped_uv * region.uv_scale;
    let final_uv = scaled + region.uv_offset;
    
    // Clamp to region bounds to prevent bleeding into neighboring materials
    let min_uv = region.uv_offset;
    let max_uv = region.uv_offset + region.uv_scale;
    return clamp(final_uv, min_uv, max_uv);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Check if this is terrain: sum of blend weights > 0.01 means it's terrain
    // Non-terrain objects have blend = [0,0,0,0], terrain has blend weights summing to ~1.0
    let blend_sum = in.material_blend.x + in.material_blend.y + in.material_blend.z;
    let is_terrain = blend_sum > 0.01;
    
    var albedo: vec3<f32>;
    var uv_for_sampling: vec2<f32>;
    var normal_sample: vec3<f32>;
    var roughness: f32;
    var metallic: f32;
    var ao: f32;
    
    if is_terrain {
        // Terrain: Blend multiple materials from texture array
        // Layer 0 = grass, Layer 1 = dirt, Layer 2 = stone
        let terrain_uv = in.uv * 10.0;
        
        let grass_color = textureSample(terrain_albedo_array, terrain_sampler, terrain_uv, 0).rgb;
        let dirt_color = textureSample(terrain_albedo_array, terrain_sampler, terrain_uv, 1).rgb;
        let stone_color = textureSample(terrain_albedo_array, terrain_sampler, terrain_uv, 2).rgb;
        
        // Weighted blend based on material_blend weights
        albedo = grass_color * in.material_blend.x 
               + dirt_color * in.material_blend.y 
               + stone_color * in.material_blend.z;
        
        // TASK 2.3: Sample terrain-specific normals
        let grass_normal = textureSample(terrain_normal_array, terrain_sampler, terrain_uv, 0).rgb;
        let dirt_normal = textureSample(terrain_normal_array, terrain_sampler, terrain_uv, 1).rgb;
        let stone_normal = textureSample(terrain_normal_array, terrain_sampler, terrain_uv, 2).rgb;
        
        normal_sample = grass_normal * in.material_blend.x
                      + dirt_normal * in.material_blend.y
                      + stone_normal * in.material_blend.z;
        
        // TASK 2.3: Sample terrain-specific roughness (MRA packing)
        let grass_mra = textureSample(terrain_roughness_array, terrain_sampler, terrain_uv, 0);
        let dirt_mra = textureSample(terrain_roughness_array, terrain_sampler, terrain_uv, 1);
        let stone_mra = textureSample(terrain_roughness_array, terrain_sampler, terrain_uv, 2);
        
        let blended_mra = grass_mra * in.material_blend.x
                        + dirt_mra * in.material_blend.y
                        + stone_mra * in.material_blend.z;
        
        metallic = blended_mra.r;
        roughness = blended_mra.g;
        ao = blended_mra.b;
        
        // Terrain uses original UVs for normal/roughness
        uv_for_sampling = in.uv;
    } else {
        // Standard object: Use material atlas with remapped UVs
        let atlas_uv = remap_atlas_uv(in.uv, in.material_id);
        albedo = textureSample(albedo_texture, texture_sampler, atlas_uv).rgb;
        
        // Sample normal and roughness from atlas
        normal_sample = textureSample(normal_texture, texture_sampler, atlas_uv).rgb;
        
        // FIX: MRA packing (Metallic-Roughness-AO)
        let mra_sample = textureSample(roughness_texture, texture_sampler, atlas_uv);
        metallic = mra_sample.r;
        roughness = mra_sample.g;
        ao = mra_sample.b;
        
        // Use remapped UVs for all texture sampling
        uv_for_sampling = atlas_uv;
    }
    
    // Apply normal map
    let normal = apply_normal_map(in.world_normal, in.world_tangent, normal_sample);
    
    // Calculate lighting
    let view_dir = normalize(uniforms.camera_pos - in.world_position);
    let light_dir = normalize(vec3<f32>(0.3, 0.8, 0.4)); // Directional light
    
    let final_color = simple_pbr(albedo, normal, roughness, view_dir, light_dir);
    
    return vec4<f32>(final_color, 1.0);
}

// PHASE 4.3: Transparent fragment shader with alpha cutoff (for foliage/glass)
@fragment
fn fs_main_transparent(in: VertexOutput) -> @location(0) vec4<f32> {
    // Standard object: Use material atlas with remapped UVs
    let atlas_uv = remap_atlas_uv(in.uv, in.material_id);
    let albedo_sample = textureSample(albedo_texture, texture_sampler, atlas_uv);
    
    // Alpha test: discard fragments with alpha < 0.5
    if albedo_sample.a < 0.5 {
        discard;
    }
    
    let albedo = albedo_sample.rgb;
    
    // Sample normal and roughness from atlas
    let normal_sample = textureSample(normal_texture, texture_sampler, atlas_uv).rgb;
    
    // FIX: MRA packing (Metallic-Roughness-AO)
    let mra_sample = textureSample(roughness_texture, texture_sampler, atlas_uv);
    let roughness = mra_sample.g;
    
    // Apply normal map
    let normal = apply_normal_map(in.world_normal, in.world_tangent, normal_sample);
    
    // Calculate lighting
    let view_dir = normalize(uniforms.camera_pos - in.world_position);
    let light_dir = normalize(vec3<f32>(0.3, 0.8, 0.4)); // Directional light
    
    let final_color = simple_pbr(albedo, normal, roughness, view_dir, light_dir);
    
    return vec4<f32>(final_color, albedo_sample.a);
}

