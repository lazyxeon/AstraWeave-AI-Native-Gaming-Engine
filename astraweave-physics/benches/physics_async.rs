// physics_async.rs - Async Physics Benchmarks
//
// Benchmarks for parallel physics pipeline measuring:
// - Single-thread vs multi-thread speedup
// - Scalability with thread count (1, 2, 4, 8)
// - Performance with varying NPC counts
// - Telemetry overhead

#![cfg(feature = "async-physics")]

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use astraweave_physics::{PhysicsWorld, Layers};
use glam::vec3;
use rand::{Rng, SeedableRng};
use std::hint::black_box;

/// Create a world with N characters for benchmarking
fn create_world_with_characters(char_count: usize) -> PhysicsWorld {
    let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    
    // Create ground
    world.create_ground_plane(vec3(100.0, 0.5, 100.0), 0.9);
    
    // Create characters in deterministic grid
    let grid_size = (char_count as f32).sqrt().ceil() as usize;
    let spacing = 3.0;
    
    for i in 0..char_count {
        let x = (i % grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
        let z = (i / grid_size) as f32 * spacing - (grid_size as f32 * spacing / 2.0);
        world.add_character(vec3(x, 1.0, z), vec3(0.4, 0.9, 0.4));
    }
    
    world
}

/// Create a world with N dynamic rigid bodies
fn create_world_with_rigid_bodies(body_count: usize) -> PhysicsWorld {
    let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
    
    // Create ground
    world.create_ground_plane(vec3(100.0, 0.5, 100.0), 0.9);
    
    // Create bodies in deterministic positions
    let mut rng = rand::rngs::StdRng::seed_from_u64(12345);
    
    for _ in 0..body_count {
        let x = rng.gen_range(-40.0..40.0);
        let y = rng.gen_range(5.0..15.0);
        let z = rng.gen_range(-40.0..40.0);
        
        world.add_dynamic_box(
            vec3(x, y, z),
            vec3(0.5, 0.5, 0.5),
            1.0,
            Layers::DEFAULT,
        );
    }
    
    world
}

/// Benchmark: Full physics tick (single-threaded baseline)
fn physics_full_tick_baseline(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_full_tick_baseline");
    
    for npc_count in [100, 500, 1000, 2500].iter() {
        let mut world = create_world_with_characters(*npc_count);
        
        group.throughput(Throughput::Elements(*npc_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(npc_count),
            npc_count,
            |b, _| {
                b.iter(|| {
                    world.step();
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Full physics tick with async enabled (4 threads)
fn physics_async_full_tick(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_async_full_tick");
    
    for npc_count in [100, 500, 1000, 2500].iter() {
        let mut world = create_world_with_characters(*npc_count);
        
        // Enable async physics with 4 threads
        world.enable_async_physics(4);
        
        group.throughput(Throughput::Elements(*npc_count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(npc_count),
            npc_count,
            |b, _| {
                b.iter(|| {
                    world.step();
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Thread scaling (2, 4, 8 threads with 2500 NPCs)
fn physics_async_thread_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_async_thread_scaling");
    const NPC_COUNT: usize = 2500;
    
    for thread_count in [1, 2, 4, 8].iter() {
        let mut world = create_world_with_characters(NPC_COUNT);
        
        // Enable async physics with variable thread count
        world.enable_async_physics(*thread_count);
        
        group.throughput(Throughput::Elements(NPC_COUNT as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(thread_count),
            thread_count,
            |b, _| {
                b.iter(|| {
                    world.step();
                });
            },
        );
    }
    
    group.finish();
}

/// Benchmark: Rigid body simulation (async vs sync)
fn physics_async_rigid_bodies(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_async_rigid_bodies");
    const BODY_COUNT: usize = 500;
    
    // Baseline (single-thread)
    {
        let mut world = create_world_with_rigid_bodies(BODY_COUNT);
        
        group.throughput(Throughput::Elements(BODY_COUNT as u64));
        group.bench_function("sync", |b| {
            b.iter(|| {
                world.step();
            });
        });
    }
    
    // Async (4 threads)
    {
        let mut world = create_world_with_rigid_bodies(BODY_COUNT);
        world.enable_async_physics(4);
        
        group.throughput(Throughput::Elements(BODY_COUNT as u64));
        group.bench_function("async_4_threads", |b| {
            b.iter(|| {
                world.step();
            });
        });
    }
    
    group.finish();
}

/// Benchmark: Telemetry overhead
fn physics_async_telemetry_overhead(c: &mut Criterion) {
    const NPC_COUNT: usize = 1000;
    
    c.bench_function("physics_async_telemetry_overhead", |b| {
        let mut world = create_world_with_characters(NPC_COUNT);
        world.enable_async_physics(4);
        
        b.iter(|| {
            world.step();
            // Get telemetry (should be very fast)
            black_box(world.get_last_profile());
        });
    });
}

/// Benchmark: Character movement + physics (realistic workload)
fn physics_async_character_simulation(c: &mut Criterion) {
    const CHAR_COUNT: usize = 2500;
    let mut world = create_world_with_characters(CHAR_COUNT);
    world.enable_async_physics(4);
    
    // Get all character IDs
    let char_ids: Vec<u64> = (1..=(CHAR_COUNT as u64)).collect();
    
    c.bench_function("physics_async_character_simulation", |b| {
        let mut frame = 0;
        b.iter(|| {
            // Move characters (simulating AI/input)
            for &id in &char_ids {
                let move_dir = vec3(
                    ((frame as f32 + id as f32) * 0.1).sin(),
                    0.0,
                    ((frame as f32 + id as f32) * 0.1).cos(),
                );
                world.control_character(id, move_dir, 1.0 / 60.0, false);
            }
            
            // Step physics
            world.step();
            
            frame += 1;
        });
    });
}

/// Benchmark: Mixed workload (characters + rigid bodies)
fn physics_async_mixed_workload(c: &mut Criterion) {
    let mut group = c.benchmark_group("physics_async_mixed_workload");
    
    for (char_count, body_count) in [(500, 200), (1000, 400), (2000, 800)].iter() {
        let mut world = PhysicsWorld::new(vec3(0.0, -9.8, 0.0));
        world.create_ground_plane(vec3(100.0, 0.5, 100.0), 0.9);
        
        // Add characters
        for i in 0..*char_count {
            let x = (i % 50) as f32 * 2.0 - 50.0;
            let z = (i / 50) as f32 * 2.0 - 50.0;
            world.add_character(vec3(x, 1.0, z), vec3(0.4, 0.9, 0.4));
        }
        
        // Add rigid bodies
        let mut rng = rand::rngs::StdRng::seed_from_u64(54321);
        for _ in 0..*body_count {
            let x = rng.gen_range(-40.0..40.0);
            let y = rng.gen_range(5.0..15.0);
            let z = rng.gen_range(-40.0..40.0);
            world.add_dynamic_box(
                vec3(x, y, z),
                vec3(0.5, 0.5, 0.5),
                1.0,
                Layers::DEFAULT,
            );
        }
        
        // Enable async
        world.enable_async_physics(4);
        
        let label = format!("{}chars_{}bodies", char_count, body_count);
        group.throughput(Throughput::Elements((char_count + body_count) as u64));
        group.bench_function(&label, |b| {
            b.iter(|| {
                world.step();
            });
        });
    }
    
    group.finish();
}

criterion_group!(
    async_physics_benches,
    physics_full_tick_baseline,
    physics_async_full_tick,
    physics_async_thread_scaling,
    physics_async_rigid_bodies,
    physics_async_telemetry_overhead,
    physics_async_character_simulation,
    physics_async_mixed_workload,
);
criterion_main!(async_physics_benches);
