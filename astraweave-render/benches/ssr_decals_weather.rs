//! SSR, Decals, and Weather Effects Benchmark Suite
//!
//! Comprehensive benchmarks for screen-space reflections, deferred decal rendering,
//! and weather particle effects. All benchmarks are CPU-side algorithm simulations
//! that validate algorithmic complexity; actual GPU execution would be 10-100× faster.
//!
//! # Categories
//! 
//! ## Screen-Space Reflections (SSR)
//! - Ray marching: Per-pixel raycast against depth buffer
//! - Hit detection: Binary search refinement
//! - Cone tracing: Roughness-based filtering
//!
//! ## Decal System
//! - Projection matrix: World-to-decal space transform
//! - Atlas UV lookup: Grid-based texture coordinate calculation
//! - Fade update: Per-decal lifetime management
//! - GPU buffer preparation: CPU-to-GPU data marshalling
//!
//! ## Weather Effects
//! - Particle spawning: Random distribution across bounds
//! - Particle update: Position integration with lifetime
//! - Instance matrix generation: Per-particle transform calculation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::hint::black_box;
use std::f32::consts::PI;

// ============================================================================
// SSR (Screen-Space Reflections) Benchmarks
// ============================================================================

/// Ray marching step for SSR
#[derive(Clone, Copy)]
struct SsrRay {
    origin: [f32; 3],
    direction: [f32; 3],
    max_distance: f32,
}

impl SsrRay {
    fn new(origin: [f32; 3], direction: [f32; 3]) -> Self {
        Self {
            origin,
            direction,
            max_distance: 100.0,
        }
    }
}

/// Mock depth buffer (1D representation of 2D depth)
struct MockDepthBuffer {
    width: usize,
    height: usize,
    data: Vec<f32>,
}

impl MockDepthBuffer {
    fn new(width: usize, height: usize) -> Self {
        // Generate depth values (far to near gradient with some variation)
        let mut data = Vec::with_capacity(width * height);
        for y in 0..height {
            for x in 0..width {
                let base_depth = 1.0 - (y as f32 / height as f32) * 0.8;
                let variation = ((x as f32 * 0.1).sin() * 0.02).abs();
                data.push(base_depth - variation);
            }
        }
        Self {
            width,
            height,
            data,
        }
    }

    fn sample(&self, x: f32, y: f32) -> f32 {
        let ix = ((x * self.width as f32) as usize).min(self.width - 1);
        let iy = ((y * self.height as f32) as usize).min(self.height - 1);
        self.data[iy * self.width + ix]
    }
}

/// Ray march against depth buffer (SSR core algorithm)
fn ssr_ray_march(
    ray: &SsrRay,
    depth_buffer: &MockDepthBuffer,
    max_steps: u32,
    step_size: f32,
) -> Option<[f32; 2]> {
    let mut pos = ray.origin;
    let mut hit_uv = None;

    for _ in 0..max_steps {
        // March ray
        pos[0] += ray.direction[0] * step_size;
        pos[1] += ray.direction[1] * step_size;
        pos[2] += ray.direction[2] * step_size;

        // Project to screen space (simplified)
        let screen_x = (pos[0] + 1.0) * 0.5;
        let screen_y = (pos[1] + 1.0) * 0.5;

        if screen_x < 0.0 || screen_x > 1.0 || screen_y < 0.0 || screen_y > 1.0 {
            break;
        }

        // Sample depth
        let sampled_depth = depth_buffer.sample(screen_x, screen_y);
        let ray_depth = pos[2];

        // Check for intersection
        if ray_depth > sampled_depth && ray_depth < sampled_depth + 0.05 {
            hit_uv = Some([screen_x, screen_y]);
            break;
        }
    }

    hit_uv
}

