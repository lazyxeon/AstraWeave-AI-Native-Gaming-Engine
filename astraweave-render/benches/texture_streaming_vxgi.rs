//! Texture Streaming & VXGI Benchmarks
//!
//! Comprehensive benchmark suite covering:
//! - Texture streaming: LRU cache operations, priority queue, memory management
//! - VXGI (Voxel Global Illumination): Voxel cone tracing, radiance field sampling
//!
//! These benchmarks measure CPU-side performance of streaming and GI systems.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Mat4, Vec3, Vec4};
use std::collections::{BinaryHeap, HashMap, VecDeque};
use std::cmp::Ordering;

// =============================================================================
// TEXTURE STREAMING TYPES
// =============================================================================

/// Unique identifier for textures
type AssetId = String;

/// Texture handle for benchmarking (simplified)
#[derive(Debug, Clone)]
struct TextureHandle {
    id: AssetId,
    width: u32,
    height: u32,
    mip_levels: u32,
    memory_bytes: usize,
}

/// Asset state in the streaming system
#[derive(Debug, Clone)]
enum AssetState {
    Loading,
    Resident(TextureHandle),
    Failed(String),
}

/// Load request with priority
#[derive(Debug, Clone)]
struct LoadRequest {
    id: AssetId,
    priority: u32,
    distance: f32,
}

impl Eq for LoadRequest {}

impl PartialEq for LoadRequest {
    fn eq(&self, other: &Self) -> bool {
        self.priority == other.priority && self.id == other.id
    }
}

impl PartialOrd for LoadRequest {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for LoadRequest {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then closer distance
        match self.priority.cmp(&other.priority) {
            Ordering::Equal => {
                other.distance.partial_cmp(&self.distance).unwrap_or(Ordering::Equal)
            }
            other_cmp => other_cmp,
        }
    }
}

/// Texture streaming manager (CPU simulation)
struct TextureStreamingManager {
    assets: HashMap<AssetId, AssetState>,
    lru_queue: VecDeque<AssetId>,
    load_queue: BinaryHeap<LoadRequest>,
    max_memory_bytes: usize,
    current_memory_bytes: usize,
}

impl TextureStreamingManager {
    fn new(max_memory_mb: usize) -> Self {
        Self {
            assets: HashMap::new(),
            lru_queue: VecDeque::new(),
            load_queue: BinaryHeap::new(),
            max_memory_bytes: max_memory_mb * 1024 * 1024,
            current_memory_bytes: 0,
        }
    }

    /// Request texture - returns handle if resident, queues if not
    fn request_texture(&mut self, id: AssetId, priority: u32, distance: f32) -> Option<TextureHandle> {
        // Check if resident
        if let Some(AssetState::Resident(handle)) = self.assets.get(&id) {
            // Update LRU (move to back)
            if let Some(pos) = self.lru_queue.iter().position(|x| x == &id) {
                self.lru_queue.remove(pos);
                self.lru_queue.push_back(id.clone());
            }
            return Some(handle.clone());
        }

        // Already loading or failed
        if self.assets.contains_key(&id) {
            return None;
        }

        // Queue for load
        self.assets.insert(id.clone(), AssetState::Loading);
        self.load_queue.push(LoadRequest { id, priority, distance });
        None
    }

    /// Simulate loading a texture (instant for benchmarking)
    fn simulate_load(&mut self, id: &str, width: u32, height: u32, mip_levels: u32) {
        let memory_bytes = (width * height * 4) as usize + ((width * height * 4) as usize / 3);
        
        // Evict if needed
        while self.current_memory_bytes + memory_bytes > self.max_memory_bytes {
            if !self.evict_lru() {
                return;
            }
        }

        let handle = TextureHandle {
            id: id.to_string(),
            width,
            height,
            mip_levels,
            memory_bytes,
        };

        self.current_memory_bytes += memory_bytes;
        self.assets.insert(id.to_string(), AssetState::Resident(handle));
        self.lru_queue.push_back(id.to_string());
    }

