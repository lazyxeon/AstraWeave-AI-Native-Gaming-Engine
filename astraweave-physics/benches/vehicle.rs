//! Vehicle Physics Benchmarks (Phase 8.8)
//!
//! Benchmarks for vehicle simulation including suspension, drivetrain, and friction.

use astraweave_physics::vehicle::{
    EngineConfig, FrictionCurve, TransmissionConfig, Vehicle, VehicleConfig, 
    VehicleInput, VehicleManager,
};
use astraweave_physics::PhysicsWorld;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::Vec3;
use std::hint::black_box;

const DT: f32 = 1.0 / 60.0;

fn vehicle_creation(c: &mut Criterion) {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    
    c.bench_function("vehicle_creation", |b| {
        b.iter(|| {
            let mut mgr = VehicleManager::new();
            black_box(mgr.spawn(&mut physics, Vec3::ZERO, VehicleConfig::default()))
        });
    });
}

fn vehicle_update_single(c: &mut Criterion) {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(100.0, 0.5, 100.0), 0.9);
    
    let mut mgr = VehicleManager::new();
    mgr.spawn(&mut physics, Vec3::new(0.0, 1.0, 0.0), VehicleConfig::default());

    c.bench_function("vehicle_update_single", |b| {
        b.iter(|| {
            mgr.update(&mut physics, DT);
            black_box(())
        });
    });
}

fn vehicle_update_with_input(c: &mut Criterion) {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(100.0, 0.5, 100.0), 0.9);
    
    let mut mgr = VehicleManager::new();
    let id = mgr.spawn(&mut physics, Vec3::new(0.0, 1.0, 0.0), VehicleConfig::default());
    
    let input = VehicleInput {
        throttle: 1.0,
        steering: 0.3,
        ..Default::default()
    };

    c.bench_function("vehicle_update_with_input", |b| {
        b.iter(|| {
            mgr.update_with_input(id, &mut physics, &input, DT);
            black_box(())
        });
    });
}

fn vehicle_multiple(c: &mut Criterion) {
    let mut group = c.benchmark_group("vehicle_multiple");
    
    for count in [1, 5, 10, 20].iter() {
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(count), count, |b, &count| {
            let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
            physics.create_ground_plane(Vec3::new(200.0, 0.5, 200.0), 0.9);
            
            let mut mgr = VehicleManager::new();
            for i in 0..count {
                mgr.spawn(&mut physics, Vec3::new(i as f32 * 10.0, 1.0, 0.0), VehicleConfig::default());
            }
            
            b.iter(|| {
                mgr.update(&mut physics, DT);
                black_box(())
            });
        });
    }
    
    group.finish();
}

fn friction_curve_evaluation(c: &mut Criterion) {
    let curves = [
        ("tarmac", FrictionCurve::tarmac()),
        ("gravel", FrictionCurve::gravel()),
        ("ice", FrictionCurve::ice()),
        ("mud", FrictionCurve::mud()),
    ];
    
    let mut group = c.benchmark_group("friction_curve");
    
    for (name, curve) in curves.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), &curve, |b, curve| {
            b.iter(|| {
                // Evaluate friction at various slip values
                let mut sum = 0.0f32;
                for i in 0..100 {
                    sum += curve.friction_at_slip(i as f32 * 0.01);
                }
                black_box(sum)
            });
        });
    }
    
    group.finish();
}

fn engine_torque_curve(c: &mut Criterion) {
    let engine = EngineConfig::default();
    
    c.bench_function("engine_torque_curve", |b| {
        b.iter(|| {
            let mut sum = 0.0f32;
            // Evaluate across RPM range
            for rpm in (800..7000).step_by(100) {
                sum += engine.torque_at_rpm(rpm as f32);
            }
            black_box(sum)
        });
    });
}

fn transmission_gear_ratios(c: &mut Criterion) {
    let trans = TransmissionConfig::default();
    
    c.bench_function("transmission_effective_ratio", |b| {
        b.iter(|| {
            let mut sum = 0.0f32;
            for gear in -1..=6 {
                sum += trans.effective_ratio(gear);
            }
            black_box(sum)
        });
    });
}

fn vehicle_shifting(c: &mut Criterion) {
    let config = VehicleConfig::default();
    let mut vehicle = Vehicle::new(1, 42, config);

    c.bench_function("vehicle_shifting", |b| {
        b.iter(|| {
            vehicle.shift_up();
            vehicle.shift_timer = 0.0;
            vehicle.shift_up();
            vehicle.shift_timer = 0.0;
            vehicle.shift_down();
            vehicle.shift_timer = 0.0;
            black_box(vehicle.current_gear)
        });
    });
}

criterion_group!(
    benches,
    vehicle_creation,
    vehicle_update_single,
    vehicle_update_with_input,
    vehicle_multiple,
    friction_curve_evaluation,
    engine_torque_curve,
    transmission_gear_ratios,
    vehicle_shifting,
);
criterion_main!(benches);
