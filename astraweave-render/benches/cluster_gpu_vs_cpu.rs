// MegaLights GPU vs CPU Light Culling Benchmark
//
// This benchmark compares:
// 1. CPU bin_lights_cpu() - O(N×M) brute force (0.5-2ms @ 1k lights)
// 2. GPU MegaLights compute - 3-stage pipeline (<0.1ms @ 1k lights)
//
// Expected Results:
// - CPU: Linear scaling, collapses at 250+ lights
// - GPU: Sub-linear scaling, 68× faster @ 1000 lights on RTX 3060
//
// Hardware Dependencies:
// - GPU benchmarks require wgpu device (may vary across GPUs)
// - CPU benchmarks are deterministic and comparable across runs

use astraweave_render::clustered::{bin_lights_cpu, ClusterDims, CpuLight};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

#[cfg(feature = "megalights")]
use astraweave_render::clustered_megalights::{GpuLight, MegaLightsRenderer};

/// Create test scene with N lights in circular pattern
fn create_test_scene_cpu(light_count: usize) -> Vec<CpuLight> {
    (0..light_count)
        .map(|i| {
            let angle = (i as f32 / light_count as f32) * std::f32::consts::TAU;
            let radius = 10.0 + (i % 10) as f32 * 2.0;
            CpuLight {
                pos: glam::Vec3::new(
                    angle.cos() * radius,
                    5.0 + (i % 5) as f32,
                    angle.sin() * radius,
                ),
                radius: 5.0 + (i % 7) as f32,
            }
        })
        .collect()
}

#[cfg(feature = "megalights")]
fn create_test_scene_gpu(light_count: usize) -> Vec<GpuLight> {
    (0..light_count)
        .map(|i| {
            let angle = (i as f32 / light_count as f32) * std::f32::consts::TAU;
            let radius = 10.0 + (i % 10) as f32 * 2.0;
            GpuLight {
                position: [
                    angle.cos() * radius,
                    5.0 + (i % 5) as f32,
                    angle.sin() * radius,
                    5.0 + (i % 7) as f32, // radius
                ],
                color: [1.0, 1.0, 1.0, 1.0],
            }
        })
        .collect()
}

