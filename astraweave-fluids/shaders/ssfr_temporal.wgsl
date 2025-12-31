// SSFR Temporal Reprojection Pass
// Reduces flickering by blending current frame with reprojected history

struct TemporalParams {
    current_view_proj: mat4x4<f32>,
    prev_view_proj: mat4x4<f32>,
    jitter_offset: vec2<f32>,
    blend_factor: f32,
    _pad: f32,
};

@group(0) @binding(0) var<uniform> params: TemporalParams;
@group(0) @binding(1) var current_color: texture_2d<f32>;
@group(0) @binding(2) var current_depth: texture_2d<f32>;
@group(0) @binding(3) var history_color: texture_2d<f32>;
@group(0) @binding(4) var default_sampler: sampler;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let u = f32((vertex_index << 1u) & 2u);
    let v = f32(vertex_index & 2u);
    out.uv = vec2<f32>(u, v);
    out.clip_position = vec4<f32>(u * 2.0 - 1.0, 1.0 - v * 2.0, 0.0, 1.0);
    return out;
}

// Reconstruct clip-space position from UV and depth
fn clip_from_uv_depth(uv: vec2<f32>, depth: f32) -> vec4<f32> {
    return vec4<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0, depth, 1.0);
}

// Compute motion vector from current to previous frame
fn get_motion_vector(uv: vec2<f32>, depth: f32) -> vec2<f32> {
    // Current clip position
    let curr_clip = clip_from_uv_depth(uv, depth);
    
    // Reproject to previous frame
    let curr_world = params.current_view_proj * curr_clip;
    let prev_clip = params.prev_view_proj * curr_world;
    let prev_ndc = prev_clip.xy / prev_clip.w;
    let prev_uv = vec2<f32>(prev_ndc.x * 0.5 + 0.5, 0.5 - prev_ndc.y * 0.5);
    
    return uv - prev_uv;
}

// Neighborhood clipping for ghosting prevention
fn ycocg_from_rgb(rgb: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(
        rgb.r * 0.25 + rgb.g * 0.5 + rgb.b * 0.25,
        rgb.r * 0.5 - rgb.b * 0.5,
        rgb.g * 0.5 - rgb.r * 0.25 - rgb.b * 0.25
    );
}

fn rgb_from_ycocg(ycocg: vec3<f32>) -> vec3<f32> {
    let tmp = ycocg.x - ycocg.z;
    return vec3<f32>(
        tmp + ycocg.y,
        ycocg.x + ycocg.z,
        tmp - ycocg.y
    );
}

fn clip_aabb(color: vec3<f32>, box_min: vec3<f32>, box_max: vec3<f32>) -> vec3<f32> {
    let center = (box_max + box_min) * 0.5;
    let half_size = (box_max - box_min) * 0.5 + 0.001;
    
    let clip = color - center;
    let clip_unit = abs(clip / half_size);
    let max_clip = max(clip_unit.x, max(clip_unit.y, clip_unit.z));
    
    if (max_clip > 1.0) {
        return center + clip / max_clip;
    }
    return color;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let size = vec2<f32>(textureDimensions(current_color));
    let texel_size = 1.0 / size;
    
    // Sample current frame with jitter offset removed
    let uv = in.uv - params.jitter_offset * texel_size;
    let current = textureSample(current_color, default_sampler, uv);
    let depth = textureSample(current_depth, default_sampler, uv).r;
    
    // Skip reprojection for background
    if (depth >= 1.0) {
        return current;
    }
    
    // Compute motion vector
    let motion = get_motion_vector(uv, depth);
    let history_uv = uv + motion;
    
    // Reject if reprojected UV is outside frame
    if (any(history_uv < vec2<f32>(0.0)) || any(history_uv > vec2<f32>(1.0))) {
        return current;
    }
    
    // Sample history
    var history = textureSample(history_color, default_sampler, history_uv);
    
    // Neighborhood clamping in YCoCg space to prevent ghosting
    var box_min = ycocg_from_rgb(current.rgb);
    var box_max = box_min;
    
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let sample_uv = uv + vec2<f32>(f32(x), f32(y)) * texel_size;
            let sample_rgb = textureSample(current_color, default_sampler, sample_uv).rgb;
            let sample_ycocg = ycocg_from_rgb(sample_rgb);
            box_min = min(box_min, sample_ycocg);
            box_max = max(box_max, sample_ycocg);
        }
    }
    
    let history_ycocg = ycocg_from_rgb(history.rgb);
    let clamped_ycocg = clip_aabb(history_ycocg, box_min, box_max);
    let clamped_history = rgb_from_ycocg(clamped_ycocg);
    
    // Blend current and clamped history
    let blend = params.blend_factor;
    let final_rgb = mix(clamped_history, current.rgb, 1.0 - blend);
    
    return vec4<f32>(final_rgb, current.a);
}