    /// Evict least recently used texture
    fn evict_lru(&mut self) -> bool {
        if let Some(id) = self.lru_queue.pop_front() {
            if let Some(AssetState::Resident(handle)) = self.assets.remove(&id) {
                self.current_memory_bytes = self.current_memory_bytes.saturating_sub(handle.memory_bytes);
                return true;
            }
        }
        false
    }

    /// Pop next load request from priority queue
    fn pop_load_request(&mut self) -> Option<LoadRequest> {
        self.load_queue.pop()
    }

    /// Check if texture is resident
    fn is_resident(&self, id: &AssetId) -> bool {
        matches!(self.assets.get(id), Some(AssetState::Resident(_)))
    }

    /// Get statistics
    fn get_stats(&self) -> (usize, usize, usize, usize) {
        let loaded = self.assets.values().filter(|s| matches!(s, AssetState::Resident(_))).count();
        let pending = self.load_queue.len();
        (loaded, pending, self.current_memory_bytes, self.max_memory_bytes)
    }
}

// =============================================================================
// VXGI TYPES
// =============================================================================

/// VXGI configuration
#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct VxgiConfig {
    voxel_resolution: u32,
    world_size: f32,
    cone_count: u32,
    max_trace_distance: f32,
    cone_aperture: f32,
}

impl Default for VxgiConfig {
    fn default() -> Self {
        Self {
            voxel_resolution: 256,
            world_size: 1000.0,
            cone_count: 6,
            max_trace_distance: 100.0,
            cone_aperture: 0.577, // ~33 degrees
        }
    }
}

/// Voxel radiance data
#[repr(C)]
#[derive(Debug, Clone, Copy, Default)]
struct VoxelRadiance {
    radiance: [f32; 4], // RGB + opacity
}

/// 3D voxel grid for radiance field
struct VoxelGrid {
    data: Vec<VoxelRadiance>,
    resolution: u32,
    world_size: f32,
}

impl VoxelGrid {
    fn new(resolution: u32, world_size: f32) -> Self {
        let size = (resolution * resolution * resolution) as usize;
        Self {
            data: vec![VoxelRadiance::default(); size],
            resolution,
            world_size,
        }
    }

    /// World position to voxel index
    fn world_to_voxel(&self, pos: Vec3) -> Option<(u32, u32, u32)> {
        let half_size = self.world_size * 0.5;
        let normalized = (pos + Vec3::splat(half_size)) / self.world_size;
        
        if normalized.x < 0.0 || normalized.x >= 1.0 ||
           normalized.y < 0.0 || normalized.y >= 1.0 ||
           normalized.z < 0.0 || normalized.z >= 1.0 {
            return None;
        }

        let x = (normalized.x * self.resolution as f32) as u32;
        let y = (normalized.y * self.resolution as f32) as u32;
        let z = (normalized.z * self.resolution as f32) as u32;
        Some((x.min(self.resolution - 1), y.min(self.resolution - 1), z.min(self.resolution - 1)))
    }

    /// Get voxel at index
    fn get(&self, x: u32, y: u32, z: u32) -> Option<&VoxelRadiance> {
        if x >= self.resolution || y >= self.resolution || z >= self.resolution {
            return None;
        }
        let idx = (z * self.resolution * self.resolution + y * self.resolution + x) as usize;
        self.data.get(idx)
    }

    /// Set voxel at index
    fn set(&mut self, x: u32, y: u32, z: u32, radiance: VoxelRadiance) {
        if x < self.resolution && y < self.resolution && z < self.resolution {
            let idx = (z * self.resolution * self.resolution + y * self.resolution + x) as usize;
            if idx < self.data.len() {
                self.data[idx] = radiance;
            }
        }
    }