/// Binary search refinement for SSR hit
fn ssr_binary_search_refine(
    ray: &SsrRay,
    depth_buffer: &MockDepthBuffer,
    initial_t: f32,
    iterations: u32,
) -> [f32; 2] {
    let mut t_min = initial_t - 0.1;
    let mut t_max = initial_t + 0.1;

    for _ in 0..iterations {
        let t_mid = (t_min + t_max) * 0.5;
        let pos = [
            ray.origin[0] + ray.direction[0] * t_mid,
            ray.origin[1] + ray.direction[1] * t_mid,
            ray.origin[2] + ray.direction[2] * t_mid,
        ];

        let screen_x = ((pos[0] + 1.0) * 0.5).clamp(0.0, 1.0);
        let screen_y = ((pos[1] + 1.0) * 0.5).clamp(0.0, 1.0);
        let sampled_depth = depth_buffer.sample(screen_x, screen_y);

        if pos[2] > sampled_depth {
            t_max = t_mid;
        } else {
            t_min = t_mid;
        }
    }

    let t_final = (t_min + t_max) * 0.5;
    let pos_final = [
        ray.origin[0] + ray.direction[0] * t_final,
        ray.origin[1] + ray.direction[1] * t_final,
        ray.origin[2] + ray.direction[2] * t_final,
    ];

    [
        ((pos_final[0] + 1.0) * 0.5).clamp(0.0, 1.0),
        ((pos_final[1] + 1.0) * 0.5).clamp(0.0, 1.0),
    ]
}

/// Cone tracing for rough reflections (prefiltered)
fn ssr_cone_trace(
    ray: &SsrRay,
    depth_buffer: &MockDepthBuffer,
    roughness: f32,
    _mip_levels: u32,
) -> [f32; 4] {
    // Larger roughness = wider cone = more samples averaged
    let sample_count = ((roughness * 8.0) as u32 + 1).min(16);
    let mut accumulated_color = [0.0f32; 4];

    for i in 0..sample_count {
        // Jitter direction based on roughness
        let angle = (i as f32 / sample_count as f32) * PI * 2.0;
        let jitter = roughness * 0.1;
        let jittered_dir = [
            ray.direction[0] + jitter * angle.cos(),
            ray.direction[1] + jitter * angle.sin(),
            ray.direction[2],
        ];
        let len = (jittered_dir[0].powi(2) + jittered_dir[1].powi(2) + jittered_dir[2].powi(2))
            .sqrt()
            .max(0.001);
        let norm_dir = [
            jittered_dir[0] / len,
            jittered_dir[1] / len,
            jittered_dir[2] / len,
        ];

        let jittered_ray = SsrRay {
            origin: ray.origin,
            direction: norm_dir,
            max_distance: ray.max_distance,
        };

        if let Some(uv) = ssr_ray_march(&jittered_ray, depth_buffer, 32, 0.05) {
            // Sample color (mock - use UV as color)
            accumulated_color[0] += uv[0];
            accumulated_color[1] += uv[1];
            accumulated_color[2] += 0.5;
            accumulated_color[3] += 1.0;
        }
    }

    let weight = accumulated_color[3].max(1.0);
    [
        accumulated_color[0] / weight,
        accumulated_color[1] / weight,
        accumulated_color[2] / weight,
        accumulated_color[3] / sample_count as f32,
    ]
}

// ============================================================================
// Decal System Benchmarks
// ============================================================================

/// GPU decal representation (matches astraweave-render)
#[repr(C)]
#[derive(Clone, Copy)]
struct GpuDecal {
    inv_projection: [[f32; 4]; 4],
    albedo_tint: [f32; 4],
    params: [f32; 4],
    atlas_uv: [f32; 4],
}

impl GpuDecal {
    fn size() -> usize {
        std::mem::size_of::<Self>()
    }
}

/// CPU decal definition
struct Decal {
    position: [f32; 3],
    rotation: [f32; 4], // quaternion
    scale: [f32; 3],
    albedo_tint: [f32; 4],
    normal_strength: f32,
    roughness: f32,
    metallic: f32,
    blend_mode: u32,
    atlas_uv: ([f32; 2], [f32; 2]),
    fade_duration: f32,
    fade_time: f32,
}

impl Decal {
    fn new(position: [f32; 3], scale: [f32; 3], atlas_index: u32, grid_size: u32) -> Self {
        let cell_size = 1.0 / grid_size as f32;
        let atlas_x = (atlas_index % grid_size) as f32 * cell_size;
        let atlas_y = (atlas_index / grid_size) as f32 * cell_size;

        Self {
            position,
            rotation: [0.0, 0.0, 0.0, 1.0],
            scale,
            albedo_tint: [1.0, 1.0, 1.0, 1.0],
            normal_strength: 1.0,
            roughness: 0.5,
            metallic: 0.0,
            blend_mode: 2, // AlphaBlend
            atlas_uv: ([atlas_x, atlas_y], [cell_size, cell_size]),
            fade_duration: 0.0,
            fade_time: 0.0,
        }
    }

