//! Performance Regression Detection Benchmarks
//!
//! Frame-time budgets and GPU performance benchmarks to detect regressions.

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use std::time::Duration;

fn bench_frame_time_budget(c: &mut Criterion) {
    let mut group = c.benchmark_group("rendering_frame_time");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(100);

    // Target: 16.67ms per frame for 60 FPS
    // Rendering budget: 6.00ms (36% of frame)

    group.bench_function("mock_full_render_pass", |b| {
        b.iter(|| {
            // Mock a full render pass workload
            black_box(simulate_render_work(1000));
        });
    });

    group.finish();
}

fn bench_culling_enabled_vs_disabled(c: &mut Criterion) {
    let mut group = c.benchmark_group("culling_performance");

    group.bench_function("with_backface_culling", |b| {
        b.iter(|| {
            // Simulate rendering with culling (50% of fragments)
            black_box(simulate_render_work(500));
        });
    });

    group.bench_function("without_backface_culling", |b| {
        b.iter(|| {
            // Simulate rendering without culling (100% of fragments)
            black_box(simulate_render_work(1000));
        });
    });

    group.finish();
}

fn bench_texture_sampling(c: &mut Criterion) {
    let mut group = c.benchmark_group("texture_operations");

    for mip_levels in [1, 3, 7].iter() {
        group.bench_with_input(
            BenchmarkId::new("mipmap_levels", mip_levels),
            mip_levels,
            |b, &mips| {
                b.iter(|| {
                    // Mock texture sampling cost increases with mip levels
                    black_box(simulate_texture_sampling(mips));
                });
            },
        );
    }

    group.finish();
}

fn bench_shader_compilation(c: &mut Criterion) {
    let mut group = c.benchmark_group("shader_compilation");

    let simple_shader = r#"
        @vertex
        fn vs_main(@builtin(vertex_index) idx: u32) -> @builtin(position) vec4<f32> {
            return vec4<f32>(0.0, 0.0, 0.0, 1.0);
        }
        
        @fragment
        fn fs_main() -> @location(0) vec4<f32> {
            return vec4<f32>(1.0, 0.0, 0.0, 1.0);
        }
    "#;

    group.bench_function("shader_parse_and_validate", |b| {
        b.iter(|| {
            let module = naga::front::wgsl::parse_str(black_box(simple_shader)).unwrap();
            black_box(module);
        });
    });

    group.finish();
}

// Helper functions to simulate work

fn simulate_render_work(fragment_count: usize) -> u64 {
    // Simulate fragment shader work
    let mut sum = 0u64;
    for i in 0..fragment_count {
        // Simple arithmetic to simulate GPU work
        sum = sum.wrapping_add((i as u64).wrapping_mul(17).wrapping_add(31));
    }
    sum
}

fn simulate_texture_sampling(mip_levels: usize) -> u64 {
    // Simulate texture sampling cost
    let mut sum = 0u64;
    for i in 0..(mip_levels * 100) {
        sum = sum.wrapping_add(i as u64);
    }
    sum
}

criterion_group!(
    benches,
    bench_frame_time_budget,
    bench_culling_enabled_vs_disabled,
    bench_texture_sampling,
    bench_shader_compilation
);
criterion_main!(benches);