    /// Trilinear sample at world position
    fn sample_trilinear(&self, pos: Vec3) -> VoxelRadiance {
        let half_size = self.world_size * 0.5;
        let normalized = (pos + Vec3::splat(half_size)) / self.world_size;
        
        // Clamp to valid range
        let u = normalized.x.clamp(0.0, 0.9999) * self.resolution as f32;
        let v = normalized.y.clamp(0.0, 0.9999) * self.resolution as f32;
        let w = normalized.z.clamp(0.0, 0.9999) * self.resolution as f32;

        let x0 = u.floor() as u32;
        let y0 = v.floor() as u32;
        let z0 = w.floor() as u32;
        let x1 = (x0 + 1).min(self.resolution - 1);
        let y1 = (y0 + 1).min(self.resolution - 1);
        let z1 = (z0 + 1).min(self.resolution - 1);

        let fx = u.fract();
        let fy = v.fract();
        let fz = w.fract();

        // Fetch 8 corners
        let c000 = self.get(x0, y0, z0).copied().unwrap_or_default();
        let c100 = self.get(x1, y0, z0).copied().unwrap_or_default();
        let c010 = self.get(x0, y1, z0).copied().unwrap_or_default();
        let c110 = self.get(x1, y1, z0).copied().unwrap_or_default();
        let c001 = self.get(x0, y0, z1).copied().unwrap_or_default();
        let c101 = self.get(x1, y0, z1).copied().unwrap_or_default();
        let c011 = self.get(x0, y1, z1).copied().unwrap_or_default();
        let c111 = self.get(x1, y1, z1).copied().unwrap_or_default();

        // Trilinear interpolation
        let lerp = |a: f32, b: f32, t: f32| a + (b - a) * t;
        
        let mut result = VoxelRadiance::default();
        for i in 0..4 {
            let c00 = lerp(c000.radiance[i], c100.radiance[i], fx);
            let c01 = lerp(c001.radiance[i], c101.radiance[i], fx);
            let c10 = lerp(c010.radiance[i], c110.radiance[i], fx);
            let c11 = lerp(c011.radiance[i], c111.radiance[i], fx);
            
            let c0 = lerp(c00, c10, fy);
            let c1 = lerp(c01, c11, fy);
            
            result.radiance[i] = lerp(c0, c1, fz);
        }

        result
    }
}

/// Cone direction generator for hemisphere sampling
fn generate_cone_directions(count: u32) -> Vec<Vec3> {
    let mut directions = Vec::with_capacity(count as usize);
    
    // Predefined directions for common counts
    match count {
        1 => {
            directions.push(Vec3::Y); // Up
        }
        4 => {
            // Tetrahedral pattern
            directions.push(Vec3::new(0.0, 1.0, 0.0));
            directions.push(Vec3::new(0.943, -0.333, 0.0));
            directions.push(Vec3::new(-0.471, -0.333, 0.816));
            directions.push(Vec3::new(-0.471, -0.333, -0.816));
        }
        6 => {
            // Axis-aligned pattern (default VXGI)
            directions.push(Vec3::Y);  // +Y
            directions.push(Vec3::X);  // +X
            directions.push(-Vec3::X); // -X
            directions.push(Vec3::Z);  // +Z
            directions.push(-Vec3::Z); // -Z
            directions.push(-Vec3::Y); // -Y (bounce)
        }
        _ => {
            // Fibonacci sphere distribution
            let golden_ratio = (1.0 + 5.0_f32.sqrt()) / 2.0;
            for i in 0..count {
                let theta = 2.0 * std::f32::consts::PI * i as f32 / golden_ratio;
                let phi = (1.0 - 2.0 * (i as f32 + 0.5) / count as f32).acos();
                directions.push(Vec3::new(
                    phi.sin() * theta.cos(),
                    phi.cos(),
                    phi.sin() * theta.sin(),
                ));
            }
        }
    }
    
    directions
}

