//! Clustered MegaLights & GPU Residency Benchmarks
//!
//! Benchmarks for:
//! 1. Clustered Light Culling (CPU simulation of GPU algorithm)
//!    - Light-cluster intersection tests
//!    - Prefix sum for offset calculation
//!    - Cluster grid operations
//!    - Light binning at various scales
//!
//! 2. GPU Residency Management
//!    - Asset loading/unloading
//!    - LRU eviction strategies
//!    - Memory budget enforcement
//!    - Hot reload handling
//!
//! Run: cargo bench -p astraweave-render --bench clustered_megalights_residency

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::hint::black_box;

// ============================================================================
// MegaLights Data Structures (CPU simulation)
// ============================================================================

/// GPU light representation (32 bytes)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct GpuLight {
    pub position: [f32; 4], // xyz = position, w = radius
    pub color: [f32; 4],    // rgb = color, a = intensity
}

impl GpuLight {
    pub fn new(x: f32, y: f32, z: f32, radius: f32) -> Self {
        Self {
            position: [x, y, z, radius],
            color: [1.0, 1.0, 1.0, 1.0],
        }
    }

    #[inline]
    pub fn pos(&self) -> [f32; 3] {
        [self.position[0], self.position[1], self.position[2]]
    }

    #[inline]
    pub fn radius(&self) -> f32 {
        self.position[3]
    }
}

/// Cluster AABB (32 bytes, 16-byte aligned)
#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ClusterBounds {
    pub min_pos: [f32; 3],
    pub _pad1: f32,
    pub max_pos: [f32; 3],
    pub _pad2: f32,
}

impl ClusterBounds {
    pub fn new(min: [f32; 3], max: [f32; 3]) -> Self {
        Self {
            min_pos: min,
            _pad1: 0.0,
            max_pos: max,
            _pad2: 0.0,
        }
    }

    /// Test sphere-AABB intersection (light culling)
    #[inline]
    pub fn intersects_sphere(&self, center: [f32; 3], radius: f32) -> bool {
        let mut dist_sq = 0.0f32;
        for i in 0..3 {
            let v = center[i];
            if v < self.min_pos[i] {
                let d = self.min_pos[i] - v;
                dist_sq += d * d;
            } else if v > self.max_pos[i] {
                let d = v - self.max_pos[i];
                dist_sq += d * d;
            }
        }
        dist_sq <= radius * radius
    }
}

/// Cluster grid for light culling
pub struct ClusterGrid {
    pub dims: (u32, u32, u32),
    pub bounds: Vec<ClusterBounds>,
    pub world_min: [f32; 3],
    pub world_max: [f32; 3],
}

impl ClusterGrid {
    pub fn new(dims: (u32, u32, u32), world_min: [f32; 3], world_max: [f32; 3]) -> Self {
        let total = (dims.0 * dims.1 * dims.2) as usize;
        let mut bounds = Vec::with_capacity(total);

        let cell_size = [
            (world_max[0] - world_min[0]) / dims.0 as f32,
            (world_max[1] - world_min[1]) / dims.1 as f32,
            (world_max[2] - world_min[2]) / dims.2 as f32,
        ];

        for z in 0..dims.2 {
            for y in 0..dims.1 {
                for x in 0..dims.0 {
                    let min = [
                        world_min[0] + x as f32 * cell_size[0],
                        world_min[1] + y as f32 * cell_size[1],
                        world_min[2] + z as f32 * cell_size[2],
                    ];
                    let max = [
                        min[0] + cell_size[0],
                        min[1] + cell_size[1],
                        min[2] + cell_size[2],
                    ];
                    bounds.push(ClusterBounds::new(min, max));
                }
            }
        }

        Self {
            dims,
            bounds,
            world_min,
            world_max,
        }
    }

    pub fn total_clusters(&self) -> usize {
        (self.dims.0 * self.dims.1 * self.dims.2) as usize
    }

    /// CPU light binning (reference implementation)
    pub fn bin_lights_cpu(&self, lights: &[GpuLight]) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
        let cluster_count = self.total_clusters();

