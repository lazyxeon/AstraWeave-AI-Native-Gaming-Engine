//! Nanite GPU Culling & Shadow CSM Benchmarks
//!
//! Comprehensive benchmarks for GPU-driven rendering and cascaded shadow mapping:
//! - Hi-Z pyramid operations (hierarchical depth buffer)
//! - Meshlet culling (frustum, occlusion, backface)
//! - Visibility buffer operations
//! - Shadow cascade splitting & matrix computation
//! - PCF shadow sampling simulation
//! - Shadow atlas management

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Mat4, Vec3, Vec4};

// ============================================================================
// NANITE GPU CULLING STRUCTURES (CPU-side representations)
// ============================================================================

/// GPU camera uniform data
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GpuCamera {
    view_proj: [[f32; 4]; 4],
    inv_view_proj: [[f32; 4]; 4],
    position: [f32; 3],
    _padding0: f32,
    view_dir: [f32; 3],
    _padding1: f32,
    frustum_planes: [[f32; 4]; 6],
    hiz_size: [u32; 2],
    hiz_mip_count: u32,
    screen_width: u32,
    screen_height: u32,
    enable_occlusion: u32,
    enable_backface: u32,
    lod_scale: f32,
}

impl GpuCamera {
    fn from_matrix(view_proj: Mat4, position: Vec3, width: u32, height: u32) -> Self {
        let inv_view_proj = view_proj.inverse();
        let forward = Vec3::new(0.0, 0.0, -1.0);
        
        // Extract frustum planes (Gribb-Hartmann)
        let vp = view_proj;
        let planes = [
            (vp.row(3) + vp.row(0)).normalize(), // Left
            (vp.row(3) - vp.row(0)).normalize(), // Right
            (vp.row(3) + vp.row(1)).normalize(), // Bottom
            (vp.row(3) - vp.row(1)).normalize(), // Top
            (vp.row(3) + vp.row(2)).normalize(), // Near
            (vp.row(3) - vp.row(2)).normalize(), // Far
        ];
        
        Self {
            view_proj: view_proj.to_cols_array_2d(),
            inv_view_proj: inv_view_proj.to_cols_array_2d(),
            position: position.to_array(),
            _padding0: 0.0,
            view_dir: forward.to_array(),
            _padding1: 0.0,
            frustum_planes: planes.map(|p| p.to_array()),
            hiz_size: [width, height],
            hiz_mip_count: (width.max(height) as f32).log2().ceil() as u32,
            screen_width: width,
            screen_height: height,
            enable_occlusion: 1,
            enable_backface: 1,
            lod_scale: 1.0,
        }
    }
}

/// Culling statistics
#[repr(C)]
#[derive(Copy, Clone, Debug, Default)]
struct CullStats {
    total_clusters: u32,
    frustum_culled: u32,
    occlusion_culled: u32,
    backface_culled: u32,
    visible_count: u32,
}

/// GPU meshlet representation
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GpuMeshlet {
    // Bounding sphere (center + radius)
    center: [f32; 3],
    radius: f32,
    // Cone for backface culling
    cone_axis: [f32; 3],
    cone_cutoff: f32,
    // Index data
    vertex_offset: u32,
    vertex_count: u32,
    index_offset: u32,
    index_count: u32,
    // LOD level
    lod_level: u32,
    error_metric: f32,
    _padding: [u32; 2],
}

impl GpuMeshlet {
    fn new(center: Vec3, radius: f32, vertex_count: u32, index_count: u32) -> Self {
        Self {
            center: center.to_array(),
            radius,
            cone_axis: [0.0, 1.0, 0.0],
            cone_cutoff: 0.0,
            vertex_offset: 0,
            vertex_count,
            index_offset: 0,
            index_count,
            lod_level: 0,
            error_metric: 0.0,
            _padding: [0; 2],
        }
    }
}

// ============================================================================
// HI-Z PYRAMID SIMULATION
// ============================================================================

/// Hi-Z pyramid level (simulates hierarchical depth buffer)
struct HiZLevel {
    width: u32,
    height: u32,
    data: Vec<f32>,
}

