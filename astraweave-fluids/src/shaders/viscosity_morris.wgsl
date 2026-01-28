// ============================================================================
// viscosity_morris.wgsl - Research-Grade SPH Viscosity Shader
// ============================================================================
//
// Implements Morris et al. 1997 viscosity formulation with extensions:
// - Explicit Morris viscosity (standard SPH)
// - Matrix-free implicit Jacobi iteration (Weiler et al. 2018)
// - Non-Newtonian support (Carreau, Power Law, Bingham)
// - Temperature-dependent viscosity (Arrhenius, VTF)
//
// References:
// - Morris, Fox & Zhu (1997) "Modeling Low Reynolds Number Incompressible Flows"
// - Weiler et al. (2018) "A Physically Consistent Implicit Viscosity Solver"
// ============================================================================

// ============================================================================
// STRUCTURES
// ============================================================================

struct ViscosityParams {
    // Core parameters
    smoothing_radius: f32,      // h
    dt: f32,                    // Time step
    particle_count: u32,        // Number of particles
    particle_mass: f32,         // m

    // Grid parameters
    grid_width: u32,
    grid_height: u32,
    grid_depth: u32,
    cell_size: f32,

    // Viscosity settings
    base_viscosity: f32,        // μ₀ (Pa·s)
    solver_type: u32,           // 0=XSPH, 1=Morris, 2=ImplicitJacobi
    iteration: u32,             // Current Jacobi iteration
    max_iterations: u32,        // Max Jacobi iterations

    // Implicit solver settings
    omega: f32,                 // SOR relaxation (0.5-1.0)
    tolerance: f32,             // Convergence threshold
    _pad0: f32,
    _pad1: f32,

    // Non-Newtonian parameters
    enable_non_newtonian: u32,  // 0/1
    nn_model_type: u32,         // 0=Newtonian, 1=PowerLaw, 2=Carreau, 3=Cross, 4=Bingham
    viscosity_0: f32,           // Zero-shear viscosity
    viscosity_inf: f32,         // Infinite-shear viscosity

    power_index: f32,           // n (power law index)
    relaxation_time: f32,       // λ (Carreau)
    yield_stress: f32,          // τ_y (Bingham)
    cross_exponent: f32,        // m (Cross model)

    // Temperature parameters
    enable_temperature: u32,    // 0/1
    temp_model_type: u32,       // 0=Constant, 1=Arrhenius, 2=VTF
    reference_temp: f32,        // T_ref (K)
    activation_energy: f32,     // E_a (J/mol)

    temp_coefficient: f32,      // B (VTF)
    vogel_temp: f32,            // T₀ (VTF)
    _pad2: f32,
    _pad3: f32,
}

struct Particle {
    position: vec4<f32>,
    velocity: vec4<f32>,
    predicted_position: vec4<f32>,
    
    lambda: f32,
    density: f32,
    phase: u32,
    temperature: f32,

    alpha: f32,
    kappa: f32,
    velocity_divergence: f32,
    density_derivative: f32,
    previous_pressure: f32,

    viscosity_coefficient: f32,
    shear_rate: f32,

    shift_delta: vec3<f32>,
    is_surface: u32,

    vorticity: vec3<f32>,
    angular_velocity: vec3<f32>,

    phase_gradient: vec3<f32>,
    is_gas: u32,

    color: vec4<f32>,

    reserved0: f32,
    reserved1: f32,
    _pad: f32,
}

// ============================================================================
// BINDINGS
// ============================================================================

@group(0) @binding(0) var<uniform> params: ViscosityParams;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(2) var<storage, read> grid_cells: array<u32>;
@group(0) @binding(3) var<storage, read> cell_counts: array<u32>;

// Double-buffered velocity for implicit solver
@group(0) @binding(4) var<storage, read> velocity_in: array<vec4<f32>>;
@group(0) @binding(5) var<storage, read_write> velocity_out: array<vec4<f32>>;
@group(0) @binding(6) var<storage, read_write> residuals: array<f32>;

// ============================================================================
// KERNEL FUNCTIONS
// ============================================================================

const PI: f32 = 3.14159265359;

/// Cubic spline kernel W(r, h)
fn kernel_w(r: f32, h: f32) -> f32 {
    if (r >= h) {
        return 0.0;
    }

    let q = r / h;
    let sigma = 8.0 / (PI * h * h * h);

    if (q <= 0.5) {
        return sigma * (6.0 * q * q * q - 6.0 * q * q + 1.0);
    } else {
        let one_minus_q = 1.0 - q;
        return sigma * 2.0 * one_minus_q * one_minus_q * one_minus_q;
    }
}

