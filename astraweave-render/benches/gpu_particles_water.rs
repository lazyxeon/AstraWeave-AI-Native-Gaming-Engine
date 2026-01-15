//! GPU Particles & Water Rendering Benchmarks
//!
//! Comprehensive benchmark suite for:
//! - GPU particle system operations
//! - Gerstner wave calculations
//! - Water surface tessellation
//! - Particle emission and update algorithms
//!
//! All benchmarks are CPU-side algorithm simulations that validate
//! algorithmic complexity. Actual GPU execution would be 10-100× faster.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::f32::consts::PI;
use std::hint::black_box;

// =============================================================================
// GPU PARTICLE STRUCTURES
// =============================================================================

/// GPU particle (64 bytes, matches gpu_particles.rs)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GpuParticle {
    /// Position (xyz) and lifetime (w)
    position: [f32; 4],
    /// Velocity (xyz) and age (w)
    velocity: [f32; 4],
    /// Color (rgba)
    color: [f32; 4],
    /// Scale (xyz) and mass (w)
    scale: [f32; 4],
}

impl Default for GpuParticle {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 5.0], // 5s lifetime
            velocity: [0.0, 1.0, 0.0, 0.0], // upward, age 0
            color: [1.0, 1.0, 1.0, 1.0],
            scale: [1.0, 1.0, 1.0, 1.0], // mass 1
        }
    }
}

/// Emitter parameters
#[allow(dead_code)]
#[derive(Copy, Clone, Debug)]
struct EmitterParams {
    position: [f32; 3],
    velocity: [f32; 3],
    emission_rate: f32,
    lifetime: f32,
    velocity_randomness: f32,
    delta_time: f32,
    gravity: [f32; 3],
    max_particles: u32,
}

impl Default for EmitterParams {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            velocity: [0.0, 5.0, 0.0],
            emission_rate: 100.0,
            lifetime: 5.0,
            velocity_randomness: 0.2,
            delta_time: 1.0 / 60.0,
            gravity: [0.0, -9.81, 0.0],
            max_particles: 10000,
        }
    }
}

// =============================================================================
// GERSTNER WAVE STRUCTURES
// =============================================================================

/// Single Gerstner wave component
#[derive(Copy, Clone, Debug)]
struct GerstnerWave {
    /// Wave direction (normalized)
    direction: [f32; 2],
    /// Wave amplitude
    amplitude: f32,
    /// Wave frequency
    frequency: f32,
    /// Wave phase speed
    phase: f32,
    /// Steepness factor (0-1)
    steepness: f32,
}

impl GerstnerWave {
    fn new(dir_x: f32, dir_y: f32, amplitude: f32, wavelength: f32, steepness: f32) -> Self {
        let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
        Self {
            direction: [dir_x / len, dir_y / len],
            amplitude,
            frequency: 2.0 * PI / wavelength,
            phase: (9.81 * 2.0 * PI / wavelength).sqrt(),
            steepness,
        }
    }
}

/// Standard 4-wave ocean configuration
fn standard_ocean_waves() -> [GerstnerWave; 4] {
    [
        GerstnerWave::new(1.0, 0.0, 0.5, 20.0, 0.5),
        GerstnerWave::new(0.707, 0.707, 0.3, 15.0, 0.4),
        GerstnerWave::new(0.0, 1.0, 0.2, 10.0, 0.3),
        GerstnerWave::new(-0.5, 0.866, 0.15, 8.0, 0.2),
    ]
}

// =============================================================================
// PARTICLE SIMULATION ALGORITHMS
// =============================================================================

/// Simple pseudo-random number generator (matches GPU shader)
fn rand_lcg(seed: u32) -> f32 {
    let mut state = seed;
    state = state.wrapping_mul(1103515245).wrapping_add(12345);
    (state & 0x7FFFFFFF) as f32 / 2147483648.0
}