impl HiZLevel {
    fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![1.0; (width * height) as usize], // Initialize to far depth
        }
    }
    
    /// Downsample from previous level (max operation for depth)
    fn downsample_from(prev: &HiZLevel) -> Self {
        let width = (prev.width + 1) / 2;
        let height = (prev.height + 1) / 2;
        let mut data = Vec::with_capacity((width * height) as usize);
        
        for y in 0..height {
            for x in 0..width {
                let x0 = (x * 2).min(prev.width - 1);
                let x1 = (x * 2 + 1).min(prev.width - 1);
                let y0 = (y * 2).min(prev.height - 1);
                let y1 = (y * 2 + 1).min(prev.height - 1);
                
                let d00 = prev.data[(y0 * prev.width + x0) as usize];
                let d10 = prev.data[(y0 * prev.width + x1) as usize];
                let d01 = prev.data[(y1 * prev.width + x0) as usize];
                let d11 = prev.data[(y1 * prev.width + x1) as usize];
                
                // Max for occlusion culling (conservative depth)
                data.push(d00.max(d10).max(d01).max(d11));
            }
        }
        
        Self { width, height, data }
    }
    
    /// Sample depth at UV coordinates
    fn sample(&self, u: f32, v: f32) -> f32 {
        let x = ((u * self.width as f32) as u32).min(self.width - 1);
        let y = ((v * self.height as f32) as u32).min(self.height - 1);
        self.data[(y * self.width + x) as usize]
    }
}

/// Build complete Hi-Z pyramid
fn build_hiz_pyramid(base_width: u32, base_height: u32) -> Vec<HiZLevel> {
    let mip_count = (base_width.max(base_height) as f32).log2().ceil() as usize;
    let mut pyramid = Vec::with_capacity(mip_count);
    
    // Base level (full resolution depth)
    pyramid.push(HiZLevel::new(base_width, base_height));
    
    // Build mip chain
    for _ in 1..mip_count {
        let prev = pyramid.last().unwrap();
        pyramid.push(HiZLevel::downsample_from(prev));
    }
    
    pyramid
}

// ============================================================================
// MESHLET CULLING (CPU SIMULATION OF GPU COMPUTE)
// ============================================================================

/// Frustum cull a single meshlet
fn frustum_cull_meshlet(meshlet: &GpuMeshlet, planes: &[[f32; 4]; 6]) -> bool {
    let center = Vec3::from_array(meshlet.center);
    let radius = meshlet.radius;
    
    for plane in planes {
        let normal = Vec3::new(plane[0], plane[1], plane[2]);
        let d = plane[3];
        let distance = normal.dot(center) + d;
        
        if distance < -radius {
            return true; // Culled
        }
    }
    
    false // Visible
}

/// Backface cull a meshlet based on cone
fn backface_cull_meshlet(meshlet: &GpuMeshlet, view_dir: Vec3) -> bool {
    let cone_axis = Vec3::from_array(meshlet.cone_axis);
    let cone_cutoff = meshlet.cone_cutoff;
    
    // If cone_cutoff >= 1.0, meshlet is double-sided (never backface cull)
    if cone_cutoff >= 1.0 {
        return false;
    }
    
    // Backface if view direction is within cone
    let dot = view_dir.dot(cone_axis);
    dot > cone_cutoff
}

/// Occlusion cull using Hi-Z pyramid
fn occlusion_cull_meshlet(
    meshlet: &GpuMeshlet,
    view_proj: Mat4,
    hiz: &[HiZLevel],
    screen_width: u32,
    screen_height: u32,
) -> bool {
    let center = Vec3::from_array(meshlet.center);
    let radius = meshlet.radius;
    
    // Project bounding sphere to screen
    let clip = view_proj * center.extend(1.0);
    if clip.w <= 0.0 {
        return false; // Behind camera, don't cull (will be frustum culled)
    }
    
    let ndc = clip.truncate() / clip.w;
    let screen_x = (ndc.x * 0.5 + 0.5) * screen_width as f32;
    let screen_y = (ndc.y * 0.5 + 0.5) * screen_height as f32;
    
    // Project radius to screen space
    let proj_radius = (radius / clip.w) * screen_width as f32 * 0.5;
    
    // Select mip level based on projected size
    let mip = ((proj_radius * 2.0).log2().ceil() as usize).max(0).min(hiz.len() - 1);
    
    // Sample Hi-Z at projected position
    let u = screen_x / screen_width as f32;
    let v = screen_y / screen_height as f32;
    let hiz_depth = hiz[mip].sample(u, v);
    
    // Depth of sphere front
    let sphere_depth = (clip.z / clip.w) - radius * 0.1; // Slight bias
    
    // Occluded if sphere is behind Hi-Z depth
    sphere_depth > hiz_depth
}