/// Voxel cone trace from position in direction
fn cone_trace(
    grid: &VoxelGrid,
    origin: Vec3,
    direction: Vec3,
    aperture: f32,
    max_distance: f32,
    step_scale: f32,
) -> Vec4 {
    let mut color = Vec4::ZERO;
    let mut alpha = 0.0_f32;
    let mut distance = grid.world_size / grid.resolution as f32; // Start one voxel away
    
    let direction = direction.normalize();
    
    while distance < max_distance && alpha < 0.99 {
        // Cone radius at this distance
        let cone_radius = distance * aperture.tan();
        
        // Sample position
        let sample_pos = origin + direction * distance;
        
        // Determine mip level based on cone radius
        let voxel_size = grid.world_size / grid.resolution as f32;
        let mip_level = (cone_radius / voxel_size).log2().max(0.0);
        
        // Sample voxel (using trilinear for simplicity, production uses mip sampling)
        let sample = grid.sample_trilinear(sample_pos);
        
        // Front-to-back compositing
        let sample_alpha = sample.radiance[3] * (1.0 - alpha);
        color += Vec4::new(
            sample.radiance[0],
            sample.radiance[1],
            sample.radiance[2],
            sample.radiance[3],
        ) * sample_alpha;
        alpha += sample_alpha;
        
        // Adaptive step size based on mip level
        distance += voxel_size * (1.0 + mip_level * step_scale);
    }
    
    color
}

/// Compute indirect lighting for a surface point using cone tracing
fn compute_indirect_lighting(
    grid: &VoxelGrid,
    position: Vec3,
    normal: Vec3,
    config: &VxgiConfig,
) -> Vec3 {
    let cone_directions = generate_cone_directions(config.cone_count);
    let mut indirect = Vec3::ZERO;
    
    // Transform directions to world space (hemisphere around normal)
    let tangent = if normal.y.abs() < 0.999 {
        normal.cross(Vec3::Y).normalize()
    } else {
        normal.cross(Vec3::X).normalize()
    };
    let bitangent = normal.cross(tangent);
    
    for local_dir in &cone_directions {
        // Transform to world space
        let world_dir = tangent * local_dir.x + normal * local_dir.y + bitangent * local_dir.z;
        
        // Weight by cosine (diffuse BRDF)
        let weight = world_dir.dot(normal).max(0.0);
        
        // Trace cone
        let traced = cone_trace(
            grid,
            position + normal * 0.1, // Offset to avoid self-intersection
            world_dir,
            config.cone_aperture,
            config.max_trace_distance,
            0.5, // Step scale
        );
        
        indirect += Vec3::new(traced.x, traced.y, traced.z) * weight;
    }
    
    // Average by cone count
    indirect / config.cone_count as f32
}

/// Voxelize a triangle into the grid
fn voxelize_triangle(
    grid: &mut VoxelGrid,
    v0: Vec3,
    v1: Vec3,
    v2: Vec3,
    radiance: VoxelRadiance,
) {
    // Simple conservative rasterization
    let min = v0.min(v1).min(v2);
    let max = v0.max(v1).max(v2);
    
    let voxel_size = grid.world_size / grid.resolution as f32;
    
    // Get voxel bounds
    if let (Some(min_v), Some(max_v)) = (grid.world_to_voxel(min), grid.world_to_voxel(max)) {
        for z in min_v.2..=max_v.2 {
            for y in min_v.1..=max_v.1 {
                for x in min_v.0..=max_v.0 {
                    // Voxel center
                    let half_size = grid.world_size * 0.5;
                    let voxel_center = Vec3::new(
                        (x as f32 + 0.5) * voxel_size - half_size,
                        (y as f32 + 0.5) * voxel_size - half_size,
                        (z as f32 + 0.5) * voxel_size - half_size,
                    );
                    
                    // Simple point-in-AABB test (conservative)
                    if voxel_center.x >= min.x - voxel_size && voxel_center.x <= max.x + voxel_size &&
                       voxel_center.y >= min.y - voxel_size && voxel_center.y <= max.y + voxel_size &&
                       voxel_center.z >= min.z - voxel_size && voxel_center.z <= max.z + voxel_size {
                        grid.set(x, y, z, radiance);
                    }
                }
            }
        }
    }
}