/// Update a single particle (CPU simulation of GPU compute)
fn update_particle(particle: &mut GpuParticle, params: &EmitterParams, idx: u32) {
    let dt = params.delta_time;

    // Update age
    particle.velocity[3] += dt;

    // Check if particle is dead
    if particle.velocity[3] >= particle.position[3] {
        // Reset particle
        particle.velocity[3] = 0.0;

        // Respawn at emitter position with random velocity
        let r1 = rand_lcg(idx.wrapping_mul(1103515245));
        let r2 = rand_lcg(idx.wrapping_mul(747796405));
        let r3 = rand_lcg(idx.wrapping_mul(2891336453));

        particle.position[0] = params.position[0] + (r1 - 0.5) * 2.0;
        particle.position[1] = params.position[1];
        particle.position[2] = params.position[2] + (r2 - 0.5) * 2.0;

        particle.velocity[0] =
            params.velocity[0] + (r1 - 0.5) * params.velocity_randomness * params.velocity[0];
        particle.velocity[1] =
            params.velocity[1] + (r2 - 0.5) * params.velocity_randomness * params.velocity[1];
        particle.velocity[2] =
            params.velocity[2] + (r3 - 0.5) * params.velocity_randomness * params.velocity[2];

        return;
    }

    // Apply gravity
    let mass = particle.scale[3];
    particle.velocity[0] += params.gravity[0] * mass * dt;
    particle.velocity[1] += params.gravity[1] * mass * dt;
    particle.velocity[2] += params.gravity[2] * mass * dt;

    // Update position
    particle.position[0] += particle.velocity[0] * dt;
    particle.position[1] += particle.velocity[1] * dt;
    particle.position[2] += particle.velocity[2] * dt;
}

/// Emit new particles
fn emit_particles(
    particles: &mut [GpuParticle],
    params: &EmitterParams,
    start_idx: usize,
    count: usize,
) {
    for i in 0..count {
        let idx = (start_idx + i) % particles.len();
        let seed = (idx as u32).wrapping_mul(1103515245);

        let r1 = rand_lcg(seed);
        let r2 = rand_lcg(seed.wrapping_add(1));
        let r3 = rand_lcg(seed.wrapping_add(2));

        particles[idx] = GpuParticle {
            position: [
                params.position[0] + (r1 - 0.5) * 2.0,
                params.position[1],
                params.position[2] + (r2 - 0.5) * 2.0,
                params.lifetime,
            ],
            velocity: [
                params.velocity[0] * (1.0 + (r1 - 0.5) * params.velocity_randomness),
                params.velocity[1] * (1.0 + (r2 - 0.5) * params.velocity_randomness),
                params.velocity[2] * (1.0 + (r3 - 0.5) * params.velocity_randomness),
                0.0, // age
            ],
            color: [1.0, 1.0, 1.0, 1.0],
            scale: [1.0, 1.0, 1.0, 1.0],
        };
    }
}

/// Sort particles by depth for alpha blending
fn sort_particles_by_depth(particles: &mut [GpuParticle], camera_pos: [f32; 3]) {
    particles.sort_by(|a, b| {
        let da = (a.position[0] - camera_pos[0]).powi(2)
            + (a.position[1] - camera_pos[1]).powi(2)
            + (a.position[2] - camera_pos[2]).powi(2);
        let db = (b.position[0] - camera_pos[0]).powi(2)
            + (b.position[1] - camera_pos[1]).powi(2)
            + (b.position[2] - camera_pos[2]).powi(2);
        db.partial_cmp(&da).unwrap_or(std::cmp::Ordering::Equal)
    });
}

// =============================================================================
// GERSTNER WAVE ALGORITHMS
// =============================================================================

/// Calculate Gerstner wave displacement at a point
fn gerstner_displacement(x: f32, z: f32, time: f32, wave: &GerstnerWave) -> [f32; 3] {
    let k = wave.frequency;
    let dot = wave.direction[0] * x + wave.direction[1] * z;
    let phase_offset = k * dot - wave.phase * time;
    let cos_phase = phase_offset.cos();
    let sin_phase = phase_offset.sin();

    let q = wave.steepness / (wave.frequency * wave.amplitude);

    [
        q * wave.amplitude * wave.direction[0] * cos_phase,
        wave.amplitude * sin_phase,
        q * wave.amplitude * wave.direction[1] * cos_phase,
    ]
}

/// Calculate combined 4-wave displacement
fn combined_wave_displacement(x: f32, z: f32, time: f32, waves: &[GerstnerWave; 4]) -> [f32; 3] {
    let mut result = [0.0f32; 3];
    for wave in waves {
        let d = gerstner_displacement(x, z, time, wave);
        result[0] += d[0];
        result[1] += d[1];
        result[2] += d[2];
    }
    result
}