/// Full meshlet culling pass (frustum + backface + occlusion)
fn cull_meshlets(
    meshlets: &[GpuMeshlet],
    camera: &GpuCamera,
    hiz: &[HiZLevel],
) -> (Vec<u32>, CullStats) {
    let mut visible = Vec::with_capacity(meshlets.len());
    let mut stats = CullStats {
        total_clusters: meshlets.len() as u32,
        ..Default::default()
    };
    
    let view_proj = Mat4::from_cols_array_2d(&camera.view_proj);
    let view_dir = Vec3::from_array(camera.view_dir);
    
    for (i, meshlet) in meshlets.iter().enumerate() {
        // Stage 1: Frustum culling
        if frustum_cull_meshlet(meshlet, &camera.frustum_planes) {
            stats.frustum_culled += 1;
            continue;
        }
        
        // Stage 2: Backface culling (if enabled)
        if camera.enable_backface != 0 && backface_cull_meshlet(meshlet, view_dir) {
            stats.backface_culled += 1;
            continue;
        }
        
        // Stage 3: Occlusion culling (if enabled)
        if camera.enable_occlusion != 0 {
            if occlusion_cull_meshlet(meshlet, view_proj, hiz, camera.screen_width, camera.screen_height) {
                stats.occlusion_culled += 1;
                continue;
            }
        }
        
        visible.push(i as u32);
    }
    
    stats.visible_count = visible.len() as u32;
    (visible, stats)
}

// ============================================================================
// SHADOW CSM STRUCTURES
// ============================================================================

/// Shadow cascade data
#[derive(Debug, Clone, Copy)]
struct ShadowCascade {
    near: f32,
    far: f32,
    view_matrix: Mat4,
    proj_matrix: Mat4,
    view_proj_matrix: Mat4,
    atlas_offset: Vec4,
}

/// GPU-compatible cascade data
#[repr(C)]
#[derive(Copy, Clone, Debug)]
struct GpuShadowCascade {
    view_proj: [[f32; 4]; 4],
    split_distances: [f32; 4],
    atlas_transform: [f32; 4],
}

impl From<&ShadowCascade> for GpuShadowCascade {
    fn from(cascade: &ShadowCascade) -> Self {
        Self {
            view_proj: cascade.view_proj_matrix.to_cols_array_2d(),
            split_distances: [cascade.near, cascade.far, 0.0, 0.0],
            atlas_transform: cascade.atlas_offset.to_array(),
        }
    }
}

/// Calculate cascade split distances using logarithmic distribution
fn calculate_cascade_splits(
    near: f32,
    far: f32,
    cascade_count: usize,
    lambda: f32, // 0.0 = linear, 1.0 = logarithmic
) -> Vec<f32> {
    let mut splits = Vec::with_capacity(cascade_count + 1);
    splits.push(near);
    
    for i in 1..cascade_count {
        let ratio = i as f32 / cascade_count as f32;
        let log_split = near * (far / near).powf(ratio);
        let linear_split = near + (far - near) * ratio;
        let split = lambda * log_split + (1.0 - lambda) * linear_split;
        splits.push(split);
    }
    
    splits.push(far);
    splits
}

