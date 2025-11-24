struct Uniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec3<f32>,
    _pad: f32,
}

struct VertexInput {
    @location(0) position: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: Uniforms;
@group(0) @binding(1) var skybox_texture: texture_2d<f32>;
@group(0) @binding(2) var skybox_sampler: sampler;

const PI: f32 = 3.14159265359;

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    // Center skybox at camera position so it moves with the camera
    let centered_pos = input.position + uniforms.camera_pos;
    output.clip_position = uniforms.view_proj * vec4<f32>(centered_pos, 1.0);
    output.world_pos = input.position;
    return output;
}

fn direction_to_equirectangular_uv(dir: vec3<f32>) -> vec2<f32> {
    let normalized = normalize(dir);
    let u = atan2(normalized.z, normalized.x) / (2.0 * PI) + 0.5;
    let v = asin(normalized.y) / PI + 0.5;
    return vec2<f32>(u, 1.0 - v);
}

fn reinhard_tonemap(hdr: vec3<f32>) -> vec3<f32> {
    return hdr / (hdr + vec3<f32>(1.0));
}

fn aces_tonemap(hdr: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((hdr * (a * hdr + b)) / (hdr * (c * hdr + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let dir = normalize(input.world_pos);
    let uv = direction_to_equirectangular_uv(dir);
    
    let hdr_color = textureSample(skybox_texture, skybox_sampler, uv).rgb;
    
    // Apply ACES tonemapping
    let tonemapped = aces_tonemap(hdr_color * 1.2);
    
    return vec4<f32>(tonemapped, 1.0);
}
