//! Ragdoll System Benchmarks (Phase 8.8)
//!
//! Benchmarks for ragdoll physics components.

use astraweave_physics::ragdoll::{BoneDef, BoneShape, RagdollBuilder, RagdollConfig};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::Vec3;
use std::hint::black_box;

fn ragdoll_config_creation(c: &mut Criterion) {
    c.bench_function("ragdoll_config_default", |b| {
        b.iter(|| {
            black_box(RagdollConfig::default())
        });
    });
}

fn ragdoll_builder_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("ragdoll_builder");
    
    for bone_count in [5, 10, 20, 30].iter() {
        group.throughput(Throughput::Elements(*bone_count as u64));
        group.bench_with_input(BenchmarkId::from_parameter(bone_count), bone_count, |b, &count| {
            b.iter(|| {
                let config = RagdollConfig::default();
                let mut builder = RagdollBuilder::new(config);
                
                // Add root bone
                builder.add_bone(
                    "root",
                    None,
                    Vec3::ZERO,
                    BoneShape::Capsule { radius: 0.15, half_height: 0.1 },
                    2.0,
                );
                
                // Add child bones
                for i in 1..count {
                    builder.add_bone(
                        &format!("bone_{}", i),
                        Some("root"),
                        Vec3::new(0.0, 0.2 * i as f32, 0.0),
                        BoneShape::Capsule { radius: 0.05, half_height: 0.1 },
                        1.0,
                    );
                }
                
                black_box(builder)
            });
        });
    }
    
    group.finish();
}

fn bone_def_creation(c: &mut Criterion) {
    c.bench_function("bone_def_default", |b| {
        b.iter(|| {
            black_box(BoneDef::default())
        });
    });
}

fn bone_shape_volume(c: &mut Criterion) {
    let mut group = c.benchmark_group("bone_shape_volume");
    
    let shapes = [
        ("capsule", BoneShape::Capsule { radius: 0.1, half_height: 0.2 }),
        ("sphere", BoneShape::Sphere { radius: 0.15 }),
        ("box", BoneShape::Box { half_extents: Vec3::splat(0.1) }),
    ];
    
    for (name, shape) in shapes.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), shape, |b, shape| {
            b.iter(|| {
                black_box(shape.volume())
            });
        });
    }
    
    group.finish();
}

fn ragdoll_hinge_bone_add(c: &mut Criterion) {
    c.bench_function("ragdoll_add_hinge_bone", |b| {
        b.iter_with_setup(
            || {
                let config = RagdollConfig::default();
                let mut builder = RagdollBuilder::new(config);
                builder.add_bone("root", None, Vec3::ZERO, BoneShape::Capsule { radius: 0.1, half_height: 0.15 }, 2.0);
                builder
            },
            |mut builder| {
                builder.add_hinge_bone(
                    "elbow",
                    "root",
                    Vec3::new(0.3, 0.0, 0.0),
                    BoneShape::Capsule { radius: 0.04, half_height: 0.15 },
                    0.5,
                    Vec3::Y,
                    Some((-2.0, 0.0)),
                );
                black_box(builder)
            },
        );
    });
}

fn ragdoll_ball_bone_add(c: &mut Criterion) {
    c.bench_function("ragdoll_add_ball_bone", |b| {
        b.iter_with_setup(
            || {
                let config = RagdollConfig::default();
                let mut builder = RagdollBuilder::new(config);
                builder.add_bone("root", None, Vec3::ZERO, BoneShape::Capsule { radius: 0.1, half_height: 0.15 }, 2.0);
                builder
            },
            |mut builder| {
                builder.add_ball_bone(
                    "shoulder",
                    "root",
                    Vec3::new(0.2, 0.3, 0.0),
                    BoneShape::Capsule { radius: 0.05, half_height: 0.15 },
                    0.8,
                    Some(std::f32::consts::FRAC_PI_2),
                );
                black_box(builder)
            },
        );
    });
}

fn ragdoll_config_variations(c: &mut Criterion) {
    let mut group = c.benchmark_group("ragdoll_config_variations");
    
    let configs = [
        ("stiff", RagdollConfig { joint_stiffness: 1.0, joint_damping: 0.9, ..Default::default() }),
        ("floppy", RagdollConfig { joint_stiffness: 0.1, joint_damping: 0.1, ..Default::default() }),
        ("heavy", RagdollConfig { mass_scale: 5.0, ..Default::default() }),
    ];
    
    for (name, config) in configs.iter() {
        group.bench_with_input(BenchmarkId::from_parameter(name), config, |b, config| {
            b.iter(|| {
                let mut builder = RagdollBuilder::new(config.clone());
                builder.add_bone("pelvis", None, Vec3::ZERO, BoneShape::Capsule { radius: 0.15, half_height: 0.1 }, 2.0);
                builder.add_bone("spine", Some("pelvis"), Vec3::new(0.0, 0.2, 0.0), BoneShape::Capsule { radius: 0.12, half_height: 0.15 }, 1.5);
                builder.add_bone("head", Some("spine"), Vec3::new(0.0, 0.35, 0.0), BoneShape::Sphere { radius: 0.12 }, 1.0);
                black_box(builder)
            });
        });
    }
    
    group.finish();
}