/// Calculate orthographic projection matrix for shadow cascade
fn calculate_cascade_projection(
    corners: &[Vec3; 8],
    light_dir: Vec3,
) -> (Mat4, Mat4, Mat4) {
    // Light view matrix
    let light_pos = Vec3::ZERO - light_dir * 100.0;
    let view = Mat4::look_at_rh(light_pos, Vec3::ZERO, Vec3::Y);
    
    // Transform frustum corners to light space
    let mut min = Vec3::splat(f32::MAX);
    let mut max = Vec3::splat(f32::MIN);
    
    for corner in corners {
        let light_space = (view * corner.extend(1.0)).truncate();
        min = min.min(light_space);
        max = max.max(light_space);
    }
    
    // Add margin for shadow casters outside frustum
    let margin = 10.0;
    min -= Vec3::splat(margin);
    max += Vec3::splat(margin);
    
    // Orthographic projection
    let proj = Mat4::orthographic_rh(min.x, max.x, min.y, max.y, min.z, max.z);
    let view_proj = proj * view;
    
    (view, proj, view_proj)
}

/// Build shadow cascades for a directional light
fn build_shadow_cascades(
    camera_view_proj: Mat4,
    light_dir: Vec3,
    near: f32,
    far: f32,
    cascade_count: usize,
) -> Vec<ShadowCascade> {
    let splits = calculate_cascade_splits(near, far, cascade_count, 0.75);
    let inv_view_proj = camera_view_proj.inverse();
    
    let mut cascades = Vec::with_capacity(cascade_count);
    
    for i in 0..cascade_count {
        let cascade_near = splits[i];
        let cascade_far = splits[i + 1];
        
        // Get frustum corners for this cascade
        let corners = get_frustum_corners(inv_view_proj, cascade_near, cascade_far);
        
        // Calculate light-space matrices
        let (view, proj, view_proj) = calculate_cascade_projection(&corners, light_dir);
        
        // Atlas offset (2x2 grid layout)
        let atlas_x = (i % 2) as f32 * 0.5;
        let atlas_y = (i / 2) as f32 * 0.5;
        
        cascades.push(ShadowCascade {
            near: cascade_near,
            far: cascade_far,
            view_matrix: view,
            proj_matrix: proj,
            view_proj_matrix: view_proj,
            atlas_offset: Vec4::new(atlas_x, atlas_y, 0.5, 0.5),
        });
    }
    
    cascades
}

/// Get frustum corners for a depth range
fn get_frustum_corners(inv_view_proj: Mat4, near: f32, far: f32) -> [Vec3; 8] {
    let near_ndc = near; // Assuming linear depth
    let far_ndc = far;
    
    let ndc_corners = [
        Vec4::new(-1.0, -1.0, near_ndc, 1.0),
        Vec4::new( 1.0, -1.0, near_ndc, 1.0),
        Vec4::new(-1.0,  1.0, near_ndc, 1.0),
        Vec4::new( 1.0,  1.0, near_ndc, 1.0),
        Vec4::new(-1.0, -1.0, far_ndc, 1.0),
        Vec4::new( 1.0, -1.0, far_ndc, 1.0),
        Vec4::new(-1.0,  1.0, far_ndc, 1.0),
        Vec4::new( 1.0,  1.0, far_ndc, 1.0),
    ];
    
    let mut world_corners = [Vec3::ZERO; 8];
    for (i, ndc) in ndc_corners.iter().enumerate() {
        let world = inv_view_proj * *ndc;
        world_corners[i] = world.truncate() / world.w;
    }
    
    world_corners
}

// ============================================================================
// PCF SHADOW SAMPLING SIMULATION
// ============================================================================

/// Simulated shadow map (depth values)
struct ShadowMap {
    resolution: u32,
    data: Vec<f32>,
}

impl ShadowMap {
    fn new(resolution: u32) -> Self {
        // Initialize with random depths for realistic simulation
        let mut data = Vec::with_capacity((resolution * resolution) as usize);
        let mut seed = 12345u64;
        for _ in 0..(resolution * resolution) {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
            let depth = (seed as f32 / u64::MAX as f32) * 0.5 + 0.3;
            data.push(depth);
        }
        Self { resolution, data }
    }
    
    fn sample(&self, u: f32, v: f32) -> f32 {
        let x = ((u * self.resolution as f32) as u32).min(self.resolution - 1);
        let y = ((v * self.resolution as f32) as u32).min(self.resolution - 1);
        self.data[(y * self.resolution + x) as usize]
    }
}

