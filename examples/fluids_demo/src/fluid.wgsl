struct CameraUniform {
    view_proj: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

@group(0) @binding(1)
var skybox_texture: texture_2d<f32>;

@group(0) @binding(2)
var skybox_sampler: sampler;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) world_pos: vec3<f32>,
}

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    // Generate quad vertices from vertex index (0-3)
    let vertex_id = in.vertex_index % 4u;
    var quad_offset = vec2<f32>(0.0, 0.0);
    
    if (vertex_id == 0u) {
        quad_offset = vec2<f32>(-1.0, -1.0);
        out.uv = vec2<f32>(0.0, 0.0);
    } else if (vertex_id == 1u) {
        quad_offset = vec2<f32>(1.0, -1.0);
        out.uv = vec2<f32>(1.0, 0.0);
    } else if (vertex_id == 2u) {
        quad_offset = vec2<f32>(-1.0, 1.0);
        out.uv = vec2<f32>(0.0, 1.0);
    } else {
        quad_offset = vec2<f32>(1.0, 1.0);
        out.uv = vec2<f32>(1.0, 1.0);
    }
    
    // Billboard size
    let particle_size = 0.8;
    
    // World position of particle
    out.world_pos = in.position.xyz;
    
    // Create billboard in view space (use xyz from position)
    let view_pos = camera.view_proj * vec4<f32>(in.position.xyz, 1.0);
    let billboard_pos = view_pos.xy + quad_offset * particle_size;
    
    out.clip_position = vec4<f32>(billboard_pos, view_pos.z, view_pos.w);
    
    return out;
}

const PI: f32 = 3.14159265359;

fn direction_to_equirectangular_uv(dir: vec3<f32>) -> vec2<f32> {
    let normalized = normalize(dir);
    let u = atan2(normalized.z, normalized.x) / (2.0 * PI) + 0.5;
    let v = asin(normalized.y) / PI + 0.5;
    return vec2<f32>(u, v);
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Create billboarded sphere with lighting
    let center = vec2<f32>(0.5, 0.5);
    let uv_centered = in.uv - center;
    let dist_sq = dot(uv_centered, uv_centered);
    let radius_sq = 0.25; // radius = 0.5
    
    // Discard fragments outside the sphere
    if (dist_sq > radius_sq) {
        discard;
    }
    
    // Compute normal for the sphere fragment
    let z = sqrt(radius_sq - dist_sq);
    let normal = normalize(vec3<f32>(uv_centered.x, uv_centered.y, z));
    
    // Light direction (hardcoded directional light)
    let light_dir = normalize(vec3<f32>(0.5, 0.8, 0.6));
    
    // View direction approximation (toward camera from world pos)
    // For particles, we assume camera is above looking down
    let view_dir = normalize(vec3<f32>(0.0, 1.0, 0.0));
    
    // Reflection direction for skybox sampling
    let reflection_dir = reflect(-view_dir, normal);
    let reflection_uv = direction_to_equirectangular_uv(reflection_dir);
    let skybox_color = textureSample(skybox_texture, skybox_sampler, reflection_uv).rgb;
    
    // Diffuse lighting
    let diffuse = max(dot(normal, light_dir), 0.0);
    
    // Specular (Blinn-Phong)
    let half_dir = normalize(light_dir + view_dir);
    let specular = pow(max(dot(normal, half_dir), 0.0), 128.0);
    
    // Water base color (cyan-blue)
    let base_color = vec3<f32>(0.1, 0.3, 0.6);
    
    // Fresnel approximation
    let fresnel = pow(1.0 - max(dot(normal, view_dir), 0.0), 3.0);
    
    // Mix base color with skybox reflection based on fresnel
    let reflection_color = mix(base_color, skybox_color, fresnel * 0.7);
    
    // Combine lighting
    let ambient = 0.2;
    let final_color = reflection_color * (ambient + diffuse * 0.5) + vec3<f32>(1.0, 1.0, 1.0) * specular * 0.8;
    
    // Alpha falloff based on distance from center
    let dist = sqrt(dist_sq);
    let alpha = 1.0 - smoothstep(0.3, 0.5, dist);
    
    if (alpha < 0.01) {
        discard;
    }
    
    return vec4<f32>(final_color, alpha * 0.85);
}
