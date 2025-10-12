// Copyright 2025 AstraWeave Contributors
// SPDX-License-Identifier: MIT

//! Benchmark comparing naive vs SIMD movement implementations.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use glam::Vec3;
use astraweave_math::simd_movement::{update_positions_simd, update_positions_naive};

fn benchmark_movement(c: &mut Criterion) {
    let mut group = c.benchmark_group("movement");
    
    // Test various entity counts
    for count in [100, 1000, 10000].iter() {
        let mut positions = vec![Vec3::ZERO; *count];
        let velocities = vec![Vec3::new(1.0, 2.0, 3.0); *count];
        let dt = 0.016; // 60 FPS

        group.bench_with_input(
            BenchmarkId::new("naive", count),
            count,
            |b, _count| {
                b.iter(|| {
                    update_positions_naive(
                        black_box(&mut positions),
                        black_box(&velocities),
                        black_box(dt),
                    )
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("simd", count),
            count,
            |b, _count| {
                b.iter(|| {
                    update_positions_simd(
                        black_box(&mut positions),
                        black_box(&velocities),
                        black_box(dt),
                    )
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_movement);
criterion_main!(benches);