/// PCF shadow sampling (NxN kernel)
fn pcf_sample(
    shadow_map: &ShadowMap,
    uv: (f32, f32),
    depth: f32,
    kernel_size: u32,
    bias: f32,
) -> f32 {
    let texel_size = 1.0 / shadow_map.resolution as f32;
    let half_kernel = (kernel_size / 2) as i32;
    let mut shadow = 0.0;
    let mut count = 0;
    
    for y in -half_kernel..=half_kernel {
        for x in -half_kernel..=half_kernel {
            let sample_u = uv.0 + x as f32 * texel_size;
            let sample_v = uv.1 + y as f32 * texel_size;
            
            if sample_u >= 0.0 && sample_u <= 1.0 && sample_v >= 0.0 && sample_v <= 1.0 {
                let sample_depth = shadow_map.sample(sample_u, sample_v);
                if depth - bias <= sample_depth {
                    shadow += 1.0;
                }
                count += 1;
            }
        }
    }
    
    if count > 0 { shadow / count as f32 } else { 1.0 }
}

/// Select cascade for a fragment based on view-space depth
fn select_cascade(cascades: &[ShadowCascade], view_depth: f32) -> usize {
    for (i, cascade) in cascades.iter().enumerate() {
        if view_depth <= cascade.far {
            return i;
        }
    }
    cascades.len() - 1
}

/// VSM (Variance Shadow Maps) moment calculation
fn calculate_vsm_moments(depth: f32) -> (f32, f32) {
    (depth, depth * depth)
}

/// Chebyshev upper bound for VSM
fn chebyshev_upper_bound(moments: (f32, f32), t: f32, min_variance: f32) -> f32 {
    let p: f32 = if t <= moments.0 { 1.0 } else { 0.0 };
    let variance = (moments.1 - moments.0 * moments.0).max(min_variance);
    let d = t - moments.0;
    let p_max = variance / (variance + d * d);
    p.max(p_max)
}

// ============================================================================
// BENCHMARKS
// ============================================================================

fn bench_hiz_pyramid(c: &mut Criterion) {
    let mut group = c.benchmark_group("HiZ_Pyramid");
    
    // Build pyramid at different resolutions
    for &(width, height) in &[(1920, 1080), (2560, 1440), (3840, 2160)] {
        group.throughput(Throughput::Elements(1));
        group.bench_with_input(
            BenchmarkId::new("build", format!("{}x{}", width, height)),
            &(width, height),
            |b, &(w, h)| {
                b.iter(|| build_hiz_pyramid(black_box(w), black_box(h)));
            },
        );
    }
    
    // Downsample single level
    let base = HiZLevel::new(1920, 1080);
    group.bench_function("downsample_single_level", |b| {
        b.iter(|| HiZLevel::downsample_from(black_box(&base)));
    });
    
    // Sample from pyramid
    let pyramid = build_hiz_pyramid(1920, 1080);
    group.bench_function("sample_mip0", |b| {
        b.iter(|| pyramid[0].sample(black_box(0.5), black_box(0.5)));
    });
    
    group.bench_function("sample_mip4", |b| {
        b.iter(|| pyramid[4].sample(black_box(0.5), black_box(0.5)));
    });
    
    group.finish();
}

fn bench_gpu_camera(c: &mut Criterion) {
    let mut group = c.benchmark_group("GpuCamera");
    
    let view_proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0/9.0, 0.1, 1000.0)
        * Mat4::look_at_rh(Vec3::new(0.0, 10.0, 20.0), Vec3::ZERO, Vec3::Y);
    let position = Vec3::new(0.0, 10.0, 20.0);
    
    group.bench_function("from_matrix_1080p", |b| {
        b.iter(|| {
            GpuCamera::from_matrix(
                black_box(view_proj),
                black_box(position),
                black_box(1920),
                black_box(1080),
            )
        });
    });
    
    group.bench_function("from_matrix_4K", |b| {
        b.iter(|| {
            GpuCamera::from_matrix(
                black_box(view_proj),
                black_box(position),
                black_box(3840),
                black_box(2160),
            )
        });
    });
    
    group.finish();
}

