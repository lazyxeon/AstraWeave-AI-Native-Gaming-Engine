// MegaLights: Write Indices Pass
// Writes light indices into the global compacted light list

struct GpuLight {
    position: vec4<f32>, // xyz = pos, w = radius
    color: vec4<f32>,    // rgb = color, a = intensity
}

struct ClusterBounds {
    min_pos: vec3<f32>,
    pad1: f32,
    max_pos: vec3<f32>,
    pad2: f32,
}

struct ClusterParams {
    cluster_dims: vec3<u32>,
    pad1: u32,
    total_clusters: u32,
    light_count: u32,
    pad2: u32,
    pad3: u32,
}

@group(0) @binding(0) var<storage, read> lights: array<GpuLight>;
@group(0) @binding(1) var<storage, read> clusters: array<ClusterBounds>;
@group(0) @binding(2) var<storage, read> light_offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> light_indices: array<u32>;
@group(0) @binding(4) var<uniform> params: ClusterParams;

fn sphere_aabb_intersect(center: vec3<f32>, radius: f32, aabb_min: vec3<f32>, aabb_max: vec3<f32>) -> bool {
    let closest = clamp(center, aabb_min, aabb_max);
    let dist_sq = dot(center - closest, center - closest);
    return dist_sq <= (radius * radius);
}

@compute @workgroup_size(64, 1, 1)
fn write_light_indices(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let cluster_index = global_id.x + 
                        global_id.y * params.cluster_dims.x + 
                        global_id.z * params.cluster_dims.x * params.cluster_dims.y;

    if (cluster_index >= params.total_clusters) {
        return;
    }

    let bounds = clusters[cluster_index];
    let start_offset = light_offsets[cluster_index];
    var current_count = 0u;

    // Re-run intersection to find which lights to write
    for (var i = 0u; i < params.light_count; i = i + 1u) {
        let light = lights[i];
        if (sphere_aabb_intersect(light.position.xyz, light.position.w, bounds.min_pos, bounds.max_pos)) {
            // Write light index
            light_indices[start_offset + current_count] = i;
            current_count = current_count + 1u;
        }
    }
}