/// Calculate Gerstner wave normal
fn gerstner_normal(x: f32, z: f32, time: f32, wave: &GerstnerWave) -> [f32; 3] {
    let k = wave.frequency;
    let dot = wave.direction[0] * x + wave.direction[1] * z;
    let phase_offset = k * dot - wave.phase * time;
    let cos_phase = phase_offset.cos();
    let sin_phase = phase_offset.sin();

    let wa = wave.frequency * wave.amplitude;
    let s = wave.steepness;

    [
        -wave.direction[0] * wa * cos_phase,
        1.0 - s * wa * sin_phase,
        -wave.direction[1] * wa * cos_phase,
    ]
}

/// Calculate combined 4-wave normal
fn combined_wave_normal(x: f32, z: f32, time: f32, waves: &[GerstnerWave; 4]) -> [f32; 3] {
    let mut result = [0.0f32, 0.0, 0.0];
    for wave in waves {
        let n = gerstner_normal(x, z, time, wave);
        result[0] += n[0];
        result[1] += n[1];
        result[2] += n[2];
    }
    // Normalize
    let len = (result[0] * result[0] + result[1] * result[1] + result[2] * result[2]).sqrt();
    [result[0] / len, result[1] / len, result[2] / len]
}

/// Calculate foam factor based on wave crest
fn calculate_foam(displacement_y: f32, threshold: f32) -> f32 {
    ((displacement_y - threshold) / (1.0 - threshold)).clamp(0.0, 1.0)
}

/// Fresnel reflection coefficient (Schlick approximation)
fn fresnel_schlick(cos_theta: f32, f0: f32) -> f32 {
    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
}

// =============================================================================
// WATER GRID GENERATION
// =============================================================================

/// Generate water grid vertices
fn generate_water_grid(size: f32, subdivisions: u32) -> Vec<[f32; 5]> {
    let mut vertices = Vec::with_capacity(((subdivisions + 1) * (subdivisions + 1)) as usize);
    let step = size / subdivisions as f32;
    let half = size / 2.0;

    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let px = -half + x as f32 * step;
            let pz = -half + z as f32 * step;
            let u = x as f32 / subdivisions as f32;
            let v = z as f32 / subdivisions as f32;
            vertices.push([px, 0.0, pz, u, v]);
        }
    }
    vertices
}

/// Generate water grid indices
fn generate_water_indices(subdivisions: u32) -> Vec<u32> {
    let mut indices = Vec::with_capacity((subdivisions * subdivisions * 6) as usize);
    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let i = z * (subdivisions + 1) + x;
            indices.push(i);
            indices.push(i + subdivisions + 1);
            indices.push(i + 1);
            indices.push(i + 1);
            indices.push(i + subdivisions + 1);
            indices.push(i + subdivisions + 2);
        }
    }
    indices
}

// =============================================================================
// BENCHMARKS: PARTICLE OPERATIONS
// =============================================================================

fn bench_particle_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_particles/update");

    for count in [1000, 10000, 50000, 100000] {
        group.throughput(Throughput::Elements(count as u64));

        let mut particles: Vec<GpuParticle> = (0..count).map(|_| GpuParticle::default()).collect();

        // Initialize with random ages
        for (i, p) in particles.iter_mut().enumerate() {
            p.velocity[3] = rand_lcg(i as u32 * 12345) * 5.0;
            p.position[0] = rand_lcg(i as u32 * 67890) * 10.0 - 5.0;
            p.position[2] = rand_lcg(i as u32 * 11111) * 10.0 - 5.0;
        }

        let params = EmitterParams::default();

        group.bench_with_input(
            BenchmarkId::new("update_particles", count),
            &count,
            |b, _| {
                b.iter(|| {
                    for (i, particle) in particles.iter_mut().enumerate() {
                        update_particle(particle, &params, i as u32);
                    }
                    black_box(particles.len())
                })
            },
        );
    }
    group.finish();
}

fn bench_particle_emission(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_particles/emission");

    for count in [100, 500, 1000, 5000] {
        group.throughput(Throughput::Elements(count as u64));

        let mut particles: Vec<GpuParticle> = (0..10000).map(|_| GpuParticle::default()).collect();
        let params = EmitterParams::default();

        group.bench_with_input(
            BenchmarkId::new("emit_particles", count),
            &count,
            |b, &count| {
                b.iter(|| {
                    emit_particles(&mut particles, &params, 0, count as usize);
                    black_box(count)
                })
            },
        );
    }
    group.finish();
}