fn bench_meshlet_culling(c: &mut Criterion) {
    let mut group = c.benchmark_group("Meshlet_Culling");
    
    // Create test meshlets
    fn generate_meshlets(count: usize) -> Vec<GpuMeshlet> {
        let mut meshlets = Vec::with_capacity(count);
        let mut seed = 42u64;
        for i in 0..count {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let x = ((seed >> 32) as f32 / u32::MAX as f32) * 100.0 - 50.0;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let y = ((seed >> 32) as f32 / u32::MAX as f32) * 50.0;
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let z = ((seed >> 32) as f32 / u32::MAX as f32) * 100.0 - 50.0;
            
            meshlets.push(GpuMeshlet::new(
                Vec3::new(x, y, z),
                1.0 + (i % 3) as f32,
                64,
                128,
            ));
        }
        meshlets
    }
    
    let view_proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0/9.0, 0.1, 1000.0)
        * Mat4::look_at_rh(Vec3::new(0.0, 10.0, 20.0), Vec3::ZERO, Vec3::Y);
    let camera = GpuCamera::from_matrix(view_proj, Vec3::new(0.0, 10.0, 20.0), 1920, 1080);
    let hiz = build_hiz_pyramid(1920, 1080);
    
    // Single meshlet tests
    let meshlet_visible = GpuMeshlet::new(Vec3::new(0.0, 5.0, 0.0), 2.0, 64, 128);
    let meshlet_culled = GpuMeshlet::new(Vec3::new(1000.0, 0.0, 0.0), 1.0, 64, 128);
    
    group.bench_function("frustum_single_visible", |b| {
        b.iter(|| frustum_cull_meshlet(black_box(&meshlet_visible), black_box(&camera.frustum_planes)));
    });
    
    group.bench_function("frustum_single_culled", |b| {
        b.iter(|| frustum_cull_meshlet(black_box(&meshlet_culled), black_box(&camera.frustum_planes)));
    });
    
    group.bench_function("backface_single", |b| {
        b.iter(|| backface_cull_meshlet(black_box(&meshlet_visible), black_box(Vec3::new(0.0, 0.0, -1.0))));
    });
    
    group.bench_function("occlusion_single", |b| {
        b.iter(|| {
            occlusion_cull_meshlet(
                black_box(&meshlet_visible),
                black_box(view_proj),
                black_box(&hiz),
                black_box(1920),
                black_box(1080),
            )
        });
    });
    
    // Batch culling
    for &count in &[1000, 5000, 10000, 50000] {
        let meshlets = generate_meshlets(count);
        
        group.throughput(Throughput::Elements(count as u64));
        group.bench_with_input(
            BenchmarkId::new("full_cull", count),
            &meshlets,
            |b, meshlets| {
                b.iter(|| cull_meshlets(black_box(meshlets), black_box(&camera), black_box(&hiz)));
            },
        );
    }
    
    group.finish();
}

fn bench_cascade_splitting(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cascade_Splitting");
    
    // Cascade split calculation
    group.bench_function("calculate_splits_4_cascades", |b| {
        b.iter(|| calculate_cascade_splits(black_box(0.1), black_box(1000.0), black_box(4), black_box(0.75)));
    });
    
    group.bench_function("calculate_splits_8_cascades", |b| {
        b.iter(|| calculate_cascade_splits(black_box(0.1), black_box(1000.0), black_box(8), black_box(0.75)));
    });
    
    // Full cascade building
    let camera_view_proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0/9.0, 0.1, 1000.0)
        * Mat4::look_at_rh(Vec3::new(0.0, 10.0, 20.0), Vec3::ZERO, Vec3::Y);
    let light_dir = Vec3::new(0.5, -1.0, 0.3).normalize();
    
    for &cascade_count in &[2, 4, 6, 8] {
        group.bench_with_input(
            BenchmarkId::new("build_cascades", cascade_count),
            &cascade_count,
            |b, &count| {
                b.iter(|| {
                    build_shadow_cascades(
                        black_box(camera_view_proj),
                        black_box(light_dir),
                        black_box(0.1),
                        black_box(1000.0),
                        black_box(count),
                    )
                });
            },
        );
    }
    
    // Cascade selection
    let cascades = build_shadow_cascades(camera_view_proj, light_dir, 0.1, 1000.0, 4);
    
    group.bench_function("select_cascade_near", |b| {
        b.iter(|| select_cascade(black_box(&cascades), black_box(5.0)));
    });
    
    group.bench_function("select_cascade_mid", |b| {
        b.iter(|| select_cascade(black_box(&cascades), black_box(100.0)));
    });
    
    group.bench_function("select_cascade_far", |b| {
        b.iter(|| select_cascade(black_box(&cascades), black_box(800.0)));
    });
    
    group.finish();
}

