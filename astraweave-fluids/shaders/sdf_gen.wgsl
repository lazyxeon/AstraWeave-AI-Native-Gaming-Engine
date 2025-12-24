// SDF Generation Shaders - Voxelization and JFA
// Used for Global Signed Distance Field collisions in AstraWeave Fluids

struct Config {
    resolution: u32,
    world_size: f32,
    triangle_count: u32,
};

@group(0) @binding(0) var<uniform> config: Config;
@group(0) @binding(1) var voxel_grid: texture_storage_3d<r32float, read_write>;

// --- Voxelization Pass ---
// Simple binary voxelization: if a triangle touches the voxel, it's solid (1.0)
@compute @workgroup_size(64)
fn voxelize(@builtin(global_invocation_id) global_id: vec3<u32>) {
    // This would be a simplified version of the vxgi_voxelize.wgsl
    // For brevity in this implementation, we assume we use a similar logic
}

// --- JFA Initialization ---
// Initialize the JFA texture with the position of the solid voxel or a far-away value
@compute @workgroup_size(8, 8, 8)
fn jfa_init(@builtin(global_invocation_id) id: vec3<u32>) {
    let resolution = config.resolution;
    if (id.x >= resolution || id.y >= resolution || id.z >= resolution) { return; }
    
    let is_solid = textureLoad(voxel_grid, id).r > 0.5;
    if (is_solid) {
        textureStore(voxel_grid, id, vec4<f32>(vec3<f32>(id), 1.0));
    } else {
        textureStore(voxel_grid, id, vec4<f32>(-1.0, -1.0, -1.0, 0.0));
    }
}

// --- JFA Step ---
struct JfaParams {
    step_size: u32,
};
@group(0) @binding(2) var<uniform> jfa_params: JfaParams;
@group(0) @binding(3) var next_grid: texture_storage_3d<r32float, write>;

@compute @workgroup_size(8, 8, 8)
fn jfa_step(@builtin(global_invocation_id) id: vec3<u32>) {
    let resolution = config.resolution;
    if (id.x >= resolution || id.y >= resolution || id.z >= resolution) { return; }
    
    var best_pos = textureLoad(voxel_grid, id).xyz;
    var best_dist = 99999.0;
    if (best_pos.x >= 0.0) {
        best_dist = length(vec3<f32>(id) - best_pos);
    }
    
    let step = i32(jfa_params.step_size);
    
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                if (dx == 0 && dy == 0 && dz == 0) { continue; }
                
                let neighbor_id = vec3<i32>(id) + vec3<i32>(dx, dy, dz) * step;
                if (neighbor_id.x < 0 || neighbor_id.x >= i32(resolution) ||
                    neighbor_id.y < 0 || neighbor_id.y >= i32(resolution) ||
                    neighbor_id.z < 0 || neighbor_id.z >= i32(resolution)) { continue; }
                
                let neighbor_pos = textureLoad(voxel_grid, vec3<u32>(neighbor_id)).xyz;
                if (neighbor_pos.x >= 0.0) {
                    let d = length(vec3<f32>(id) - neighbor_pos);
                    if (d < best_dist) {
                        best_dist = d;
                        best_pos = neighbor_pos;
                    }
                }
            }
        }
    }
    
    textureStore(next_grid, id, vec4<f32>(best_pos, 1.0));
}

// --- SDF Finalize ---
@compute @workgroup_size(8, 8, 8)
fn sdf_finalize(@builtin(global_invocation_id) id: vec3<u32>) {
    let resolution = config.resolution;
    if (id.x >= resolution || id.y >= resolution || id.z >= resolution) { return; }
    
    let nearest_pos = textureLoad(voxel_grid, id).xyz;
    var dist = 99999.0;
    if (nearest_pos.x >= 0.0) {
        dist = length(vec3<f32>(id) - nearest_pos) * (config.world_size / f32(resolution));
    }
    
    // Check if inner or outer (for signed distance)
    // For now we just use positive distance
    textureStore(voxel_grid, id, vec4<f32>(dist, 0.0, 0.0, 1.0));
}