fn bench_particle_sorting(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_particles/sorting");

    for count in [1000, 5000, 10000, 50000] {
        group.throughput(Throughput::Elements(count as u64));

        let camera_pos = [0.0, 5.0, -10.0];

        group.bench_with_input(
            BenchmarkId::new("depth_sort", count),
            &count,
            |b, &count| {
                // Create fresh particles each iteration
                let mut particles: Vec<GpuParticle> = (0..count)
                    .map(|i| {
                        let mut p = GpuParticle::default();
                        p.position[0] = rand_lcg(i as u32 * 12345) * 20.0 - 10.0;
                        p.position[1] = rand_lcg(i as u32 * 67890) * 10.0;
                        p.position[2] = rand_lcg(i as u32 * 11111) * 20.0 - 10.0;
                        p
                    })
                    .collect();

                b.iter(|| {
                    sort_particles_by_depth(&mut particles, camera_pos);
                    black_box(particles.len())
                })
            },
        );
    }
    group.finish();
}

fn bench_particle_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_particles/culling");

    // Frustum culling simulation
    for count in [10000, 50000, 100000] {
        group.throughput(Throughput::Elements(count as u64));

        let particles: Vec<GpuParticle> = (0..count)
            .map(|i| {
                let mut p = GpuParticle::default();
                p.position[0] = rand_lcg(i as u32 * 12345) * 100.0 - 50.0;
                p.position[1] = rand_lcg(i as u32 * 67890) * 50.0;
                p.position[2] = rand_lcg(i as u32 * 11111) * 100.0 - 50.0;
                p
            })
            .collect();

        // Simple frustum bounds (box approximation)
        let frustum_min = [-20.0f32, -5.0, -30.0];
        let frustum_max = [20.0f32, 40.0, 30.0];

        group.bench_with_input(BenchmarkId::new("frustum_cull", count), &count, |b, _| {
            b.iter(|| {
                let visible_count = particles
                    .iter()
                    .filter(|p| {
                        p.position[0] >= frustum_min[0]
                            && p.position[0] <= frustum_max[0]
                            && p.position[1] >= frustum_min[1]
                            && p.position[1] <= frustum_max[1]
                            && p.position[2] >= frustum_min[2]
                            && p.position[2] <= frustum_max[2]
                    })
                    .count();
                black_box(visible_count)
            })
        });
    }
    group.finish();
}

// =============================================================================
// BENCHMARKS: GERSTNER WAVES
// =============================================================================

fn bench_gerstner_single_wave(c: &mut Criterion) {
    let mut group = c.benchmark_group("water/gerstner_single");

    let wave = GerstnerWave::new(1.0, 0.0, 0.5, 20.0, 0.5);
    let time = 1.5f32;

    group.bench_function("displacement", |b| {
        b.iter(|| {
            let d = gerstner_displacement(black_box(5.0), black_box(3.0), time, &wave);
            black_box(d)
        })
    });

    group.bench_function("normal", |b| {
        b.iter(|| {
            let n = gerstner_normal(black_box(5.0), black_box(3.0), time, &wave);
            black_box(n)
        })
    });

    group.finish();
}

fn bench_gerstner_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("water/gerstner_combined");

    let waves = standard_ocean_waves();
    let time = 1.5f32;

    group.bench_function("displacement_4waves", |b| {
        b.iter(|| {
            let d = combined_wave_displacement(black_box(5.0), black_box(3.0), time, &waves);
            black_box(d)
        })
    });

    group.bench_function("normal_4waves", |b| {
        b.iter(|| {
            let n = combined_wave_normal(black_box(5.0), black_box(3.0), time, &waves);
            black_box(n)
        })
    });

    group.bench_function("foam_calculation", |b| {
        b.iter(|| {
            let d = combined_wave_displacement(black_box(5.0), black_box(3.0), time, &waves);
            let foam = calculate_foam(d[1], 0.6);
            black_box(foam)
        })
    });

    group.bench_function("fresnel", |b| {
        b.iter(|| {
            let f = fresnel_schlick(black_box(0.5), 0.02);
            black_box(f)
        })
    });

    group.finish();
}

