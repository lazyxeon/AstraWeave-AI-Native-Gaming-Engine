// Week 5 Action 19: Mesh Optimization Benchmarks
// Validates memory reduction, performance, and draw call savings

use astraweave_render::instancing::{Instance, InstanceManager, InstancePatternBuilder};
use astraweave_render::lod_generator::{LODConfig, LODGenerator, SimplificationMesh};
use astraweave_render::vertex_compression::{
    CompressedVertex, HalfFloatEncoder, OctahedralEncoder, VertexCompressor,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{Quat, Vec2, Vec3};

// ============================================================================
// VERTEX COMPRESSION BENCHMARKS
// ============================================================================

fn bench_octahedral_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_compression/octahedral");

    let normals = vec![
        Vec3::Y,
        Vec3::new(1.0, 1.0, 1.0).normalize(),
        Vec3::new(-0.5, 0.8, 0.3).normalize(),
    ];

    group.bench_function("encode", |b| {
        b.iter(|| {
            for normal in &normals {
                black_box(OctahedralEncoder::encode(black_box(*normal)));
            }
        });
    });

    let encoded = normals
        .iter()
        .map(|n| OctahedralEncoder::encode(*n))
        .collect::<Vec<_>>();

    group.bench_function("decode", |b| {
        b.iter(|| {
            for enc in &encoded {
                black_box(OctahedralEncoder::decode(black_box(*enc)));
            }
        });
    });

    group.finish();
}

fn bench_half_float_encoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_compression/half_float");

    let uvs = vec![
        Vec2::new(0.0, 0.0),
        Vec2::new(0.5, 0.5),
        Vec2::new(1.0, 1.0),
        Vec2::new(0.123, 0.456),
    ];

    group.bench_function("encode_vec2", |b| {
        b.iter(|| {
            for uv in &uvs {
                black_box(HalfFloatEncoder::encode_vec2(black_box(*uv)));
            }
        });
    });

    let encoded = uvs
        .iter()
        .map(|uv| HalfFloatEncoder::encode_vec2(*uv))
        .collect::<Vec<_>>();

    group.bench_function("decode_vec2", |b| {
        b.iter(|| {
            for enc in &encoded {
                black_box(HalfFloatEncoder::decode_vec2(black_box(*enc)));
            }
        });
    });

    group.finish();
}

fn bench_vertex_compression_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_compression/throughput");

    for vertex_count in [100, 1000, 10000, 100000] {
        group.throughput(Throughput::Elements(vertex_count));

        let positions: Vec<Vec3> = (0..vertex_count)
            .map(|i| Vec3::new(i as f32, (i * 2) as f32, (i * 3) as f32))
            .collect();

        let normals: Vec<Vec3> = (0..vertex_count)
            .map(|i| {
                Vec3::new((i % 3) as f32, ((i + 1) % 3) as f32, ((i + 2) % 3) as f32).normalize()
            })
            .collect();

        let uvs: Vec<Vec2> = (0..vertex_count)
            .map(|i| Vec2::new((i % 10) as f32 / 10.0, ((i + 5) % 10) as f32 / 10.0))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("compress_batch", vertex_count),
            &vertex_count,
            |b, _| {
                b.iter(|| {
                    black_box(VertexCompressor::compress_batch(
                        black_box(&positions),
                        black_box(&normals),
                        black_box(&uvs),
                    ))
                });
            },
        );
    }

    group.finish();
}

fn bench_vertex_compression_memory(c: &mut Criterion) {
    let mut group = c.benchmark_group("vertex_compression/memory_savings");

    for vertex_count in [1000, 10000, 100000, 1000000] {
        group.bench_with_input(
            BenchmarkId::new("calculate_savings", vertex_count),
            &vertex_count,
            |b, &count| {
                b.iter(|| black_box(VertexCompressor::calculate_savings(black_box(count))));
            },
        );
    }

    group.finish();
}

// ============================================================================
// LOD GENERATION BENCHMARKS
// ============================================================================