fn bench_shadow_matrix(c: &mut Criterion) {
    let mut group = c.benchmark_group("Shadow_Matrix");
    
    let corners = [
        Vec3::new(-10.0, -10.0, 0.1),
        Vec3::new(10.0, -10.0, 0.1),
        Vec3::new(-10.0, 10.0, 0.1),
        Vec3::new(10.0, 10.0, 0.1),
        Vec3::new(-50.0, -50.0, 100.0),
        Vec3::new(50.0, -50.0, 100.0),
        Vec3::new(-50.0, 50.0, 100.0),
        Vec3::new(50.0, 50.0, 100.0),
    ];
    let light_dir = Vec3::new(0.5, -1.0, 0.3).normalize();
    
    group.bench_function("calculate_projection", |b| {
        b.iter(|| calculate_cascade_projection(black_box(&corners), black_box(light_dir)));
    });
    
    // GPU cascade conversion
    let cascade = ShadowCascade {
        near: 0.1,
        far: 100.0,
        view_matrix: Mat4::IDENTITY,
        proj_matrix: Mat4::IDENTITY,
        view_proj_matrix: Mat4::IDENTITY,
        atlas_offset: Vec4::new(0.0, 0.0, 0.5, 0.5),
    };
    
    group.bench_function("to_gpu_cascade", |b| {
        b.iter(|| GpuShadowCascade::from(black_box(&cascade)));
    });
    
    group.finish();
}

fn bench_pcf_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("PCF_Sampling");
    
    let shadow_map = ShadowMap::new(2048);
    let uv = (0.5, 0.5);
    let depth = 0.5;
    let bias = 0.005;
    
    // Different kernel sizes
    for &kernel_size in &[3, 5, 7, 9] {
        group.bench_with_input(
            BenchmarkId::new("kernel", format!("{}x{}", kernel_size, kernel_size)),
            &kernel_size,
            |b, &ks| {
                b.iter(|| {
                    pcf_sample(
                        black_box(&shadow_map),
                        black_box(uv),
                        black_box(depth),
                        black_box(ks),
                        black_box(bias),
                    )
                });
            },
        );
    }
    
    // Batch PCF sampling (typical frame: many pixels)
    for &pixel_count in &[1000, 10000, 100000] {
        group.throughput(Throughput::Elements(pixel_count as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_5x5", pixel_count),
            &pixel_count,
            |b, &count| {
                b.iter(|| {
                    let mut total_shadow = 0.0;
                    for i in 0..count {
                        let u = (i % 1000) as f32 / 1000.0;
                        let v = (i / 1000) as f32 / 1000.0;
                        total_shadow += pcf_sample(&shadow_map, (u, v), depth, 5, bias);
                    }
                    total_shadow
                });
            },
        );
    }
    
    group.finish();
}

fn bench_vsm_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("VSM_Sampling");
    
    group.bench_function("calculate_moments", |b| {
        b.iter(|| calculate_vsm_moments(black_box(0.5)));
    });
    
    let moments = (0.5, 0.26); // Mean and variance
    group.bench_function("chebyshev_bound", |b| {
        b.iter(|| chebyshev_upper_bound(black_box(moments), black_box(0.6), black_box(0.0001)));
    });
    
    // Batch VSM sampling
    for &pixel_count in &[1000, 10000, 100000] {
        group.throughput(Throughput::Elements(pixel_count as u64));
        group.bench_with_input(
            BenchmarkId::new("batch", pixel_count),
            &pixel_count,
            |b, &count| {
                b.iter(|| {
                    let mut total_shadow = 0.0;
                    for i in 0..count {
                        let t = (i as f32 / count as f32) * 0.2 + 0.4;
                        total_shadow += chebyshev_upper_bound(moments, t, 0.0001);
                    }
                    total_shadow
                });
            },
        );
    }
    
    group.finish();
}