// =============================================================================
// BENCHMARKS
// =============================================================================

fn bench_texture_streaming(c: &mut Criterion) {
    let mut group = c.benchmark_group("Texture_Streaming");

    // Manager creation
    for memory_mb in [256, 512, 1024, 2048] {
        group.bench_with_input(
            BenchmarkId::new("manager_create", format!("{}MB", memory_mb)),
            &memory_mb,
            |b, &mem| {
                b.iter(|| {
                    black_box(TextureStreamingManager::new(mem))
                });
            },
        );
    }

    // Texture request (resident - cache hit)
    {
        let mut manager = TextureStreamingManager::new(1024);
        for i in 0..100 {
            manager.simulate_load(&format!("texture_{}", i), 1024, 1024, 10);
        }
        
        group.bench_function("request_resident", |b| {
            let mut idx = 0;
            b.iter(|| {
                let id = format!("texture_{}", idx % 100);
                let result = black_box(manager.request_texture(id, 100, 10.0));
                idx += 1;
                result
            });
        });
    }

    // Texture request (not resident - queue)
    group.bench_function("request_queue", |b| {
        let mut manager = TextureStreamingManager::new(1024);
        let mut idx = 0;
        b.iter(|| {
            let id = format!("new_texture_{}", idx);
            let result = black_box(manager.request_texture(id, 100, 10.0));
            idx += 1;
            result
        });
    });

    // LRU update (touch)
    {
        let mut manager = TextureStreamingManager::new(1024);
        for i in 0..1000 {
            manager.simulate_load(&format!("texture_{}", i), 512, 512, 8);
        }
        
        group.bench_function("lru_touch_1000", |b| {
            let mut idx = 0;
            b.iter(|| {
                let id = format!("texture_{}", idx % 1000);
                manager.request_texture(id, 100, 10.0);
                idx += 1;
            });
        });
    }

    // LRU eviction
    for count in [100, 500, 1000] {
        group.bench_with_input(
            BenchmarkId::new("evict_lru", count),
            &count,
            |b, &cnt| {
                b.iter(|| {
                    let mut manager = TextureStreamingManager::new(256); // Small budget to force evictions
                    for i in 0..cnt {
                        manager.simulate_load(&format!("tex_{}", i), 256, 256, 6);
                    }
                    // Evict all
                    let mut evicted = 0;
                    while manager.evict_lru() {
                        evicted += 1;
                    }
                    black_box(evicted)
                });
            },
        );
    }

    // Priority queue operations
    group.bench_function("priority_queue_push_pop_100", |b| {
        b.iter(|| {
            let mut queue: BinaryHeap<LoadRequest> = BinaryHeap::new();
            for i in 0..100 {
                queue.push(LoadRequest {
                    id: format!("tex_{}", i),
                    priority: (i % 10) as u32,
                    distance: (i as f32) * 10.0,
                });
            }
            let mut count = 0;
            while queue.pop().is_some() {
                count += 1;
            }
            black_box(count)
        });
    });

    // Simulate load with budget check
    for budget_mb in [128, 512, 2048] {
        group.bench_with_input(
            BenchmarkId::new("simulate_load", format!("{}MB_budget", budget_mb)),
            &budget_mb,
            |b, &budget| {
                b.iter(|| {
                    let mut manager = TextureStreamingManager::new(budget);
                    for i in 0..50 {
                        manager.simulate_load(&format!("tex_{}", i), 1024, 1024, 10);
                    }
                    black_box(manager.get_stats())
                });
            },
        );
    }

    // Get stats
    {
        let mut manager = TextureStreamingManager::new(1024);
        for i in 0..500 {
            manager.simulate_load(&format!("texture_{}", i), 256, 256, 6);
        }
        
        group.bench_function("get_stats_500_textures", |b| {
            b.iter(|| {
                black_box(manager.get_stats())
            });
        });
    }

    group.finish();
}

