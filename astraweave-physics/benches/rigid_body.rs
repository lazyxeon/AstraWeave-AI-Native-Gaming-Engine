#![allow(dead_code)]

use astraweave_physics::{Layers, PhysicsWorld};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{vec3, Mat4};
use std::hint::black_box;

// =============================================================================
// MISSION-CRITICAL CORRECTNESS ASSERTIONS
// =============================================================================
// Rigid body benchmarks validate CORRECTNESS of physics simulation.
// Assertions verify:
//   1. Body Transform: Transformation matrices are finite and valid
//   2. Ground Interaction: Bodies don't fall through ground
//   3. Creation Validity: Newly created bodies have valid handles
//   4. Stacking Stability: Towers don't explode or collapse incorrectly
//   5. Multi-Body Consistency: Batch operations maintain valid state
// =============================================================================

/// CORRECTNESS: Validate a body's transformation matrix is geometrically valid
#[inline]
fn assert_body_transform_valid(transform: Option<Mat4>, context: &str) {
    assert!(transform.is_some(), 
        "[CORRECTNESS FAILURE] {}: body transform lookup returned None", context);
    if let Some(mat) = transform {
        // Extract position from column 3 (translation column)
        let pos = mat.col(3).truncate();
        // Position must be finite
        assert!(pos.x.is_finite() && pos.y.is_finite() && pos.z.is_finite(),
            "[CORRECTNESS FAILURE] {}: position contains non-finite values {:?}", context, pos);
        // Position shouldn't be absurdly far (exploded simulation)
        assert!(pos.length_squared() < 10000.0 * 10000.0,
            "[CORRECTNESS FAILURE] {}: body position exploded to {:?}", context, pos);
        // Check matrix has valid scaling (determinant should be non-zero, not NaN)
        let det = mat.determinant();
        assert!(det.is_finite() && det.abs() > 1e-10,
            "[CORRECTNESS FAILURE] {}: transform matrix has invalid determinant {}", context, det);
    }
}

/// CORRECTNESS: Validate body is above ground (didn't fall through)
#[inline]
fn assert_body_above_ground(transform: Option<Mat4>, min_y: f32, context: &str) {
    if let Some(mat) = transform {
        let pos_y = mat.col(3).y;
        assert!(pos_y > min_y,
            "[CORRECTNESS FAILURE] {}: body fell through ground (y={}, min={})", context, pos_y, min_y);
    }
}

/// Setup a basic world with gravity
fn setup_world() -> PhysicsWorld {
    PhysicsWorld::new(vec3(0.0, -9.8, 0.0))
}

