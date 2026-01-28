// =============================================================================
// PCISPH (Predictive-Corrective Incompressible SPH) Compute Shader
// =============================================================================
//
// Implementation of Solenthaler & Pajarola 2009: "Predictive-Corrective
// Incompressible SPH"
//
// Key Features:
// - Predictive-corrective pressure solving (iterative)
// - Warm-starting from previous frame pressure
// - δ-SPH particle shifting (Marrone et al. 2011)
// - Improved incompressibility over PBD
// - GPU-optimized with shared memory where beneficial
//
// Performance Targets:
// - 100-200k particles @ 60 FPS
// - <0.1% density error after convergence
// - 3-8 pressure iterations typical
//
// =============================================================================

// Research-grade particle structure (176 bytes)
struct ResearchParticle {
    position: vec4<f32>,              // xyz + mass
    velocity: vec4<f32>,              // xyz + pad
    predicted_position: vec4<f32>,    // xyz + pad
    
    lambda: f32,                      // PBD constraint multiplier
    density: f32,                     // Current density
    phase: u32,                       // Phase ID
    temperature: f32,                 // Temperature in Kelvin
    
    alpha: f32,                       // DFSPH α factor
    kappa: f32,                       // DFSPH κ factor
    velocity_divergence: f32,         // ∇·v
    density_derivative: f32,          // Dρ/Dt
    previous_pressure: f32,           // For warm-starting
    
    viscosity_coefficient: f32,       // Dynamic viscosity
    shear_rate: f32,                  // Non-Newtonian shear rate
    
    shift_delta: vec3<f32>,           // δ-SPH shift vector
    is_surface: u32,                  // Surface particle flag
    
    vorticity: vec3<f32>,             // Curl of velocity
    angular_velocity: vec3<f32>,      // Micropolar spin
    
    phase_gradient: vec3<f32>,        // Interface normal
    is_gas: u32,                      // Air phase flag
    
    color: vec4<f32>,                 // Rendering color
    
    reserved0: f32,                   // Future use
    reserved1: f32,                   // Future use
    _pad: f32,                        // 16-byte alignment
};

// Extended simulation parameters for research solvers
struct ResearchSimParams {
    // Core parameters (64 bytes)
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
    frame: u32,
    warm_start_factor: f32,
    _pad0: f32,
    
    // Solver parameters (32 bytes)
    max_iterations: u32,
    min_iterations: u32,
    density_error_threshold: f32,
    divergence_error_threshold: f32,
    pcisph_delta: f32,                // Pre-computed pressure coefficient
    shifting_strength: f32,           // δ-SPH coefficient C_δ
    vorticity_epsilon: f32,           // Vorticity confinement ε
    sor_omega: f32,                   // SOR relaxation
    
    // Flags (16 bytes)
    solver_type: u32,                 // 0=PBD, 1=PCISPH, 2=DFSPH, 3=IISPH
    enable_shifting: u32,             // Enable δ-SPH
    enable_vorticity: u32,            // Enable vorticity confinement
    enable_warm_start: u32,           // Enable warm-starting
};

// Dynamic scene objects
struct DynamicObject {
    transform: mat4x4<f32>,
    inv_transform: mat4x4<f32>,
    half_extents: vec4<f32>,          // w = type (0=box, 1=sphere)
};

// PCISPH iteration state
struct IterationState {
    iteration: u32,
    max_density_error: f32,
    avg_density_error: f32,
    converged: u32,
};

// =============================================================================
// BINDINGS
// =============================================================================

// Group 0: Global Infrastructure
@group(0) @binding(0) var<uniform> params: ResearchSimParams;
@group(0) @binding(1) var<storage, read_write> head_pointers: array<atomic<i32>>;
@group(0) @binding(2) var<storage, read_write> next_pointers: array<i32>;
@group(0) @binding(3) var<storage, read_write> iteration_state: IterationState;
@group(0) @binding(4) var<storage, read_write> density_errors: array<f32>;

// Group 1: Particles (Ping-Pong)
@group(1) @binding(0) var<storage, read_write> particles: array<ResearchParticle>;
@group(1) @binding(1) var<storage, read_write> pressure_buffer: array<f32>;

// Group 2: Scene Data
@group(2) @binding(0) var<storage, read> dynamic_objects: array<DynamicObject>;

// =============================================================================
// CONSTANTS AND KERNEL FUNCTIONS
// =============================================================================

