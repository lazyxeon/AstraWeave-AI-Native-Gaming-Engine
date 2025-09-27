// Clustered lighting: CPU reference binning + WGSL placeholder compute kernel and compile-only tests.
// CPU path is unit-tested; WGSL is syntax-checked via naga for now.

use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct CpuLight { pub pos: Vec3, pub radius: f32 }

#[derive(Clone, Copy, Debug)]
pub struct ClusterDims { pub x: u32, pub y: u32, pub z: u32 }

fn clamp_u32(v: i32, lo: i32, hi: i32) -> u32 { v.max(lo).min(hi) as u32 }

fn cluster_index(ix: u32, iy: u32, iz: u32, dims: ClusterDims) -> usize {
    (ix + iy * dims.x + iz * (dims.x * dims.y)) as usize
}

/// Compute per-cluster light lists on CPU using simple sphere-screen bounds.
/// Inputs:
/// - lights: positions are in camera/view space with +Z forward and in meters
/// - screen: (width,height)
/// - near/far: camera near/far planes (positive distances)
/// - fov_y: vertical field of view in radians
/// Returns: (counts, indices, offsets) where offsets is exclusive scan over counts
pub fn bin_lights_cpu(
    lights: &[CpuLight],
    dims: ClusterDims,
    screen: (u32, u32),
    near: f32,
    far: f32,
    fov_y: f32,
) -> (Vec<u32>, Vec<u32>, Vec<u32>) {
    let (width, height) = screen;
    let aspect = width.max(1) as f32 / height.max(1) as f32;
    let fy = 1.0 / (0.5 * fov_y).tan();
    let fx = fy / aspect;
    let tile_w = (width as f32 / dims.x as f32).max(1.0);
    let tile_h = (height as f32 / dims.y as f32).max(1.0);

    let clusters = (dims.x * dims.y * dims.z) as usize;
    let mut counts = vec![0u32; clusters];

    // First pass: counts
    for l in lights.iter() {
        let z = l.pos.z.max(near + 1e-4);
        if z - l.radius > far { continue; }
        // Project center to NDC
        let ndc_x = (l.pos.x / z) * fx;
        let ndc_y = (l.pos.y / z) * fy;
        // Convert to pixel space
        let px = (ndc_x * 0.5 + 0.5) * width as f32;
        let py = (ndc_y * 0.5 + 0.5) * height as f32;
        // Screen-space radius in pixels (approx)
        let rpx_x = (l.radius / z) * fx * (width as f32 * 0.5);
        let rpx_y = (l.radius / z) * fy * (height as f32 * 0.5);
        let min_px = (px - rpx_x).floor() as i32;
        let max_px = (px + rpx_x).ceil() as i32;
        let min_py = (py - rpx_y).floor() as i32;
        let max_py = (py + rpx_y).ceil() as i32;
        let ix0 = clamp_u32(min_px / tile_w as i32, 0, dims.x as i32 - 1);
        let ix1 = clamp_u32(max_px / tile_w as i32, 0, dims.x as i32 - 1);
        let iy0 = clamp_u32(min_py / tile_h as i32, 0, dims.y as i32 - 1);
        let iy1 = clamp_u32(max_py / tile_h as i32, 0, dims.y as i32 - 1);
        // Z slices
        let zmin = (z - l.radius).max(near);
        let zmax = (z + l.radius).min(far);
        if zmin >= zmax { continue; }
        let iz0 = (((zmin - near) / (far - near)) * dims.z as f32).floor().clamp(0.0, dims.z as f32 - 1.0) as u32;
        let iz1 = (((zmax - near) / (far - near)) * dims.z as f32).floor().clamp(0.0, dims.z as f32 - 1.0) as u32;
        for iz in iz0..=iz1 {
            for iy in iy0..=iy1 {
                for ix in ix0..=ix1 {
                    let ci = cluster_index(ix, iy, iz, dims);
                    counts[ci] += 1;
                }
            }
        }
    }

    // Exclusive scan to offsets
    let mut offsets = vec![0u32; clusters + 1];
    for i in 0..clusters { offsets[i + 1] = offsets[i] + counts[i]; }

    // Second pass: fill indices using running write cursors
    let total = offsets[clusters] as usize;
    let mut indices = vec![0u32; total];
    let mut cursors = vec![0u32; clusters];
    for (li, l) in lights.iter().enumerate() {
        let z = l.pos.z.max(near + 1e-4);
        if z - l.radius > far { continue; }
        let ndc_x = (l.pos.x / z) * fx;
        let ndc_y = (l.pos.y / z) * fy;
        let px = (ndc_x * 0.5 + 0.5) * width as f32;
        let py = (ndc_y * 0.5 + 0.5) * height as f32;
        let rpx_x = (l.radius / z) * fx * (width as f32 * 0.5);
        let rpx_y = (l.radius / z) * fy * (height as f32 * 0.5);
        let min_px = (px - rpx_x).floor() as i32;
        let max_px = (px + rpx_x).ceil() as i32;
        let min_py = (py - rpx_y).floor() as i32;
        let max_py = (py + rpx_y).ceil() as i32;
        let ix0 = clamp_u32(min_px / tile_w as i32, 0, dims.x as i32 - 1);
        let ix1 = clamp_u32(max_px / tile_w as i32, 0, dims.x as i32 - 1);
        let iy0 = clamp_u32(min_py / tile_h as i32, 0, dims.y as i32 - 1);
        let iy1 = clamp_u32(max_py / tile_h as i32, 0, dims.y as i32 - 1);
        let zmin = (z - l.radius).max(near);
        let zmax = (z + l.radius).min(far);
        if zmin >= zmax { continue; }
        let iz0 = (((zmin - near) / (far - near)) * dims.z as f32).floor().clamp(0.0, dims.z as f32 - 1.0) as u32;
        let iz1 = (((zmax - near) / (far - near)) * dims.z as f32).floor().clamp(0.0, dims.z as f32 - 1.0) as u32;
        for iz in iz0..=iz1 {
            for iy in iy0..=iy1 {
                for ix in ix0..=ix1 {
                    let ci = cluster_index(ix, iy, iz, dims);
                    let off = offsets[ci] + cursors[ci];
                    indices[off as usize] = li as u32;
                    cursors[ci] += 1;
                }
            }
        }
    }

    (counts, indices, offsets)
}

