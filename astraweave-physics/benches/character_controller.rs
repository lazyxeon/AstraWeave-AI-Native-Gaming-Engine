use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use astraweave_physics::PhysicsWorld;
use glam::vec3;
use std::hint::black_box;

/// Setup a simple test world with ground
fn setup_simple_world() -> PhysicsWorld {
    let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    world.create_ground_plane(vec3(50.0, 0.5, 50.0), 0.9);
    world
}

/// Benchmark: Single character controller update (straight movement)
fn character_move_straight(c: &mut Criterion) {
    let mut world = setup_simple_world();
    let char_id = world.add_character(vec3(0.0, 1.0, 0.0), vec3(0.4, 0.9, 0.4));
    
    c.bench_function("character_move_straight", |b| {
        b.iter(|| {
            let desired_move = black_box(vec3(1.0, 0.0, 0.0));
            world.control_character(char_id, desired_move, 1.0 / 60.0, false);
        });
    });
}

/// Benchmark: Character controller with diagonal movement
fn character_move_diagonal(c: &mut Criterion) {
    let mut world = setup_simple_world();
    let char_id = world.add_character(vec3(0.0, 1.0, 0.0), vec3(0.4, 0.9, 0.4));
    
    c.bench_function("character_move_diagonal", |b| {
        b.iter(|| {
            let desired_move = black_box(vec3(1.0, 0.0, 1.0).normalize());
            world.control_character(char_id, desired_move, 1.0 / 60.0, false);
        });
    });
}

/// Benchmark: Multiple characters moving simultaneously
fn character_move_batch(c: &mut Criterion) {
    let mut group = c.benchmark_group("character_move_batch");
    
    for char_count in [1, 10, 50, 100].iter() {
        let mut world = setup_simple_world();
        let mut char_ids = Vec::new();
        
        // Spawn characters in a grid
        let grid_size = (*char_count as f32).sqrt().ceil() as usize;
        let spacing = 3.0;
        
        for i in 0..*char_count {
            let x = (i % grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            let z = (i / grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
            let id = world.add_character(vec3(x, 1.0, z), vec3(0.4, 0.9, 0.4));
            char_ids.push(id);
        }
        
        group.throughput(Throughput::Elements(*char_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(char_count),
            char_count,
            |b, _| {
                b.iter(|| {
                    for &id in &char_ids {
                        let desired_move = black_box(vec3(1.0, 0.0, 0.0));
                        world.control_character(id, desired_move, 1.0 / 60.0, false);
                    }
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Character controller with obstacle avoidance
fn character_move_with_obstacles(c: &mut Criterion) {
    let mut world = setup_simple_world();
    
    // Add wall of obstacles in front of character
    for i in 0..10 {
        world.add_dynamic_box(
            vec3(5.0, 1.0, (i as f32) - 5.0),
            vec3(0.5, 1.0, 0.5),
            10.0,
            astraweave_physics::Layers::DEFAULT,
        );
    }
    
    let char_id = world.add_character(vec3(0.0, 1.0, 0.0), vec3(0.4, 0.9, 0.4));
    
    c.bench_function("character_move_with_obstacles", |b| {
        b.iter(|| {
            let desired_move = black_box(vec3(1.0, 0.0, 0.0));
            world.control_character(char_id, desired_move, 1.0 / 60.0, false);
        });
    });
}

/// Benchmark: Character controller step climbing
fn character_step_climbing(c: &mut Criterion) {
    let mut world = setup_simple_world();
    
    // Add step obstacles (small boxes to climb over)
    for i in 0..5 {
        world.add_dynamic_box(
            vec3((i as f32) * 2.0, 0.2, 0.0),
            vec3(0.8, 0.2, 1.0),
            100.0, // Heavy so they don't move
            astraweave_physics::Layers::DEFAULT,
        );
    }
    
    let char_id = world.add_character(vec3(-2.0, 1.0, 0.0), vec3(0.4, 0.9, 0.4));
    
    c.bench_function("character_step_climbing", |b| {
        b.iter(|| {
            let desired_move = black_box(vec3(1.0, 0.0, 0.0));
            world.control_character(char_id, desired_move, 1.0 / 60.0, true);
        });
    });
}

/// Benchmark: Full character simulation tick (move + physics step)
fn character_full_tick(c: &mut Criterion) {
    let mut world = setup_simple_world();
    let char_id = world.add_character(vec3(0.0, 1.0, 0.0), vec3(0.4, 0.9, 0.4));
    
    c.bench_function("character_full_tick", |b| {
        b.iter(|| {
            let desired_move = black_box(vec3(1.0, 0.0, 0.0));
            world.control_character(char_id, desired_move, 1.0 / 60.0, false);
            world.step();
        });
    });
}

/// Benchmark: Character transforms retrieval
fn character_transform_lookup(c: &mut Criterion) {
    let mut world = setup_simple_world();
    let char_id = world.add_character(vec3(0.0, 1.0, 0.0), vec3(0.4, 0.9, 0.4));
    
    c.bench_function("character_transform_lookup", |b| {
        b.iter(|| {
            black_box(world.body_transform(black_box(char_id)))
        });
    });
}

criterion_group!(
    character_controller_benches,
    character_move_straight,
    character_move_diagonal,
    character_move_batch,
    character_move_with_obstacles,
    character_step_climbing,
    character_full_tick,
    character_transform_lookup,
);
criterion_main!(character_controller_benches);