        // Stage 1: Count lights per cluster
        let mut counts: Vec<u32> = vec![0; cluster_count];
        for (cluster_idx, cluster) in self.bounds.iter().enumerate() {
            for light in lights {
                if cluster.intersects_sphere(light.pos(), light.radius()) {
                    counts[cluster_idx] += 1;
                }
            }
        }

        // Stage 2: Prefix sum (exclusive scan)
        let mut offsets: Vec<u32> = vec![0; cluster_count + 1];
        for i in 0..cluster_count {
            offsets[i + 1] = offsets[i] + counts[i];
        }

        // Stage 3: Write light indices
        let total_indices = offsets[cluster_count] as usize;
        let mut indices: Vec<u32> = vec![0; total_indices];
        let mut write_pos: Vec<u32> = offsets[..cluster_count].to_vec();

        for (cluster_idx, cluster) in self.bounds.iter().enumerate() {
            for (light_idx, light) in lights.iter().enumerate() {
                if cluster.intersects_sphere(light.pos(), light.radius()) {
                    let pos = write_pos[cluster_idx] as usize;
                    if pos < indices.len() {
                        indices[pos] = light_idx as u32;
                        write_pos[cluster_idx] += 1;
                    }
                }
            }
        }

        (counts, offsets, indices)
    }
}

/// Blelloch prefix sum (GPU-style parallel scan)
pub fn prefix_sum_blelloch(input: &[u32]) -> Vec<u32> {
    let n = input.len();
    if n == 0 {
        return vec![];
    }

    // Pad to power of 2
    let padded_len = n.next_power_of_two();
    let mut data: Vec<u32> = input.to_vec();
    data.resize(padded_len, 0);

    // Up-sweep (reduce)
    let mut offset = 1;
    while offset < padded_len {
        let step = offset * 2;
        for i in (0..padded_len).step_by(step) {
            if i + offset < padded_len {
                data[i + step - 1] += data[i + offset - 1];
            }
        }
        offset *= 2;
    }

    // Clear last element
    data[padded_len - 1] = 0;

    // Down-sweep
    offset = padded_len / 2;
    while offset > 0 {
        let step = offset * 2;
        for i in (0..padded_len).step_by(step) {
            if i + offset < padded_len {
                let temp = data[i + offset - 1];
                data[i + offset - 1] = data[i + step - 1];
                data[i + step - 1] += temp;
            }
        }
        offset /= 2;
    }

    data.truncate(n);
    data
}

/// Sequential prefix sum (baseline)
pub fn prefix_sum_sequential(input: &[u32]) -> Vec<u32> {
    let mut output = vec![0u32; input.len() + 1];
    for i in 0..input.len() {
        output[i + 1] = output[i] + input[i];
    }
    output
}

// ============================================================================
// Residency Management (CPU simulation)
// ============================================================================

#[derive(Clone, Debug)]
pub struct AssetInfo {
    pub guid: String,
    pub memory_mb: usize,
    pub last_used_frame: u64,
    pub priority: u32, // Higher = more important
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct EvictionCandidate {
    guid: String,
    score: u64, // Lower = evict first
}

impl Ord for EvictionCandidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Min-heap: lower score evicted first
        other.score.cmp(&self.score)
    }
}

impl PartialOrd for EvictionCandidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub struct ResidencyManager {
    loaded_assets: HashMap<String, AssetInfo>,
    lru_queue: VecDeque<String>,
    max_memory_mb: usize,
    current_memory_mb: usize,
    current_frame: u64,
}

impl ResidencyManager {
    pub fn new(max_memory_mb: usize) -> Self {
        Self {
            loaded_assets: HashMap::new(),
            lru_queue: VecDeque::new(),
            max_memory_mb,
            current_memory_mb: 0,
            current_frame: 0,
        }
    }

    pub fn load_asset(&mut self, guid: &str, memory_mb: usize, priority: u32) -> bool {
        if self.loaded_assets.contains_key(guid) {
            self.touch_asset(guid);
            return true;
        }

        // Evict if necessary
        while self.current_memory_mb + memory_mb > self.max_memory_mb {
            if !self.evict_lru() {
                return false; // Can't evict anything
            }
        }

        let info = AssetInfo {
            guid: guid.to_string(),
            memory_mb,
            last_used_frame: self.current_frame,
            priority,
        };

        self.loaded_assets.insert(guid.to_string(), info);
        self.lru_queue.push_back(guid.to_string());
        self.current_memory_mb += memory_mb;
        true
    }