fn create_test_sphere(segments: usize) -> SimplificationMesh {
    let mut positions = Vec::new();
    let mut normals = Vec::new();
    let mut uvs = Vec::new();
    let mut indices = Vec::new();

    // Generate sphere vertices
    for lat in 0..=segments {
        let theta = (lat as f32 / segments as f32) * std::f32::consts::PI;
        for lon in 0..=segments {
            let phi = (lon as f32 / segments as f32) * std::f32::consts::TAU;

            let x = theta.sin() * phi.cos();
            let y = theta.cos();
            let z = theta.sin() * phi.sin();

            positions.push(Vec3::new(x, y, z));
            normals.push(Vec3::new(x, y, z).normalize());
            uvs.push([lon as f32 / segments as f32, lat as f32 / segments as f32]);
        }
    }

    // Generate indices
    for lat in 0..segments {
        for lon in 0..segments {
            let first = lat * (segments + 1) + lon;
            let second = first + segments + 1;

            indices.push(first as u32);
            indices.push(second as u32);
            indices.push((first + 1) as u32);

            indices.push(second as u32);
            indices.push((second + 1) as u32);
            indices.push((first + 1) as u32);
        }
    }

    SimplificationMesh::new(positions, normals, uvs, indices)
}

fn bench_lod_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("lod_generation/simplification");
    group.sample_size(10); // Reduce sample size for slow operations

    for segments in [8, 16, 32] {
        let mesh = create_test_sphere(segments);
        let vertex_count = mesh.vertex_count();

        group.throughput(Throughput::Elements(vertex_count as u64));

        let config = LODConfig {
            reduction_targets: vec![0.50], // Single 50% reduction
            max_error: 0.1,
            preserve_boundaries: false,
        };
        let generator = LODGenerator::new(config);

        group.bench_with_input(
            BenchmarkId::new("sphere_50_percent", vertex_count),
            &mesh,
            |b, mesh| {
                b.iter(|| black_box(generator.simplify(black_box(mesh), vertex_count / 2)));
            },
        );
    }

    group.finish();
}

fn bench_lod_multi_level(c: &mut Criterion) {
    let mut group = c.benchmark_group("lod_generation/multi_level");
    group.sample_size(10);

    let mesh = create_test_sphere(16); // ~300 vertices

    let config = LODConfig {
        reduction_targets: vec![0.75, 0.50, 0.25],
        max_error: 0.1,
        preserve_boundaries: false,
    };
    let generator = LODGenerator::new(config);

    group.bench_function("generate_3_lods", |b| {
        b.iter(|| black_box(generator.generate_lods(black_box(&mesh))));
    });

    group.finish();
}

// ============================================================================
// INSTANCING BENCHMARKS
// ============================================================================

fn bench_instance_manager_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("instancing/instance_manager");

    for instance_count in [100, 1000, 10000] {
        group.throughput(Throughput::Elements(instance_count));

        let instances: Vec<Instance> = (0..instance_count)
            .map(|i| Instance::new(Vec3::new(i as f32, 0.0, 0.0), Quat::IDENTITY, Vec3::ONE))
            .collect();

        group.bench_with_input(
            BenchmarkId::new("add_instances", instance_count),
            &instances,
            |b, instances| {
                b.iter(|| {
                    let mut manager = InstanceManager::new();
                    for instance in instances {
                        manager.add_instance(1, instance.clone());
                    }
                    black_box(manager)
                });
            },
        );
    }

    group.finish();
}

fn bench_instance_to_raw(c: &mut Criterion) {
    let mut group = c.benchmark_group("instancing/transformation");

    let instance = Instance::new(
        Vec3::new(10.0, 20.0, 30.0),
        Quat::from_rotation_y(1.57),
        Vec3::splat(2.0),
    );

    group.bench_function("instance_to_raw", |b| {
        b.iter(|| black_box(instance.to_raw()));
    });

    group.finish();
}

