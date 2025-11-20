// Terrain Shader with Triplanar Mapping

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
@group(2) @binding(0) var t_grass: texture_2d<f32>;
@group(2) @binding(1) var t_rock: texture_2d<f32>;
@group(2) @binding(2) var s_sampler: sampler;
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

// Triplanar mapping function
fn triplanar_sample(pos: vec3<f32>, normal: vec3<f32>, tex_grass: texture_2d<f32>, tex_rock: texture_2d<f32>, s: sampler) -> vec4<f32> {
    // Blend weights based on world normal
    let blend = abs(normal);
    let blend_normalized = blend / (blend.x + blend.y + blend.z);
    
    // UV scale for triplanar - explicit tiling factor multiplied with world position
    let uv_scale = 0.5; // Tiling factor: smaller = more tiling/repeats
    
    // Slope detection: use Y component of normal to determine grass vs rock
    let slope = abs(normal.y);
    let grass_weight = smoothstep(0.6, 0.8, slope); // Flat areas = grass
    
    // Sample each axis for grass
    let sample_x_grass = textureSample(tex_grass, s, pos.yz * uv_scale);
    let sample_y_grass = textureSample(tex_grass, s, pos.xz * uv_scale);
    let sample_z_grass = textureSample(tex_grass, s, pos.xy * uv_scale);
    let grass_color = sample_x_grass * blend_normalized.x + 
                      sample_y_grass * blend_normalized.y + 
                      sample_z_grass * blend_normalized.z;
    
    // Sample each axis for rock
    let sample_x_rock = textureSample(tex_rock, s, pos.yz * uv_scale);
    let sample_y_rock = textureSample(tex_rock, s, pos.xz * uv_scale);
    let sample_z_rock = textureSample(tex_rock, s, pos.xy * uv_scale);
    let rock_color = sample_x_rock * blend_normalized.x + 
                     sample_y_rock * blend_normalized.y + 
                     sample_z_rock * blend_normalized.z;
    
    // Blend between grass and rock based on slope
    return mix(rock_color, grass_color, grass_weight);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Triplanar sample
    let object_color = triplanar_sample(in.world_position, in.world_normal, t_grass, t_rock, s_sampler) * in.color;
    
    // Lighting (from shader_v2.wgsl)
    let ambient_strength = 0.5;
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