    fn new_fading(position: [f32; 3], fade_duration: f32) -> Self {
        let mut decal = Self::new(position, [1.0, 1.0, 1.0], 0, 4);
        decal.fade_duration = fade_duration;
        decal
    }

    /// Update fade and return if still alive
    fn update(&mut self, dt: f32) -> bool {
        if self.fade_duration > 0.0 {
            self.fade_time += dt;
            if self.fade_time >= self.fade_duration {
                return false;
            }
            let fade_alpha = 1.0 - (self.fade_time / self.fade_duration);
            self.albedo_tint[3] = fade_alpha;
        }
        true
    }

    /// Convert to GPU representation
    fn to_gpu(&self) -> GpuDecal {
        // Build transformation matrix components
        let qx = self.rotation[0];
        let qy = self.rotation[1];
        let qz = self.rotation[2];
        let qw = self.rotation[3];

        // Rotation matrix from quaternion
        let r00 = 1.0 - 2.0 * (qy * qy + qz * qz);
        let r01 = 2.0 * (qx * qy - qz * qw);
        let r02 = 2.0 * (qx * qz + qy * qw);
        let r10 = 2.0 * (qx * qy + qz * qw);
        let r11 = 1.0 - 2.0 * (qx * qx + qz * qz);
        let r12 = 2.0 * (qy * qz - qx * qw);
        let r20 = 2.0 * (qx * qz - qy * qw);
        let r21 = 2.0 * (qy * qz + qx * qw);
        let r22 = 1.0 - 2.0 * (qx * qx + qy * qy);

        // Scale and position
        let sx = self.scale[0];
        let sy = self.scale[1];
        let sz = self.scale[2];
        let tx = self.position[0];
        let ty = self.position[1];
        let tz = self.position[2];

        // Construct transformation matrix
        let transform = [
            [r00 * sx, r01 * sy, r02 * sz, tx],
            [r10 * sx, r11 * sy, r12 * sz, ty],
            [r20 * sx, r21 * sy, r22 * sz, tz],
            [0.0, 0.0, 0.0, 1.0],
        ];

        // Compute inverse (for simple scale + translation + identity rotation)
        // For identity rotation quaternion, inverse is just negated translation and reciprocal scale
        let inv_sx = 1.0 / sx;
        let inv_sy = 1.0 / sy;
        let inv_sz = 1.0 / sz;

        let inv_projection = [
            [r00 * inv_sx, r10 * inv_sx, r20 * inv_sx, 0.0],
            [r01 * inv_sy, r11 * inv_sy, r21 * inv_sy, 0.0],
            [r02 * inv_sz, r12 * inv_sz, r22 * inv_sz, 0.0],
            [
                -(transform[0][3] * inv_sx),
                -(transform[1][3] * inv_sy),
                -(transform[2][3] * inv_sz),
                1.0,
            ],
        ];

        GpuDecal {
            inv_projection,
            albedo_tint: self.albedo_tint,
            params: [
                self.normal_strength,
                self.roughness,
                self.metallic,
                self.blend_mode as f32,
            ],
            atlas_uv: [
                self.atlas_uv.0[0],
                self.atlas_uv.0[1],
                self.atlas_uv.1[0],
                self.atlas_uv.1[1],
            ],
        }
    }
}

/// Atlas UV coordinate lookup
fn atlas_get_uv(atlas_x: u32, atlas_y: u32, grid_size: u32) -> ([f32; 2], [f32; 2]) {
    let cell_size = 1.0 / grid_size as f32;
    let offset = [atlas_x as f32 * cell_size, atlas_y as f32 * cell_size];
    let scale = [cell_size, cell_size];
    (offset, scale)
}

// ============================================================================
// Weather Effects Benchmarks
// ============================================================================

/// Weather particle
#[derive(Clone, Copy)]
struct WeatherParticle {
    pos: [f32; 3],
    vel: [f32; 3],
    life: f32,
    color: [f32; 4],
    scale: [f32; 3],
}