const PI: f32 = 3.14159265359;
const EPSILON: f32 = 1e-6;

// Cubic spline kernel (Monaghan 1992) - better conservation properties
fn kernel_cubic_spline(r: f32, h: f32) -> f32 {
    let q = r / h;
    if (q >= 2.0) { return 0.0; }
    
    let sigma = 1.0 / (PI * h * h * h);
    
    if (q < 1.0) {
        return sigma * (1.0 - 1.5 * q * q + 0.75 * q * q * q);
    } else {
        let t = 2.0 - q;
        return sigma * 0.25 * t * t * t;
    }
}

fn kernel_cubic_spline_grad(r: f32, diff: vec3<f32>, h: f32) -> vec3<f32> {
    if (r < EPSILON || r >= 2.0 * h) { return vec3<f32>(0.0); }
    
    let q = r / h;
    let sigma = 1.0 / (PI * h * h * h * h);
    
    var grad_w: f32;
    if (q < 1.0) {
        grad_w = sigma * (-3.0 * q + 2.25 * q * q);
    } else {
        let t = 2.0 - q;
        grad_w = sigma * (-0.75 * t * t);
    }
    
    return (grad_w / r) * diff;
}

// Wendland C2 kernel - compact support, better stability
fn kernel_wendland(r: f32, h: f32) -> f32 {
    let q = r / h;
    if (q >= 1.0) { return 0.0; }
    
    let sigma = 21.0 / (2.0 * PI * h * h * h);
    let t = 1.0 - q;
    return sigma * t * t * t * t * (1.0 + 4.0 * q);
}

fn kernel_wendland_grad(r: f32, diff: vec3<f32>, h: f32) -> vec3<f32> {
    if (r < EPSILON || r >= h) { return vec3<f32>(0.0); }
    
    let q = r / h;
    let sigma = 21.0 / (2.0 * PI * h * h * h * h);
    let t = 1.0 - q;
    
    // d/dq of (1-q)^4 * (1+4q) = -20q(1-q)^3
    let grad_w = sigma * (-20.0 * q * t * t * t);
    
    return (grad_w / r) * diff;
}

// Grid utility functions
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
    
    return grid_pos.x + grid_pos.y * i32(params.grid_width) + 
           grid_pos.z * i32(params.grid_width * params.grid_height);
}

fn get_cell_index_clamped(x: i32, y: i32, z: i32) -> i32 {
    if (x < 0 || x >= i32(params.grid_width) ||
        y < 0 || y >= i32(params.grid_height) ||
        z < 0 || z >= i32(params.grid_depth)) {
        return -1;
    }
    return x + y * i32(params.grid_width) + 
           z * i32(params.grid_width * params.grid_height);
}

// =============================================================================
// PCISPH KERNELS
// =============================================================================

// Step 1: Predict positions (external forces)
@compute @workgroup_size(64)
fn pcisph_predict(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    
    let pos = particles[id].position.xyz;
    var vel = particles[id].velocity.xyz;
    let temp = particles[id].temperature;
    let mass = particles[id].position.w;
    
    // External forces
    // 1. Gravity
    vel += vec3<f32>(0.0, params.gravity, 0.0) * params.dt;
    
    // 2. Thermal buoyancy (Boussinesq approximation)
    let ambient_temp = 293.0;
    let thermal_expansion = 0.0002;
    let buoyancy = thermal_expansion * (temp - ambient_temp) * abs(params.gravity);
    vel.y += buoyancy * params.dt;
    
    // Store predicted position
    particles[id].predicted_position = vec4<f32>(pos + vel * params.dt, 1.0);
    particles[id].velocity = vec4<f32>(vel, 0.0);
    
    // Initialize pressure with warm-starting
    if (params.enable_warm_start != 0u) {
        pressure_buffer[id] = particles[id].previous_pressure * params.warm_start_factor;
    } else {
        pressure_buffer[id] = 0.0;
    }
}

// Step 2: Clear spatial hash grid
@compute @workgroup_size(64)
fn pcisph_clear_grid(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    let grid_size = params.grid_width * params.grid_height * params.grid_depth;
    if (id >= grid_size) { return; }
    atomicStore(&head_pointers[id], -1);
}