fn bench_pattern_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("instancing/pattern_generation");

    group.bench_function("grid_10x10", |b| {
        b.iter(|| black_box(InstancePatternBuilder::new().grid(10, 10, 5.0).build()));
    });

    group.bench_function("circle_100", |b| {
        b.iter(|| black_box(InstancePatternBuilder::new().circle(100, 50.0).build()));
    });

    group.bench_function("grid_with_variations", |b| {
        b.iter(|| {
            black_box(
                InstancePatternBuilder::new()
                    .grid(10, 10, 5.0)
                    .with_position_jitter(1.0)
                    .with_scale_variation(0.8, 1.2)
                    .with_random_rotation_y()
                    .build(),
            )
        });
    });

    group.finish();
}

fn bench_draw_call_reduction(c: &mut Criterion) {
    let mut group = c.benchmark_group("instancing/draw_call_reduction");

    for (mesh_count, instances_per_mesh) in [(1, 1000), (10, 100), (100, 10)] {
        let total_instances = mesh_count * instances_per_mesh;
        group.throughput(Throughput::Elements(total_instances as u64));

        group.bench_with_input(
            BenchmarkId::new(
                "calculate_savings",
                format!("{}x{}", mesh_count, instances_per_mesh),
            ),
            &(mesh_count, instances_per_mesh),
            |b, &(meshes, instances)| {
                b.iter(|| {
                    let mut manager = InstanceManager::new();
                    for mesh_id in 0..meshes {
                        for _ in 0..instances {
                            manager.add_instance(mesh_id as u64, Instance::identity());
                        }
                    }
                    black_box(manager.draw_calls_saved())
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// INTEGRATED BENCHMARKS (Combined optimizations)
// ============================================================================

fn bench_full_optimization_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("integrated/full_pipeline");
    group.sample_size(10);

    let mesh = create_test_sphere(16);
    let vertex_count = mesh.vertex_count();

    group.bench_function("compress_generate_lods_instance", |b| {
        b.iter(|| {
            // Step 1: Compress vertices
            let uvs_vec2: Vec<Vec2> = mesh.uvs.iter().map(|&[u, v]| Vec2::new(u, v)).collect();
            let compressed =
                VertexCompressor::compress_batch(&mesh.positions, &mesh.normals, &uvs_vec2);

            // Step 2: Generate LODs
            let lod_config = LODConfig {
                reduction_targets: vec![0.50],
                max_error: 0.1,
                preserve_boundaries: false,
            };
            let lod_gen = LODGenerator::new(lod_config);
            let lods = lod_gen.generate_lods(&mesh);

            // Step 3: Create instances
            let instances = InstancePatternBuilder::new().grid(10, 10, 5.0).build();

            let mut manager = InstanceManager::new();
            for instance in instances {
                manager.add_instance(1, instance);
            }

            black_box((compressed, lods, manager))
        });
    });

    group.finish();
}

fn bench_memory_savings_combined(c: &mut Criterion) {
    let mut group = c.benchmark_group("integrated/memory_savings");

    for vertex_count in [1000, 10000, 100000] {
        group.bench_with_input(
            BenchmarkId::new("vertex_compression", vertex_count),
            &vertex_count,
            |b, &count| {
                b.iter(|| {
                    let (_, _, savings, _) = VertexCompressor::calculate_savings(count);
                    black_box(savings)
                });
            },
        );
    }

    group.finish();
}

// ============================================================================
// CRITERION GROUPS
// ============================================================================

criterion_group!(
    vertex_compression_benches,
    bench_octahedral_encoding,
    bench_half_float_encoding,
    bench_vertex_compression_throughput,
    bench_vertex_compression_memory,
);

criterion_group!(
    lod_generation_benches,
    bench_lod_generation,
    bench_lod_multi_level,
);

criterion_group!(
    instancing_benches,
    bench_instance_manager_add,
    bench_instance_to_raw,
    bench_pattern_generation,
    bench_draw_call_reduction,
);

criterion_group!(
    integrated_benches,
    bench_full_optimization_pipeline,
    bench_memory_savings_combined,
);

criterion_main!(
    vertex_compression_benches,
    lod_generation_benches,
    instancing_benches,
    integrated_benches,
);
