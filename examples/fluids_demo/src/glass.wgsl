struct Uniforms {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(0) @binding(1)
var skybox_texture: texture_2d<f32>;

@group(0) @binding(2)
var skybox_sampler: sampler;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    output.clip_position = uniforms.view_proj * vec4<f32>(input.position, 1.0);
    output.world_pos = input.position;
    output.normal = input.normal;
    
    return output;
}

const PI: f32 = 3.14159265359;

fn direction_to_equirectangular_uv(dir: vec3<f32>) -> vec2<f32> {
    let normalized = normalize(dir);
    let u = atan2(normalized.z, normalized.x) / (2.0 * PI) + 0.5;
    let v = asin(normalized.y) / PI + 0.5;
    return vec2<f32>(u, v);
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    let normal = normalize(input.normal);
    
    // View direction approximation (toward camera - assume above)
    let view_dir = normalize(vec3<f32>(0.0, 1.0, 0.0));
    
    // Calculate reflection for skybox
    let reflection_dir = reflect(-view_dir, normal);
    let reflection_uv = direction_to_equirectangular_uv(reflection_dir);
    let skybox_color = textureSample(skybox_texture, skybox_sampler, reflection_uv).rgb;
    
    // Glass tint (slight cyan)
    let glass_tint = vec3<f32>(0.8, 0.95, 1.0);
    
    // Fresnel effect
    let fresnel = pow(1.0 - abs(dot(normal, view_dir)), 2.0);
    
    // Mix glass tint with skybox reflection
    let final_color = mix(glass_tint * 0.3, skybox_color * glass_tint, fresnel * 0.5);
    
    // Glass alpha (high transparency with subtle fresnel)
    let alpha = 0.02 + fresnel * 0.2;
    
    return vec4<f32>(final_color, alpha);
}