// Step 3: Build spatial hash grid
@compute @workgroup_size(64)
fn pcisph_build_grid(@builtin(global_invocation_id) global_id: vec3<u32>) {
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

// Step 4: Compute density and density error
@compute @workgroup_size(64)
fn pcisph_compute_density(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    
    let pos = particles[id].predicted_position.xyz;
    let pressure_i = pressure_buffer[id];
    let h = params.smoothing_radius;
    
    // Compute density from neighbors
    var density = 0.0;
    
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );
    
    // Iterate over 27 neighboring cells
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(
                    grid_pos.x + dx, 
                    grid_pos.y + dy, 
                    grid_pos.z + dz
                );
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    let j = u32(current);
                    let neighbor_pos = particles[j].predicted_position.xyz;
                    let diff = pos - neighbor_pos;
                    let r = length(diff);
                    
                    if (r < h) {
                        // Use Wendland kernel for better stability
                        density += kernel_wendland(r, h);
                    }
                    
                    current = next_pointers[current];
                }
            }
        }
    }
    
    // Store density and compute error
    particles[id].density = density;
    let density_error = (density - params.target_density) / params.target_density;
    density_errors[id] = abs(density_error);
}

// Step 5: Compute pressure from density error (PCISPH correction)
@compute @workgroup_size(64)
fn pcisph_pressure_solve(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    
    let density = particles[id].density;
    let density_error = max(0.0, density - params.target_density);
    
    // PCISPH pressure update:
    // p_i += δ * max(0, ρ_i - ρ₀)
    //
    // δ is pre-computed as:
    // δ = -1 / (β * (-Σ∇W_ij · Σ∇W_ij - Σ(∇W_ij · ∇W_ij)))
    // β = Δt² * m² * 2 / ρ₀²
    //
    // For simplicity, we use the pressure_multiplier as approximation
    let pressure_correction = params.pcisph_delta * density_error;
    pressure_buffer[id] += pressure_correction;
    
    // Clamp to prevent negative pressure
    pressure_buffer[id] = max(0.0, pressure_buffer[id]);
}

// Step 6: Compute pressure acceleration and update velocities
@compute @workgroup_size(64)
fn pcisph_apply_pressure(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    
    let pos = particles[id].predicted_position.xyz;
    let vel_old = particles[id].velocity.xyz;
    let density_i = particles[id].density;
    let pressure_i = pressure_buffer[id];
    let h = params.smoothing_radius;
    
    // Compute pressure force: -∇p / ρ
    var pressure_force = vec3<f32>(0.0);
    
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );
    
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(
                    grid_pos.x + dx, 
                    grid_pos.y + dy, 
                    grid_pos.z + dz
                );
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    let j = u32(current);
                    if (j != id) {
                        let neighbor_pos = particles[j].predicted_position.xyz;
                        let diff = pos - neighbor_pos;
                        let r = length(diff);
                        
                        if (r > EPSILON && r < h) {
                            let density_j = particles[j].density;
                            let pressure_j = pressure_buffer[j];
                            
                            // Symmetric pressure force (momentum conserving)
                            // a_pressure = -m Σ (p_i/ρ_i² + p_j/ρ_j²) ∇W_ij
                            let grad_w = kernel_wendland_grad(r, diff, h);
                            let pressure_term = pressure_i / (density_i * density_i + EPSILON) +
                                               pressure_j / (density_j * density_j + EPSILON);
                            
                            pressure_force -= pressure_term * grad_w;
                        }
                    }
                    
                    current = next_pointers[current];
                }
            }
        }
    }
    
    // Update velocity with pressure acceleration
    let vel_new = vel_old + pressure_force * params.dt;
    particles[id].velocity = vec4<f32>(vel_new, 0.0);
    
    // Update predicted position
    let pos_old = particles[id].position.xyz;
    particles[id].predicted_position = vec4<f32>(pos_old + vel_new * params.dt, 1.0);
}