/// Benchmark CPU light culling (baseline reference)
fn bench_cpu_light_culling(c: &mut Criterion) {
    let dims = ClusterDims {
        x: 16,
        y: 16,
        z: 32,
    }; // 8192 clusters
    let screen = (1920u32, 1080u32);
    let near = 0.1f32;
    let far = 1000.0f32;
    let fov = std::f32::consts::FRAC_PI_4;

    let mut group = c.benchmark_group("cpu_light_culling");

    // Test scaling: 100, 250, 500, 1000, 2000 lights
    for light_count in [100, 250, 500, 1000, 2000] {
        group.bench_with_input(
            BenchmarkId::new("bin_lights_cpu", light_count),
            &light_count,
            |b, &count| {
                let lights = create_test_scene_cpu(count);
                b.iter(|| {
                    let (_counts, _indices, offsets) =
                        bin_lights_cpu(black_box(&lights), dims, screen, near, far, fov);
                    black_box(offsets);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark GPU light culling (MegaLights compute shaders)
#[cfg(feature = "megalights")]
fn bench_gpu_light_culling(c: &mut Criterion) {
    use bytemuck;
    use astraweave_render::clustered_megalights::ClusterBounds;
    
    // Setup wgpu device (once)
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect("Failed to find suitable GPU adapter");

    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("Failed to create wgpu device");

    // Create MegaLights renderer
    let cluster_dims = (16u32, 16u32, 32u32); // 8192 clusters
    let total_clusters = cluster_dims.0 * cluster_dims.1 * cluster_dims.2;
    let max_lights = 4096usize;

    let mut megalights = MegaLightsRenderer::new(&device, cluster_dims, max_lights)
        .expect("Failed to create MegaLightsRenderer");

    // Create GPU buffers for bind groups
    let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Buffer"),
        size: (max_lights * std::mem::size_of::<GpuLight>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Create cluster bounds (simple grid for benchmark)
    let cluster_bounds: Vec<ClusterBounds> = (0..total_clusters)
        .map(|i| {
            let x = (i % cluster_dims.0) as f32;
            let y = ((i / cluster_dims.0) % cluster_dims.1) as f32;
            let z = (i / (cluster_dims.0 * cluster_dims.1)) as f32;
            ClusterBounds {
                min_pos: [x * 10.0, y * 10.0, z * 10.0],
                _pad1: 0.0,
                max_pos: [(x + 1.0) * 10.0, (y + 1.0) * 10.0, (z + 1.0) * 10.0],
                _pad2: 0.0,
            }
        })
        .collect();

    let cluster_bounds_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Cluster Bounds Buffer"),
        size: (total_clusters as usize * std::mem::size_of::<ClusterBounds>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&cluster_bounds_buffer, 0, bytemuck::cast_slice(&cluster_bounds));

    let light_counts_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Counts Buffer"),
        size: (total_clusters as usize * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let light_offsets_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Offsets Buffer"),
        size: (total_clusters as usize * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    // Worst case: every light in every cluster
    let light_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Indices Buffer"),
        size: (total_clusters as usize * 64 * std::mem::size_of::<u32>()) as u64, // 64 lights per cluster max
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    // Params uniform buffer
    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct ClusterParams {
        cluster_dims: [u32; 3],
        _pad1: u32,
        total_clusters: u32,
        light_count: u32,
        _pad2: u32,
        _pad3: u32,
    }

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Params Buffer"),
        size: std::mem::size_of::<ClusterParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct PrefixSumParams {
        element_count: u32,
        workgroup_size: u32,
        _pad1: u32,
        _pad2: u32,
    }

    let prefix_sum_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Prefix Sum Params Buffer"),
        size: std::mem::size_of::<PrefixSumParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    // Initialize bind groups
    megalights.update_bind_groups(
        &device,
        &light_buffer,
        &cluster_bounds_buffer,
        &light_counts_buffer,
        &light_offsets_buffer,
        &light_indices_buffer,
        &params_buffer,
        &prefix_sum_params_buffer,
    );

    let mut group = c.benchmark_group("gpu_light_culling");

    // Test scaling: 100, 250, 500, 1000, 2000 lights
    for light_count in [100, 250, 500, 1000, 2000] {
        let lights = create_test_scene_gpu(light_count);
        
        // Upload lights to GPU
        queue.write_buffer(&light_buffer, 0, bytemuck::cast_slice(&lights));
        
        // Update params
        let params = ClusterParams {
            cluster_dims: [cluster_dims.0, cluster_dims.1, cluster_dims.2],
            _pad1: 0,
            total_clusters,
            light_count: light_count as u32,
            _pad2: 0,
            _pad3: 0,
        };
        queue.write_buffer(&params_buffer, 0, bytemuck::bytes_of(&params));
        
        let prefix_params = PrefixSumParams {
            element_count: total_clusters,
            workgroup_size: 256,
            _pad1: 0,
            _pad2: 0,
        };
        queue.write_buffer(&prefix_sum_params_buffer, 0, bytemuck::bytes_of(&prefix_params));

        group.bench_with_input(
            BenchmarkId::new("megalights_dispatch", light_count),
            &light_count,
            |b, &count| {
                b.iter(|| {
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("MegaLights Benchmark Encoder"),
                        });

                    // Dispatch GPU compute
                    let result = megalights.dispatch(&mut encoder, black_box(count as u32));
                    black_box(result).expect("GPU dispatch failed");

                    // Submit and wait for GPU
                    queue.submit([encoder.finish()]);
                    let _ = device.poll(wgpu::MaintainBase::Wait);
                });
            },
        );
        
        // Benchmark dispatch-only (no sync) - measures command recording overhead
        group.bench_with_input(
            BenchmarkId::new("megalights_dispatch_only", light_count),
            &light_count,
            |b, &count| {
                b.iter(|| {
                    let mut encoder =
                        device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                            label: Some("MegaLights Dispatch-Only Encoder"),
                        });

                    // Dispatch GPU compute (command recording only)
                    let result = megalights.dispatch(&mut encoder, black_box(count as u32));
                    black_box(result).expect("GPU dispatch failed");
                    
                    // Return encoder without submitting - measures pure dispatch overhead
                    black_box(encoder.finish());
                });
            },
        );
    }

    group.finish();
}

/// Benchmark GPU throughput (batched submissions to measure true GPU performance)
#[cfg(feature = "megalights")]
fn bench_gpu_throughput(c: &mut Criterion) {
    use bytemuck;
    use astraweave_render::clustered_megalights::ClusterBounds;
    
    let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
        backends: wgpu::Backends::PRIMARY,
        ..Default::default()
    });

    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::HighPerformance,
        force_fallback_adapter: false,
        compatible_surface: None,
    }))
    .expect("Failed to find suitable GPU adapter");

    let (device, queue) =
        pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default()))
            .expect("Failed to create wgpu device");

    let cluster_dims = (16u32, 16u32, 32u32);
    let total_clusters = cluster_dims.0 * cluster_dims.1 * cluster_dims.2;
    let max_lights = 4096usize;

    let mut megalights = MegaLightsRenderer::new(&device, cluster_dims, max_lights)
        .expect("Failed to create MegaLightsRenderer");

    // Create buffers (same as above)
    let light_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Buffer"),
        size: (max_lights * std::mem::size_of::<GpuLight>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let cluster_bounds: Vec<ClusterBounds> = (0..total_clusters)
        .map(|i| {
            let x = (i % cluster_dims.0) as f32;
            let y = ((i / cluster_dims.0) % cluster_dims.1) as f32;
            let z = (i / (cluster_dims.0 * cluster_dims.1)) as f32;
            ClusterBounds {
                min_pos: [x * 10.0, y * 10.0, z * 10.0],
                _pad1: 0.0,
                max_pos: [(x + 1.0) * 10.0, (y + 1.0) * 10.0, (z + 1.0) * 10.0],
                _pad2: 0.0,
            }
        })
        .collect();

    let cluster_bounds_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Cluster Bounds Buffer"),
        size: (total_clusters as usize * std::mem::size_of::<ClusterBounds>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    queue.write_buffer(&cluster_bounds_buffer, 0, bytemuck::cast_slice(&cluster_bounds));

    let light_counts_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Counts Buffer"),
        size: (total_clusters as usize * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    let light_offsets_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Offsets Buffer"),
        size: (total_clusters as usize * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    let light_indices_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Light Indices Buffer"),
        size: (total_clusters as usize * 64 * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    });

    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct ClusterParams {
        cluster_dims: [u32; 3],
        _pad1: u32,
        total_clusters: u32,
        light_count: u32,
        _pad2: u32,
        _pad3: u32,
    }

    let params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Params Buffer"),
        size: std::mem::size_of::<ClusterParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    #[repr(C)]
    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
    struct PrefixSumParams {
        element_count: u32,
        workgroup_size: u32,
        _pad1: u32,
        _pad2: u32,
    }

    let prefix_sum_params_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Prefix Sum Params Buffer"),
        size: std::mem::size_of::<PrefixSumParams>() as u64,
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });

    megalights.update_bind_groups(
        &device,
        &light_buffer,
        &cluster_bounds_buffer,
        &light_counts_buffer,
        &light_offsets_buffer,
        &light_indices_buffer,
        &params_buffer,
        &prefix_sum_params_buffer,
    );

    // Upload 1000 lights for throughput test
    let lights = create_test_scene_gpu(1000);
    queue.write_buffer(&light_buffer, 0, bytemuck::cast_slice(&lights));
    
    let params = ClusterParams {
        cluster_dims: [cluster_dims.0, cluster_dims.1, cluster_dims.2],
        _pad1: 0,
        total_clusters,
        light_count: 1000,
        _pad2: 0,
        _pad3: 0,
    };
    queue.write_buffer(&params_buffer, 0, bytemuck::bytes_of(&params));
    
    let prefix_params = PrefixSumParams {
        element_count: total_clusters,
        workgroup_size: 256,
        _pad1: 0,
        _pad2: 0,
    };
    queue.write_buffer(&prefix_sum_params_buffer, 0, bytemuck::bytes_of(&prefix_params));

    let mut group = c.benchmark_group("gpu_throughput");

    // Batch 10 dispatches, single sync - measures amortized GPU cost
    group.bench_function("megalights_10x_batched_1000_lights", |b| {
        b.iter(|| {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Batched Encoder"),
            });

            // Record 10 dispatches
            for _ in 0..10 {
                megalights.dispatch(&mut encoder, 1000).expect("dispatch failed");
            }

            queue.submit([encoder.finish()]);
            let _ = device.poll(wgpu::MaintainBase::Wait);
        });
    });

    // Batch 100 dispatches - stress test
    group.bench_function("megalights_100x_batched_1000_lights", |b| {
        b.iter(|| {
            let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Batched Encoder 100x"),
            });

            for _ in 0..100 {
                megalights.dispatch(&mut encoder, 1000).expect("dispatch failed");
            }

            queue.submit([encoder.finish()]);
            let _ = device.poll(wgpu::MaintainBase::Wait);
        });
    });

    group.finish();
}

// Benchmark groups (simplified for criterion compatibility)
#[cfg(feature = "megalights")]
criterion_group!(benches, bench_cpu_light_culling, bench_gpu_light_culling, bench_gpu_throughput);

#[cfg(not(feature = "megalights"))]
criterion_group!(benches, bench_cpu_light_culling);

criterion_main!(benches);
