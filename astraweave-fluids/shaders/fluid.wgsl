// Smoothed Particle Hydrodynamics (SPH) Compute Shader

struct Particle {
    position: vec4<f32>, // xyz = pos, w = padding
    velocity: vec4<f32>, // xyz = vel, w = padding
    force: vec4<f32>,    // xyz = force, w = padding
    density: f32,        // density
    pressure: f32,       // pressure
    padding1: f32,
    padding2: f32,
};

struct SimParams {
    delta_time: f32,
    smoothing_radius: f32,
    target_density: f32,
    pressure_multiplier: f32,
    viscosity: f32,
    particle_count: u32,
    gravity: f32,
    padding: u32,
    cell_size: f32,
    grid_width: u32,
    grid_height: u32,
    grid_depth: u32,
};

@group(0) @binding(0) var<uniform> params: SimParams;
@group(0) @binding(1) var<storage, read_write> particles_src: array<Particle>;
@group(0) @binding(2) var<storage, read_write> particles_dst: array<Particle>;
@group(0) @binding(3) var<storage, read_write> head_pointers: array<atomic<i32>>;
@group(0) @binding(4) var<storage, read_write> next_pointers: array<i32>;

const PI: f32 = 3.14159265359;

fn get_cell_index(pos: vec3<f32>) -> i32 {
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 20.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 20.0) / params.cell_size))
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

fn poly6_kernel(r2: f32, h: f32) -> f32 {
    let h2 = h * h;
    if (r2 < 0.0 || r2 > h2) { return 0.0; }
    let term = h2 - r2;
    return (315.0 / (64.0 * PI * pow(h, 9.0))) * term * term * term;
}

fn spiky_kernel_grad(r: f32, dist_vec: vec3<f32>, h: f32) -> vec3<f32> {
    if (r <= 0.0001 || r > h) { return vec3<f32>(0.0); }
    let term = h - r;
    let scalar = -45.0 / (PI * pow(h, 6.0)) * term * term / r;
    return scalar * dist_vec;
}

fn viscosity_kernel_lap(r: f32, h: f32) -> f32 {
    if (r > h) { return 0.0; }
    let term = h - r;
    return (45.0 / (PI * pow(h, 6.0))) * term;
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
    
    let pos = particles_src[id].position.xyz;
    let cell_idx = get_cell_index(pos);
    
    if (cell_idx >= 0) {
        let old_head = atomicExchange(&head_pointers[cell_idx], i32(id));
        next_pointers[id] = old_head;
    } else {
        next_pointers[id] = -1;
    }
}

@compute @workgroup_size(64)
fn compute_density_pressure(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    let pos = particles_src[id].position.xyz;
    var density = 0.0;
    let h = params.smoothing_radius;
    let h2 = h * h;

    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 20.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 20.0) / params.cell_size))
    );

    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let nx = grid_pos.x + dx;
                let ny = grid_pos.y + dy;
                let nz = grid_pos.z + dz;
                let cell_idx = get_cell_index_clamped(nx, ny, nz);
                
                if (cell_idx >= 0) {
                    var current = atomicLoad(&head_pointers[cell_idx]);
                    while (current >= 0) {
                        let neighbor_pos = particles_src[current].position.xyz;
                        let diff = pos - neighbor_pos;
                        let r2 = dot(diff, diff);
                        if (r2 < h2) {
                            density += poly6_kernel(r2, h);
                        }
                        current = next_pointers[current];
                    }
                }
            }
        }
    }

    particles_dst[id].density = max(density, params.target_density);
    particles_dst[id].pressure = params.pressure_multiplier * (particles_dst[id].density - params.target_density);
    particles_dst[id].position = particles_src[id].position;
    particles_dst[id].velocity = particles_src[id].velocity;
    particles_dst[id].force = vec4<f32>(0.0);
}

@compute @workgroup_size(64)
fn compute_forces(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    let pos = particles_src[id].position.xyz;
    let vel = particles_src[id].velocity.xyz;
    let pressure = particles_src[id].pressure;
    let density = particles_src[id].density;
    
    var pressure_force = vec3<f32>(0.0);
    var viscosity_force = vec3<f32>(0.0);
    let h = params.smoothing_radius;
    let h2 = h * h;

    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 20.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 20.0) / params.cell_size))
    );

    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let nx = grid_pos.x + dx;
                let ny = grid_pos.y + dy;
                let nz = grid_pos.z + dz;
                let cell_idx = get_cell_index_clamped(nx, ny, nz);
                
                if (cell_idx >= 0) {
                    var current = atomicLoad(&head_pointers[cell_idx]);
                    while (current >= 0) {
                        if (u32(current) != id) {
                            let neighbor_pos = particles_src[current].position.xyz;
                            let diff = pos - neighbor_pos;
                            let r2 = dot(diff, diff);

                            if (r2 < h2) {
                                let r = sqrt(r2);
                                let neighbor_pressure = particles_src[current].pressure;
                                let neighbor_density = particles_src[current].density;
                                let neighbor_vel = particles_src[current].velocity.xyz;

                                let p_term = (pressure + neighbor_pressure) / (2.0 * neighbor_density);
                                pressure_force -= p_term * spiky_kernel_grad(r, diff, h);
                                viscosity_force += (neighbor_vel - vel) * viscosity_kernel_lap(r, h) / neighbor_density;
                            }
                        }
                        current = next_pointers[current];
                    }
                }
            }
        }
    }

    pressure_force *= params.target_density; 
    viscosity_force *= params.viscosity * params.target_density; 
    let gravity = vec3<f32>(0.0, params.gravity, 0.0) * density;
    particles_dst[id].force = vec4<f32>(pressure_force + viscosity_force + gravity, 0.0);
    particles_dst[id].position = particles_src[id].position;
    particles_dst[id].velocity = particles_src[id].velocity;
    particles_dst[id].density = particles_src[id].density;
    particles_dst[id].pressure = particles_src[id].pressure;
}

@compute @workgroup_size(64)
fn integrate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }

    var pos = particles_src[id].position.xyz;
    var vel = particles_src[id].velocity.xyz;
    let force = particles_src[id].force.xyz;
    let density = particles_src[id].density;

    let accel = force / density;
    vel += accel * params.delta_time;
    pos += vel * params.delta_time;

    let bounds = 20.0;
    let damping = 0.1;
    if (pos.y < 0.0) { pos.y = 0.0; vel.y *= -damping; }
    if (pos.x < -bounds) { pos.x = -bounds; vel.x *= -damping; }
    if (pos.x > bounds) { pos.x = bounds; vel.x *= -damping; }
    if (pos.z < -bounds) { pos.z = -bounds; vel.z *= -damping; }
    if (pos.z > bounds) { pos.z = bounds; vel.z *= -damping; }

    particles_dst[id].position = vec4<f32>(pos, 1.0);
    particles_dst[id].velocity = vec4<f32>(vel, 0.0);
    particles_dst[id].density = density;
    particles_dst[id].pressure = particles_src[id].pressure;
    particles_dst[id].force = vec4<f32>(0.0);
}
