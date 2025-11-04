use astraweave_physics::{Layers, PhysicsWorld};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::vec3;
use std::hint::black_box;

/// Setup a basic world with gravity
fn setup_world() -> PhysicsWorld {
    PhysicsWorld::new(vec3(0.0, -9.8, 0.0))
}

/// Benchmark: Single rigid body physics step (free fall)
fn rigid_body_single_step(c: &mut Criterion) {
    let mut world = setup_world();
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    world.add_dynamic_box(
        vec3(0.0, 10.0, 0.0),
        vec3(0.5, 0.5, 0.5),
        1.0,
        Layers::DEFAULT,
    );

    c.bench_function("rigid_body_single_step", |b| {
        b.iter(|| {
            world.step();
        });
    });
}

/// Benchmark: Physics step with multiple rigid bodies
fn rigid_body_batch_step(c: &mut Criterion) {
    let mut group = c.benchmark_group("rigid_body_batch_step");

    for body_count in [1, 10, 50, 100, 200].iter() {
        let mut world = setup_world();
        world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);

        // Add bodies in a grid pattern above ground
        let grid_size = (*body_count as f32).sqrt().ceil() as usize;
        let spacing = 2.0;

        for i in 0..*body_count {
            let x = (i % grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            let z = (i / grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            world.add_dynamic_box(
                vec3(x, 10.0 + (i / grid_size) as f32 * 2.0, z),
                vec3(0.5, 0.5, 0.5),
                1.0,
                Layers::DEFAULT,
            );
        }

        group.throughput(Throughput::Elements(*body_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(body_count),
            body_count,
            |b, _| {
                b.iter(|| {
                    world.step();
                });
            },
        );
    }

    group.finish();
}

/// Benchmark: Rigid body creation overhead
fn rigid_body_creation(c: &mut Criterion) {
    c.bench_function("rigid_body_creation", |b| {
        b.iter(|| {
            let mut world = setup_world();
            black_box(world.add_dynamic_box(
                vec3(0.0, 10.0, 0.0),
                vec3(0.5, 0.5, 0.5),
                1.0,
                Layers::DEFAULT,
            ))
        });
    });
}

/// Benchmark: Static trimesh creation (navmesh scenario)
fn rigid_body_trimesh_creation(c: &mut Criterion) {
    // Create a simple triangle mesh (quad made of 2 triangles)
    let vertices = vec![
        vec3(0.0, 0.0, 0.0),
        vec3(10.0, 0.0, 0.0),
        vec3(10.0, 0.0, 10.0),
        vec3(0.0, 0.0, 10.0),
    ];
    let indices = vec![[0, 1, 2], [0, 2, 3]];

    c.bench_function("rigid_body_trimesh_creation", |b| {
        b.iter(|| {
            let mut world = setup_world();
            black_box(world.add_static_trimesh(&vertices, &indices, Layers::DEFAULT))
        });
    });
}

/// Benchmark: Body transform retrieval
fn rigid_body_transform_lookup(c: &mut Criterion) {
    let mut world = setup_world();
    let body_id = world.add_dynamic_box(
        vec3(0.0, 10.0, 0.0),
        vec3(0.5, 0.5, 0.5),
        1.0,
        Layers::DEFAULT,
    );

    c.bench_function("rigid_body_transform_lookup", |b| {
        b.iter(|| black_box(world.body_transform(black_box(body_id))));
    });
}

/// Benchmark: Stacked bodies simulation (worst case for physics)
fn rigid_body_stacked_simulation(c: &mut Criterion) {
    let mut world = setup_world();
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);

    // Create a tower of boxes (challenging for solver)
    for i in 0..10 {
        world.add_dynamic_box(
            vec3(0.0, 1.0 + (i as f32) * 1.1, 0.0),
            vec3(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
    }

    c.bench_function("rigid_body_stacked_simulation", |b| {
        b.iter(|| {
            world.step();
        });
    });
}

/// Benchmark: Destructible box creation
fn rigid_body_destructible_creation(c: &mut Criterion) {
    c.bench_function("rigid_body_destructible_creation", |b| {
        b.iter(|| {
            let mut world = setup_world();
            black_box(world.add_destructible_box(
                vec3(0.0, 10.0, 0.0),
                vec3(0.5, 0.5, 0.5),
                1.0,
                100.0, // health
                50.0,  // break_impulse
            ))
        });
    });
}

/// Benchmark: Physics simulation with mixed body types
fn rigid_body_mixed_simulation(c: &mut Criterion) {
    let mut world = setup_world();
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);

    // Add dynamic boxes
    for i in 0..10 {
        world.add_dynamic_box(
            vec3((i as f32) * 2.0, 5.0, 0.0),
            vec3(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
    }

    // Add characters (kinematic bodies)
    for i in 0..5 {
        world.add_character(vec3(0.0, 1.0, (i as f32) * 3.0), vec3(0.4, 0.9, 0.4));
    }

    // Add destructible boxes
    for i in 0..5 {
        world.add_destructible_box(
            vec3(-(i as f32) * 2.0, 5.0, 0.0),
            vec3(0.5, 0.5, 0.5),
            1.0,
            100.0,
            50.0,
        );
    }

    c.bench_function("rigid_body_mixed_simulation", |b| {
        b.iter(|| {
            world.step();
        });
    });
}

/// Benchmark: Ground plane creation
fn rigid_body_ground_creation(c: &mut Criterion) {
    c.bench_function("rigid_body_ground_creation", |b| {
        b.iter(|| {
            let mut world = setup_world();
            black_box(world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9))
        });
    });
}

criterion_group!(
    rigid_body_benches,
    rigid_body_single_step,
    rigid_body_batch_step,
    rigid_body_creation,
    rigid_body_trimesh_creation,
    rigid_body_transform_lookup,
    rigid_body_stacked_simulation,
    rigid_body_destructible_creation,
    rigid_body_mixed_simulation,
    rigid_body_ground_creation,
);
criterion_main!(rigid_body_benches);