fn bench_vxgi_grid(c: &mut Criterion) {
    let mut group = c.benchmark_group("VXGI_Grid");

    // Grid creation
    for resolution in [64, 128, 256] {
        group.throughput(Throughput::Elements((resolution * resolution * resolution) as u64));
        group.bench_with_input(
            BenchmarkId::new("create", format!("{}^3", resolution)),
            &resolution,
            |b, &res| {
                b.iter(|| {
                    black_box(VoxelGrid::new(res, 1000.0))
                });
            },
        );
    }

    // World to voxel conversion
    {
        let grid = VoxelGrid::new(256, 1000.0);
        
        group.bench_function("world_to_voxel", |b| {
            let mut idx = 0.0_f32;
            b.iter(|| {
                let pos = Vec3::new(
                    (idx * 0.1).sin() * 400.0,
                    (idx * 0.2).cos() * 400.0,
                    (idx * 0.3).sin() * 400.0,
                );
                idx += 0.01;
                black_box(grid.world_to_voxel(pos))
            });
        });
    }

    // Get/Set voxel
    {
        let mut grid = VoxelGrid::new(256, 1000.0);
        let radiance = VoxelRadiance { radiance: [1.0, 0.5, 0.2, 0.8] };
        
        group.bench_function("set_voxel", |b| {
            let mut x = 0_u32;
            b.iter(|| {
                grid.set(x % 256, (x / 256) % 256, (x / 65536) % 256, radiance);
                x = x.wrapping_add(1);
            });
        });

        group.bench_function("get_voxel", |b| {
            let mut x = 0_u32;
            b.iter(|| {
                let result = black_box(grid.get(x % 256, (x / 256) % 256, (x / 65536) % 256));
                x = x.wrapping_add(1);
                result
            });
        });
    }

    // Trilinear sampling
    for resolution in [64, 128, 256] {
        let grid = VoxelGrid::new(resolution, 1000.0);
        
        group.bench_with_input(
            BenchmarkId::new("trilinear_sample", format!("{}^3", resolution)),
            &resolution,
            |b, _| {
                let mut idx = 0.0_f32;
                b.iter(|| {
                    let pos = Vec3::new(
                        (idx * 0.1).sin() * 400.0,
                        (idx * 0.2).cos() * 400.0,
                        (idx * 0.3).sin() * 400.0,
                    );
                    idx += 0.01;
                    black_box(grid.sample_trilinear(pos))
                });
            },
        );
    }

    // Batch trilinear sampling
    for count in [100, 1000, 10000] {
        let grid = VoxelGrid::new(256, 1000.0);
        
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("trilinear_batch", count),
            &count,
            |b, &cnt| {
                let positions: Vec<Vec3> = (0..cnt)
                    .map(|i| Vec3::new(
                        ((i as f32) * 0.1).sin() * 400.0,
                        ((i as f32) * 0.2).cos() * 400.0,
                        ((i as f32) * 0.3).sin() * 400.0,
                    ))
                    .collect();
                
                b.iter(|| {
                    let sum: f32 = positions.iter()
                        .map(|p| grid.sample_trilinear(*p).radiance[0])
                        .sum();
                    black_box(sum)
                });
            },
        );
    }

    group.finish();
}