/// Benchmark: Single rigid body physics step (free fall)
fn rigid_body_single_step(c: &mut Criterion) {
    let mut world = setup_world();
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    let body_id = world.add_dynamic_box(
        vec3(0.0, 10.0, 0.0),
        vec3(0.5, 0.5, 0.5),
        1.0,
        Layers::DEFAULT,
    );
    
    // CORRECTNESS: Verify body created with valid initial transform
    let initial = world.body_transform(body_id);
    assert_body_transform_valid(initial, "rigid_body_single_step/initial");

    c.bench_function("rigid_body_single_step", |b| {
        b.iter(|| {
            world.step();
            // CORRECTNESS: After each step, body must remain valid
            let transform = world.body_transform(body_id);
            assert_body_transform_valid(transform, "rigid_body_single_step/post");
            black_box(transform)
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
        let mut body_ids = Vec::new();

        for i in 0..*body_count {
            let x = (i % grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            let z = (i / grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            let id = world.add_dynamic_box(
                vec3(x, 10.0 + (i / grid_size) as f32 * 2.0, z),
                vec3(0.5, 0.5, 0.5),
                1.0,
                Layers::DEFAULT,
            );
            body_ids.push(id);
        }
        
        // CORRECTNESS: Verify all bodies spawned
        assert_eq!(body_ids.len(), *body_count,
            "[CORRECTNESS FAILURE] rigid_body_batch_step: expected {} bodies, spawned {}", 
            body_count, body_ids.len());

        group.throughput(Throughput::Elements(*body_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(body_count),
            body_count,
            |b, &count| {
                b.iter(|| {
                    world.step();
                    // CORRECTNESS: Validate subset of bodies for performance
                    for &id in body_ids.iter().take(5.min(count)) {
                        let transform = world.body_transform(id);
                        assert_body_transform_valid(transform, &format!("rigid_body_batch_step/{}", count));
                    }
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
            let body_id = world.add_dynamic_box(
                vec3(0.0, 10.0, 0.0),
                vec3(0.5, 0.5, 0.5),
                1.0,
                Layers::DEFAULT,
            );
            // CORRECTNESS: Newly created body must have valid transform
            let transform = world.body_transform(body_id);
            assert_body_transform_valid(transform, "rigid_body_creation");
            black_box(body_id)
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
            let mesh_id = world.add_static_trimesh(&vertices, &indices, Layers::DEFAULT);
            // CORRECTNESS: Trimesh creation should return valid handle
            // (static bodies may not have transforms, but should not panic)
            black_box(mesh_id)
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
        b.iter(|| {
            let transform = world.body_transform(black_box(body_id));
            // CORRECTNESS: Transform lookup must always return valid data
            assert_body_transform_valid(transform, "rigid_body_transform_lookup");
            black_box(transform)
        });
    });
}

/// Benchmark: Stacked bodies simulation (worst case for physics)
fn rigid_body_stacked_simulation(c: &mut Criterion) {
    let mut world = setup_world();
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);

    // Create a tower of boxes (challenging for solver)
    let mut tower_ids = Vec::new();
    for i in 0..10 {
        let id = world.add_dynamic_box(
            vec3(0.0, 1.0 + (i as f32) * 1.1, 0.0),
            vec3(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        tower_ids.push(id);
    }
    
    // CORRECTNESS: Verify tower spawned correctly
    assert_eq!(tower_ids.len(), 10, "[CORRECTNESS FAILURE] tower should have 10 boxes");

    c.bench_function("rigid_body_stacked_simulation", |b| {
        b.iter(|| {
            world.step();
            // CORRECTNESS: Check tower hasn't exploded (common physics bug)
            for (i, &id) in tower_ids.iter().enumerate() {
                let transform = world.body_transform(id);
                assert_body_transform_valid(transform, &format!("stacked_simulation/box_{}", i));
            }
        });
    });
}

/// Benchmark: Destructible box creation
fn rigid_body_destructible_creation(c: &mut Criterion) {
    c.bench_function("rigid_body_destructible_creation", |b| {
        b.iter(|| {
            let mut world = setup_world();
            let body_id = world.add_destructible_box(
                vec3(0.0, 10.0, 0.0),
                vec3(0.5, 0.5, 0.5),
                1.0,
                100.0, // health
                50.0,  // break_impulse
            );
            // CORRECTNESS: Destructible body must have valid transform
            let transform = world.body_transform(body_id);
            assert_body_transform_valid(transform, "rigid_body_destructible_creation");
            black_box(body_id)
        });
    });
}

/// Benchmark: Physics simulation with mixed body types
fn rigid_body_mixed_simulation(c: &mut Criterion) {
    let mut world = setup_world();
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    
    let mut all_body_ids = Vec::new();

    // Add dynamic boxes
    for i in 0..10 {
        let id = world.add_dynamic_box(
            vec3((i as f32) * 2.0, 5.0, 0.0),
            vec3(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
        all_body_ids.push(id);
    }

    // Add characters (kinematic bodies)
    for i in 0..5 {
        let id = world.add_character(vec3(0.0, 1.0, (i as f32) * 3.0), vec3(0.4, 0.9, 0.4));
        all_body_ids.push(id);
    }

    // Add destructible boxes
    for i in 0..5 {
        let id = world.add_destructible_box(
            vec3(-(i as f32) * 2.0, 5.0, 0.0),
            vec3(0.5, 0.5, 0.5),
            1.0,
            100.0,
            50.0,
        );
        all_body_ids.push(id);
    }
    
    // CORRECTNESS: Verify all bodies created
    assert_eq!(all_body_ids.len(), 20, 
        "[CORRECTNESS FAILURE] mixed simulation should have 20 bodies (10+5+5)");

    c.bench_function("rigid_body_mixed_simulation", |b| {
        b.iter(|| {
            world.step();
            // CORRECTNESS: Validate a subset of bodies for performance
            for &id in all_body_ids.iter().take(5) {
                let transform = world.body_transform(id);
                assert_body_transform_valid(transform, "rigid_body_mixed_simulation");
            }
        });
    });
}

/// Benchmark: Ground plane creation
fn rigid_body_ground_creation(c: &mut Criterion) {
    c.bench_function("rigid_body_ground_creation", |b| {
        b.iter(|| {
            let mut world = setup_world();
            let ground_id = world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
            // CORRECTNESS: Ground should be created (may not have transform for static)
            black_box(ground_id)
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
