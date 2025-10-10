use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use astraweave_physics::{Layers, PhysicsWorld};
use glam::vec3;
use rapier3d::prelude::*;
use std::hint::black_box;

/// Setup a physics world with ground plane and optional obstacles
fn setup_world_with_obstacles(obstacle_count: usize) -> PhysicsWorld {
    let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    
    // Ground plane
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    
    // Add obstacles in a grid pattern
    let grid_size = (obstacle_count as f32).sqrt().ceil() as usize;
    let spacing = 5.0;
    
    for i in 0..obstacle_count {
        let x = (i % grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
        let z = (i / grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
        world.add_dynamic_box(
            vec3(x, 2.0, z),
            vec3(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
    }
    
    world
}

/// Benchmark: Single raycast against empty scene
fn raycast_empty_scene(c: &mut Criterion) {
    let world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    
    c.bench_function("raycast_empty_scene", |b| {
        b.iter(|| {
            let origin = black_box(vec3(0.0, 1.0, 0.0));
            let direction = black_box(vec3(1.0, 0.0, 0.0).normalize());
            
            let ray = Ray::new(
                point![origin.x, origin.y, origin.z],
                vector![direction.x, direction.y, direction.z],
            );
            
            let filter = QueryFilter::default();
            world.query_pipeline.cast_ray_and_get_normal(
                &world.bodies,
                &world.colliders,
                &ray,
                100.0,
                true,
                filter,
            )
        });
    });
}

/// Benchmark: Single raycast against ground plane
fn raycast_ground_plane(c: &mut Criterion) {
    let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    
    c.bench_function("raycast_ground_plane", |b| {
        b.iter(|| {
            let origin = black_box(vec3(0.0, 5.0, 0.0));
            let direction = black_box(vec3(0.0, -1.0, 0.0));
            
            let ray = Ray::new(
                point![origin.x, origin.y, origin.z],
                vector![direction.x, direction.y, direction.z],
            );
            
            let filter = QueryFilter::default();
            world.query_pipeline.cast_ray_and_get_normal(
                &world.bodies,
                &world.colliders,
                &ray,
                10.0,
                true,
                filter,
            )
        });
    });
}

/// Benchmark: Raycast through varying obstacle density
fn raycast_obstacle_density(c: &mut Criterion) {
    let mut group = c.benchmark_group("raycast_obstacle_density");
    
    for obstacle_count in [0, 10, 50, 100].iter() {
        let world = setup_world_with_obstacles(*obstacle_count);
        
        group.throughput(Throughput::Elements(*obstacle_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(obstacle_count),
            obstacle_count,
            |b, _| {
                b.iter(|| {
                    let origin = black_box(vec3(-20.0, 1.0, 0.0));
                    let direction = black_box(vec3(1.0, 0.0, 0.0));
                    
                    let ray = Ray::new(
                        point![origin.x, origin.y, origin.z],
                        vector![direction.x, direction.y, direction.z],
                    );
                    
                    let filter = QueryFilter::default();
                    world.query_pipeline.cast_ray_and_get_normal(
                        &world.bodies,
                        &world.colliders,
                        &ray,
                        50.0,
                        true,
                        filter,
                    )
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Multiple raycasts (simulating combat/vision systems)
fn raycast_batch_8_rays(c: &mut Criterion) {
    let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    
    // Add some obstacles
    for i in 0..20 {
        let angle = (i as f32) * std::f32::consts::PI * 2.0 / 20.0;
        world.add_dynamic_box(
            vec3(angle.cos() * 10.0, 2.0, angle.sin() * 10.0),
            vec3(0.5, 1.0, 0.5),
            1.0,
            Layers::DEFAULT,
        );
    }
    
    c.bench_function("raycast_batch_8_rays", |b| {
        b.iter(|| {
            let origin = black_box(vec3(0.0, 1.5, 0.0));
            
            // Cast 8 rays in a circle (vision cone simulation)
            for i in 0..8 {
                let angle = (i as f32) * std::f32::consts::PI * 2.0 / 8.0;
                let direction = vec3(angle.cos(), 0.0, angle.sin());
                
                let ray = Ray::new(
                    point![origin.x, origin.y, origin.z],
                    vector![direction.x, direction.y, direction.z],
                );
                
                let filter = QueryFilter::default();
                black_box(world.query_pipeline.cast_ray_and_get_normal(
                    &world.bodies,
                    &world.colliders,
                    &ray,
                    20.0,
                    true,
                    filter,
                ));
            }
        });
    });
}

/// Benchmark: Raycast with normal retrieval vs without
fn raycast_with_and_without_normal(c: &mut Criterion) {
    let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    
    let mut group = c.benchmark_group("raycast_normal_retrieval");
    
    // With normal
    group.bench_function("with_normal", |b| {
        b.iter(|| {
            let origin = black_box(vec3(0.0, 5.0, 0.0));
            let direction = black_box(vec3(0.0, -1.0, 0.0));
            
            let ray = Ray::new(
                point![origin.x, origin.y, origin.z],
                vector![direction.x, direction.y, direction.z],
            );
            
            let filter = QueryFilter::default();
            world.query_pipeline.cast_ray_and_get_normal(
                &world.bodies,
                &world.colliders,
                &ray,
                10.0,
                true,
                filter,
            )
        });
    });
    
    // Without normal (just hit detection)
    group.bench_function("without_normal", |b| {
        b.iter(|| {
            let origin = black_box(vec3(0.0, 5.0, 0.0));
            let direction = black_box(vec3(0.0, -1.0, 0.0));
            
            let ray = Ray::new(
                point![origin.x, origin.y, origin.z],
                vector![direction.x, direction.y, direction.z],
            );
            
            let filter = QueryFilter::default();
            world.query_pipeline.cast_ray(
                &world.bodies,
                &world.colliders,
                &ray,
                10.0,
                true,
                filter,
            )
        });
    });
    
    group.finish();
}

criterion_group!(
    raycast_benches,
    raycast_empty_scene,
    raycast_ground_plane,
    raycast_obstacle_density,
    raycast_batch_8_rays,
    raycast_with_and_without_normal,
);
criterion_main!(raycast_benches);