/// Gradient magnitude of cubic spline kernel (always positive)
/// The gradient points toward higher density (center), direction is -r̂
fn kernel_gradient_mag(r: f32, h: f32) -> f32 {
    if (r >= h || r < 1e-10) {
        return 0.0;
    }

    let q = r / h;
    let sigma = 48.0 / (PI * h * h * h * h);

    // Return positive magnitude (flip signs from derivative)
    if (q <= 0.5) {
        // dW/dr is negative here, so we flip to get positive magnitude
        return sigma * q * (2.0 - 3.0 * q);
    } else {
        let one_minus_q = 1.0 - q;
        return sigma * one_minus_q * one_minus_q;
    }
}

/// Laplacian of kernel for viscosity
fn kernel_laplacian(r: f32, h: f32) -> f32 {
    if (r >= h) {
        return 0.0;
    }

    let q = r / h;
    let sigma = 48.0 / (PI * h * h * h * h * h);

    if (q <= 0.5) {
        return sigma * (6.0 * q - 2.0);
    } else {
        let one_minus_q = 1.0 - q;
        return sigma * 2.0 * one_minus_q;
    }
}

// ============================================================================
// VISCOSITY MODELS
// ============================================================================

/// Compute non-Newtonian effective viscosity
fn compute_non_newtonian_viscosity(shear_rate: f32) -> f32 {
    let gamma_dot = max(shear_rate, 1e-6);

    switch (params.nn_model_type) {
        // Newtonian
        case 0u: {
            return params.viscosity_0;
        }
        // Power Law: μ = K γ̇^(n-1)
        case 1u: {
            return params.viscosity_0 * pow(gamma_dot, params.power_index - 1.0);
        }
        // Carreau: μ = μ∞ + (μ₀ - μ∞)(1 + (λγ̇)²)^((n-1)/2)
        case 2u: {
            let lambda_gamma = params.relaxation_time * gamma_dot;
            let factor = pow(1.0 + lambda_gamma * lambda_gamma, (params.power_index - 1.0) / 2.0);
            return params.viscosity_inf + (params.viscosity_0 - params.viscosity_inf) * factor;
        }
        // Cross: μ = μ∞ + (μ₀ - μ∞)/(1 + (λγ̇)^m)
        case 3u: {
            let lambda_gamma = params.relaxation_time * gamma_dot;
            let denom = 1.0 + pow(lambda_gamma, params.cross_exponent);
            return params.viscosity_inf + (params.viscosity_0 - params.viscosity_inf) / denom;
        }
        // Bingham: μ = μ₀ + τ_y/γ̇
        case 4u: {
            return params.viscosity_0 + params.yield_stress / gamma_dot;
        }
        default: {
            return params.viscosity_0;
        }
    }
}

/// Compute temperature-dependent viscosity
fn compute_temperature_viscosity(temperature: f32) -> f32 {
    let R = 8.314; // Gas constant

    switch (params.temp_model_type) {
        // Constant
        case 0u: {
            return params.base_viscosity;
        }
        // Arrhenius: μ(T) = μ_ref exp(E_a/R (1/T - 1/T_ref))
        case 1u: {
            let exponent = (params.activation_energy / R) * 
                (1.0 / temperature - 1.0 / params.reference_temp);
            return params.base_viscosity * exp(exponent);
        }
        // VTF: μ(T) = μ_ref exp(B/(T - T₀) - B/(T_ref - T₀))
        case 2u: {
            let t_minus_t0 = max(temperature - params.vogel_temp, 1.0);
            let tref_minus_t0 = max(params.reference_temp - params.vogel_temp, 1.0);
            let exponent = params.temp_coefficient * (1.0 / t_minus_t0 - 1.0 / tref_minus_t0);
            return params.base_viscosity * exp(exponent);
        }
        default: {
            return params.base_viscosity;
        }
    }
}

/// Get effective viscosity for a particle
fn get_effective_viscosity(id: u32) -> f32 {
    var mu = params.base_viscosity;

    // Temperature dependence
    if (params.enable_temperature != 0u) {
        mu = compute_temperature_viscosity(particles[id].temperature);
    }

    // Non-Newtonian behavior
    if (params.enable_non_newtonian != 0u) {
        let shear_rate = particles[id].shear_rate;
        let nn_mu = compute_non_newtonian_viscosity(shear_rate);
        // Scale by ratio to base model
        mu = mu * (nn_mu / params.viscosity_0);
    }

    return mu;
}