fn ragdoll_humanoid_skeleton(c: &mut Criterion) {
    // Benchmark creating a complete humanoid ragdoll skeleton
    c.bench_function("ragdoll_humanoid_skeleton", |b| {
        b.iter(|| {
            let config = RagdollConfig::default();
            let mut builder = RagdollBuilder::new(config);
            
            // Torso
            builder.add_bone("pelvis", None, Vec3::ZERO, BoneShape::Capsule { radius: 0.15, half_height: 0.1 }, 2.0);
            builder.add_bone("spine", Some("pelvis"), Vec3::new(0.0, 0.2, 0.0), BoneShape::Capsule { radius: 0.12, half_height: 0.15 }, 1.5);
            builder.add_bone("chest", Some("spine"), Vec3::new(0.0, 0.3, 0.0), BoneShape::Capsule { radius: 0.14, half_height: 0.12 }, 1.5);
            builder.add_bone("head", Some("chest"), Vec3::new(0.0, 0.25, 0.0), BoneShape::Sphere { radius: 0.12 }, 1.0);
            
            // Left arm
            builder.add_ball_bone("l_shoulder", "chest", Vec3::new(-0.2, 0.1, 0.0), BoneShape::Capsule { radius: 0.05, half_height: 0.12 }, 0.6, Some(1.57));
            builder.add_hinge_bone("l_elbow", "l_shoulder", Vec3::new(-0.25, 0.0, 0.0), BoneShape::Capsule { radius: 0.04, half_height: 0.12 }, 0.4, Vec3::Y, Some((-2.5, 0.0)));
            builder.add_bone("l_hand", Some("l_elbow"), Vec3::new(-0.25, 0.0, 0.0), BoneShape::Sphere { radius: 0.05 }, 0.2);
            
            // Right arm
            builder.add_ball_bone("r_shoulder", "chest", Vec3::new(0.2, 0.1, 0.0), BoneShape::Capsule { radius: 0.05, half_height: 0.12 }, 0.6, Some(1.57));
            builder.add_hinge_bone("r_elbow", "r_shoulder", Vec3::new(0.25, 0.0, 0.0), BoneShape::Capsule { radius: 0.04, half_height: 0.12 }, 0.4, Vec3::Y, Some((-2.5, 0.0)));
            builder.add_bone("r_hand", Some("r_elbow"), Vec3::new(0.25, 0.0, 0.0), BoneShape::Sphere { radius: 0.05 }, 0.2);
            
            // Left leg
            builder.add_ball_bone("l_hip", "pelvis", Vec3::new(-0.1, -0.1, 0.0), BoneShape::Capsule { radius: 0.06, half_height: 0.2 }, 1.2, Some(1.2));
            builder.add_hinge_bone("l_knee", "l_hip", Vec3::new(0.0, -0.4, 0.0), BoneShape::Capsule { radius: 0.05, half_height: 0.2 }, 0.8, Vec3::X, Some((0.0, 2.5)));
            builder.add_bone("l_foot", Some("l_knee"), Vec3::new(0.0, -0.4, 0.0), BoneShape::Box { half_extents: Vec3::new(0.05, 0.03, 0.1) }, 0.3);
            
            // Right leg
            builder.add_ball_bone("r_hip", "pelvis", Vec3::new(0.1, -0.1, 0.0), BoneShape::Capsule { radius: 0.06, half_height: 0.2 }, 1.2, Some(1.2));
            builder.add_hinge_bone("r_knee", "r_hip", Vec3::new(0.0, -0.4, 0.0), BoneShape::Capsule { radius: 0.05, half_height: 0.2 }, 0.8, Vec3::X, Some((0.0, 2.5)));
            builder.add_bone("r_foot", Some("r_knee"), Vec3::new(0.0, -0.4, 0.0), BoneShape::Box { half_extents: Vec3::new(0.05, 0.03, 0.1) }, 0.3);
            
            black_box(builder)
        });
    });
}

criterion_group!(
    benches,
    ragdoll_config_creation,
    ragdoll_builder_creation,
    bone_def_creation,
    bone_shape_volume,
    ragdoll_hinge_bone_add,
    ragdoll_ball_bone_add,
    ragdoll_config_variations,
    ragdoll_humanoid_skeleton,
);
criterion_main!(benches);