/// Instance data for GPU rendering
#[repr(C)]
#[derive(Clone, Copy)]
struct InstanceRaw {
    model: [[f32; 4]; 4],
    normal_matrix: [[f32; 3]; 3],
    color: [f32; 4],
    material_id: u32,
    _padding: [u32; 3],
}

/// LCG random number generator
struct LcgRng {
    state: u64,
}

impl LcgRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next(&mut self) -> f32 {
        self.state = self.state.wrapping_mul(6364136223846793005).wrapping_add(1);
        ((self.state >> 33) as f32) / (u32::MAX as f32)
    }

    fn range(&mut self, min: f32, max: f32) -> f32 {
        min + self.next() * (max - min)
    }
}

/// Spawn rain particle
fn spawn_rain_particle(rng: &mut LcgRng) -> WeatherParticle {
    WeatherParticle {
        pos: [rng.range(-25.0, 25.0), rng.range(8.0, 18.0), rng.range(-25.0, 25.0)],
        vel: [0.0, -20.0, 0.0],
        life: rng.range(0.5, 1.5),
        color: [0.7, 0.8, 1.0, 0.9],
        scale: [0.02, 0.5, 0.02],
    }
}

/// Spawn wind particle
fn spawn_wind_particle(rng: &mut LcgRng) -> WeatherParticle {
    WeatherParticle {
        pos: [rng.range(-25.0, 25.0), rng.range(0.5, 4.0), rng.range(-25.0, 25.0)],
        vel: [5.0, 0.0, 1.0],
        life: rng.range(1.0, 3.0),
        color: [1.0, 1.0, 1.0, 0.3],
        scale: [0.05, 0.05, 0.8],
    }
}

/// Update weather particle
fn update_particle(particle: &mut WeatherParticle, dt: f32) -> bool {
    particle.life -= dt;
    particle.pos[0] += particle.vel[0] * dt;
    particle.pos[1] += particle.vel[1] * dt;
    particle.pos[2] += particle.vel[2] * dt;
    particle.life > 0.0 && particle.pos[1] > 0.0
}

/// Generate instance matrix for particle
fn particle_to_instance(particle: &WeatherParticle) -> InstanceRaw {
    let sx = particle.scale[0];
    let sy = particle.scale[1];
    let sz = particle.scale[2];
    let tx = particle.pos[0];
    let ty = particle.pos[1];
    let tz = particle.pos[2];

    // Simple scale + translation matrix
    let model = [
        [sx, 0.0, 0.0, 0.0],
        [0.0, sy, 0.0, 0.0],
        [0.0, 0.0, sz, 0.0],
        [tx, ty, tz, 1.0],
    ];

    // Normal matrix for uniform scale is identity (simplified)
    let normal_matrix = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    InstanceRaw {
        model,
        normal_matrix,
        color: particle.color,
        material_id: 0,
        _padding: [0, 0, 0],
    }
}

// ============================================================================
// Benchmark Functions
// ============================================================================

