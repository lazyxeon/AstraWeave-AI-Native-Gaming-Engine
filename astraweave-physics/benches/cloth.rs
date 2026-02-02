//! Cloth Physics Benchmarks (Phase 8.8)
//!
//! Benchmarks for cloth simulation performance across various configurations.

use astraweave_physics::cloth::{ClothConfig, ClothManager};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::Vec3;
use std::hint::black_box;

const DT: f32 = 1.0 / 60.0;

fn cloth_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("cloth_creation");
    
    for size in [4, 8, 16, 32].iter() {
        group.throughput(Throughput::Elements((*size * *size) as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &size| {
            b.iter(|| {
                let mut mgr = ClothManager::new();
                let config = ClothConfig {
                    width: size,
                    height: size,
                    ..Default::default()
                };
                black_box(mgr.create(config, Vec3::ZERO))
            });
        });
    }
    
    group.finish();
}

fn cloth_update_small(c: &mut Criterion) {
    let mut mgr = ClothManager::new();
    let config = ClothConfig {
        width: 8,
        height: 8,
        ..Default::default()
    };
    mgr.create(config, Vec3::ZERO);

    c.bench_function("cloth_update_8x8", |b| {
        b.iter(|| {
            mgr.update(DT);
            black_box(())
        });
    });
}

fn cloth_update_medium(c: &mut Criterion) {
    let mut mgr = ClothManager::new();
    let config = ClothConfig {
        width: 16,
        height: 16,
        ..Default::default()
    };
    mgr.create(config, Vec3::ZERO);

    c.bench_function("cloth_update_16x16", |b| {
        b.iter(|| {
            mgr.update(DT);
            black_box(())
        });
    });
}

fn cloth_update_large(c: &mut Criterion) {
    let mut mgr = ClothManager::new();
    let config = ClothConfig {
        width: 32,
        height: 32,
        ..Default::default()
    };
    mgr.create(config, Vec3::ZERO);

    c.bench_function("cloth_update_32x32", |b| {
        b.iter(|| {
            mgr.update(DT);
            black_box(())
        });
    });
}

fn cloth_multiple_instances(c: &mut Criterion) {
    let mut group = c.benchmark_group("cloth_multiple");
    
    for count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut mgr = ClothManager::new();
            let config = ClothConfig {
                width: 8,
                height: 8,
                ..Default::default()
            };
            for i in 0..count {
                mgr.create(config.clone(), Vec3::new(i as f32 * 5.0, 0.0, 0.0));
            }
            
            b.iter(|| {
                mgr.update(DT);
                black_box(())
            });
        });
    }
    
    group.finish();
}

fn cloth_stiffness_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("cloth_stiffness");
    
    for stiffness in [0.3, 0.6, 0.9].iter() {
        group.bench_with_input(BenchmarkId::from_parameter(format!("{:.1}", stiffness)), stiffness, |b, &stiffness| {
            let mut mgr = ClothManager::new();
            let config = ClothConfig {
                width: 16,
                height: 16,
                stiffness,
                ..Default::default()
            };
            mgr.create(config, Vec3::ZERO);
            
            b.iter(|| {
                mgr.update(DT);
                black_box(())
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    cloth_creation,
    cloth_update_small,
    cloth_update_medium,
    cloth_update_large,
    cloth_multiple_instances,
    cloth_stiffness_variations,
);
criterion_main!(benches);