    pub fn touch_asset(&mut self, guid: &str) {
        if let Some(info) = self.loaded_assets.get_mut(guid) {
            info.last_used_frame = self.current_frame;
            // Move to back of LRU
            if let Some(pos) = self.lru_queue.iter().position(|g| g == guid) {
                self.lru_queue.remove(pos);
                self.lru_queue.push_back(guid.to_string());
            }
        }
    }

    pub fn evict_lru(&mut self) -> bool {
        if let Some(guid) = self.lru_queue.pop_front() {
            if let Some(info) = self.loaded_assets.remove(&guid) {
                self.current_memory_mb = self.current_memory_mb.saturating_sub(info.memory_mb);
                return true;
            }
        }
        false
    }

    /// Priority-aware eviction (evict lowest priority first)
    pub fn evict_by_priority(&mut self) -> bool {
        if self.loaded_assets.is_empty() {
            return false;
        }

        // Build min-heap of eviction candidates
        let mut heap: BinaryHeap<EvictionCandidate> = self
            .loaded_assets
            .values()
            .map(|info| {
                // Score: priority * 1000 + recency (lower = evict first)
                let recency = self.current_frame.saturating_sub(info.last_used_frame);
                let score = (info.priority as u64) * 1000 + (1000 - recency.min(1000));
                EvictionCandidate {
                    guid: info.guid.clone(),
                    score,
                }
            })
            .collect();

        if let Some(candidate) = heap.pop() {
            if let Some(info) = self.loaded_assets.remove(&candidate.guid) {
                self.current_memory_mb = self.current_memory_mb.saturating_sub(info.memory_mb);
                self.lru_queue.retain(|g| g != &candidate.guid);
                return true;
            }
        }
        false
    }

    pub fn advance_frame(&mut self) {
        self.current_frame += 1;
    }

    pub fn get_stats(&self) -> (usize, usize, usize) {
        (
            self.loaded_assets.len(),
            self.current_memory_mb,
            self.max_memory_mb,
        )
    }

    /// Simulate hot reload (clear all)
    pub fn hot_reload(&mut self) {
        self.loaded_assets.clear();
        self.lru_queue.clear();
        self.current_memory_mb = 0;
    }
}

// ============================================================================
// Light Intersection Tests
// ============================================================================

/// Sphere-sphere intersection for light merging
#[inline]
pub fn sphere_sphere_intersect(c1: [f32; 3], r1: f32, c2: [f32; 3], r2: f32) -> bool {
    let dx = c1[0] - c2[0];
    let dy = c1[1] - c2[1];
    let dz = c1[2] - c2[2];
    let dist_sq = dx * dx + dy * dy + dz * dz;
    let r_sum = r1 + r2;
    dist_sq <= r_sum * r_sum
}

