// Shader V2 - Clean PBR + Skybox

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
@group(2) @binding(0) var t_diffuse: texture_2d<f32>;
@group(2) @binding(1) var s_diffuse: sampler;
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
    @location(0) world_normal: vec3<f32>,
    @location(1) world_position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    
    let normal_matrix = mat3x3<f32>(model.model[0].xyz, model.model[1].xyz, model.model[2].xyz);
    out.world_normal = normalize(normal_matrix * in.normal);
    
    out.uv = in.uv;
    out.color = in.color;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let object_color = textureSample(t_diffuse, s_diffuse, in.uv) * in.color;
    
    let ambient_strength = 0.5; // Increased ambient for better visibility
    let ambient = ambient_strength * light.color;
    
    let norm = normalize(in.world_normal);
    let light_dir = normalize(light.position - in.world_position);
    let diff = max(dot(norm, light_dir), 0.0);
    let diffuse = diff * light.color;
    
    let view_dir = normalize(camera.camera_pos - in.world_position);
    let reflect_dir = reflect(-light_dir, norm);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    let specular = 0.5 * spec * light.color;
    
    let result = (ambient + diffuse + specular) * object_color.rgb;
    
    // Tone mapping
    let mapped = result / (result + vec3<f32>(1.0));
    let gamma = pow(mapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma, object_color.a);
}

// Skybox Shader
@group(1) @binding(0) var t_sky: texture_2d<f32>;
@group(1) @binding(1) var s_sky: sampler;

struct SkyboxOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs_skybox(in: VertexInput) -> SkyboxOutput {
    var out: SkyboxOutput;
    // Remove translation from view matrix for skybox
    let view_rot = mat4x4<f32>(
        camera.view_proj[0],
        camera.view_proj[1],
        camera.view_proj[2],
        vec4<f32>(0.0, 0.0, 0.0, 1.0) // Reset translation
    );
    
    // We use the position directly as it's a sphere centered at 0,0,0
    // But we need to apply the camera projection and rotation
    // Actually, simpler: Just render the sphere at camera position
    let world_pos = vec4<f32>(in.position + camera.camera_pos, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    
    // Equirectangular mapping
    // Normal is inverted in mesh generation, so we use it to find UV
    let n = normalize(in.position);
    let u = 0.5 + atan2(n.z, n.x) / (2.0 * 3.14159);
    let v = 0.5 - asin(n.y) / 3.14159;
    
    out.uv = vec2<f32>(u, v);
    return out;
}

@fragment
fn fs_skybox(in: SkyboxOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_sky, s_sky, in.uv).rgb;
    // No lighting on skybox
    return vec4<f32>(color, 1.0);
}
