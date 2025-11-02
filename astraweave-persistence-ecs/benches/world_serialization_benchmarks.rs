use astraweave_core::ecs_components::*;
use astraweave_core::IVec2;
use astraweave_ecs::World;
use astraweave_persistence_ecs::{
    calculate_world_hash, deserialize_ecs_world, serialize_ecs_world,
};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};

/// Helper to create a world with N entities and various components
fn create_test_world(entity_count: usize) -> World {
    let mut world = World::new();

    for i in 0..entity_count {
        let entity = world.spawn();

        // Distribute components across entities for realism
        // ~80% have position
        if i % 5 != 0 {
            world.insert(entity, CPos {
                pos: IVec2 {
                    x: (i as i32) % 100,
                    y: (i as i32) / 100,
                },
            });
        }

        // ~60% have health
        if i % 5 < 3 {
            world.insert(entity, CHealth {
                hp: 100 - (i as i32) % 100,
            });
        }

        // ~40% have team
        if i % 5 < 2 {
            world.insert(entity, CTeam {
                id: (i % 4) as u8,
            });
        }

        // ~20% have ammo
        if i % 5 == 0 {
            world.insert(entity, CAmmo {
                rounds: 30 - (i as i32) % 30,
            });
        }

        // ~10% have AI agent marker
        if i % 10 == 0 {
            world.insert(entity, CAiAgent);
        }

        // ~5% have cooldowns (expensive component)
        if i % 20 == 0 {
            let mut cooldowns = CCooldowns::default();
            cooldowns.map.insert(
                cooldowns::CooldownKey::ThrowSmoke,
                (i % 10) as f32 * 0.1,
            );
            world.insert(entity, cooldowns);
        }
    }

    world
}

fn bench_serialize_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialize_world");

    for size in [10, 100, 500, 1_000, 2_000].iter() {
        let world = create_test_world(*size);
        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let blob = serialize_ecs_world(black_box(&world)).unwrap();
                black_box(blob);
            });
        });
    }

    group.finish();
}

fn bench_deserialize_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("deserialize_world");

    for size in [10, 100, 500, 1_000, 2_000].iter() {
        let world = create_test_world(*size);
        let blob = serialize_ecs_world(&world).unwrap();

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let mut new_world = World::new();
                deserialize_ecs_world(black_box(&blob), black_box(&mut new_world)).unwrap();
                black_box(new_world);
            });
        });
    }

    group.finish();
}

fn bench_roundtrip_world(c: &mut Criterion) {
    let mut group = c.benchmark_group("roundtrip_world");

    for size in [10, 100, 500, 1_000, 2_000].iter() {
        let world = create_test_world(*size);

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                // Serialize
                let blob = serialize_ecs_world(black_box(&world)).unwrap();

                // Deserialize
                let mut new_world = World::new();
                deserialize_ecs_world(black_box(&blob), black_box(&mut new_world)).unwrap();

                black_box(new_world);
            });
        });
    }

    group.finish();
}

fn bench_calculate_hash(c: &mut Criterion) {
    let mut group = c.benchmark_group("calculate_hash");

    for size in [10, 100, 500, 1_000, 2_000].iter() {
        let world = create_test_world(*size);

        group.throughput(Throughput::Elements(*size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let hash = calculate_world_hash(black_box(&world));
                black_box(hash);
            });
        });
    }

    group.finish();
}

fn bench_blob_size(c: &mut Criterion) {
    let mut group = c.benchmark_group("blob_size");

    for size in [10, 100, 500, 1_000, 2_000].iter() {
        let world = create_test_world(*size);
        let blob = serialize_ecs_world(&world).unwrap();

        println!(
            "World with {} entities serializes to {} bytes ({:.2} bytes/entity)",
            size,
            blob.len(),
            blob.len() as f64 / *size as f64
        );

        group.throughput(Throughput::Bytes(blob.len() as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
            b.iter(|| {
                let blob = serialize_ecs_world(black_box(&world)).unwrap();
                black_box(blob.len());
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_serialize_world,
    bench_deserialize_world,
    bench_roundtrip_world,
    bench_calculate_hash,
    bench_blob_size
);
criterion_main!(benches);
