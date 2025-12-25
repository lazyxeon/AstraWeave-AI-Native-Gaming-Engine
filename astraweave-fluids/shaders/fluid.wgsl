// Position-Based Fluids (PBD) Compute Shader
// Optimized for stability and professional production use

struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    predicted_position: vec4<f32>,
    lambda: f32,
    density: f32,
    padding1: f32,
    padding2: f32,
};

struct DynamicObject {
    transform: mat4x4<f32>,
    inv_transform: mat4x4<f32>,
    half_extents: vec4<f32>, // w = type (0=box, 1=sphere)
};

struct SimParams {
    smoothing_radius: f32,
    target_density: f32,
    pressure_multiplier: f32,
    viscosity: f32,
    surface_tension: f32,
    gravity: f32,
    dt: f32,
    particle_count: u32,
    grid_width: u32,
    grid_height: u32,
    grid_depth: u32,
    cell_size: f32,
    object_count: u32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
};

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<storage, read_write> particles_dst_unused: array<Particle>;
@group(0) @binding(3) var<storage, read_write> head_pointers: array<atomic<i32>>;
@group(0) @binding(4) var<storage, read_write> next_pointers: array<i32>;
@group(0) @binding(5) var sdf_texture: texture_3d<f32>;
@group(0) @binding(6) var default_sampler: sampler;

@group(1) @binding(0) var<storage, read> dynamic_objects: array<DynamicObject>;

const PI: f32 = 3.14159265359;

fn get_cell_index(pos: vec3<f32>) -> i32 {
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );
    
    if (grid_pos.x < 0 || grid_pos.x >= i32(params.grid_width) ||
        grid_pos.y < 0 || grid_pos.y >= i32(params.grid_height) ||
        grid_pos.z < 0 || grid_pos.z >= i32(params.grid_depth)) {
        return -1;
    }
    
    return grid_pos.x + grid_pos.y * i32(params.grid_width) + grid_pos.z * i32(params.grid_width * params.grid_height);
}

fn get_cell_index_clamped(x: i32, y: i32, z: i32) -> i32 {
    if (x < 0 || x >= i32(params.grid_width) ||
        y < 0 || y >= i32(params.grid_height) ||
        z < 0 || z >= i32(params.grid_depth)) {
        return -1;
    }
    return x + y * i32(params.grid_width) + z * i32(params.grid_width * params.grid_height);
}

// Cubic spline kernel for better stability in PBD
fn kernel_w(r: f32, h: f32) -> f32 {
    let q = r / h;
    if (q >= 1.0) { return 0.0; }
    let alpha = 3.0 / (2.0 * PI * h * h * h);
    if (q < 0.5) {
        return alpha * (2.0 * (1.0 - q) * (1.0 - q) * (1.0 - q) - (1.0 - 2.0 * q) * (1.0 - 2.0 * q) * (1.0 - 2.0 * q));
    } else {
        return alpha * (1.0 - q) * (1.0 - q) * (1.0 - q);
    }
}

fn kernel_grad_w(r: f32, diff: vec3<f32>, h: f32) -> vec3<f32> {
    let q = r / h;
    if (q >= 1.0 || r <= 0.0001) { return vec3<f32>(0.0); }
    let alpha = 3.0 / (2.0 * PI * h * h * h);
    var grad_q = 0.0;
    if (q < 0.5) {
        grad_q = alpha * (-6.0 * (1.0 - q) * (1.0 - q) + 6.0 * (1.0 - 2.0 * q) * (1.0 - 2.0 * q)) / h;
    } else {
        grad_q = alpha * (-3.0 * (1.0 - q) * (1.0 - q)) / h;
    }
    return (grad_q / r) * diff;
}

@compute @workgroup_size(64)
fn predict(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    let pos = particles[id].position.xyz;
    var vel = particles[id].velocity.xyz;

    // External forces (Gravity)
    vel += vec3<f32>(0.0, params.gravity, 0.0) * params.dt;
    
    particles[id].predicted_position = vec4<f32>(pos + vel * params.dt, 1.0);
    particles[id].velocity = vec4<f32>(vel, 0.0);
}

@compute @workgroup_size(64)
fn clear_grid(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    let grid_size = params.grid_width * params.grid_height * params.grid_depth;
    if (id >= grid_size) { return; }
    atomicStore(&head_pointers[id], -1);
}

@compute @workgroup_size(64)
fn build_grid(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    
    let pos = particles[id].predicted_position.xyz;
    let cell_idx = get_cell_index(pos);
    
    if (cell_idx >= 0) {
        let old_head = atomicExchange(&head_pointers[cell_idx], i32(id));
        next_pointers[id] = old_head;
    } else {
        next_pointers[id] = -1;
    }
}

@compute @workgroup_size(64)
fn compute_lambda(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    let pos = particles[id].predicted_position.xyz;
    let h = params.smoothing_radius;
    
    var density = 0.0;
    var sum_grad_c2 = 0.0;
    var grad_ci = vec3<f32>(0.0);

    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );

    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(grid_pos.x + dx, grid_pos.y + dy, grid_pos.z + dz);
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    let neighbor_pos = particles[current].predicted_position.xyz;
                    let diff = pos - neighbor_pos;
                    let r = length(diff);
                    if (r < h) {
                        let w = kernel_w(r, h);
                        density += w;
                        
                        if (u32(current) != id) {
                            let grad_wj = kernel_grad_w(r, diff, h) / params.target_density;
                            sum_grad_c2 += dot(grad_wj, grad_wj);
                            grad_ci += grad_wj;
                        }
                    }
                    current = next_pointers[current];
                }
            }
        }
    }

    sum_grad_c2 += dot(grad_ci, grad_ci);
    let constraint = (density / params.target_density) - 1.0;
    
    // epsilon for constraint softening
    let epsilon = 100.0; 
    particles[id].lambda = -constraint / (sum_grad_c2 + epsilon);
    particles[id].density = density;
}