fn bench_ssr(c: &mut Criterion) {
    let mut group = c.benchmark_group("ssr");

    // Ray marching benchmarks
    for steps in [16, 32, 64, 128] {
        let depth_buffer = MockDepthBuffer::new(256, 256);
        let ray = SsrRay::new([0.0, 0.0, 0.5], [0.1, 0.1, -0.98]);

        group.throughput(Throughput::Elements(steps as u64));
        group.bench_with_input(BenchmarkId::new("ray_march", steps), &steps, |b, &steps| {
            b.iter(|| {
                black_box(ssr_ray_march(
                    black_box(&ray),
                    black_box(&depth_buffer),
                    steps,
                    0.05,
                ))
            });
        });
    }

    // Binary search refinement
    for iterations in [4, 8, 16] {
        let depth_buffer = MockDepthBuffer::new(256, 256);
        let ray = SsrRay::new([0.0, 0.0, 0.5], [0.1, 0.1, -0.98]);

        group.bench_with_input(
            BenchmarkId::new("binary_refine", iterations),
            &iterations,
            |b, &iterations| {
                b.iter(|| {
                    black_box(ssr_binary_search_refine(
                        black_box(&ray),
                        black_box(&depth_buffer),
                        0.5,
                        iterations,
                    ))
                });
            },
        );
    }

    // Cone tracing for rough reflections
    for roughness_pct in [0, 25, 50, 100] {
        let depth_buffer = MockDepthBuffer::new(128, 128);
        let ray = SsrRay::new([0.0, 0.0, 0.5], [0.1, 0.1, -0.98]);
        let roughness = roughness_pct as f32 / 100.0;

        group.bench_with_input(
            BenchmarkId::new("cone_trace_r", roughness_pct),
            &roughness,
            |b, &roughness| {
                b.iter(|| {
                    black_box(ssr_cone_trace(
                        black_box(&ray),
                        black_box(&depth_buffer),
                        roughness,
                        5,
                    ))
                });
            },
        );
    }

    // Full-screen SSR simulation (per-pixel)
    for resolution in [(320, 180), (640, 360), (1280, 720)] {
        let (width, height) = resolution;
        let pixel_count = width * height;
        let depth_buffer = MockDepthBuffer::new(width, height);

        group.throughput(Throughput::Elements(pixel_count as u64));
        group.bench_with_input(
            BenchmarkId::new("fullscreen", format!("{}x{}", width, height)),
            &resolution,
            |b, _| {
                b.iter(|| {
                    let mut hits = 0u32;
                    // Sample every 8th pixel (simulation)
                    for y in (0..height).step_by(8) {
                        for x in (0..width).step_by(8) {
                            let nx = (x as f32 / width as f32) * 2.0 - 1.0;
                            let ny = (y as f32 / height as f32) * 2.0 - 1.0;
                            let ray = SsrRay::new([nx, ny, 0.5], [nx * 0.1, ny * 0.1, -0.98]);
                            if ssr_ray_march(&ray, &depth_buffer, 32, 0.05).is_some() {
                                hits += 1;
                            }
                        }
                    }
                    black_box(hits)
                });
            },
        );
    }

    group.finish();
}

fn bench_decals(c: &mut Criterion) {
    let mut group = c.benchmark_group("decals");

    // Single decal to GPU conversion
    group.bench_function("to_gpu_single", |b| {
        let decal = Decal::new([1.0, 2.0, 3.0], [0.5, 0.5, 0.1], 0, 4);
        b.iter(|| black_box(decal.to_gpu()));
    });

    // Batch decal conversion
    for count in [10, 100, 500, 1000] {
        let decals: Vec<Decal> = (0..count)
            .map(|i| {
                Decal::new(
                    [i as f32 * 0.5, 0.0, i as f32 * 0.3],
                    [0.5, 0.5, 0.1],
                    (i % 16) as u32,
                    4,
                )
            })
            .collect();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("to_gpu_batch", count), &decals, |b, decals| {
            b.iter(|| {
                let gpu_decals: Vec<GpuDecal> = decals.iter().map(|d| d.to_gpu()).collect();
                black_box(gpu_decals)
            });
        });
    }

    // Decal fade update
    for count in [100, 500, 1000] {
        let mut decals: Vec<Decal> = (0..count)
            .map(|i| Decal::new_fading([i as f32, 0.0, 0.0], 5.0))
            .collect();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("fade_update", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let mut alive_count = 0;
                    for decal in decals.iter_mut() {
                        if decal.update(black_box(0.016)) {
                            alive_count += 1;
                        }
                    }
                    black_box(alive_count)
                });
            },
        );
    }

    // Atlas UV lookup
    group.bench_function("atlas_uv_lookup", |b| {
        b.iter(|| {
            let mut uvs = [[0.0f32; 4]; 16];
            for i in 0..16 {
                let (offset, scale) = atlas_get_uv(i % 4, i / 4, 4);
                uvs[i as usize] = [offset[0], offset[1], scale[0], scale[1]];
            }
            black_box(uvs)
        });
    });

    // GpuDecal memory layout
    group.bench_function("gpu_decal_size", |b| {
        b.iter(|| black_box(GpuDecal::size()));
    });

    // Full decal system update simulation
    for count in [50, 200, 500] {
        let mut decals: Vec<Decal> = (0..count)
            .map(|i| {
                let mut d = Decal::new(
                    [i as f32 * 0.5, 0.0, i as f32 * 0.3],
                    [0.5, 0.5, 0.1],
                    (i % 16) as u32,
                    4,
                );
                if i % 3 == 0 {
                    d.fade_duration = 5.0;
                }
                d
            })
            .collect();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("full_update", count),
            &count,
            |b, _| {
                b.iter(|| {
                    // Update fades and remove dead decals
                    decals.retain_mut(|d| d.update(black_box(0.016)));
                    // Convert to GPU format
                    let gpu_decals: Vec<GpuDecal> = decals.iter().map(|d| d.to_gpu()).collect();
                    black_box(gpu_decals.len())
                });
            },
        );
    }

    group.finish();
}

