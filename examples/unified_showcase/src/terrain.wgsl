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
@group(1) @binding(1) var t_shadow: texture_depth_2d;
@group(1) @binding(2) var s_shadow: sampler_comparison;
@group(2) @binding(0) var t_grass_diff: texture_2d<f32>;
@group(2) @binding(1) var t_grass_norm: texture_2d<f32>;
@group(2) @binding(2) var t_grass_rough: texture_2d<f32>;
@group(2) @binding(3) var t_rock_diff: texture_2d<f32>;
@group(2) @binding(4) var t_rock_norm: texture_2d<f32>;
@group(2) @binding(5) var t_rock_rough: texture_2d<f32>;
@group(2) @binding(6) var s_terrain: sampler;
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
    @location(4) light_space_pos: vec4<f32>,
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
    out.light_space_pos = light.view_proj * world_pos;
    return out;
}

// Triplanar sampling helper
fn triplanar_sample(pos: vec3<f32>, normal: vec3<f32>, tex: texture_2d<f32>, s: sampler) -> vec4<f32> {
    let blend = abs(normal);
    let blend_normalized = blend / (blend.x + blend.y + blend.z);
    let uv_scale = 0.5;
    
    let sample_x = textureSample(tex, s, pos.yz * uv_scale);
    let sample_y = textureSample(tex, s, pos.xz * uv_scale);
    let sample_z = textureSample(tex, s, pos.xy * uv_scale);
    
    return sample_x * blend_normalized.x + 
           sample_y * blend_normalized.y + 
           sample_z * blend_normalized.z;
}

// Triplanar normal mapping (simplified blending approach)
fn triplanar_normal(pos: vec3<f32>, normal: vec3<f32>, tex: texture_2d<f32>, s: sampler) -> vec3<f32> {
    let blend = abs(normal);
    let blend_normalized = blend / (blend.x + blend.y + blend.z);
    let uv_scale = 0.5;
    
    // Sample normal maps
    let sample_x = textureSample(tex, s, pos.yz * uv_scale).xyz * 2.0 - 1.0;
    let sample_y = textureSample(tex, s, pos.xz * uv_scale).xyz * 2.0 - 1.0;
    let sample_z = textureSample(tex, s, pos.xy * uv_scale).xyz * 2.0 - 1.0;
    
    // Blend tangent-space normals
    let blended = sample_x * blend_normalized.x + 
                  sample_y * blend_normalized.y + 
                  sample_z * blend_normalized.z;
    
    // Simple approach: blend with world normal
    return normalize(normal + blended * 0.5);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Slope detection for grass vs rock blending
    let slope = abs(in.world_normal.y);
    let grass_weight = smoothstep(0.6, 0.8, slope); // Flat areas = grass
    
    // Sample diffuse textures using triplanar
    let grass_color = triplanar_sample(in.world_position, in.world_normal, t_grass_diff, s_terrain);
    let rock_color = triplanar_sample(in.world_position, in.world_normal, t_rock_diff, s_terrain);
    let base_color = mix(rock_color, grass_color, grass_weight) * in.color;
    
    // Sample normal maps using triplanar
    let grass_normal = triplanar_normal(in.world_position, in.world_normal, t_grass_norm, s_terrain);
    let rock_normal = triplanar_normal(in.world_position, in.world_normal, t_rock_norm, s_terrain);
    let final_normal = normalize(mix(rock_normal, grass_normal, grass_weight));
    
    // Sample roughness textures using triplanar
    let grass_roughness = triplanar_sample(in.world_position, in.world_normal, t_grass_rough, s_terrain).r;
    let rock_roughness = triplanar_sample(in.world_position, in.world_normal, t_rock_rough, s_terrain).r;
    let roughness = mix(rock_roughness, grass_roughness, grass_weight);
    
    // Shadow calculation
    let shadow_coords = in.light_space_pos.xyz / in.light_space_pos.w;
    let shadow_uv = shadow_coords.xy * 0.5 + 0.5;
    let shadow_uv_flipped = vec2<f32>(shadow_uv.x, 1.0 - shadow_uv.y);
    var shadow = 0.0;
    if shadow_uv_flipped.x >= 0.0 && shadow_uv_flipped.x <= 1.0 && 
       shadow_uv_flipped.y >= 0.0 && shadow_uv_flipped.y <= 1.0 &&
       shadow_coords.z >= 0.0 && shadow_coords.z <= 1.0 {
        shadow = textureSampleCompare(t_shadow, s_shadow, shadow_uv_flipped, shadow_coords.z);
    } else {
        shadow = 1.0;
    }
    
    // Lighting calculations with normal and roughness
    let ambient_strength = 0.5;
    let ambient = ambient_strength * light.color;
    
    let light_dir = normalize(light.position - in.world_position);
    let diff = max(dot(final_normal, light_dir), 0.0);
    let diffuse = diff * light.color * shadow;
    
    let view_dir = normalize(camera.camera_pos - in.world_position);
    let reflect_dir = reflect(-light_dir, final_normal);
    
    // Use roughness to modulate specular
    let shininess = mix(64.0, 8.0, roughness); // Lower shininess for rough surfaces
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), shininess);
    let specular_strength = mix(0.5, 0.1, roughness); // Lower strength for rough surfaces
    let specular = specular_strength * spec * light.color * shadow;
    
    let result = (ambient + diffuse + specular) * base_color.rgb;
    
    // Tone mapping
    let mapped = result / (result + vec3<f32>(1.0));
    let gamma = pow(mapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma, base_color.a);
}