// ============================================================================
// GRID UTILITIES
// ============================================================================

fn get_cell_index(pos: vec3<f32>) -> i32 {
    let grid_pos = vec3<i32>(floor(pos / params.cell_size));
    
    if (grid_pos.x < 0 || grid_pos.x >= i32(params.grid_width) ||
        grid_pos.y < 0 || grid_pos.y >= i32(params.grid_height) ||
        grid_pos.z < 0 || grid_pos.z >= i32(params.grid_depth)) {
        return -1;
    }
    
    return grid_pos.x + 
           grid_pos.y * i32(params.grid_width) + 
           grid_pos.z * i32(params.grid_width * params.grid_height);
}

// ============================================================================
// XSPH VISCOSITY (Simple, for games)
// ============================================================================

@compute @workgroup_size(64)
fn compute_xsph_viscosity(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.particle_count) {
        return;
    }

    let pos_i = particles[id].position.xyz;
    let vel_i = particles[id].velocity.xyz;
    let h = params.smoothing_radius;
    let xsph_factor = 0.01;

    var correction = vec3<f32>(0.0);

    // Search neighboring cells
    let cell_idx = get_cell_index(pos_i);
    if (cell_idx < 0) {
        return;
    }

    let grid_pos = vec3<i32>(floor(pos_i / params.cell_size));
    
    for (var dz = -1; dz <= 1; dz++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let neighbor_pos = grid_pos + vec3<i32>(dx, dy, dz);
                
                if (neighbor_pos.x < 0 || neighbor_pos.x >= i32(params.grid_width) ||
                    neighbor_pos.y < 0 || neighbor_pos.y >= i32(params.grid_height) ||
                    neighbor_pos.z < 0 || neighbor_pos.z >= i32(params.grid_depth)) {
                    continue;
                }

                let neighbor_cell = neighbor_pos.x + 
                                   neighbor_pos.y * i32(params.grid_width) +
                                   neighbor_pos.z * i32(params.grid_width * params.grid_height);
                
                let cell_start = grid_cells[u32(neighbor_cell) * 2u];
                let cell_end = cell_start + cell_counts[u32(neighbor_cell)];

                for (var j = cell_start; j < cell_end; j++) {
                    if (j == id) {
                        continue;
                    }

                    let pos_j = particles[j].position.xyz;
                    let diff = pos_i - pos_j;
                    let r = length(diff);

                    if (r >= h) {
                        continue;
                    }

                    let vel_j = particles[j].velocity.xyz;
                    let w = kernel_w(r, h);

                    correction += xsph_factor * (vel_j - vel_i) * w;
                }
            }
        }
    }

    particles[id].velocity = vec4<f32>(vel_i + correction, 0.0);
}

// ============================================================================
// MORRIS VISCOSITY (Explicit, physically-based)
// ============================================================================

