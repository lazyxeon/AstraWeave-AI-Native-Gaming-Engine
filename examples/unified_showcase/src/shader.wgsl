// Clean PBR Shader with Shadow Mapping

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

struct ModelUniforms {
    model: mat4x4<f32>,
}

struct LightUniforms {
    view_proj: mat4x4<f32>, // Light's view-projection matrix
    position: vec3<f32>,
    color: vec3<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniforms;

@group(1) @binding(0)
var albedo_texture: texture_2d<f32>;

@group(1) @binding(1)
var texture_sampler: sampler;

@group(1) @binding(2)
var normal_texture: texture_2d<f32>;

@group(1) @binding(3)
var roughness_texture: texture_2d<f32>;

@group(2) @binding(0)
var<uniform> model: ModelUniforms;

@group(3) @binding(0)
var<uniform> light: LightUniforms;

@group(3) @binding(1)
var shadow_texture: texture_depth_2d;

@group(3) @binding(2)
var shadow_sampler: sampler_comparison;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) tangent: vec4<f32>,
    @location(5) shadow_pos: vec4<f32>, // Position in light space
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    
    let normal_matrix = mat3x3<f32>(
        model.model[0].xyz,
        model.model[1].xyz,
        model.model[2].xyz
    );
    out.world_normal = normalize(normal_matrix * in.normal);
    out.uv = in.uv;
    out.color = in.color;
    
    let tangent_ws = normalize(normal_matrix * in.tangent.xyz);
    out.tangent = vec4<f32>(tangent_ws, in.tangent.w);

    // Transform world position to light space
    // Bias matrix to map [-1, 1] to [0, 1] is handled in shadow calculation or matrix
    // Here we just pass the projected coordinate
    out.shadow_pos = light.view_proj * world_pos;
    
    return out;
}

// Shadow pass vertex shader
@vertex
fn vs_shadow(in: VertexInput) -> @builtin(position) vec4<f32> {
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    return light.view_proj * world_pos;
}

fn fetch_shadow(shadow_pos: vec4<f32>, normal: vec3<f32>, light_dir: vec3<f32>) -> f32 {
    if (shadow_pos.w <= 0.0) {
        return 1.0;
    }
    
    // Perspective divide
    let proj_coords = shadow_pos.xyz / shadow_pos.w;
    
    // Transform to [0, 1] range (flip Y for wgpu/vulkan)
    let flip_correction = vec2<f32>(0.5, -0.5);
    let uv = proj_coords.xy * flip_correction + vec2<f32>(0.5, 0.5);
    
    // Check if outside shadow map
    if (uv.x < 0.0 || uv.x > 1.0 || uv.y < 0.0 || uv.y > 1.0 || proj_coords.z < 0.0 || proj_coords.z > 1.0) {
        return 1.0;
    }
    
    let current_depth = proj_coords.z;
    
    // Bias to prevent shadow acne
    let bias = max(0.005 * (1.0 - dot(normal, light_dir)), 0.0005);
    
    // PCF (Percentage Closer Filtering)
    var shadow = 0.0;
    let size = textureDimensions(shadow_texture);
    let texel_size = vec2<f32>(1.0 / f32(size.x), 1.0 / f32(size.y));
    
    for (var x = -1; x <= 1; x++) {
        for (var y = -1; y <= 1; y++) {
            let pcf_depth = textureSampleCompare(
                shadow_texture,
                shadow_sampler,
                uv + vec2<f32>(f32(x), f32(y)) * texel_size,
                current_depth - bias
            );
            shadow += pcf_depth;
        }
    }
    shadow /= 9.0;
    
    return shadow;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texture_color = textureSample(albedo_texture, texture_sampler, in.uv);
    if (texture_color.a < 0.5) {
        discard;
    }
    let albedo = in.color.rgb * texture_color.rgb;
    
    let normal_sample = textureSample(normal_texture, texture_sampler, in.uv).rgb;
    // Roughness is in Green channel for glTF MRA workflow (Red=Occlusion, Green=Roughness, Blue=Metalness)
    let roughness_sample = textureSample(roughness_texture, texture_sampler, in.uv).g;
    
    // Normal mapping
    let n_sample = normal_sample * 2.0 - vec3<f32>(1.0, 1.0, 1.0);
    let tangent_vec = normalize(in.tangent.xyz);
    let handedness = in.tangent.w;
    let bitangent = cross(in.world_normal, tangent_vec) * handedness;
    let TBN = mat3x3<f32>(tangent_vec, bitangent, in.world_normal);
    let mapped_normal = normalize(TBN * n_sample);
    
    let view_dir = normalize(camera.camera_pos - in.world_position);
    
    // Directional light (sun)
    // light.position is the sun's position, so direction TO light is normalize(light.position)
    let light_dir = normalize(light.position); 
    
    // Diffuse
    let diffuse_strength = max(dot(mapped_normal, light_dir), 0.0);
    
    // Specular
    let halfway = normalize(light_dir + view_dir);
    let specular_strength = pow(max(dot(mapped_normal, halfway), 0.0), 32.0);
    
    // Shadow
    let shadow = fetch_shadow(in.shadow_pos, mapped_normal, light_dir);
    
    // Ambient
    let ambient = vec3<f32>(0.3, 0.3, 0.4) * albedo; // Blue-ish ambient
    
    // Combine
    let lighting = (ambient + (diffuse_strength + specular_strength * 0.5) * shadow * light.color) * albedo;
    
    // Tone mapping (Reinhard)
    let mapped = lighting / (lighting + vec3<f32>(1.0));
    // Gamma correction
    let gamma = pow(mapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma, in.color.a);
}

// ========================================================================
// SKYBOX SHADER
// ========================================================================

struct SkyOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
};

@group(1) @binding(0)
var sky_texture: texture_2d<f32>;
@group(1) @binding(1)
var sky_sampler: sampler;

@vertex
fn vs_sky(in: VertexInput) -> SkyOutput {
    var out: SkyOutput;
    // Center the skybox on the camera
    let world_pos = in.position * 500.0 + camera.camera_pos;
    out.clip_position = camera.view_proj * vec4<f32>(world_pos, 1.0);
    // Force z to be at the far plane (1.0)
    out.clip_position.z = out.clip_position.w; 
    out.world_pos = in.position;
    return out;
}

@fragment
fn fs_sky(in: SkyOutput) -> @location(0) vec4<f32> {
    let dir = normalize(in.world_pos);
    // Equirectangular mapping
    // atan2(z, x) gives angle in [-PI, PI]
    // We map to [0, 1]
    let u = 0.5 + atan2(dir.z, dir.x) / (2.0 * 3.14159265);
    let v = 0.5 - asin(dir.y) / 3.14159265;
    
    let color = textureSample(sky_texture, sky_sampler, vec2<f32>(u, v));
    
    // Apply simple tone mapping to match the scene
    let mapped = color.rgb / (color.rgb + vec3<f32>(1.0));
    let gamma = pow(mapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma, 1.0);
}
