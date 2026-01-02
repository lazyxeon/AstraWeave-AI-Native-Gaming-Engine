//! Adversarial Fluids Benchmarks
//!
//! Stress testing for GPU SPH fluid simulation, particles, and grid operations.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-fluids API)
// ============================================================================

#[derive(Clone, Copy, Debug)]
struct Particle {
    position: [f32; 3],
    velocity: [f32; 3],
    force: [f32; 3],
    density: f32,
    pressure: f32,
    mass: f32,
}

impl Particle {
    fn new(position: [f32; 3]) -> Self {
        Self {
            position,
            velocity: [0.0, 0.0, 0.0],
            force: [0.0, 0.0, 0.0],
            density: 1000.0, // Water density
            pressure: 0.0,
            mass: 0.02, // 20 grams
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct SimParams {
    smoothing_radius: f32,
    rest_density: f32,
    stiffness: f32,
    viscosity: f32,
    gravity: [f32; 3],
    time_step: f32,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            smoothing_radius: 0.1,
            rest_density: 1000.0,
            stiffness: 1000.0,
            viscosity: 0.01,
            gravity: [0.0, -9.81, 0.0],
            time_step: 0.001,
        }
    }
}

struct SpatialGrid {
    cells: HashMap<(i32, i32, i32), Vec<usize>>,
    cell_size: f32,
}

impl SpatialGrid {
    fn new(cell_size: f32) -> Self {
        Self {
            cells: HashMap::new(),
            cell_size,
        }
    }

    fn cell_coord(&self, pos: &[f32; 3]) -> (i32, i32, i32) {
        (
            (pos[0] / self.cell_size).floor() as i32,
            (pos[1] / self.cell_size).floor() as i32,
            (pos[2] / self.cell_size).floor() as i32,
        )
    }

    fn insert(&mut self, particle_idx: usize, pos: &[f32; 3]) {
        let cell = self.cell_coord(pos);
        self.cells.entry(cell).or_default().push(particle_idx);
    }

    fn query_neighbors(&self, pos: &[f32; 3]) -> Vec<usize> {
        let center = self.cell_coord(pos);
        let mut neighbors = Vec::new();

        for dx in -1..=1 {
            for dy in -1..=1 {
                for dz in -1..=1 {
                    let cell = (center.0 + dx, center.1 + dy, center.2 + dz);
                    if let Some(indices) = self.cells.get(&cell) {
                        neighbors.extend_from_slice(indices);
                    }
                }
            }
        }

        neighbors
    }

    fn clear(&mut self) {
        self.cells.clear();
    }
}

fn generate_particles(count: usize, bounds: [f32; 3]) -> Vec<Particle> {
    (0..count)
        .map(|i| {
            let x = (i % 100) as f32 * bounds[0] / 100.0;
            let y = ((i / 100) % 100) as f32 * bounds[1] / 100.0;
            let z = (i / 10000) as f32 * bounds[2] / 100.0;
            Particle::new([x, y, z])
        })
        .collect()
}

// SPH Kernel functions
fn poly6_kernel(r_sq: f32, h: f32) -> f32 {
    let h_sq = h * h;
    if r_sq > h_sq {
        return 0.0;
    }
    let diff = h_sq - r_sq;
    let coeff = 315.0 / (64.0 * std::f32::consts::PI * h.powi(9));
    coeff * diff.powi(3)
}

fn spiky_gradient_kernel(r: f32, h: f32) -> f32 {
    if r > h || r < 0.0001 {
        return 0.0;
    }
    let diff = h - r;
    let coeff = -45.0 / (std::f32::consts::PI * h.powi(6));
    coeff * diff.powi(2) / r
}

fn viscosity_laplacian_kernel(r: f32, h: f32) -> f32 {
    if r > h {
        return 0.0;
    }
    let coeff = 45.0 / (std::f32::consts::PI * h.powi(6));
    coeff * (h - r)
}

// ============================================================================
// CATEGORY 1: PARTICLE OPERATIONS
// ============================================================================

fn bench_particle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluids_adversarial/particle_operations");

