//! Gravity System Benchmarks (Phase 8.8)
//!
//! Benchmarks for gravity zone calculations and per-body gravity.

use astraweave_physics::gravity::{GravityManager, GravityZone, GravityZoneShape};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::Vec3;
use std::hint::black_box;

fn gravity_calculation_global(c: &mut Criterion) {
    let manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));

    c.bench_function("gravity_global_only", |b| {
        b.iter(|| {
            let gravity = manager.calculate_gravity(1, Vec3::new(10.0, 5.0, 3.0));
            black_box(gravity)
        });
    });
}

fn gravity_calculation_with_scale(c: &mut Criterion) {
    let mut manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
    manager.set_gravity_scale(1, 0.5);

    c.bench_function("gravity_with_scale", |b| {
        b.iter(|| {
            let gravity = manager.calculate_gravity(1, Vec3::ZERO);
            black_box(gravity)
        });
    });
}

fn gravity_zone_contains_check(c: &mut Criterion) {
    let shapes = [
        ("box", GravityZoneShape::Box {
            min: Vec3::new(-10.0, -10.0, -10.0),
            max: Vec3::new(10.0, 10.0, 10.0),
        }),
        ("sphere", GravityZoneShape::Sphere {
            center: Vec3::ZERO,
            radius: 15.0,
        }),
        ("point", GravityZoneShape::Point {
            center: Vec3::ZERO,
            radius: 20.0,
            strength: 100.0,
        }),
    ];

    let mut group = c.benchmark_group("gravity_zone_contains");
    
    for (name, shape) in shapes.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), shape, |b, shape| {
            b.iter(|| {
                let mut count = 0;
                for i in 0..100 {
                    let pos = Vec3::new(i as f32 * 0.5 - 25.0, 0.0, 0.0);
                    if shape.contains(pos) {
                        count += 1;
                    }
                }
                black_box(count)
            });
        });
    }
    
    group.finish();
}

fn gravity_zone_get_gravity(c: &mut Criterion) {
    let shapes = [
        ("box", GravityZoneShape::Box {
            min: Vec3::splat(-50.0),
            max: Vec3::splat(50.0),
        }, Vec3::new(0.0, 10.0, 0.0)),
        ("point_attractor", GravityZoneShape::Point {
            center: Vec3::ZERO,
            radius: 100.0,
            strength: 500.0,
        }, Vec3::ZERO),
    ];

    let mut group = c.benchmark_group("gravity_zone_get_gravity");
    
    for (name, shape, zone_gravity) in shapes.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), &(shape, zone_gravity), |b, (shape, zone_gravity)| {
            b.iter(|| {
                let mut sum = Vec3::ZERO;
                for i in 0..100 {
                    let pos = Vec3::new(i as f32 * 0.5 - 25.0, i as f32 * 0.3, 0.0);
                    if let Some(g) = shape.get_gravity(pos, **zone_gravity) {
                        sum += g;
                    }
                }
                black_box(sum)
            });
        });
    }
    
    group.finish();
}

fn gravity_multiple_zones(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity_multiple_zones");
    
    for zone_count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*zone_count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(zone_count), zone_count, |b, &count| {
            let mut manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
            
            for i in 0..count {
                manager.add_zone(GravityZone {
                    shape: GravityZoneShape::Box {
                        min: Vec3::new(i as f32 * 20.0 - 10.0, -10.0, -10.0),
                        max: Vec3::new(i as f32 * 20.0 + 10.0, 10.0, 10.0),
                    },
                    gravity: Vec3::new(0.0, i as f32, 0.0),
                    priority: i as i32,
                    ..Default::default()
                });
            }
            
            b.iter(|| {
                let gravity = manager.calculate_gravity(1, Vec3::new(5.0, 0.0, 0.0));
                black_box(gravity)
            });
        });
    }
    
    group.finish();
}

fn gravity_many_bodies(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity_many_bodies");
    
    for body_count in [100, 500, 1000, 5000].iter() {
        group.throughput(Throughput::Elements(*body_count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(body_count), body_count, |b, &count| {
            let mut manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
            
            // Add a zone
            manager.add_zero_g_box(
                Vec3::new(-50.0, -50.0, -50.0),
                Vec3::new(50.0, 50.0, 50.0),
                1,
            );
            
            // Set per-body gravity for some
            for i in 0..count / 10 {
                manager.set_gravity_scale(i as u64, 0.5);
            }
            
            b.iter(|| {
                let mut sum = Vec3::ZERO;
                for i in 0..count {
                    let pos = Vec3::new(
                        (i % 100) as f32 - 50.0,
                        ((i / 100) % 100) as f32 - 50.0,
                        0.0,
                    );
                    sum += manager.calculate_gravity(i as u64, pos);
                }
                black_box(sum)
            });
        });
    }
    
    group.finish();
}

fn gravity_point_attractor(c: &mut Criterion) {
    let mut manager = GravityManager::new(Vec3::ZERO);
    manager.add_attractor(Vec3::ZERO, 100.0, 500.0, 1);

    c.bench_function("gravity_point_attractor", |b| {
        b.iter(|| {
            let mut sum = Vec3::ZERO;
            for i in 0..100 {
                let angle = (i as f32) * 0.0628; // 2*PI/100
                let pos = Vec3::new(angle.cos() * 50.0, angle.sin() * 50.0, 0.0);
                sum += manager.calculate_gravity(1, pos);
            }
            black_box(sum)
        });
    });
}

fn gravity_bodies_in_zone(c: &mut Criterion) {
    let mut group = c.benchmark_group("gravity_bodies_in_zone");
    
    for body_count in [100, 500, 1000].iter() {
        group.throughput(Throughput::Elements(*body_count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(body_count), body_count, |b, &count| {
            let mut manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));
            let zone_id = manager.add_zero_g_box(
                Vec3::new(-25.0, -25.0, -25.0),
                Vec3::new(25.0, 25.0, 25.0),
                1,
            );
            
            let bodies: Vec<(u64, Vec3)> = (0..count)
                .map(|i| {
                    let x = (i % 100) as f32 - 50.0;
                    let y = ((i / 100) % 100) as f32 - 50.0;
                    (i as u64, Vec3::new(x, y, 0.0))
                })
                .collect();
            
            b.iter(|| {
                let inside = manager.bodies_in_zone(zone_id, &bodies);
                black_box(inside.len())
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    benches,
    gravity_calculation_global,
    gravity_calculation_with_scale,
    gravity_zone_contains_check,
    gravity_zone_get_gravity,
    gravity_multiple_zones,
    gravity_many_bodies,
    gravity_point_attractor,
    gravity_bodies_in_zone,
);
criterion_main!(benches);