fn bench_vxgi_cone_tracing(c: &mut Criterion) {
    let mut group = c.benchmark_group("VXGI_Cone_Tracing");

    // Generate cone directions
    for count in [1, 4, 6, 12, 32] {
        group.bench_with_input(
            BenchmarkId::new("generate_directions", count),
            &count,
            |b, &cnt| {
                b.iter(|| {
                    black_box(generate_cone_directions(cnt))
                });
            },
        );
    }

    // Single cone trace
    {
        let mut grid = VoxelGrid::new(128, 1000.0);
        // Fill with some data
        for z in 0..128 {
            for y in 0..128 {
                for x in 0..128 {
                    let radiance = VoxelRadiance {
                        radiance: [
                            (x as f32 / 128.0),
                            (y as f32 / 128.0),
                            (z as f32 / 128.0),
                            0.1,
                        ],
                    };
                    grid.set(x, y, z, radiance);
                }
            }
        }

        for max_dist in [25.0, 50.0, 100.0] {
            group.bench_with_input(
                BenchmarkId::new("single_cone_trace", format!("{:.0}m", max_dist)),
                &max_dist,
                |b, &dist| {
                    let origin = Vec3::ZERO;
                    let direction = Vec3::new(1.0, 1.0, 1.0).normalize();
                    let aperture = 0.577_f32;
                    
                    b.iter(|| {
                        black_box(cone_trace(&grid, origin, direction, aperture, dist, 0.5))
                    });
                },
            );
        }
    }

    // Full indirect lighting (multiple cones)
    {
        let mut grid = VoxelGrid::new(64, 1000.0);
        // Sparse fill
        for i in 0..1000 {
            let x = (i * 7) % 64;
            let y = (i * 11) % 64;
            let z = (i * 13) % 64;
            grid.set(x, y, z, VoxelRadiance { radiance: [1.0, 0.8, 0.6, 0.5] });
        }

        for cone_count in [1, 4, 6] {
            let config = VxgiConfig {
                cone_count,
                max_trace_distance: 50.0,
                ..Default::default()
            };
            
            group.bench_with_input(
                BenchmarkId::new("indirect_lighting", format!("{}_cones", cone_count)),
                &config,
                |b, cfg| {
                    let position = Vec3::new(10.0, 10.0, 10.0);
                    let normal = Vec3::Y;
                    
                    b.iter(|| {
                        black_box(compute_indirect_lighting(&grid, position, normal, cfg))
                    });
                },
            );
        }
    }

    // Batch indirect lighting
    for pixel_count in [100, 1000] {
        let mut grid = VoxelGrid::new(64, 1000.0);
        for i in 0..500 {
            let x = (i * 7) % 64;
            let y = (i * 11) % 64;
            let z = (i * 13) % 64;
            grid.set(x, y, z, VoxelRadiance { radiance: [0.8, 0.6, 0.4, 0.3] });
        }
        let config = VxgiConfig::default();
        
        group.throughput(Throughput::Elements(pixel_count as u64));
        group.bench_with_input(
            BenchmarkId::new("indirect_batch", pixel_count),
            &pixel_count,
            |b, &cnt| {
                let samples: Vec<(Vec3, Vec3)> = (0..cnt)
                    .map(|i| {
                        let pos = Vec3::new(
                            ((i as f32) * 0.37).sin() * 30.0,
                            ((i as f32) * 0.41).cos() * 30.0,
                            ((i as f32) * 0.47).sin() * 30.0,
                        );
                        let normal = Vec3::new(
                            ((i as f32) * 0.23).sin(),
                            ((i as f32) * 0.29).cos().abs() + 0.1,
                            ((i as f32) * 0.31).sin(),
                        ).normalize();
                        (pos, normal)
                    })
                    .collect();
                
                b.iter(|| {
                    let sum: Vec3 = samples.iter()
                        .map(|(p, n)| compute_indirect_lighting(&grid, *p, *n, &config))
                        .fold(Vec3::ZERO, |a, b| a + b);
                    black_box(sum)
                });
            },
        );
    }

    group.finish();
}