/// Cone-sphere intersection for spotlight culling
#[inline]
pub fn cone_sphere_intersect(
    cone_apex: [f32; 3],
    cone_dir: [f32; 3],
    cone_angle_cos: f32,
    sphere_center: [f32; 3],
    sphere_radius: f32,
) -> bool {
    // Vector from apex to sphere center
    let to_sphere = [
        sphere_center[0] - cone_apex[0],
        sphere_center[1] - cone_apex[1],
        sphere_center[2] - cone_apex[2],
    ];

    // Distance along cone axis
    let dist_along =
        to_sphere[0] * cone_dir[0] + to_sphere[1] * cone_dir[1] + to_sphere[2] * cone_dir[2];

    if dist_along < -sphere_radius {
        return false; // Behind cone
    }

    // Distance from cone axis
    let dist_from_axis_sq =
        to_sphere[0] * to_sphere[0] + to_sphere[1] * to_sphere[1] + to_sphere[2] * to_sphere[2]
            - dist_along * dist_along;

    // Cone radius at this distance
    let sin_angle = (1.0 - cone_angle_cos * cone_angle_cos).sqrt();
    let cone_radius_at_dist = dist_along.max(0.0) * sin_angle / cone_angle_cos;

    let combined_radius = cone_radius_at_dist + sphere_radius;
    dist_from_axis_sq <= combined_radius * combined_radius
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_light_intersection(c: &mut Criterion) {
    let mut group = c.benchmark_group("Light_Intersection");

    // Sphere-AABB (cluster culling)
    let cluster = ClusterBounds::new([0.0, 0.0, 0.0], [10.0, 10.0, 10.0]);

    group.bench_function("sphere_aabb_hit", |b| {
        let center = [5.0, 5.0, 5.0];
        let radius = 3.0;
        b.iter(|| black_box(cluster.intersects_sphere(black_box(center), black_box(radius))))
    });

    group.bench_function("sphere_aabb_miss", |b| {
        let center = [50.0, 50.0, 50.0];
        let radius = 3.0;
        b.iter(|| black_box(cluster.intersects_sphere(black_box(center), black_box(radius))))
    });

    // Sphere-sphere (light merging)
    group.bench_function("sphere_sphere_hit", |b| {
        let c1 = [0.0, 0.0, 0.0];
        let c2 = [5.0, 0.0, 0.0];
        b.iter(|| {
            black_box(sphere_sphere_intersect(
                black_box(c1),
                black_box(5.0),
                black_box(c2),
                black_box(3.0),
            ))
        })
    });

    group.bench_function("sphere_sphere_miss", |b| {
        let c1 = [0.0, 0.0, 0.0];
        let c2 = [100.0, 0.0, 0.0];
        b.iter(|| {
            black_box(sphere_sphere_intersect(
                black_box(c1),
                black_box(5.0),
                black_box(c2),
                black_box(3.0),
            ))
        })
    });

    // Cone-sphere (spotlight culling)
    group.bench_function("cone_sphere_hit", |b| {
        let apex = [0.0, 0.0, 0.0];
        let dir = [1.0, 0.0, 0.0];
        let angle_cos = 0.866; // 30 degrees
        let sphere = [10.0, 2.0, 0.0];
        b.iter(|| {
            black_box(cone_sphere_intersect(
                black_box(apex),
                black_box(dir),
                black_box(angle_cos),
                black_box(sphere),
                black_box(3.0),
            ))
        })
    });

    group.bench_function("cone_sphere_miss", |b| {
        let apex = [0.0, 0.0, 0.0];
        let dir = [1.0, 0.0, 0.0];
        let angle_cos = 0.866;
        let sphere = [10.0, 50.0, 0.0]; // Far outside cone
        b.iter(|| {
            black_box(cone_sphere_intersect(
                black_box(apex),
                black_box(dir),
                black_box(angle_cos),
                black_box(sphere),
                black_box(3.0),
            ))
        })
    });

    group.finish();
}

fn bench_cluster_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cluster_Grid");

    // Grid creation
    for dims in [(16, 9, 24), (32, 18, 48), (64, 36, 96)] {
        let total = dims.0 * dims.1 * dims.2;
        group.throughput(Throughput::Elements(total as u64));
        group.bench_with_input(
            BenchmarkId::new("create", format!("{}x{}x{}", dims.0, dims.1, dims.2)),
            &dims,
            |b, &dims| {
                b.iter(|| {
                    black_box(ClusterGrid::new(
                        dims,
                        [-100.0, -100.0, 0.1],
                        [100.0, 100.0, 1000.0],
                    ))
                })
            },
        );
    }

    // Single cluster lookup
    let grid = ClusterGrid::new((16, 9, 24), [-100.0, -100.0, 0.1], [100.0, 100.0, 1000.0]);
    group.bench_function("single_cluster_access", |b| {
        b.iter(|| black_box(&grid.bounds[black_box(1000)]))
    });

    group.finish();
}

fn bench_prefix_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("Prefix_Sum");

    for size in [1024, 4096, 16384, 65536] {
        let input: Vec<u32> = (0..size).map(|i| (i % 10) as u32).collect();

        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::new("sequential", size), &input, |b, input| {
            b.iter(|| black_box(prefix_sum_sequential(black_box(input))))
        });

        group.bench_with_input(BenchmarkId::new("blelloch", size), &input, |b, input| {
            b.iter(|| black_box(prefix_sum_blelloch(black_box(input))))
        });
    }

    group.finish();
}