fn sd_box(p: vec3<f32>, b: vec3<f32>) -> f32 {
    let q = abs(p) - b;
    return length(max(q, vec3<f32>(0.0))) + min(max(q.x, max(q.y, q.z)), 0.0);
}

@compute @workgroup_size(64)
fn compute_delta_pos(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    let pos = particles[id].predicted_position.xyz;
    let lambda_i = particles[id].lambda;
    let h = params.smoothing_radius;
    
    var delta_p = vec3<f32>(0.0);

    // Dynamic Object Collisions
    for (var i: u32 = 0; i < params.object_count; i++) {
        let obj = dynamic_objects[i];
        let p_local = (obj.inv_transform * vec4<f32>(pos, 1.0)).xyz;
        
        var dist = 0.0;
        var local_normal = vec3<f32>(0.0);
        
        if (obj.half_extents.w < 0.5) { // Box
            dist = sd_box(p_local, obj.half_extents.xyz);
            // Gradient approximation for box
            local_normal = normalize(sign(p_local) * max(abs(p_local) - obj.half_extents.xyz, vec3<f32>(0.001)));
        } else { // Sphere
            dist = length(p_local) - obj.half_extents.x;
            local_normal = normalize(p_local);
        }
        
        if (dist < 0.1) {
             let world_normal = normalize((obj.transform * vec4<f32>(local_normal, 0.0)).xyz);
             delta_p += world_normal * (0.1 - dist) * 0.5; // Soft collision resolve
        }
    }

    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );

    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(grid_pos.x + dx, grid_pos.y + dy, grid_pos.z + dz);
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    if (u32(current) != id) {
                        let neighbor_pos = particles[current].predicted_position.xyz;
                        let diff = pos - neighbor_pos;
                        let r = length(diff);
                        if (r < h) {
                            let lambda_j = particles[current].lambda;
                            
                            // Tensile instability correction (scorr)
                            let scorr = -0.001 * pow(kernel_w(r, h) / kernel_w(0.1 * h, h), 4.0);
                            
                            // Surface Tension Cohesion force (simple approximation)
                            let cohesion = params.surface_tension * kernel_w(r, h) * normalize(diff);
                            
                            delta_p += (lambda_i + lambda_j + scorr) * kernel_grad_w(r, diff, h) + cohesion;
                        }
                    }
                    current = next_pointers[current];
                }
            }
        }
    }

    particles[id].predicted_position += vec4<f32>(delta_p / params.target_density, 0.0);
}

@compute @workgroup_size(64)
fn integrate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    let old_pos = particles[id].position.xyz;
    var pred_pos = particles[id].predicted_position.xyz;

    // --- Global SDF Collision ---
    let voxel_pos = (pred_pos / 60.0) + 0.5; // Normalized to [0, 1] assuming world is [-30, 30]
    let sdf_dist = textureSampleLevel(sdf_texture, default_sampler, voxel_pos, 0.0).r;
    
    let particle_radius = 0.2;
    if (sdf_dist < particle_radius) {
        // Compute normal from SDF gradient
        let eps = 0.01;
        let nx = textureSampleLevel(sdf_texture, default_sampler, voxel_pos + vec3<f32>(eps, 0.0, 0.0), 0.0).r - sdf_dist;
        let ny = textureSampleLevel(sdf_texture, default_sampler, voxel_pos + vec3<f32>(0.0, eps, 0.0), 0.0).r - sdf_dist;
        let nz = textureSampleLevel(sdf_texture, default_sampler, voxel_pos + vec3<f32>(0.0, 0.0, eps), 0.0).r - sdf_dist;
        let normal = normalize(vec3<f32>(nx, ny, nz));
        
        pred_pos += normal * (particle_radius - sdf_dist);
    }

    // Boundary Handling (Simple box for now, SDFs handle the rest)
    let bounds_x = 30.0;
    let bounds_z = 30.0;
    let bounds_y = 60.0;
    if (pred_pos.y < 0.0) { pred_pos.y = 0.0; }
    if (pred_pos.y > bounds_y) { pred_pos.y = bounds_y; }
    if (pred_pos.x < -bounds_x) { pred_pos.x = -bounds_x; }
    if (pred_pos.x > bounds_x) { pred_pos.x = bounds_x; }
    if (pred_pos.z < -bounds_z) { pred_pos.z = -bounds_z; }
    if (pred_pos.z > bounds_z) { pred_pos.z = bounds_z; }

    let vel = (pred_pos - old_pos) / params.delta_time;
    
    // XSPH Viscosity
    var xsph_vel = vel;
    let h = params.smoothing_radius;
    let grid_pos = vec3<i32>(
        i32(floor((pred_pos.x + 30.0) / params.cell_size)),
        i32(floor(pred_pos.y / params.cell_size)),
        i32(floor((pred_pos.z + 30.0) / params.cell_size))
    );

    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(grid_pos.x + dx, grid_pos.y + dy, grid_pos.z + dz);
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    if (u32(current) != id) {
                        let neighbor_pos = particles[current].predicted_position.xyz;
                        let neighbor_vel = (particles[current].predicted_position.xyz - particles[current].position.xyz) / params.delta_time;
                        let diff = pred_pos - neighbor_pos;
                        let r = length(diff);
                        if (r < h) {
                            xsph_vel += 0.01 * (neighbor_vel - vel) * kernel_w(r, h);
                        }
                    }
                    current = next_pointers[current];
                }
            }
        }
    }

    particles[id].position = vec4<f32>(pred_pos, 1.0);
    particles[id].velocity = vec4<f32>(xsph_vel, 0.0);
}