fn bench_vxgi_voxelization(c: &mut Criterion) {
    let mut group = c.benchmark_group("VXGI_Voxelization");

    // Voxelize single triangle
    {
        let mut grid = VoxelGrid::new(128, 1000.0);
        let radiance = VoxelRadiance { radiance: [1.0, 0.5, 0.25, 1.0] };
        
        group.bench_function("voxelize_triangle_small", |b| {
            let v0 = Vec3::new(0.0, 0.0, 0.0);
            let v1 = Vec3::new(10.0, 0.0, 0.0);
            let v2 = Vec3::new(5.0, 10.0, 0.0);
            
            b.iter(|| {
                voxelize_triangle(&mut grid, v0, v1, v2, radiance);
            });
        });

        group.bench_function("voxelize_triangle_large", |b| {
            let v0 = Vec3::new(-100.0, 0.0, -100.0);
            let v1 = Vec3::new(100.0, 0.0, -100.0);
            let v2 = Vec3::new(0.0, 0.0, 100.0);
            
            b.iter(|| {
                voxelize_triangle(&mut grid, v0, v1, v2, radiance);
            });
        });
    }

    // Voxelize mesh (batch of triangles)
    for tri_count in [100, 500, 1000] {
        group.throughput(Throughput::Elements(tri_count as u64));
        group.bench_with_input(
            BenchmarkId::new("voxelize_mesh", format!("{}_tris", tri_count)),
            &tri_count,
            |b, &cnt| {
                // Generate random triangles
                let triangles: Vec<(Vec3, Vec3, Vec3)> = (0..cnt)
                    .map(|i| {
                        let base = Vec3::new(
                            ((i as f32) * 1.7).sin() * 200.0,
                            ((i as f32) * 2.3).cos() * 200.0,
                            ((i as f32) * 3.1).sin() * 200.0,
                        );
                        let size = 5.0 + (i as f32 * 0.1).sin().abs() * 20.0;
                        (
                            base,
                            base + Vec3::new(size, 0.0, 0.0),
                            base + Vec3::new(size * 0.5, size, 0.0),
                        )
                    })
                    .collect();
                
                b.iter(|| {
                    let mut grid = VoxelGrid::new(128, 1000.0);
                    let radiance = VoxelRadiance { radiance: [1.0, 0.8, 0.6, 1.0] };
                    
                    for (v0, v1, v2) in &triangles {
                        voxelize_triangle(&mut grid, *v0, *v1, *v2, radiance);
                    }
                    black_box(grid.data.len())
                });
            },
        );
    }

    group.finish();
}

fn bench_streaming_stress(c: &mut Criterion) {
    let mut group = c.benchmark_group("Streaming_Stress");

    // High-churn scenario (rapid load/evict)
    group.bench_function("high_churn_100_cycles", |b| {
        b.iter(|| {
            let mut manager = TextureStreamingManager::new(64); // Very limited budget
            
            for cycle in 0..100 {
                // Load 10 textures
                for i in 0..10 {
                    let id = format!("cycle{}_{}", cycle, i);
                    manager.simulate_load(&id, 512, 512, 8);
                }
                // Request them to update LRU
                for i in 0..10 {
                    let id = format!("cycle{}_{}", cycle, i);
                    manager.request_texture(id, 100, 10.0);
                }
            }
            black_box(manager.get_stats())
        });
    });

    // Large texture atlas scenario
    group.bench_function("large_atlas_1000_textures", |b| {
        b.iter(|| {
            let mut manager = TextureStreamingManager::new(2048);
            
            // Load 1000 small textures
            for i in 0..1000 {
                manager.simulate_load(&format!("atlas_{}", i), 128, 128, 4);
            }
            
            // Random access pattern
            for i in 0..1000 {
                let idx = (i * 7 + 13) % 1000;
                manager.request_texture(format!("atlas_{}", idx), 50, idx as f32);
            }
            
            black_box(manager.get_stats())
        });
    });

    // Mixed size textures
    group.bench_function("mixed_sizes_200_textures", |b| {
        b.iter(|| {
            let mut manager = TextureStreamingManager::new(512);
            
            for i in 0..200 {
                let (width, height) = match i % 4 {
                    0 => (64, 64),      // Icon
                    1 => (256, 256),    // Small
                    2 => (1024, 1024),  // Medium
                    _ => (2048, 2048),  // Large
                };
                manager.simulate_load(&format!("tex_{}", i), width, height, 10);
            }
            
            black_box(manager.get_stats())
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_texture_streaming,
    bench_vxgi_grid,
    bench_vxgi_cone_tracing,
    bench_vxgi_voxelization,
    bench_streaming_stress,
);

criterion_main!(benches);
