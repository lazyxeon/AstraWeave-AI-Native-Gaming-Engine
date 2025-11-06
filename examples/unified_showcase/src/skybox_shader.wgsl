// Skybox vertex shader
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_position: vec3<f32>,
}

struct SkyboxUniforms {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: SkyboxUniforms;

@group(1) @binding(0)
var skybox_texture: texture_cube<f32>;

@group(1) @binding(1)
var skybox_sampler: sampler;

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var output: VertexOutput;
    
    // Generate full-screen triangle from vertex index
    // Triangle covers: (-1,-1) to (3,3) to (-1,3)
    let x = f32((vertex_index << 1u) & 2u);
    let y = f32(vertex_index & 2u);
    let position = vec2<f32>(x * 2.0 - 1.0, y * 2.0 - 1.0);
    
    // Render at far plane (0.999999 depth)
    output.clip_position = vec4<f32>(position, 0.999999, 1.0);
    
    // Reconstruct world direction from screen position
    let ndc = vec4<f32>(position, 1.0, 1.0);
    let world_pos = uniforms.inv_view_proj * ndc;
    output.world_position = world_pos.xyz / world_pos.w;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Sample cubemap using world direction
    let direction = normalize(input.world_position);
    let color = textureSample(skybox_texture, skybox_sampler, direction);
    
    // Simple tonemapping to prevent blown-out HDR values
    let tonemapped = color.rgb / (color.rgb + vec3<f32>(1.0));
    let gamma_corrected = pow(tonemapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma_corrected, 1.0);
}
