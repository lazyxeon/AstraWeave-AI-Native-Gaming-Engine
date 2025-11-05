// Infinite Grid Shader
//
// Renders an infinite grid overlay on the ground plane using a screen-space technique.
// No vertex buffers needed - renders a fullscreen quad and computes grid in fragment shader.
//
// Features:
// - Infinite grid (no visible edges)
// - Distance-based fading (prevents aliasing at horizon)
// - Major/minor grid lines (1m minor, 10m major)
// - XZ axes highlighted (red X, blue Z)
//
// Performance: ~0.5ms for 1080p

// Uniforms (camera matrices + grid settings)
struct GridUniforms {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,  // For unprojecting screen pos
    camera_pos: vec3<f32>,
    _padding1: f32,
    grid_size: f32,              // Grid spacing (meters)
    major_grid_every: f32,       // Major grid every N lines
    fade_distance: f32,          // Start fading at this distance
    max_distance: f32,           // Completely fade by this distance
    grid_color: vec4<f32>,       // Base grid color (RGBA)
    major_grid_color: vec4<f32>, // Major grid color (RGBA)
    x_axis_color: vec4<f32>,     // X axis color (red)
    z_axis_color: vec4<f32>,     // Z axis color (blue)
};

@group(0) @binding(0)
var<uniform> uniforms: GridUniforms;

// Vertex shader (fullscreen quad)
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) near_point: vec3<f32>,  // Unprojected near plane point
    @location(1) far_point: vec3<f32>,   // Unprojected far plane point
};

// Fullscreen quad vertices (NDC coordinates)
const QUAD_VERTICES = array<vec2<f32>, 6>(
    vec2<f32>(-1.0, -1.0),  // Bottom-left
    vec2<f32>( 1.0, -1.0),  // Bottom-right
    vec2<f32>(-1.0,  1.0),  // Top-left
    vec2<f32>(-1.0,  1.0),  // Top-left
    vec2<f32>( 1.0, -1.0),  // Bottom-right
    vec2<f32>( 1.0,  1.0),  // Top-right
);

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    
    let pos = QUAD_VERTICES[vertex_index];
    out.position = vec4<f32>(pos, 0.0, 1.0);
    
    // Unproject to world space (near and far plane)
    let near = uniforms.inv_view_proj * vec4<f32>(pos, -1.0, 1.0);
    let far = uniforms.inv_view_proj * vec4<f32>(pos, 1.0, 1.0);
    
    out.near_point = near.xyz / near.w;
    out.far_point = far.xyz / far.w;
    
    return out;
}

// Fragment shader (compute grid)
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Ray from camera through fragment
    let ray_dir = normalize(in.far_point - in.near_point);
    
    // Intersect ray with Y=0 plane (ground)
    let t = -in.near_point.y / ray_dir.y;
    
    // Discard if ray doesn't hit ground plane or hits behind camera
    if t < 0.0 {
        discard;
    }
    
    let world_pos = in.near_point + ray_dir * t;
    
    // Distance from camera (for fading)
    let distance = length(world_pos - uniforms.camera_pos);
    
    // Fade grid at distance (prevent aliasing)
    if distance > uniforms.max_distance {
        discard;
    }
    
    // Compute grid coordinates
    let grid_coord = world_pos.xz / uniforms.grid_size;
    let grid_fract = fract(grid_coord);
    
    // Compute grid line thickness (derivative-based anti-aliasing)
    let grid_deriv = fwidth(grid_coord);
    let grid_line = smoothstep(vec2<f32>(0.0), grid_deriv * 2.0, abs(grid_fract - 0.5) - 0.5 + grid_deriv);
    let grid_alpha = 1.0 - min(grid_line.x, grid_line.y);
    
    // Major grid lines (every N lines)
    let major_coord = world_pos.xz / (uniforms.grid_size * uniforms.major_grid_every);
    let major_fract = fract(major_coord);
    let major_deriv = fwidth(major_coord);
    let major_line = smoothstep(vec2<f32>(0.0), major_deriv * 2.0, abs(major_fract - 0.5) - 0.5 + major_deriv);
    let major_alpha = 1.0 - min(major_line.x, major_line.y);
    
    // Axes (X and Z)
    let x_axis_alpha = smoothstep(0.0, uniforms.grid_size * 0.1, abs(world_pos.z));
    let z_axis_alpha = smoothstep(0.0, uniforms.grid_size * 0.1, abs(world_pos.x));
    
    // Distance fade
    let fade = 1.0 - smoothstep(uniforms.fade_distance, uniforms.max_distance, distance);
    
    // Ground plane base color (dark surface so entities appear to stand on something solid)
    // Higher alpha (0.6) makes it more visible, darker color (0.12) to not overpower grid
    let ground_base_color = vec4<f32>(0.12, 0.12, 0.15, 0.6); // Dark blue-gray, more opaque
    
    // Combine: Ground base → Grid → Major grid → Axes
    var color = ground_base_color; // Start with ground plane
    color = mix(color, uniforms.grid_color, grid_alpha); // Add grid lines
    color = mix(color, uniforms.major_grid_color, major_alpha); // Add major grid
    color = mix(color, uniforms.x_axis_color, (1.0 - x_axis_alpha) * uniforms.x_axis_color.a); // X axis
    color = mix(color, uniforms.z_axis_color, (1.0 - z_axis_alpha) * uniforms.z_axis_color.a); // Z axis
    
    // Apply distance fade
    color.a *= fade;
    
    // Discard fully transparent pixels (optimization)
    if color.a < 0.01 {
        discard;
    }
    
    return color;
}
