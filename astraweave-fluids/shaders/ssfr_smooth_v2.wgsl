// SSFR Smoothing Pass - Bilateral Filter for Fluid Depth
// Smooths the fluid surface while preserving sharp silhouette edges

@group(0) @binding(0) var src_depth: texture_depth_2d;
@group(0) @binding(1) var dst_depth: texture_storage_2d<r32float, write>;

struct Params {
    radius: i32,
    blur_scale: f32,
    blur_depth_falloff: f32,
    _pad: f32,
    _pad2: vec4<f32>,
};

@group(0) @binding(4) var<uniform> params: Params;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(src_depth);
    if (id.x >= size.x || id.y >= size.y) { return; }
    
    let center_coord = vec2<i32>(i32(id.x), i32(id.y));
    let center_depth = textureLoad(src_depth, center_coord, 0);
    
    if (center_depth >= 1.0) {
        textureStore(dst_depth, center_coord, vec4<f32>(1.0, 0.0, 0.0, 0.0));
        return;
    }
    
    var sum = 0.0;
    var wsum = 0.0;
    
    for (var x = -params.radius; x <= params.radius; x++) {
        for (var y = -params.radius; y <= params.radius; y++) {
            let sample_coord = center_coord + vec2<i32>(x, y);
            if (sample_coord.x < 0 || sample_coord.x >= i32(size.x) || 
                sample_coord.y < 0 || sample_coord.y >= i32(size.y)) { continue; }
            
            let sample_depth = textureLoad(src_depth, sample_coord, 0);
            if (sample_depth >= 1.0) { continue; }
            
            // Spatial weight
            let r2 = f32(x*x + y*y);
            let w_spatial = exp(-r2 * params.blur_scale);
            
            // Range weight (depth difference)
            let diff = (sample_depth - center_depth) * params.blur_depth_falloff;
            let w_range = exp(-diff * diff);
            
            let w = w_spatial * w_range;
            sum += sample_depth * w;
            wsum += w;
        }
    }
    
    if (wsum > 0.0) {
        textureStore(dst_depth, center_coord, vec4<f32>(sum / wsum, 0.0, 0.0, 0.0));
    } else {
        textureStore(dst_depth, center_coord, vec4<f32>(center_depth, 0.0, 0.0, 0.0));
    }
}
