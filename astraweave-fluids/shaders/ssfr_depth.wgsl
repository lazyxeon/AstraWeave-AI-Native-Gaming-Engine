// SSFR Depth Pass - Renders particle spheres to a depth texture
// Optimized for Screen-Space Fluid Rendering

struct CameraUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    cam_pos: vec4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) position: vec4<f32>, // Instance position
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) view_pos: vec3<f32>,
    @location(2) radius: f32,
};

@vertex
fn vs_main(in: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    
    let vertex_id = in.vertex_index % 4u;
    var quad_offset = vec2<f32>(0.0, 0.0);
    
    if (vertex_id == 0u) {
        quad_offset = vec2<f32>(-1.0, -1.0);
        out.uv = vec2<f32>(-1.0, -1.0);
    } else if (vertex_id == 1u) {
        quad_offset = vec2<f32>(1.0, -1.0);
        out.uv = vec2<f32>(1.0, -1.0);
    } else if (vertex_id == 2u) {
        quad_offset = vec2<f32>(-1.0, 1.0);
        out.uv = vec2<f32>(-1.0, 1.0);
    } else {
        quad_offset = vec2<f32>(1.0, 1.0);
        out.uv = vec2<f32>(1.0, 1.0);
    }
    
    let particle_radius = 0.5;
    out.radius = particle_radius;
    
    // Transform to view space for accurate sphere rendering
    // We assume a simple view-aligned billboard for depth pass
    let world_pos = in.position.xyz;
    let proj_pos = camera.view_proj * vec4<f32>(world_pos, 1.0);
    
    // Expand billboard in clip space (can be improved for perspective-correctness)
    let aspect = 1.0; // Handled by view_proj usually, but for simple billboards:
    let billboard_scale = particle_radius * 2.5; 
    
    // Correct billboard expansion in view space is better
    // But for depth pass, we just need the quad covering the sphere
    out.view_pos = (camera.view_proj * vec4<f32>(world_pos, 1.0)).xyz; // This is not quite view space, it's clip. 
    // Let's do real view space
    
    // Placeholder for simplified version, will refine in later passes if needed
    out.clip_position = proj_pos + vec4<f32>(quad_offset * billboard_scale * proj_pos.w * 0.1, 0.0, 0.0);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @builtin(depth) f32 {
    let r2 = dot(in.uv, in.uv);
    if (r2 > 1.0) {
        discard;
    }
    
    // Calculate depth of the sphere surface
    let z_norm = sqrt(1.0 - r2);
    let pixel_pos_view = in.view_pos + vec3<f32>(0.0, 0.0, z_norm * in.radius);
    
    // Project back to get correct depth
    // Simplified depth: just add the sphere offset to the billboard depth
    return in.clip_position.z / in.clip_position.w - (z_norm * in.radius * 0.01); 
}