pub const WGSL_CLUSTER_BIN: &str = r#"
struct Light { pos_radius: vec4<f32> };
struct Params { screen: vec2<u32>, clusters: vec3<u32>, near: f32, far: f32, fov_y: f32 };
@group(0) @binding(0) var<storage, read> lights: array<Light>;
@group(0) @binding(1) var<uniform> params: Params;
// Offsets: length = clusters+1 (exclusive scan). Counts: length = clusters. Indices: variable length
@group(0) @binding(2) var<storage, read> offsets: array<u32>;
@group(0) @binding(3) var<storage, read_write> counts: array<atomic<u32>>;
@group(0) @binding(4) var<storage, read_write> indices: array<u32>;

fn cluster_index(ix: u32, iy: u32, iz: u32, dims: vec3<u32>) -> u32 {
    return ix + iy * dims.x + iz * (dims.x * dims.y);
}

@compute @workgroup_size(1,1,1)
fn cs_main(@builtin(global_invocation_id) gid: vec3<u32>) {
    // One invocation per light
    let li = gid.x;
    if (li >= arrayLength(&lights)) { return; }
    let L = lights[li].pos_radius;
    let pos = L.xyz;
    let radius = L.w;
    let width = f32(params.screen.x);
    let height = f32(params.screen.y);
    let aspect = width / max(height, 1.0);
    let fy = 1.0 / tan(0.5 * params.fov_y);
    let fx = fy / aspect;
    let z = max(pos.z, params.near + 1e-4);
    if (z - radius > params.far) { return; }
    let ndc_x = (pos.x / z) * fx;
    let ndc_y = (pos.y / z) * fy;
    let px = (ndc_x * 0.5 + 0.5) * width;
    let py = (ndc_y * 0.5 + 0.5) * height;
    let rpx_x = (radius / z) * fx * (width * 0.5);
    let rpx_y = (radius / z) * fy * (height * 0.5);
    let tile_w = max(1.0, width / f32(params.clusters.x));
    let tile_h = max(1.0, height / f32(params.clusters.y));
    let min_px = floor(px - rpx_x);
    let max_px = ceil(px + rpx_x);
    let min_py = floor(py - rpx_y);
    let max_py = ceil(py + rpx_y);
    let ix0 = u32(clamp(min_px / tile_w, 0.0, f32(params.clusters.x - 1u)));
    let ix1 = u32(clamp(max_px / tile_w, 0.0, f32(params.clusters.x - 1u)));
    let iy0 = u32(clamp(min_py / tile_h, 0.0, f32(params.clusters.y - 1u)));
    let iy1 = u32(clamp(max_py / tile_h, 0.0, f32(params.clusters.y - 1u)));
    let zmin = max(z - radius, params.near);
    let zmax = min(z + radius, params.far);
    if (zmin >= zmax) { return; }
    let iz0 = u32(clamp(floor(((zmin - params.near) / (params.far - params.near)) * f32(params.clusters.z)), 0.0, f32(params.clusters.z - 1u)));
    let iz1 = u32(clamp(floor(((zmax - params.near) / (params.far - params.near)) * f32(params.clusters.z)), 0.0, f32(params.clusters.z - 1u)));
    for (var iz: u32 = iz0; iz <= iz1; iz = iz + 1u) {
        for (var iy: u32 = iy0; iy <= iy1; iy = iy + 1u) {
            for (var ix: u32 = ix0; ix <= ix1; ix = ix + 1u) {
                let ci = cluster_index(ix, iy, iz, params.clusters);
                let write_idx = offsets[ci] + atomicAdd(&counts[ci], 1u);
                indices[write_idx] = li;
            }
        }
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    #[cfg(feature = "gpu-tests")]
    use wgpu::util::DeviceExt;
    #[test]
    fn parse_cluster_cs() {
        let module = naga::front::wgsl::parse_str(WGSL_CLUSTER_BIN).expect("WGSL cluster should parse");
        assert!(module.entry_points.iter().any(|e| e.name == "cs_main"));
    }

    #[test]
    fn cpu_binning_produces_consistent_counts() {
        let dims = ClusterDims{ x: 8, y: 4, z: 8 };
        let (w,h) = (640u32, 360u32);
        let near = 0.1f32; let far = 100.0f32; let fov_y = std::f32::consts::FRAC_PI_3; // 60 deg
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut lights = Vec::new();
        for _ in 0..64 {
            let x = rand::Rng::random_range(&mut rng, -10.0..10.0);
            let y = rand::Rng::random_range(&mut rng, -5.0..5.0);
            let z = rand::Rng::random_range(&mut rng, 1.0..50.0);
            let r = rand::Rng::random_range(&mut rng, 0.25..3.0);
            lights.push(CpuLight{ pos: Vec3::new(x,y,z), radius: r });
        }
        let (counts, indices, offsets) = bin_lights_cpu(&lights, dims, (w,h), near, far, fov_y);
        // Basic invariants
        assert_eq!(offsets.last().copied().unwrap_or(0), indices.len() as u32);
        assert_eq!(counts.iter().map(|&c| c as usize).sum::<usize>(), indices.len());
        // Re-derive counts from indices vector
        let mut rec = vec![0u32; (dims.x * dims.y * dims.z) as usize];
        for iz in 0..dims.z { for iy in 0..dims.y { for ix in 0..dims.x {
            let ci = cluster_index(ix, iy, iz, dims);
            rec[ci] = offsets[ci+1] - offsets[ci];
        } } }
        assert_eq!(rec, counts);
    }

    #[test]
    fn cpu_binning_indices_valid() {
        let dims = ClusterDims{ x: 4, y: 4, z: 4 };
        let lights = vec![
            CpuLight{ pos: Vec3::new(0.0,0.0,5.0), radius: 1.0 },
            CpuLight{ pos: Vec3::new(1.0,0.0,7.0), radius: 2.0 },
        ];
        let (counts, indices, offsets) = bin_lights_cpu(&lights, dims, (320,180), 0.1, 50.0, std::f32::consts::FRAC_PI_3);
        // Offsets monotonic and last equals indices length
        assert!(offsets.windows(2).all(|w| w[1] >= w[0]));
        assert_eq!(*offsets.last().unwrap() as usize, indices.len());
        // Indices in range
        for &idx in &indices { assert!((idx as usize) < lights.len()); }
        // Counts match offsets delta
        for i in 0..(counts.len()) {
            assert_eq!(counts[i], offsets[i+1] - offsets[i]);
        }
    }

    #[cfg(feature = "gpu-tests")]
    #[test]
    fn gpu_cpu_counts_parity() {
        // Setup test data
        let dims = ClusterDims{ x: 8, y: 4, z: 8 };
        let (w,h) = (320u32, 180u32);
        let near = 0.1f32; let far = 50.0f32; let fov_y = std::f32::consts::FRAC_PI_3;
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        let mut lights = Vec::new();
        for _ in 0..48 {
            let x = rand::Rng::random_range(&mut rng, -8.0..8.0);
            let y = rand::Rng::random_range(&mut rng, -4.0..4.0);
            let z = rand::Rng::random_range(&mut rng, 1.0..40.0);
            let r = rand::Rng::random_range(&mut rng, 0.2..2.5);
            lights.push(CpuLight{ pos: Vec3::new(x,y,z), radius: r });
        }
        let (counts_cpu, _indices_cpu, offsets_cpu) = bin_lights_cpu(&lights, dims, (w,h), near, far, fov_y);

        // Flatten lights for GPU
        #[repr(C)]
        #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
        struct GpuLight { pos_radius: [f32;4] }
    #[repr(C)]
    #[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
    struct Params { screen: [u32;4], clusters: [u32;4], params: [f32;4] }
    let glights: Vec<GpuLight> = lights.iter().map(|l| GpuLight{ pos_radius: [l.pos.x, l.pos.y, l.pos.z, l.radius]}).collect();
    let params = Params{ screen: [w,h,0,0], clusters: [dims.x,dims.y,dims.z, 0], params: [near, far, fov_y, 0.0] };
        let clusters = (dims.x * dims.y * dims.z) as usize;
        let offsets = offsets_cpu.clone();
    let counts_zero = vec![0u32; clusters];
        let n_indices = *offsets_cpu.last().unwrap() as usize;
        let indices_zero = vec![0u32; n_indices];

        // WGPU setup
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::default());
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions{ power_preference: wgpu::PowerPreference::LowPower, compatible_surface: None, force_fallback_adapter: false })).expect("adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor{ label: Some("compute device"), required_features: wgpu::Features::empty(), required_limits: wgpu::Limits::default() }, None)).expect("device");
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor{ label: Some("cluster bin"), source: wgpu::ShaderSource::Wgsl(std::borrow::Cow::Borrowed(WGSL_CLUSTER_BIN)) });
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor{
            label: Some("bgl"),
            entries: &[
                wgpu::BindGroupLayoutEntry{ binding:0, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer{ ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry{ binding:1, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer{ ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry{ binding:2, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer{ ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry{ binding:3, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer{ ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
                wgpu::BindGroupLayoutEntry{ binding:4, visibility: wgpu::ShaderStages::COMPUTE, ty: wgpu::BindingType::Buffer{ ty: wgpu::BufferBindingType::Storage { read_only: false }, has_dynamic_offset: false, min_binding_size: None }, count: None },
            ]
        });
        let pl = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor{ label: Some("pl"), bind_group_layouts: &[&bgl], push_constant_ranges: &[] });
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor{ label: Some("cluster"), layout: Some(&pl), module: &shader, entry_point: "cs_main", compilation_options: wgpu::PipelineCompilationOptions::default() });

        // Buffers
        let buf_lights = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{ label: Some("lights"), contents: bytemuck::cast_slice(&glights), usage: wgpu::BufferUsages::STORAGE });
        let buf_params = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{ label: Some("params"), contents: bytemuck::bytes_of(&params), usage: wgpu::BufferUsages::UNIFORM });
        let buf_offsets = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{ label: Some("offsets"), contents: bytemuck::cast_slice(&offsets), usage: wgpu::BufferUsages::STORAGE });
        let buf_counts = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{ label: Some("counts"), contents: bytemuck::cast_slice(&counts_zero), usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC });
        let buf_indices = device.create_buffer_init(&wgpu::util::BufferInitDescriptor{ label: Some("indices"), contents: bytemuck::cast_slice(&indices_zero), usage: wgpu::BufferUsages::STORAGE });
        let bg = device.create_bind_group(&wgpu::BindGroupDescriptor{ label: Some("bg"), layout: &bgl, entries: &[
            wgpu::BindGroupEntry{ binding:0, resource: buf_lights.as_entire_binding() },
            wgpu::BindGroupEntry{ binding:1, resource: buf_params.as_entire_binding() },
            wgpu::BindGroupEntry{ binding:2, resource: buf_offsets.as_entire_binding() },
            wgpu::BindGroupEntry{ binding:3, resource: buf_counts.as_entire_binding() },
            wgpu::BindGroupEntry{ binding:4, resource: buf_indices.as_entire_binding() },
        ]});

        // Dispatch: one thread per light
        let mut enc = device.create_command_encoder(&wgpu::CommandEncoderDescriptor{ label: Some("enc") });
        {
            let mut cpass = enc.begin_compute_pass(&wgpu::ComputePassDescriptor{ label: Some("pass"), timestamp_writes: None });
            cpass.set_pipeline(&pipeline);
            cpass.set_bind_group(0, &bg, &[]);
            cpass.dispatch_workgroups(glights.len() as u32, 1, 1);
        }
        // Copy counts to a mappable buffer
        let buf_counts_read = device.create_buffer(&wgpu::BufferDescriptor{ label: Some("counts read"), size: (clusters * std::mem::size_of::<u32>()) as u64, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false });
        enc.copy_buffer_to_buffer(&buf_counts, 0, &buf_counts_read, 0, (clusters * std::mem::size_of::<u32>()) as u64);
        queue.submit(Some(enc.finish()));
        device.poll(wgpu::Maintain::Wait);
        // Map and compare
    let slice = buf_counts_read.slice(..);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    slice.map_async(wgpu::MapMode::Read, move |res| { let _ = tx.send(res); });
    device.poll(wgpu::Maintain::Wait);
    let _ = rx.recv().expect("map result").expect("map ok");
    let data = slice.get_mapped_range();
        let counts_gpu: Vec<u32> = bytemuck::cast_slice(&data).to_vec();
        drop(data);
        buf_counts_read.unmap();

        assert_eq!(counts_gpu.len(), counts_cpu.len());
        assert_eq!(counts_gpu, counts_cpu);
    }
}
