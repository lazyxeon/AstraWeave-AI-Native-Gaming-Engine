// Skybox Shader

struct CameraUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
}

@group(0) @binding(0) var<uniform> camera: CameraUniforms;
@group(1) @binding(0) var t_sky: texture_2d<f32>;
@group(1) @binding(1) var s_sky: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
    @location(4) tangent: vec4<f32>,
}

struct SkyboxOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
}

@vertex
fn vs_skybox(in: VertexInput) -> SkyboxOutput {
    var out: SkyboxOutput;
    out.world_pos = in.position;
    
    // Remove translation from view matrix for skybox
    let view_rot = mat4x4<f32>(
        camera.view_proj[0],
        camera.view_proj[1],
        camera.view_proj[2],
        vec4<f32>(0.0, 0.0, 0.0, 1.0)
    );
    
    // Apply view-projection without translation so skybox follows camera rotation
    out.clip_position = view_rot * vec4<f32>(in.position, 1.0);
    
    // Equirectangular mapping
    let n = normalize(in.position);
    let u = 0.5 + atan2(n.z, n.x) / (2.0 * 3.14159);
    let v = 0.5 - asin(n.y) / 3.14159;
    
    out.uv = vec2<f32>(u, v);
    return out;
}

@fragment
fn fs_skybox(in: SkyboxOutput) -> @location(0) vec4<f32> {
    // Equirectangular mapping using world position
    let n = normalize(in.world_pos);
    let u = 0.5 + atan2(n.z, n.x) / (2.0 * 3.14159);
    let v = 0.5 - asin(n.y) / 3.14159;
    
    let color = textureSample(t_sky, s_sky, vec2<f32>(u, v)).rgb;
    // No lighting on skybox
    return vec4<f32>(color, 1.0);
}
