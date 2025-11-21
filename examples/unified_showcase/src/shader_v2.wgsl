// Shader V2 - Clean PBR + Skybox

// Helper function: 3x3 matrix inverse
fn inverse_mat3(m: mat3x3<f32>) -> mat3x3<f32> {
    let a00 = m[0][0]; let a01 = m[0][1]; let a02 = m[0][2];
    let a10 = m[1][0]; let a11 = m[1][1]; let a12 = m[1][2];
    let a20 = m[2][0]; let a21 = m[2][1]; let a22 = m[2][2];
    
    let b01 = a22 * a11 - a12 * a21;
    let b11 = -a22 * a10 + a12 * a20;
    let b21 = a21 * a10 - a11 * a20;
    
    let det = a00 * b01 + a01 * b11 + a02 * b21;
    
    return mat3x3<f32>(
        vec3<f32>(b01, (-a22 * a01 + a02 * a21), (a12 * a01 - a02 * a11)) / det,
        vec3<f32>(b11, (a22 * a00 - a02 * a20), (-a12 * a00 + a02 * a10)) / det,
        vec3<f32>(b21, (-a21 * a00 + a01 * a20), (a11 * a00 - a01 * a10)) / det
    );
}

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
    @location(4) light_space_pos: vec4<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    let world_pos = model.model * vec4<f32>(in.position, 1.0);
    out.clip_position = camera.view_proj * world_pos;
    out.world_position = world_pos.xyz;
    
    let model_mat3 = mat3x3<f32>(model.model[0].xyz, model.model[1].xyz, model.model[2].xyz);
    let normal_matrix = transpose(inverse_mat3(model_mat3));
    out.world_normal = normalize(normal_matrix * in.normal);
    
    out.uv = in.uv;
    out.color = in.color;
    out.light_space_pos = light.view_proj * world_pos;
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Sample texture - DON'T multiply by vertex color as Kenney models have baked beige colors
    let texture_color = textureSample(t_diffuse, s_diffuse, in.uv);
    
    // Use vertex color only for alpha blending, not color tinting
    // If vertex color is close to white, use texture as-is
    // Otherwise, blend texture with vertex color (for models that intentionally use vertex coloring)
    let vertex_brightness = (in.color.r + in.color.g + in.color.b) / 3.0;
    let is_white_vertex = vertex_brightness > 0.95;
    
    let object_color = select(
        texture_color * vec4<f32>(in.color.rgb, 1.0),  // Tinted by vertex color
        texture_color,                                   // Use texture as-is
        is_white_vertex || true  // ALWAYS use texture, ignoring vertex colors
    );
    
    // Detect water (translucent materials with alpha < 1.0)
    var norm = normalize(in.world_normal);
    if object_color.a < 1.0 {
        // Procedural water waves for normal perturbation
        let wave1 = sin(in.world_position.x * 0.5 + in.world_position.z * 0.3);
        let wave2 = cos(in.world_position.x * 0.3 + in.world_position.z * 0.5);
        let wave3 = sin(in.world_position.x * 0.7 - in.world_position.z * 0.4);
        let wave_offset = vec3<f32>(wave1 * 0.15, 0.0, wave2 * 0.15 + wave3 * 0.1);
        norm = normalize(norm + wave_offset);
    }
    
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
    
    let ambient_strength = 0.5; // Increased ambient for better visibility
    let ambient = ambient_strength * light.color;
    
    let light_dir = normalize(light.position - in.world_position);
    let diff = max(dot(norm, light_dir), 0.0);
    let diffuse = diff * light.color * shadow;
    
    let view_dir = normalize(camera.camera_pos - in.world_position);
    let reflect_dir = reflect(-light_dir, norm);
    let spec = pow(max(dot(view_dir, reflect_dir), 0.0), 32.0);
    var specular = 0.5 * spec * light.color * shadow;
    
    // Enhanced specular for water
    if object_color.a < 1.0 {
        let spec_water = pow(max(dot(view_dir, reflect_dir), 0.0), 64.0);
        specular = 1.5 * spec_water * light.color * shadow;
    }
    
    let result = (ambient + diffuse + specular) * object_color.rgb;
    
    // Tone mapping
    let mapped = result / (result + vec3<f32>(1.0));
    let gamma = pow(mapped, vec3<f32>(1.0 / 2.2));
    
    return vec4<f32>(gamma, object_color.a);
}
