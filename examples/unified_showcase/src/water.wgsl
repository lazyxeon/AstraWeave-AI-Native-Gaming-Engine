// Water Shader
struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

struct LightUniforms {
    view_proj: mat4x4<f32>,
    position: vec3<f32>,
    color: vec3<f32>,
}

struct ModelUniforms {
    model: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(1) @binding(0) var<uniform> light: LightUniforms;
@group(2) @binding(0) var t_sky: texture_2d<f32>;
@group(2) @binding(1) var s_sky: sampler;
@group(3) @binding(0) var<uniform> model: ModelUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) tangent: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    out.world_normal = in.normal; // Plane normal usually up
    out.uv = in.uv;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(in.world_normal);
    let view_dir = normalize(camera.camera_pos - in.world_position);
    
    // Basic Fresnel effect
    let fresnel = pow(1.0 - max(dot(view_dir, normal), 0.0), 3.0);
    
    // Reflection: sample skybox
    // Calculate reflection vector
    let reflect_dir = reflect(-view_dir, normal);
    
    // Equirectangular mapping for reflection
    let n = normalize(reflect_dir);
    let u = 0.5 + atan2(n.z, n.x) / (2.0 * 3.14159);
    let v = 0.5 - asin(n.y) / 3.14159;
    let reflection_color = textureSample(t_sky, s_sky, vec2<f32>(u, v)).rgb;
    
    // Water color
    let base_color = vec3<f32>(0.1, 0.3, 0.5);
    
    // Mix base and reflection based on fresnel
    let final_color = mix(base_color, reflection_color, fresnel * 0.8 + 0.2);
    
    // Specular highlight
    let light_dir = normalize(light.position - in.world_position);
    let half_dir = normalize(light_dir + view_dir);
    let spec = pow(max(dot(normal, half_dir), 0.0), 128.0);
    let specular = light.color * spec;
    
    return vec4<f32>(final_color + specular, 0.8); // 0.8 Alpha
}