@compute @workgroup_size(64)
fn compute_morris_viscosity(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.particle_count) {
        return;
    }

    let pos_i = particles[id].position.xyz;
    let vel_i = particles[id].velocity.xyz;
    let rho_i = particles[id].density;
    let mu_i = get_effective_viscosity(id);
    
    let h = params.smoothing_radius;
    let h_sq = h * h;
    let epsilon = 0.01 * h_sq;
    let dt = params.dt;
    let mass = params.particle_mass;

    var visc_accel = vec3<f32>(0.0);

    // Search neighboring cells
    let grid_pos = vec3<i32>(floor(pos_i / params.cell_size));
    
    for (var dz = -1; dz <= 1; dz++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let neighbor_pos = grid_pos + vec3<i32>(dx, dy, dz);
                
                if (neighbor_pos.x < 0 || neighbor_pos.x >= i32(params.grid_width) ||
                    neighbor_pos.y < 0 || neighbor_pos.y >= i32(params.grid_height) ||
                    neighbor_pos.z < 0 || neighbor_pos.z >= i32(params.grid_depth)) {
                    continue;
                }

                let neighbor_cell = neighbor_pos.x + 
                                   neighbor_pos.y * i32(params.grid_width) +
                                   neighbor_pos.z * i32(params.grid_width * params.grid_height);
                
                let cell_start = grid_cells[u32(neighbor_cell) * 2u];
                let cell_end = cell_start + cell_counts[u32(neighbor_cell)];

                for (var j = cell_start; j < cell_end; j++) {
                    if (j == id) {
                        continue;
                    }

                    let pos_j = particles[j].position.xyz;
                    let diff = pos_i - pos_j;
                    let r_sq = dot(diff, diff);
                    let r = sqrt(r_sq);

                    if (r >= h || r < 1e-10) {
                        continue;
                    }

                    let vel_j = particles[j].velocity.xyz;
                    let rho_j = particles[j].density;
                    let mu_j = get_effective_viscosity(j);

                    // Morris formulation (Morris et al. 1997) with 3D dimension factor
                    // a_i = dim_factor * m_j * (μ_i + μ_j)/(ρ_i * ρ_j) * (v_j - v_i) * |∇W| * r / (r² + εh²)
                    // Dimension factor for 3D: 2*(d+2)/2 = 5 (divided by 2 because we use μ_i + μ_j)
                    let dim_factor = 5.0;
                    let mu_sum = mu_i + mu_j;
                    let grad_w = kernel_gradient_mag(r, h);
                    let denom = r_sq + epsilon;

                    let factor = dim_factor * mass * mu_sum / (rho_i * rho_j * denom) * grad_w * r;

                    // (vel_j - vel_i) gives diffusive effect: fast particles slow down
                    visc_accel += factor * (vel_j - vel_i);
                }
            }
        }
    }

    particles[id].velocity = vec4<f32>(vel_i + visc_accel * dt, 0.0);
}

// ============================================================================
// IMPLICIT JACOBI VISCOSITY (High viscosity stable)
// ============================================================================

/// Initialize implicit solver - copy current velocities to buffer
@compute @workgroup_size(64)
fn init_implicit_viscosity(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.particle_count) {
        return;
    }

    velocity_out[id] = particles[id].velocity;
    residuals[id] = 0.0;
}

/// Jacobi iteration for implicit viscosity solve
@compute @workgroup_size(64)
fn iterate_implicit_viscosity(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.particle_count) {
        return;
    }

    let pos_i = particles[id].position.xyz;
    let vel_original = particles[id].velocity.xyz;
    let rho_i = particles[id].density;
    let mu_i = get_effective_viscosity(id);
    
    let h = params.smoothing_radius;
    let dt = params.dt;
    let mass = params.particle_mass;
    let omega = params.omega;

    var weighted_sum = vec3<f32>(0.0);
    var weight_total = 0.0;

    // Search neighboring cells
    let grid_pos = vec3<i32>(floor(pos_i / params.cell_size));
    
    for (var dz = -1; dz <= 1; dz++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let neighbor_pos = grid_pos + vec3<i32>(dx, dy, dz);
                
                if (neighbor_pos.x < 0 || neighbor_pos.x >= i32(params.grid_width) ||
                    neighbor_pos.y < 0 || neighbor_pos.y >= i32(params.grid_height) ||
                    neighbor_pos.z < 0 || neighbor_pos.z >= i32(params.grid_depth)) {
                    continue;
                }

                let neighbor_cell = neighbor_pos.x + 
                                   neighbor_pos.y * i32(params.grid_width) +
                                   neighbor_pos.z * i32(params.grid_width * params.grid_height);
                
                let cell_start = grid_cells[u32(neighbor_cell) * 2u];
                let cell_end = cell_start + cell_counts[u32(neighbor_cell)];

                for (var j = cell_start; j < cell_end; j++) {
                    if (j == id) {
                        continue;
                    }

                    let pos_j = particles[j].position.xyz;
                    let diff = pos_i - pos_j;
                    let r = length(diff);

                    if (r >= h) {
                        continue;
                    }

                    let rho_j = particles[j].density;
                    let mu_j = get_effective_viscosity(j);

                    // Harmonic mean of viscosities
                    let mu_avg = 2.0 * mu_i * mu_j / (mu_i + mu_j + 1e-10);

                    // Laplacian kernel
                    let lap_w = kernel_laplacian(r, h);

                    // Weight for this neighbor
                    let weight = dt * mu_avg * lap_w * mass / rho_j;

                    // Use velocity from input buffer (previous iteration)
                    weighted_sum += weight * velocity_in[j].xyz;
                    weight_total += weight;
                }
            }
        }
    }

    // Jacobi update
    let denom = 1.0 + weight_total;
    let new_v = (vel_original + weighted_sum) / denom;

    // Apply SOR relaxation
    let old_v = velocity_in[id].xyz;
    let relaxed_v = old_v + omega * (new_v - old_v);

    // Store result
    velocity_out[id] = vec4<f32>(relaxed_v, 0.0);

    // Compute residual
    let residual = length(new_v - old_v);
    residuals[id] = residual;
}