fn bench_weather(c: &mut Criterion) {
    let mut group = c.benchmark_group("weather");

    // Particle spawning
    group.bench_function("spawn_rain", |b| {
        let mut rng = LcgRng::new(12345);
        b.iter(|| black_box(spawn_rain_particle(black_box(&mut rng))));
    });

    group.bench_function("spawn_wind", |b| {
        let mut rng = LcgRng::new(12345);
        b.iter(|| black_box(spawn_wind_particle(black_box(&mut rng))));
    });

    // Batch particle spawning
    for count in [100, 500, 1000, 5000] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(BenchmarkId::new("spawn_rain_batch", count), &count, |b, &count| {
            let mut rng = LcgRng::new(12345);
            b.iter(|| {
                let particles: Vec<WeatherParticle> =
                    (0..count).map(|_| spawn_rain_particle(&mut rng)).collect();
                black_box(particles)
            });
        });
    }

    // Particle update
    group.bench_function("update_single", |b| {
        let mut particle = WeatherParticle {
            pos: [0.0, 10.0, 0.0],
            vel: [0.0, -20.0, 0.0],
            life: 1.0,
            color: [1.0; 4],
            scale: [0.1; 3],
        };
        b.iter(|| black_box(update_particle(black_box(&mut particle), 0.016)));
    });

    // Batch particle update
    for count in [100, 500, 1000, 5000] {
        let mut rng = LcgRng::new(12345);
        let mut particles: Vec<WeatherParticle> =
            (0..count).map(|_| spawn_rain_particle(&mut rng)).collect();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("update_batch", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let mut alive = 0;
                    for particle in particles.iter_mut() {
                        if update_particle(particle, black_box(0.016)) {
                            alive += 1;
                        }
                    }
                    black_box(alive)
                });
            },
        );
    }

    // Instance matrix generation
    group.bench_function("to_instance_single", |b| {
        let particle = WeatherParticle {
            pos: [0.0, 10.0, 0.0],
            vel: [0.0, -20.0, 0.0],
            life: 1.0,
            color: [0.7, 0.8, 1.0, 0.9],
            scale: [0.02, 0.5, 0.02],
        };
        b.iter(|| black_box(particle_to_instance(black_box(&particle))));
    });

    // Batch instance generation
    for count in [100, 500, 1000, 5000] {
        let mut rng = LcgRng::new(12345);
        let particles: Vec<WeatherParticle> =
            (0..count).map(|_| spawn_rain_particle(&mut rng)).collect();

        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("to_instance_batch", count),
            &particles,
            |b, particles| {
                b.iter(|| {
                    let instances: Vec<InstanceRaw> =
                        particles.iter().map(|p| particle_to_instance(p)).collect();
                    black_box(instances)
                });
            },
        );
    }

    // Full weather frame simulation (spawn + update + instance gen)
    for count in [500, 1000, 2000] {
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("full_frame", count),
            &count,
            |b, &max_particles| {
                let mut rng = LcgRng::new(12345);
                let mut particles: Vec<WeatherParticle> = Vec::with_capacity(max_particles);

                b.iter(|| {
                    // Spawn up to max
                    while particles.len() < max_particles {
                        particles.push(spawn_rain_particle(&mut rng));
                    }

                    // Update and cull
                    particles.retain_mut(|p| update_particle(p, 0.016));

                    // Generate instances
                    let instances: Vec<InstanceRaw> =
                        particles.iter().map(|p| particle_to_instance(p)).collect();

                    black_box(instances.len())
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_ssr,
    bench_decals,
    bench_weather,
);

criterion_main!(benches);