    // Test 1: Particle creation
    for count in [1000, 5000, 10000] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("particle_creation", count),
            &count,
            |bencher, &count| {
                bencher.iter(|| {
                    let particles = generate_particles(count, [10.0, 10.0, 10.0]);
                    std_black_box(particles.len())
                });
            },
        );
    }

    // Test 2: Position update
    group.bench_function("position_update_10000", |bencher| {
        let mut particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let dt = 0.001f32;

        bencher.iter(|| {
            for p in &mut particles {
                p.position[0] += p.velocity[0] * dt;
                p.position[1] += p.velocity[1] * dt;
                p.position[2] += p.velocity[2] * dt;
            }
            std_black_box(particles.len())
        });
    });

    // Test 3: Velocity update
    group.bench_function("velocity_update_10000", |bencher| {
        let mut particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let dt = 0.001f32;

        bencher.iter(|| {
            for p in &mut particles {
                // F = ma, so a = F/m, v += a*dt
                let inv_mass = 1.0 / p.mass;
                p.velocity[0] += p.force[0] * inv_mass * dt;
                p.velocity[1] += p.force[1] * inv_mass * dt;
                p.velocity[2] += p.force[2] * inv_mass * dt;
            }
            std_black_box(particles.len())
        });
    });

    // Test 4: Force accumulation
    group.bench_function("force_accumulation_10000", |bencher| {
        let mut particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let gravity = [0.0f32, -9.81, 0.0];

        bencher.iter(|| {
            for p in &mut particles {
                // Reset forces and add gravity
                p.force = [
                    gravity[0] * p.mass,
                    gravity[1] * p.mass,
                    gravity[2] * p.mass,
                ];
            }
            std_black_box(particles.len())
        });
    });

    // Test 5: Boundary collision
    group.bench_function("boundary_collision_10000", |bencher| {
        let mut particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let bounds = [10.0f32, 10.0, 10.0];
        let damping = 0.5f32;

        bencher.iter(|| {
            for p in &mut particles {
                for i in 0..3 {
                    if p.position[i] < 0.0 {
                        p.position[i] = 0.0;
                        p.velocity[i] *= -damping;
                    } else if p.position[i] > bounds[i] {
                        p.position[i] = bounds[i];
                        p.velocity[i] *= -damping;
                    }
                }
            }
            std_black_box(particles.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: SPATIAL HASHING
// ============================================================================

fn bench_spatial_hashing(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluids_adversarial/spatial_hashing");

    // Test 1: Grid construction
    for count in [1000, 5000, 10000] {
        group.throughput(Throughput::Elements(count as u64));

        group.bench_with_input(
            BenchmarkId::new("grid_construction", count),
            &count,
            |bencher, &count| {
                let particles = generate_particles(count, [10.0, 10.0, 10.0]);

                bencher.iter(|| {
                    let mut grid = SpatialGrid::new(0.1);
                    for (i, p) in particles.iter().enumerate() {
                        grid.insert(i, &p.position);
                    }
                    std_black_box(grid.cells.len())
                });
            },
        );
    }

    // Test 2: Neighbor queries
    group.bench_function("neighbor_query_10000", |bencher| {
        let particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let mut grid = SpatialGrid::new(0.1);
        for (i, p) in particles.iter().enumerate() {
            grid.insert(i, &p.position);
        }

        bencher.iter(|| {
            let mut total_neighbors = 0;
            for p in &particles {
                let neighbors = grid.query_neighbors(&p.position);
                total_neighbors += neighbors.len();
            }
            std_black_box(total_neighbors)
        });
    });

    // Test 3: Grid rebuild
    group.bench_function("grid_rebuild_5000", |bencher| {
        let mut particles = generate_particles(5000, [10.0, 10.0, 10.0]);
        let mut grid = SpatialGrid::new(0.1);

        bencher.iter(|| {
            // Simulate particle movement
            for p in &mut particles {
                p.position[0] += 0.01;
                p.position[1] -= 0.005;
            }

            // Rebuild grid
            grid.clear();
            for (i, p) in particles.iter().enumerate() {
                grid.insert(i, &p.position);
            }

            std_black_box(grid.cells.len())
        });
    });

    // Test 4: Cell density distribution
    group.bench_function("cell_density_analysis", |bencher| {
        let particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let mut grid = SpatialGrid::new(0.1);
        for (i, p) in particles.iter().enumerate() {
            grid.insert(i, &p.position);
        }

        bencher.iter(|| {
            let densities: Vec<usize> = grid.cells.values().map(|v| v.len()).collect();
            let max_density = densities.iter().max().copied().unwrap_or(0);
            let avg_density = densities.iter().sum::<usize>() as f32 / densities.len() as f32;
            std_black_box((max_density, avg_density as usize))
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: SPH KERNELS
// ============================================================================

fn bench_sph_kernels(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluids_adversarial/sph_kernels");

    // Test 1: Poly6 kernel evaluation
    group.bench_function("poly6_kernel_100000", |bencher| {
        let distances: Vec<f32> = (0..100000).map(|i| i as f32 * 0.001).collect();
        let h = 0.1f32;

        bencher.iter(|| {
            let results: Vec<f32> = distances.iter().map(|&r| poly6_kernel(r * r, h)).collect();
            std_black_box(results.iter().sum::<f32>())
        });
    });

    // Test 2: Spiky gradient kernel
    group.bench_function("spiky_gradient_100000", |bencher| {
        let distances: Vec<f32> = (0..100000).map(|i| i as f32 * 0.001).collect();
        let h = 0.1f32;

        bencher.iter(|| {
            let results: Vec<f32> = distances.iter().map(|&r| spiky_gradient_kernel(r, h)).collect();
            std_black_box(results.iter().sum::<f32>())
        });
    });

    // Test 3: Viscosity laplacian kernel
    group.bench_function("viscosity_laplacian_100000", |bencher| {
        let distances: Vec<f32> = (0..100000).map(|i| i as f32 * 0.001).collect();
        let h = 0.1f32;

        bencher.iter(|| {
            let results: Vec<f32> = distances
                .iter()
                .map(|&r| viscosity_laplacian_kernel(r, h))
                .collect();
            std_black_box(results.iter().sum::<f32>())
        });
    });

    // Test 4: Combined kernel evaluation
    group.bench_function("combined_kernels_10000", |bencher| {
        let distances: Vec<f32> = (0..10000).map(|i| i as f32 * 0.01).collect();
        let h = 0.1f32;

        bencher.iter(|| {
            let results: Vec<(f32, f32, f32)> = distances
                .iter()
                .map(|&r| {
                    (
                        poly6_kernel(r * r, h),
                        spiky_gradient_kernel(r, h),
                        viscosity_laplacian_kernel(r, h),
                    )
                })
                .collect();

            let sums: (f32, f32, f32) = results.iter().fold((0.0, 0.0, 0.0), |acc, &(a, b, c)| {
                (acc.0 + a, acc.1 + b, acc.2 + c)
            });

            std_black_box(sums)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: DENSITY/PRESSURE CALCULATION
// ============================================================================

fn bench_density_pressure(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluids_adversarial/density_pressure");

    // Test 1: Density computation
    group.bench_function("density_computation_5000", |bencher| {
        let mut particles = generate_particles(5000, [5.0, 5.0, 5.0]);
        let mut grid = SpatialGrid::new(0.1);
        for (i, p) in particles.iter().enumerate() {
            grid.insert(i, &p.position);
        }

        let params = SimParams::default();

        bencher.iter(|| {
            for i in 0..particles.len() {
                let neighbors = grid.query_neighbors(&particles[i].position);
                let mut density = 0.0f32;

                for &j in &neighbors {
                    let dx = particles[i].position[0] - particles[j].position[0];
                    let dy = particles[i].position[1] - particles[j].position[1];
                    let dz = particles[i].position[2] - particles[j].position[2];
                    let r_sq = dx * dx + dy * dy + dz * dz;

                    density += particles[j].mass * poly6_kernel(r_sq, params.smoothing_radius);
                }

                particles[i].density = density.max(0.001);
            }

            std_black_box(particles.iter().map(|p| p.density).sum::<f32>())
        });
    });

    // Test 2: Pressure computation
    group.bench_function("pressure_computation_5000", |bencher| {
        let mut particles = generate_particles(5000, [5.0, 5.0, 5.0]);
        let params = SimParams::default();

        // Set densities
        for p in &mut particles {
            p.density = params.rest_density * (0.8 + rand_simple(400) as f32 / 1000.0);
        }

        bencher.iter(|| {
            for p in &mut particles {
                // Tait equation of state
                p.pressure = params.stiffness * ((p.density / params.rest_density).powi(7) - 1.0);
            }

            std_black_box(particles.iter().map(|p| p.pressure).sum::<f32>())
        });
    });

    // Test 3: Pressure force computation
    group.bench_function("pressure_force_2000", |bencher| {
        let mut particles = generate_particles(2000, [3.0, 3.0, 3.0]);
        let mut grid = SpatialGrid::new(0.1);
        for (i, p) in particles.iter().enumerate() {
            grid.insert(i, &p.position);
        }

        let params = SimParams::default();

        // Initialize densities and pressures
        for p in &mut particles {
            p.density = params.rest_density;
            p.pressure = params.stiffness * 0.1;
        }

        bencher.iter(|| {
            let mut forces: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; particles.len()];

            for i in 0..particles.len() {
                let neighbors = grid.query_neighbors(&particles[i].position);

                for &j in &neighbors {
                    if i == j {
                        continue;
                    }

                    let dx = particles[i].position[0] - particles[j].position[0];
                    let dy = particles[i].position[1] - particles[j].position[1];
                    let dz = particles[i].position[2] - particles[j].position[2];
                    let r = (dx * dx + dy * dy + dz * dz).sqrt();

                    if r < 0.0001 {
                        continue;
                    }

                    let kernel = spiky_gradient_kernel(r, params.smoothing_radius);
                    let pressure_term = (particles[i].pressure + particles[j].pressure)
                        / (2.0 * particles[j].density);
                    let force_mag = -particles[j].mass * pressure_term * kernel;

                    forces[i][0] += force_mag * dx / r;
                    forces[i][1] += force_mag * dy / r;
                    forces[i][2] += force_mag * dz / r;
                }
            }

            std_black_box(forces.len())
        });
    });

    // Test 4: Viscosity force computation
    group.bench_function("viscosity_force_2000", |bencher| {
        let mut particles = generate_particles(2000, [3.0, 3.0, 3.0]);
        let mut grid = SpatialGrid::new(0.1);
        for (i, p) in particles.iter().enumerate() {
            grid.insert(i, &p.position);
        }

        let params = SimParams::default();

        // Initialize velocities
        for (i, p) in particles.iter_mut().enumerate() {
            p.velocity = [(i as f32 * 0.001).sin(), 0.0, 0.0];
            p.density = params.rest_density;
        }

        bencher.iter(|| {
            let mut forces: Vec<[f32; 3]> = vec![[0.0, 0.0, 0.0]; particles.len()];

            for i in 0..particles.len() {
                let neighbors = grid.query_neighbors(&particles[i].position);

                for &j in &neighbors {
                    if i == j {
                        continue;
                    }

                    let dx = particles[i].position[0] - particles[j].position[0];
                    let dy = particles[i].position[1] - particles[j].position[1];
                    let dz = particles[i].position[2] - particles[j].position[2];
                    let r = (dx * dx + dy * dy + dz * dz).sqrt();

                    let kernel = viscosity_laplacian_kernel(r, params.smoothing_radius);
                    let vel_diff = [
                        particles[j].velocity[0] - particles[i].velocity[0],
                        particles[j].velocity[1] - particles[i].velocity[1],
                        particles[j].velocity[2] - particles[i].velocity[2],
                    ];

                    let force_factor = params.viscosity * particles[j].mass * kernel / particles[j].density;

                    forces[i][0] += force_factor * vel_diff[0];
                    forces[i][1] += force_factor * vel_diff[1];
                    forces[i][2] += force_factor * vel_diff[2];
                }
            }

            std_black_box(forces.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: FULL SIMULATION STEP
// ============================================================================

fn bench_simulation_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluids_adversarial/simulation_step");

    // Test 1: Single step small system
    group.bench_function("single_step_1000", |bencher| {
        let mut particles = generate_particles(1000, [2.0, 2.0, 2.0]);
        let params = SimParams::default();

        bencher.iter(|| {
            // 1. Build spatial grid
            let mut grid = SpatialGrid::new(params.smoothing_radius);
            for (i, p) in particles.iter().enumerate() {
                grid.insert(i, &p.position);
            }

            // 2. Compute density
            for i in 0..particles.len() {
                let neighbors = grid.query_neighbors(&particles[i].position);
                let mut density = 0.0f32;

                for &j in &neighbors {
                    let dx = particles[i].position[0] - particles[j].position[0];
                    let dy = particles[i].position[1] - particles[j].position[1];
                    let dz = particles[i].position[2] - particles[j].position[2];
                    let r_sq = dx * dx + dy * dy + dz * dz;
                    density += particles[j].mass * poly6_kernel(r_sq, params.smoothing_radius);
                }

                particles[i].density = density.max(0.001);
            }

            // 3. Compute pressure
            for p in &mut particles {
                p.pressure = params.stiffness * ((p.density / params.rest_density).powi(7) - 1.0);
            }

            // 4. Reset forces + gravity
            for p in &mut particles {
                p.force = [
                    params.gravity[0] * p.mass,
                    params.gravity[1] * p.mass,
                    params.gravity[2] * p.mass,
                ];
            }

            // 5. Integration
            for p in &mut particles {
                let inv_mass = 1.0 / p.mass;
                p.velocity[0] += p.force[0] * inv_mass * params.time_step;
                p.velocity[1] += p.force[1] * inv_mass * params.time_step;
                p.velocity[2] += p.force[2] * inv_mass * params.time_step;

                p.position[0] += p.velocity[0] * params.time_step;
                p.position[1] += p.velocity[1] * params.time_step;
                p.position[2] += p.velocity[2] * params.time_step;
            }

            std_black_box(particles.len())
        });
    });

    // Test 2: Multiple steps
    group.bench_function("multi_step_500_x10", |bencher| {
        let mut particles = generate_particles(500, [2.0, 2.0, 2.0]);
        let params = SimParams::default();

        bencher.iter(|| {
            for _ in 0..10 {
                // Build grid
                let mut grid = SpatialGrid::new(params.smoothing_radius);
                for (i, p) in particles.iter().enumerate() {
                    grid.insert(i, &p.position);
                }

                // Update velocities with simple Euler
                for p in &mut particles {
                    p.velocity[1] += params.gravity[1] * params.time_step;
                }

                // Update positions
                for p in &mut particles {
                    p.position[0] += p.velocity[0] * params.time_step;
                    p.position[1] += p.velocity[1] * params.time_step;
                    p.position[2] += p.velocity[2] * params.time_step;

                    // Simple boundary
                    if p.position[1] < 0.0 {
                        p.position[1] = 0.0;
                        p.velocity[1] *= -0.5;
                    }
                }
            }

            std_black_box(particles.len())
        });
    });

    // Test 3: Adaptive time stepping
    group.bench_function("adaptive_timestep_1000", |bencher| {
        let particles = generate_particles(1000, [2.0, 2.0, 2.0]);
        let params = SimParams::default();

        bencher.iter(|| {
            // Find max velocity
            let max_vel = particles
                .iter()
                .map(|p| {
                    (p.velocity[0].powi(2) + p.velocity[1].powi(2) + p.velocity[2].powi(2)).sqrt()
                })
                .fold(0.0f32, f32::max);

            // CFL condition: dt = C * h / v_max
            let cfl = 0.4f32;
            let adaptive_dt = if max_vel > 0.0001 {
                (cfl * params.smoothing_radius / max_vel).min(params.time_step)
            } else {
                params.time_step
            };

            std_black_box(adaptive_dt)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: GPU DATA PREPARATION
// ============================================================================

fn bench_gpu_data_prep(c: &mut Criterion) {
    let mut group = c.benchmark_group("fluids_adversarial/gpu_data_prep");

    // Test 1: Position buffer packing
    group.bench_function("position_buffer_10000", |bencher| {
        let particles = generate_particles(10000, [10.0, 10.0, 10.0]);

        bencher.iter(|| {
            // Pack positions into contiguous buffer (vec4 aligned)
            let buffer: Vec<[f32; 4]> = particles
                .iter()
                .map(|p| [p.position[0], p.position[1], p.position[2], 1.0])
                .collect();

            std_black_box(buffer.len())
        });
    });

    // Test 2: Velocity buffer packing
    group.bench_function("velocity_buffer_10000", |bencher| {
        let particles = generate_particles(10000, [10.0, 10.0, 10.0]);

        bencher.iter(|| {
            let buffer: Vec<[f32; 4]> = particles
                .iter()
                .map(|p| [p.velocity[0], p.velocity[1], p.velocity[2], 0.0])
                .collect();

            std_black_box(buffer.len())
        });
    });

    // Test 3: Combined particle data packing
    group.bench_function("combined_buffer_10000", |bencher| {
        let particles = generate_particles(10000, [10.0, 10.0, 10.0]);

        bencher.iter(|| {
            // Pack all particle data into single buffer
            // Format: pos.xyz, density, vel.xyz, pressure
            let buffer: Vec<[f32; 8]> = particles
                .iter()
                .map(|p| {
                    [
                        p.position[0],
                        p.position[1],
                        p.position[2],
                        p.density,
                        p.velocity[0],
                        p.velocity[1],
                        p.velocity[2],
                        p.pressure,
                    ]
                })
                .collect();

            std_black_box(buffer.len())
        });
    });

    // Test 4: Cell index buffer generation
    group.bench_function("cell_index_buffer_10000", |bencher| {
        let particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let cell_size = 0.1f32;

        bencher.iter(|| {
            let cell_indices: Vec<u32> = particles
                .iter()
                .map(|p| {
                    let cx = (p.position[0] / cell_size).floor() as u32;
                    let cy = (p.position[1] / cell_size).floor() as u32;
                    let cz = (p.position[2] / cell_size).floor() as u32;

                    // Morton code for spatial coherence
                    (cx & 0x3FF) | ((cy & 0x3FF) << 10) | ((cz & 0x3FF) << 20)
                })
                .collect();

            std_black_box(cell_indices.len())
        });
    });

    // Test 5: Uniform buffer preparation
    group.bench_function("uniform_buffer_prep", |bencher| {
        let params = SimParams::default();

        bencher.iter(|| {
            // Pack simulation parameters for GPU
            let uniform_data: [f32; 16] = [
                params.smoothing_radius,
                params.rest_density,
                params.stiffness,
                params.viscosity,
                params.gravity[0],
                params.gravity[1],
                params.gravity[2],
                params.time_step,
                0.0, 0.0, 0.0, 0.0, // Padding
                0.0, 0.0, 0.0, 0.0, // More padding for alignment
            ];

            std_black_box(uniform_data)
        });
    });

    // Test 6: Sort preparation for GPU
    group.bench_function("sort_prep_10000", |bencher| {
        let particles = generate_particles(10000, [10.0, 10.0, 10.0]);
        let cell_size = 0.1f32;

        bencher.iter(|| {
            // Create (cell_index, particle_index) pairs for sorting
            let mut sort_keys: Vec<(u32, u32)> = particles
                .iter()
                .enumerate()
                .map(|(i, p)| {
                    let cx = (p.position[0] / cell_size).floor() as u32;
                    let cy = (p.position[1] / cell_size).floor() as u32;
                    let cz = (p.position[2] / cell_size).floor() as u32;
                    let cell = (cx & 0x3FF) | ((cy & 0x3FF) << 10) | ((cz & 0x3FF) << 20);
                    (cell, i as u32)
                })
                .collect();

            sort_keys.sort_by_key(|(cell, _)| *cell);

            std_black_box(sort_keys.len())
        });
    });

    group.finish();
}

// Simple deterministic random
fn rand_simple(max: u64) -> u64 {
    static mut SEED: u64 = 12345;
    unsafe {
        SEED = SEED.wrapping_mul(1103515245).wrapping_add(12345);
        SEED % max
    }
}

criterion_group!(
    benches,
    bench_particle_operations,
    bench_spatial_hashing,
    bench_sph_kernels,
    bench_density_pressure,
    bench_simulation_step,
    bench_gpu_data_prep,
);

criterion_main!(benches);