/// Finalize implicit solve - copy result to particles
@compute @workgroup_size(64)
fn finalize_implicit_viscosity(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.particle_count) {
        return;
    }

    particles[id].velocity = velocity_out[id];
}

// ============================================================================
// SHEAR RATE COMPUTATION
// ============================================================================

@compute @workgroup_size(64)
fn compute_shear_rate(@builtin(global_invocation_id) gid: vec3<u32>) {
    let id = gid.x;
    if (id >= params.particle_count) {
        return;
    }

    let pos_i = particles[id].position.xyz;
    let vel_i = particles[id].velocity.xyz;
    let h = params.smoothing_radius;

    // Compute velocity gradient tensor ∂vᵢ/∂xⱼ
    var grad_v: mat3x3<f32>;
    grad_v[0] = vec3<f32>(0.0);
    grad_v[1] = vec3<f32>(0.0);
    grad_v[2] = vec3<f32>(0.0);

    // Search neighboring cells
    let grid_pos = vec3<i32>(floor(pos_i / params.cell_size));
    
    for (var dz = -1; dz <= 1; dz++) {
        for (var dy = -1; dy <= 1; dy++) {
            for (var dx = -1; dx <= 1; dx++) {
                let neighbor_pos = grid_pos + vec3<i32>(dx, dy, dz);
                
                if (neighbor_pos.x < 0 || neighbor_pos.x >= i32(params.grid_width) ||
                    neighbor_pos.y < 0 || neighbor_pos.y >= i32(params.grid_height) ||
                    neighbor_pos.z < 0 || neighbor_pos.z >= i32(params.grid_depth)) {
                    continue;
                }

                let neighbor_cell = neighbor_pos.x + 
                                   neighbor_pos.y * i32(params.grid_width) +
                                   neighbor_pos.z * i32(params.grid_width * params.grid_depth);
                
                let cell_start = grid_cells[u32(neighbor_cell) * 2u];
                let cell_end = cell_start + cell_counts[u32(neighbor_cell)];

                for (var j = cell_start; j < cell_end; j++) {
                    if (j == id) {
                        continue;
                    }

                    let pos_j = particles[j].position.xyz;
                    let diff = pos_i - pos_j;
                    let r = length(diff);

                    if (r >= h || r < 1e-10) {
                        continue;
                    }

                    let vel_j = particles[j].velocity.xyz;
                    let vel_diff = vel_j - vel_i;

                    let grad_w = kernel_gradient_mag(r, h);
                    let grad_dir = -diff / r;

                    // Outer product: (v_j - v_i) ⊗ ∇W
                    let mass = params.particle_mass;
                    let rho_j = particles[j].density;
                    let vol_j = mass / rho_j;

                    grad_v[0] += vel_diff.x * grad_dir * grad_w * vol_j;
                    grad_v[1] += vel_diff.y * grad_dir * grad_w * vol_j;
                    grad_v[2] += vel_diff.z * grad_dir * grad_w * vol_j;
                }
            }
        }
    }

    // Strain rate tensor: D = 0.5 (∇v + ∇vᵀ)
    // Shear rate = sqrt(2 * D:D)
    var d_sq_sum = 0.0;
    for (var a = 0; a < 3; a++) {
        for (var b = 0; b < 3; b++) {
            let d_ab = 0.5 * (grad_v[a][b] + grad_v[b][a]);
            d_sq_sum += d_ab * d_ab;
        }
    }

    let shear_rate = sqrt(2.0 * d_sq_sum);
    particles[id].shear_rate = shear_rate;

    // Also compute vorticity ω = ∇ × v
    let omega_x = grad_v[2][1] - grad_v[1][2];
    let omega_y = grad_v[0][2] - grad_v[2][0];
    let omega_z = grad_v[1][0] - grad_v[0][1];
    particles[id].vorticity = vec3<f32>(omega_x, omega_y, omega_z);
}
