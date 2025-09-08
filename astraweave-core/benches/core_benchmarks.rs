use astraweave_core::{IVec2, Team, World};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_world_creation(c: &mut Criterion) {
    c.bench_function("world_creation", |b| b.iter(|| black_box(World::new())));
}

fn bench_entity_spawning(c: &mut Criterion) {
    c.bench_function("entity_spawning", |b| {
        b.iter(|| {
            let mut world = black_box(World::new());
            for i in 0..100 {
                world.spawn(
                    &format!("entity_{}", i),
                    IVec2 {
                        x: i % 10,
                        y: i / 10,
                    },
                    Team { id: (i % 3) as u8 },
                    100,
                    30,
                );
            }
            black_box(world)
        })
    });
}

fn bench_world_tick(c: &mut Criterion) {
    c.bench_function("world_tick", |b| {
        let mut world = World::new();
        // Spawn some entities for more realistic testing
        for i in 0..50 {
            world.spawn(
                &format!("entity_{}", i),
                IVec2 {
                    x: i % 10,
                    y: i / 10,
                },
                Team { id: (i % 3) as u8 },
                100,
                30,
            );
        }

        b.iter(|| {
            world.tick(black_box(0.016)); // 60 FPS timestep
        })
    });
}

criterion_group!(
    benches,
    bench_world_creation,
    bench_entity_spawning,
    bench_world_tick
);
criterion_main!(benches);
