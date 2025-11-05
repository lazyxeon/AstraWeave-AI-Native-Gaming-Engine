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

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) world_tangent: vec3<f32>,
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
    let light_color = vec3<f32>(1.0, 0.95, 0.9) * 1.5;
    
    // Lambertian diffuse
    let ndotl = max(dot(normal, light_dir), 0.0);
    let diffuse = base_color * light_color * ndotl;
    
    // Blinn-Phong specular (simplified for now)
    let half_dir = normalize(view_dir + light_dir);
    let ndoth = max(dot(normal, half_dir), 0.0);
    let shininess = mix(256.0, 16.0, roughness);
    let specular = pow(ndoth, shininess) * light_color * (1.0 - roughness) * 0.5;
    
    // Ambient (simplified IBL replacement)
    let ambient = base_color * vec3<f32>(0.03, 0.03, 0.04);
    
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

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample textures
    let albedo = textureSample(albedo_texture, texture_sampler, in.uv).rgb;
    let normal_sample = textureSample(normal_texture, texture_sampler, in.uv).rgb;
    let roughness = textureSample(roughness_texture, texture_sampler, in.uv).r;
    
    // Apply normal map
    let normal = apply_normal_map(in.world_normal, in.world_tangent, normal_sample);
    
    // Calculate lighting
    let view_dir = normalize(uniforms.camera_pos - in.world_position);
    let light_dir = normalize(vec3<f32>(0.3, 0.8, 0.4)); // Directional light
    
    let final_color = simple_pbr(albedo, normal, roughness, view_dir, light_dir);
    
    return vec4<f32>(final_color, 1.0);
}
