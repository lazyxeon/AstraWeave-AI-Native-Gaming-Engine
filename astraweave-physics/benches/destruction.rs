//! Destruction System Benchmarks (Phase 8.8)
//!
//! Benchmarks for destruction physics performance.

use astraweave_physics::destruction::{DestructibleConfig, DestructionManager, FracturePattern};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::Vec3;
use std::hint::black_box;

fn destruction_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("destruction_creation");
    
    for debris_count in [4, 8, 16, 32].iter() {
        group.throughput(Throughput::Elements(*debris_count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(debris_count), debris_count, |b, &count| {
            b.iter(|| {
                let mut mgr = DestructionManager::new();
                let config = DestructibleConfig {
                    fracture_pattern: FracturePattern::uniform(count, Vec3::splat(0.5), 10.0),
                    ..Default::default()
                };
                black_box(mgr.add_destructible(config, Vec3::ZERO))
            });
        });
    }
    
    group.finish();
}

fn destruction_apply_damage(c: &mut Criterion) {
    let mut group = c.benchmark_group("destruction_damage");
    
    for count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut mgr = DestructionManager::new();
            let mut ids = Vec::new();
            let config = DestructibleConfig::default();
            
            for i in 0..count {
                ids.push(mgr.add_destructible(config.clone(), Vec3::new(i as f32, 0.0, 0.0)));
            }
            
            b.iter(|| {
                for id in &ids {
                    mgr.apply_damage(*id, 1.0);
                }
                black_box(())
            });
        });
    }
    
    group.finish();
}

const GRAVITY: Vec3 = Vec3::new(0.0, -9.81, 0.0);

fn destruction_update(c: &mut Criterion) {
    let mut group = c.benchmark_group("destruction_update");
    let dt = 1.0 / 60.0;
    
    for count in [10, 50, 100].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut mgr = DestructionManager::new();
            let config = DestructibleConfig::default();
            
            for i in 0..count {
                mgr.add_destructible(config.clone(), Vec3::new(i as f32, 0.0, 0.0));
            }
            
            b.iter(|| {
                mgr.update(dt, GRAVITY);
                black_box(())
            });
        });
    }
    
    group.finish();
}

fn destruction_full_destroy(c: &mut Criterion) {
    // Benchmark destroying objects (applying enough damage to trigger destruction)
    c.bench_function("destruction_destroy_single", |b| {
        b.iter_with_setup(
            || {
                let mut mgr = DestructionManager::new();
                let config = DestructibleConfig {
                    fracture_pattern: FracturePattern::uniform(8, Vec3::splat(0.5), 10.0),
                    ..Default::default()
                };
                let id = mgr.add_destructible(config, Vec3::ZERO);
                (mgr, id)
            },
            |(mut mgr, id)| {
                mgr.destroy(id);
                mgr.update(1.0 / 60.0, GRAVITY);
                black_box(())
            },
        );
    });
}

fn destruction_fracture_patterns(c: &mut Criterion) {
    let mut group = c.benchmark_group("destruction_patterns");
    
    let patterns = [
        ("small_4", FracturePattern::uniform(4, Vec3::splat(0.5), 5.0)),
        ("medium_16", FracturePattern::uniform(16, Vec3::splat(0.5), 10.0)),
        ("large_64", FracturePattern::uniform(64, Vec3::splat(0.5), 20.0)),
    ];
    
    for (name, pattern) in patterns.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), pattern, |b, pattern| {
            b.iter(|| {
                let mut mgr = DestructionManager::new();
                let config = DestructibleConfig {
                    fracture_pattern: pattern.clone(),
                    ..Default::default()
                };
                let id = mgr.add_destructible(config, Vec3::ZERO);
                black_box(id)
            });
        });
    }
    
    group.finish();
}

fn destruction_many_objects(c: &mut Criterion) {
    let mut group = c.benchmark_group("destruction_scale");
    
    for count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            b.iter_with_setup(
                || {
                    let mut mgr = DestructionManager::new();
                    let config = DestructibleConfig::default();
                    for i in 0..count {
                        mgr.add_destructible(config.clone(), Vec3::new(i as f32 * 2.0, 0.0, 0.0));
                    }
                    mgr
                },
                |mut mgr| {
                    mgr.update(1.0 / 60.0, GRAVITY);
                    black_box(())
                },
            );
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    destruction_creation,
    destruction_apply_damage,
    destruction_update,
    destruction_full_destroy,
    destruction_fracture_patterns,
    destruction_many_objects,
);
criterion_main!(benches);