// Step 7: Apply viscosity (Morris formulation - physically correct)
@compute @workgroup_size(64)
fn pcisph_viscosity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    
    let pos = particles[id].predicted_position.xyz;
    let vel = particles[id].velocity.xyz;
    let density_i = particles[id].density;
    let viscosity_i = particles[id].viscosity_coefficient;
    let h = params.smoothing_radius;
    
    var viscosity_force = vec3<f32>(0.0);
    
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );
    
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(
                    grid_pos.x + dx, 
                    grid_pos.y + dy, 
                    grid_pos.z + dz
                );
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    let j = u32(current);
                    if (j != id) {
                        let neighbor_pos = particles[j].predicted_position.xyz;
                        let neighbor_vel = particles[j].velocity.xyz;
                        let density_j = particles[j].density;
                        let diff = pos - neighbor_pos;
                        let r = length(diff);
                        
                        if (r > EPSILON && r < h) {
                            let vel_diff = neighbor_vel - vel;
                            let grad_w = kernel_wendland_grad(r, diff, h);
                            
                            // Morris viscosity formulation:
                            // a_visc = 2ν Σ (m_j/ρ_j) * (v_ij · r_ij) / (|r_ij|² + 0.01h²) * ∇W_ij
                            let dot_rv = dot(vel_diff, diff);
                            let viscosity_term = 2.0 * viscosity_i * dot_rv / 
                                                (r * r + 0.01 * h * h);
                            
                            viscosity_force += viscosity_term * grad_w / (density_j + EPSILON);
                        }
                    }
                    
                    current = next_pointers[current];
                }
            }
        }
    }
    
    // Apply viscosity acceleration
    let vel_new = vel + viscosity_force * params.dt;
    particles[id].velocity = vec4<f32>(vel_new, 0.0);
}

// Step 8: Particle shifting (δ-SPH for tensile instability)
@compute @workgroup_size(64)
fn pcisph_particle_shifting(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    if (params.enable_shifting == 0u) { return; }
    
    let pos = particles[id].predicted_position.xyz;
    let density_i = particles[id].density;
    let h = params.smoothing_radius;
    
    // Compute shift vector: δr = -C_δ * h * Σ (∇W_ij / (Σ_k W_ik))
    var shift = vec3<f32>(0.0);
    var weight_sum = 0.0;
    var is_surface = 0u;
    
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );
    
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(
                    grid_pos.x + dx, 
                    grid_pos.y + dy, 
                    grid_pos.z + dz
                );
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    let j = u32(current);
                    if (j != id) {
                        let neighbor_pos = particles[j].predicted_position.xyz;
                        let diff = pos - neighbor_pos;
                        let r = length(diff);
                        
                        if (r > EPSILON && r < h) {
                            let w = kernel_wendland(r, h);
                            let grad_w = kernel_wendland_grad(r, diff, h);
                            
                            weight_sum += w;
                            shift += grad_w;
                        }
                    }
                    
                    current = next_pointers[current];
                }
            }
        }
    }
    
    // Surface detection based on neighbor density
    if (weight_sum < 0.7 * params.target_density / kernel_wendland(0.0, h)) {
        is_surface = 1u;
    }
    
    // Apply shift only for interior particles (not at surface)
    var final_shift = vec3<f32>(0.0);
    if (is_surface == 0u && weight_sum > EPSILON) {
        final_shift = -params.shifting_strength * h * shift / weight_sum;
        
        // Limit shift magnitude to 0.1h
        let shift_mag = length(final_shift);
        if (shift_mag > 0.1 * h) {
            final_shift = final_shift * (0.1 * h / shift_mag);
        }
    }
    
    particles[id].shift_delta = final_shift;
    particles[id].is_surface = is_surface;
}

// Step 9: Final position update and boundary handling
@compute @workgroup_size(64)
fn pcisph_integrate(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    
    var pos = particles[id].predicted_position.xyz;
    let vel = particles[id].velocity.xyz;
    let shift = particles[id].shift_delta;
    
    // Apply particle shifting
    if (params.enable_shifting != 0u) {
        pos += shift;
    }
    
    // Boundary handling (simple box for now)
    let bounds_min = vec3<f32>(-29.0, 0.1, -29.0);
    let bounds_max = vec3<f32>(29.0, 50.0, 29.0);
    let restitution = 0.3;
    
    var new_vel = vel;
    
    if (pos.x < bounds_min.x) {
        pos.x = bounds_min.x;
        new_vel.x = abs(new_vel.x) * restitution;
    }
    if (pos.x > bounds_max.x) {
        pos.x = bounds_max.x;
        new_vel.x = -abs(new_vel.x) * restitution;
    }
    if (pos.y < bounds_min.y) {
        pos.y = bounds_min.y;
        new_vel.y = abs(new_vel.y) * restitution;
    }
    if (pos.y > bounds_max.y) {
        pos.y = bounds_max.y;
        new_vel.y = -abs(new_vel.y) * restitution;
    }
    if (pos.z < bounds_min.z) {
        pos.z = bounds_min.z;
        new_vel.z = abs(new_vel.z) * restitution;
    }
    if (pos.z > bounds_max.z) {
        pos.z = bounds_max.z;
        new_vel.z = -abs(new_vel.z) * restitution;
    }
    
    // Store results
    particles[id].position = vec4<f32>(pos, particles[id].position.w);
    particles[id].velocity = vec4<f32>(new_vel, 0.0);
    
    // Store pressure for warm-starting next frame
    particles[id].previous_pressure = pressure_buffer[id];
}