fn bench_water_surface(c: &mut Criterion) {
    let mut group = c.benchmark_group("water/surface");

    let waves = standard_ocean_waves();
    let time = 1.5f32;

    // Batch processing for water grid vertices
    for subdivisions in [32, 64, 128, 256] {
        let vertex_count = (subdivisions + 1) * (subdivisions + 1);
        group.throughput(Throughput::Elements(vertex_count as u64));

        let grid = generate_water_grid(100.0, subdivisions);

        group.bench_with_input(
            BenchmarkId::new("animate_grid", subdivisions),
            &subdivisions,
            |b, _| {
                b.iter(|| {
                    let animated: Vec<[f32; 3]> = grid
                        .iter()
                        .map(|v| combined_wave_displacement(v[0], v[2], time, &waves))
                        .collect();
                    black_box(animated.len())
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("calculate_normals", subdivisions),
            &subdivisions,
            |b, _| {
                b.iter(|| {
                    let normals: Vec<[f32; 3]> = grid
                        .iter()
                        .map(|v| combined_wave_normal(v[0], v[2], time, &waves))
                        .collect();
                    black_box(normals.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_water_grid_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("water/grid_generation");

    for subdivisions in [32, 64, 128, 256] {
        group.throughput(Throughput::Elements(
            ((subdivisions + 1) * (subdivisions + 1)) as u64,
        ));

        group.bench_with_input(
            BenchmarkId::new("generate_vertices", subdivisions),
            &subdivisions,
            |b, &subs| {
                b.iter(|| {
                    let grid = generate_water_grid(100.0, subs);
                    black_box(grid.len())
                })
            },
        );

        group.bench_with_input(
            BenchmarkId::new("generate_indices", subdivisions),
            &subdivisions,
            |b, &subs| {
                b.iter(|| {
                    let indices = generate_water_indices(subs);
                    black_box(indices.len())
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// BENCHMARKS: FULL SIMULATION STEP
// =============================================================================

fn bench_full_particle_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("gpu_particles/full_frame");

    for particle_count in [10000, 50000, 100000] {
        group.throughput(Throughput::Elements(particle_count as u64));

        let mut particles: Vec<GpuParticle> = (0..particle_count)
            .map(|i| {
                let mut p = GpuParticle::default();
                p.velocity[3] = rand_lcg(i as u32 * 12345) * 5.0;
                p.position[0] = rand_lcg(i as u32 * 67890) * 20.0 - 10.0;
                p.position[1] = rand_lcg(i as u32 * 11111) * 10.0;
                p.position[2] = rand_lcg(i as u32 * 22222) * 20.0 - 10.0;
                p
            })
            .collect();

        let params = EmitterParams::default();
        let camera_pos = [0.0, 5.0, -10.0];

        group.bench_with_input(
            BenchmarkId::new("update_cull_sort", particle_count),
            &particle_count,
            |b, _| {
                b.iter(|| {
                    // 1. Update all particles
                    for (i, particle) in particles.iter_mut().enumerate() {
                        update_particle(particle, &params, i as u32);
                    }

                    // 2. Cull invisible (simplified)
                    let visible: Vec<&GpuParticle> = particles
                        .iter()
                        .filter(|p| p.velocity[3] < p.position[3]) // alive check
                        .collect();

                    // 3. Sort by depth
                    let mut to_sort: Vec<GpuParticle> = visible.iter().map(|p| **p).collect();
                    sort_particles_by_depth(&mut to_sort, camera_pos);

                    black_box(to_sort.len())
                })
            },
        );
    }

    group.finish();
}

fn bench_full_water_frame(c: &mut Criterion) {
    let mut group = c.benchmark_group("water/full_frame");

    let waves = standard_ocean_waves();

    for subdivisions in [64, 128, 256] {
        let vertex_count = (subdivisions + 1) * (subdivisions + 1);
        group.throughput(Throughput::Elements(vertex_count as u64));

        let grid = generate_water_grid(100.0, subdivisions);

        group.bench_with_input(
            BenchmarkId::new("animate_full", subdivisions),
            &subdivisions,
            |b, _| {
                b.iter(|| {
                    let time = 1.5f32; // Would increment in real usage

                    // Full vertex animation
                    let animated: Vec<([f32; 3], [f32; 3], f32)> = grid
                        .iter()
                        .map(|v| {
                            let pos = combined_wave_displacement(v[0], v[2], time, &waves);
                            let normal = combined_wave_normal(v[0], v[2], time, &waves);
                            let foam = calculate_foam(pos[1], 0.6);
                            (pos, normal, foam)
                        })
                        .collect();

                    black_box(animated.len())
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// CRITERION GROUPS
// =============================================================================

criterion_group!(
    particle_benches,
    bench_particle_update,
    bench_particle_emission,
    bench_particle_sorting,
    bench_particle_culling,
    bench_full_particle_frame,
);

criterion_group!(
    water_benches,
    bench_gerstner_single_wave,
    bench_gerstner_combined,
    bench_water_surface,
    bench_water_grid_generation,
    bench_full_water_frame,
);

criterion_main!(particle_benches, water_benches);