fn bench_full_shadow_pass(c: &mut Criterion) {
    let mut group = c.benchmark_group("Full_Shadow_Pass");
    
    let camera_view_proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0/9.0, 0.1, 1000.0)
        * Mat4::look_at_rh(Vec3::new(0.0, 10.0, 20.0), Vec3::ZERO, Vec3::Y);
    let light_dir = Vec3::new(0.5, -1.0, 0.3).normalize();
    
    // Full cascade setup + fragment sampling
    for &cascade_count in &[2, 4] {
        let cascades = build_shadow_cascades(camera_view_proj, light_dir, 0.1, 1000.0, cascade_count);
        let shadow_map = ShadowMap::new(2048);
        
        for &fragment_count in &[10000, 100000] {
            group.throughput(Throughput::Elements(fragment_count as u64));
            group.bench_with_input(
                BenchmarkId::new(format!("{}_cascades", cascade_count), fragment_count),
                &(cascade_count, fragment_count),
                |b, &(_, frag_count)| {
                    b.iter(|| {
                        let mut total_shadow = 0.0;
                        for i in 0..frag_count {
                            // Simulate fragment depth
                            let view_depth = (i as f32 / frag_count as f32) * 1000.0;
                            let cascade_idx = select_cascade(&cascades, view_depth);
                            
                            // Sample shadow
                            let u = (i % 1000) as f32 / 1000.0;
                            let v = (i / 1000) as f32 / 1000.0;
                            total_shadow += pcf_sample(&shadow_map, (u, v), 0.5, 3, 0.005);
                            
                            black_box(cascade_idx);
                        }
                        total_shadow
                    });
                },
            );
        }
    }
    
    group.finish();
}

fn bench_cull_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cull_Stats");
    
    // Generate meshlets with known visibility distribution
    fn generate_scene_meshlets(total: usize, visible_ratio: f32) -> Vec<GpuMeshlet> {
        let mut meshlets = Vec::with_capacity(total);
        let visible_count = (total as f32 * visible_ratio) as usize;
        
        // Visible meshlets (in frustum)
        for i in 0..visible_count {
            let angle = (i as f32 / visible_count as f32) * std::f32::consts::TAU;
            let radius = 10.0 + (i % 20) as f32;
            meshlets.push(GpuMeshlet::new(
                Vec3::new(angle.cos() * radius, 5.0, angle.sin() * radius - 10.0),
                1.5,
                64,
                128,
            ));
        }
        
        // Culled meshlets (outside frustum)
        for i in visible_count..total {
            meshlets.push(GpuMeshlet::new(
                Vec3::new(500.0 + i as f32, 0.0, 0.0), // Far away
                1.0,
                64,
                128,
            ));
        }
        
        meshlets
    }
    
    let view_proj = Mat4::perspective_rh(std::f32::consts::FRAC_PI_4, 16.0/9.0, 0.1, 500.0)
        * Mat4::look_at_rh(Vec3::new(0.0, 10.0, 20.0), Vec3::ZERO, Vec3::Y);
    let camera = GpuCamera::from_matrix(view_proj, Vec3::new(0.0, 10.0, 20.0), 1920, 1080);
    let hiz = build_hiz_pyramid(1920, 1080);
    
    // Test different visibility ratios
    for &(total, visible_ratio) in &[(10000, 0.1), (10000, 0.5), (10000, 0.9)] {
        let meshlets = generate_scene_meshlets(total, visible_ratio);
        
        group.throughput(Throughput::Elements(total as u64));
        group.bench_with_input(
            BenchmarkId::new("visibility", format!("{}%", (visible_ratio * 100.0) as u32)),
            &meshlets,
            |b, meshlets| {
                b.iter(|| cull_meshlets(black_box(meshlets), black_box(&camera), black_box(&hiz)));
            },
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_hiz_pyramid,
    bench_gpu_camera,
    bench_meshlet_culling,
    bench_cascade_splitting,
    bench_shadow_matrix,
    bench_pcf_sampling,
    bench_vsm_sampling,
    bench_full_shadow_pass,
    bench_cull_stats,
);

criterion_main!(benches);
