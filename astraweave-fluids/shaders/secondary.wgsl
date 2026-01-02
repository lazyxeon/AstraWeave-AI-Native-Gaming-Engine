struct ViewParams {
    view_proj: mat4x4<f32>,
    view_inv: mat4x4<f32>,
    proj_inv: mat4x4<f32>,
    camera_pos: vec4<f32>,
    screen_size: vec2<f32>,
};

@group(0) @binding(0) var<uniform> view_params: ViewParams;

struct SecondaryParticle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    info: vec4<f32>, // x: lifetime, y: type, z: alpha, w: scale
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @location(0) pos: vec4<f32>,
    @location(1) vel: vec4<f32>,
    @location(2) info: vec4<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    
    // Billboard logic
    let quad_pos = array<vec2<f32>, 4>(
        vec2<f32>(-1.0, -1.0),
        vec2<f32>(1.0, -1.0),
        vec2<f32>(-1.0, 1.0),
        vec2<f32>(1.0, 1.0)
    );
    
    let uv = array<vec2<f32>, 4>(
        vec2<f32>(0.0, 1.0),
        vec2<f32>(1.0, 1.0),
        vec2<f32>(0.0, 0.0),
        vec2<f32>(1.0, 0.0)
    );
    
    let scale = info.w;
    let alpha = info.z;
    
    // Use view_inv columns 0 and 1 for camera-aligned billboard
    let right = view_params.view_inv[0].xyz;
    let up = view_params.view_inv[1].xyz;
    
    let billboard_pos = pos.xyz + (quad_pos[v_idx].x * right + quad_pos[v_idx].y * up) * scale;
    
    out.clip_position = view_params.view_proj * vec4<f32>(billboard_pos, 1.0);
    out.uv = uv[v_idx];
    
    // Color based on type (0=Foam/Spray, 1=Bubble)
    out.color = vec4<f32>(0.9, 0.95, 1.0, alpha * 0.5);
    
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // Round particle shape
    let r = length(in.uv - 0.5) * 2.0;
    if (r > 1.0) { discard; }
    
    let soft_edge = 1.0 - smoothstep(0.5, 1.0, r);
    return vec4<f32>(in.color.rgb, in.color.a * soft_edge);
}