fn bench_light_binning(c: &mut Criterion) {
    let mut group = c.benchmark_group("Light_Binning");
    group.sample_size(30); // Reduce for longer benchmarks

    // Standard cluster grid (16×9×24 = 3456 clusters, typical for 1080p)
    let grid = ClusterGrid::new((16, 9, 24), [-100.0, -100.0, 0.1], [100.0, 100.0, 1000.0]);

    // Generate lights spread across the scene
    let generate_lights = |count: usize| -> Vec<GpuLight> {
        (0..count)
            .map(|i| {
                let t = i as f32 / count as f32;
                let x = (t * 6.28).cos() * 50.0;
                let y = (t * 6.28).sin() * 50.0;
                let z = 100.0 + t * 800.0;
                GpuLight::new(x, y, z, 10.0 + (i % 5) as f32 * 5.0)
            })
            .collect()
    };

    for light_count in [100, 500, 1000, 2000, 5000] {
        let lights = generate_lights(light_count);

        group.throughput(Throughput::Elements(
            (light_count * grid.total_clusters()) as u64,
        ));

        group.bench_with_input(
            BenchmarkId::new("cpu_full_pipeline", light_count),
            &lights,
            |b, lights| b.iter(|| black_box(grid.bin_lights_cpu(black_box(lights)))),
        );
    }

    // Test with different cluster densities
    for (dims, name) in [
        ((8, 6, 12), "low_density"),
        ((16, 9, 24), "standard"),
        ((32, 18, 48), "high_density"),
    ] {
        let grid = ClusterGrid::new(dims, [-100.0, -100.0, 0.1], [100.0, 100.0, 1000.0]);
        let lights = generate_lights(1000);

        group.bench_with_input(BenchmarkId::new("density", name), &lights, |b, lights| {
            b.iter(|| black_box(grid.bin_lights_cpu(black_box(lights))))
        });
    }

    group.finish();
}

