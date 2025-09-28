use criterion::{black_box, criterion_group, criterion_main, Criterion};

// This bench compares CPU clustered binning against a simulated GPU path dispatch cost.
// Real GPU timing is environment dependent; we measure CPU reference consistently.
use astraweave_render::clustered::{bin_lights_cpu, ClusterDims, CpuLight};

fn bench_clustered(c: &mut Criterion) {
    let dims = ClusterDims { x: 16, y: 8, z: 16 };
    let screen = (1920u32, 1080u32);
    let near = 0.1f32;
    let far = 200.0f32;
    let fov = std::f32::consts::FRAC_PI_3;
    // Generate lights
    let mut lights = Vec::new();
    for i in 0..512 {
        let t = i as f32 * 0.123;
        let r = 20.0 + (i % 50) as f32 * 0.1;
        lights.push(CpuLight {
            pos: glam::Vec3::new(t.sin() * r, 1.5 + (i % 7) as f32, t.cos() * r),
            radius: 6.0 + (i % 13) as f32,
        });
    }
    c.bench_function("clustered_cpu_binning_512", |b| {
        b.iter(|| {
            let (_counts, _indices, offsets) =
                bin_lights_cpu(black_box(&lights), dims, screen, near, far, fov);
            black_box(offsets);
        });
    });

    // GPU path placeholder: We just simulate a minimal cost to keep bench structure stable.
    // Real GPU compute dispatch would require a device and not be deterministic across machines.
    c.bench_function("clustered_gpu_dispatch_placeholder", |b| {
        b.iter(|| {
            // simulate small fixed cost
            std::hint::black_box(0u32);
        });
    });
}

criterion_group!(benches, bench_clustered);
criterion_main!(benches);