// Utility: Compute vorticity for confinement (optional post-pass)
@compute @workgroup_size(64)
fn pcisph_compute_vorticity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    if (params.enable_vorticity == 0u) { return; }
    
    let pos = particles[id].position.xyz;
    let vel = particles[id].velocity.xyz;
    let h = params.smoothing_radius;
    
    // Compute curl: ω = ∇ × v
    var omega = vec3<f32>(0.0);
    
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );
    
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(
                    grid_pos.x + dx, 
                    grid_pos.y + dy, 
                    grid_pos.z + dz
                );
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    let j = u32(current);
                    if (j != id) {
                        let neighbor_pos = particles[j].position.xyz;
                        let neighbor_vel = particles[j].velocity.xyz;
                        let density_j = particles[j].density;
                        let diff = pos - neighbor_pos;
                        let r = length(diff);
                        
                        if (r > EPSILON && r < h) {
                            let vel_diff = neighbor_vel - vel;
                            let grad_w = kernel_wendland_grad(r, diff, h);
                            
                            // ω_i += Σ (m_j/ρ_j) * v_ij × ∇W_ij
                            omega += cross(vel_diff, grad_w) / (density_j + EPSILON);
                        }
                    }
                    
                    current = next_pointers[current];
                }
            }
        }
    }
    
    particles[id].vorticity = omega;
}

// Apply vorticity confinement force
@compute @workgroup_size(64)
fn pcisph_apply_vorticity(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let id = global_id.x;
    if (id >= params.particle_count) { return; }
    if (params.enable_vorticity == 0u) { return; }
    
    let pos = particles[id].position.xyz;
    let omega = particles[id].vorticity;
    let omega_mag = length(omega);
    let h = params.smoothing_radius;
    
    if (omega_mag < EPSILON) { return; }
    
    // Compute gradient of |ω|: η = ∇|ω|
    var eta = vec3<f32>(0.0);
    
    let grid_pos = vec3<i32>(
        i32(floor((pos.x + 30.0) / params.cell_size)),
        i32(floor(pos.y / params.cell_size)),
        i32(floor((pos.z + 30.0) / params.cell_size))
    );
    
    for (var dx = -1; dx <= 1; dx++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dz = -1; dz <= 1; dz++) {
                let cell_idx = get_cell_index_clamped(
                    grid_pos.x + dx, 
                    grid_pos.y + dy, 
                    grid_pos.z + dz
                );
                if (cell_idx < 0) { continue; }
                
                var current = atomicLoad(&head_pointers[cell_idx]);
                while (current >= 0) {
                    let j = u32(current);
                    if (j != id) {
                        let neighbor_pos = particles[j].position.xyz;
                        let neighbor_omega = particles[j].vorticity;
                        let density_j = particles[j].density;
                        let diff = pos - neighbor_pos;
                        let r = length(diff);
                        
                        if (r > EPSILON && r < h) {
                            let grad_w = kernel_wendland_grad(r, diff, h);
                            let omega_j_mag = length(neighbor_omega);
                            
                            // η += Σ (m_j/ρ_j) * |ω_j| * ∇W_ij
                            eta += omega_j_mag * grad_w / (density_j + EPSILON);
                        }
                    }
                    
                    current = next_pointers[current];
                }
            }
        }
    }
    
    // Vorticity confinement force: f_vc = ε * (N × ω), N = η / |η|
    let eta_mag = length(eta);
    if (eta_mag > EPSILON) {
        let n = eta / eta_mag;
        let vorticity_force = params.vorticity_epsilon * cross(n, omega);
        
        // Apply to velocity
        let vel = particles[id].velocity.xyz;
        particles[id].velocity = vec4<f32>(vel + vorticity_force * params.dt, 0.0);
    }
}