fn bench_residency_manager(c: &mut Criterion) {
    let mut group = c.benchmark_group("Residency_Manager");

    // Manager creation
    group.bench_function("create", |b| {
        b.iter(|| black_box(ResidencyManager::new(black_box(1024))))
    });

    // Asset loading
    let mut rm = ResidencyManager::new(1024);
    group.bench_function("load_new_asset", |b| {
        let mut counter = 0u64;
        b.iter(|| {
            let guid = format!("asset_{}", counter);
            counter += 1;
            black_box(rm.load_asset(black_box(&guid), black_box(1), black_box(1)))
        });
        rm.hot_reload(); // Reset
    });

    // Touch existing asset
    let mut rm = ResidencyManager::new(1024);
    for i in 0..100 {
        rm.load_asset(&format!("asset_{}", i), 1, 1);
    }
    group.bench_function("touch_asset", |b| {
        let mut i = 0usize;
        b.iter(|| {
            let guid = format!("asset_{}", i % 100);
            i += 1;
            rm.touch_asset(black_box(&guid))
        })
    });

    // LRU eviction
    group.bench_function("evict_lru", |b| {
        b.iter_batched(
            || {
                let mut rm = ResidencyManager::new(1024);
                for i in 0..500 {
                    rm.load_asset(&format!("asset_{}", i), 2, 1);
                }
                rm
            },
            |mut rm| {
                for _ in 0..100 {
                    black_box(rm.evict_lru());
                }
                rm
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Priority-based eviction
    group.bench_function("evict_by_priority", |b| {
        b.iter_batched(
            || {
                let mut rm = ResidencyManager::new(1024);
                for i in 0..500 {
                    rm.load_asset(&format!("asset_{}", i), 2, (i % 10) as u32);
                }
                rm
            },
            |mut rm| {
                for _ in 0..100 {
                    black_box(rm.evict_by_priority());
                }
                rm
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Hot reload
    group.bench_function("hot_reload", |b| {
        b.iter_batched(
            || {
                let mut rm = ResidencyManager::new(1024);
                for i in 0..1000 {
                    rm.load_asset(&format!("asset_{}", i), 1, 1);
                }
                rm
            },
            |mut rm| {
                rm.hot_reload();
                rm
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_residency_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("Residency_Stress");
    group.sample_size(30);

    // High churn: continuous load/evict
    for asset_count in [100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("high_churn", asset_count),
            &asset_count,
            |b, &count| {
                b.iter_batched(
                    || ResidencyManager::new(count / 2), // Half capacity forces eviction
                    |mut rm| {
                        for i in 0..count {
                            rm.load_asset(&format!("asset_{}", i), 1, (i % 5) as u32);
                            rm.advance_frame();
                        }
                        rm
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    // Memory pressure: large assets
    group.bench_function("large_assets_pressure", |b| {
        b.iter_batched(
            || ResidencyManager::new(512), // 512 MB budget
            |mut rm| {
                for i in 0..100 {
                    // Each asset 10-50 MB
                    let size = 10 + (i % 5) * 10;
                    rm.load_asset(&format!("large_{}", i), size, 1);
                }
                rm
            },
            criterion::BatchSize::SmallInput,
        )
    });

    // Frame simulation
    group.bench_function("frame_simulation_100_frames", |b| {
        b.iter_batched(
            || {
                let mut rm = ResidencyManager::new(256);
                for i in 0..100 {
                    rm.load_asset(&format!("asset_{}", i), 2, 1);
                }
                rm
            },
            |mut rm| {
                for frame in 0..100 {
                    // Touch some assets (simulate camera movement)
                    for i in 0..10 {
                        let idx = (frame * 7 + i * 13) % 100;
                        rm.touch_asset(&format!("asset_{}", idx));
                    }
                    // Load new assets (streaming)
                    for i in 0..5 {
                        let new_idx = 100 + frame * 5 + i;
                        rm.load_asset(&format!("asset_{}", new_idx), 2, 1);
                    }
                    rm.advance_frame();
                }
                rm
            },
            criterion::BatchSize::SmallInput,
        )
    });

    group.finish();
}

fn bench_megalights_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("MegaLights_Scaling");
    group.sample_size(20);

    // GPU dispatch simulation (workgroup calculation only)
    for cluster_dims in [(16, 9, 24), (32, 18, 48), (64, 36, 96)] {
        let total = cluster_dims.0 * cluster_dims.1 * cluster_dims.2;

        group.bench_with_input(
            BenchmarkId::new(
                "workgroup_calc",
                format!("{}x{}x{}", cluster_dims.0, cluster_dims.1, cluster_dims.2),
            ),
            &cluster_dims,
            |b, &dims| {
                b.iter(|| {
                    let workgroups_x = (dims.0 as u32).div_ceil(64);
                    let workgroups_y = dims.1 as u32;
                    let workgroups_z = dims.2 as u32;
                    black_box((workgroups_x, workgroups_y, workgroups_z))
                })
            },
        );

        // Light count per cluster (avg intersection density)
        let grid = ClusterGrid::new(cluster_dims, [-100.0, -100.0, 0.1], [100.0, 100.0, 1000.0]);
        let lights: Vec<GpuLight> = (0..1000)
            .map(|i| {
                let t = i as f32 / 1000.0;
                GpuLight::new(
                    (t * 6.28).cos() * 50.0,
                    (t * 6.28).sin() * 50.0,
                    100.0 + t * 800.0,
                    15.0,
                )
            })
            .collect();

        group.throughput(Throughput::Elements(total as u64));
        group.bench_with_input(
            BenchmarkId::new(
                "intersection_density",
                format!("{}x{}x{}", cluster_dims.0, cluster_dims.1, cluster_dims.2),
            ),
            &(&grid, &lights),
            |b, (grid, lights)| {
                b.iter(|| {
                    let mut total_intersections = 0u32;
                    for cluster in &grid.bounds {
                        for light in *lights {
                            if cluster.intersects_sphere(light.pos(), light.radius()) {
                                total_intersections += 1;
                            }
                        }
                    }
                    black_box(total_intersections)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_light_intersection,
    bench_cluster_grid,
    bench_prefix_sum,
    bench_light_binning,
    bench_residency_manager,
    bench_residency_stress,
    bench_megalights_scaling,
);

criterion_main!(benches);
