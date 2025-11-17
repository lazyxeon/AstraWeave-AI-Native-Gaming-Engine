//! astraweave-pcg Benchmarks
//!
//! Measures performance of procedural content generation:
//! - Layout generation (rooms, connections)
//! - Encounter placement (combat, loot, ambient)
//! - RNG operations (seeding, forking, choosing)
//! - Full dungeon/level generation pipeline
//!
//! Performance targets:
//! - Small dungeon (5-10 rooms): <1 ms
//! - Medium dungeon (20-30 rooms): <10 ms
//! - Large dungeon (50-100 rooms): <50 ms
//! - Encounter placement (100 encounters): <5 ms
//! - RNG operations: <100 ns per operation

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use glam::{ivec2, IVec2};
use std::hint::black_box as std_black_box;

use astraweave_pcg::{EncounterConstraints, EncounterGenerator, LayoutGenerator, Room, SeedRng};

// ============================================================================
// Benchmark 1: RNG Operations
// ============================================================================

fn bench_rng_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("rng_operations");

    // Benchmark: RNG creation
    group.bench_function("create_rng", |b| {
        b.iter(|| {
            let rng = SeedRng::new(42, "test");
            std_black_box(rng)
        })
    });

    // Benchmark: RNG fork
    group.bench_function("fork_rng", |b| {
        let mut parent = SeedRng::new(42, "parent");

        b.iter(|| {
            let child = parent.fork("child");
            std_black_box(child)
        })
    });

    // Benchmark: gen_range (i32)
    group.bench_function("gen_range_i32", |b| {
        let mut rng = SeedRng::new(42, "test");

        b.iter(|| {
            let val = rng.gen_range(0..100);
            std_black_box(val)
        })
    });

    // Benchmark: gen_range (f32)
    group.bench_function("gen_range_f32", |b| {
        let mut rng = SeedRng::new(42, "test");

        b.iter(|| {
            let val = rng.gen_range(0.0..100.0);
            std_black_box(val)
        })
    });

    // Benchmark: gen_bool
    group.bench_function("gen_bool", |b| {
        let mut rng = SeedRng::new(42, "test");

        b.iter(|| {
            let val = rng.gen_bool();
            std_black_box(val)
        })
    });

    // Benchmark: choose from slice
    group.bench_function("choose_from_10", |b| {
        let mut rng = SeedRng::new(42, "test");
        let items = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        b.iter(|| {
            let choice = rng.choose(&items);
            std_black_box(choice)
        })
    });

    // Benchmark: shuffle 100 items
    group.bench_function("shuffle_100", |b| {
        let mut rng = SeedRng::new(42, "test");

        b.iter(|| {
            let mut items: Vec<i32> = (0..100).collect();
            rng.shuffle(&mut items);
            std_black_box(items)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 2: Room Generation
// ============================================================================

fn bench_room_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("room_generation");

    // Benchmark: Generate 5 rooms (small dungeon)
    group.bench_function("generate_5_rooms", |b| {
        let gen = LayoutGenerator::new(ivec2(100, 100));

        b.iter(|| {
            let mut rng_clone = SeedRng::new(42, "test");
            let rooms = gen.generate_rooms(&mut rng_clone, 5);
            std_black_box(rooms)
        })
    });

    // Benchmark: Generate 10 rooms
    group.bench_function("generate_10_rooms", |b| {
        let gen = LayoutGenerator::new(ivec2(100, 100));

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let rooms = gen.generate_rooms(&mut rng, 10);
            std_black_box(rooms)
        })
    });

    // Benchmark: Generate 20 rooms (medium dungeon)
    group.bench_function("generate_20_rooms", |b| {
        let gen = LayoutGenerator::new(ivec2(150, 150));

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let rooms = gen.generate_rooms(&mut rng, 20);
            std_black_box(rooms)
        })
    });

    // Benchmark: Generate 50 rooms (large dungeon)
    group.bench_function("generate_50_rooms", |b| {
        let gen = LayoutGenerator::new(ivec2(200, 200));

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let rooms = gen.generate_rooms(&mut rng, 50);
            std_black_box(rooms)
        })
    });

    // Benchmark: Generate 100 rooms (stress test)
    group.bench_function("generate_100_rooms", |b| {
        let gen = LayoutGenerator::new(ivec2(300, 300));

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let rooms = gen.generate_rooms(&mut rng, 100);
            std_black_box(rooms)
        })
    });

    // Benchmark: Room overlap check
    group.bench_function("room_overlap_check", |b| {
        let room1 = Room {
            bounds: (ivec2(0, 0), ivec2(10, 10)),
            connections: Vec::new(),
        };
        let room2 = Room {
            bounds: (ivec2(5, 5), ivec2(15, 15)),
            connections: Vec::new(),
        };

        b.iter(|| {
            let overlaps = room1.overlaps(&room2);
            std_black_box(overlaps)
        })
    });

    // Benchmark: Room center calculation
    group.bench_function("room_center", |b| {
        let room = Room {
            bounds: (ivec2(10, 20), ivec2(30, 40)),
            connections: Vec::new(),
        };

        b.iter(|| {
            let center = room.center();
            std_black_box(center)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 3: Encounter Generation
// ============================================================================

fn bench_encounter_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("encounter_generation");

    // Benchmark: Generate 10 encounters
    group.bench_function("generate_10_encounters", |b| {
        let constraints = EncounterConstraints::default();
        let gen = EncounterGenerator::new(constraints);

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let encounters = gen.generate(&mut rng, 10);
            std_black_box(encounters)
        })
    });

    // Benchmark: Generate 50 encounters
    group.bench_function("generate_50_encounters", |b| {
        let constraints = EncounterConstraints {
            bounds: (IVec2::ZERO, ivec2(200, 200)),
            min_spacing: 10.0,
            difficulty_range: (1.0, 5.0),
        };
        let gen = EncounterGenerator::new(constraints);

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let encounters = gen.generate(&mut rng, 50);
            std_black_box(encounters)
        })
    });

    // Benchmark: Generate 100 encounters (stress test)
    group.bench_function("generate_100_encounters", |b| {
        let constraints = EncounterConstraints {
            bounds: (IVec2::ZERO, ivec2(300, 300)),
            min_spacing: 8.0,
            difficulty_range: (1.0, 10.0),
        };
        let gen = EncounterGenerator::new(constraints);

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let encounters = gen.generate(&mut rng, 100);
            std_black_box(encounters)
        })
    });

    // Benchmark: Generate 200 encounters (large level)
    group.bench_function("generate_200_encounters", |b| {
        let constraints = EncounterConstraints {
            bounds: (IVec2::ZERO, ivec2(500, 500)),
            min_spacing: 5.0,
            difficulty_range: (1.0, 10.0),
        };
        let gen = EncounterGenerator::new(constraints);

        b.iter(|| {
            let mut rng = SeedRng::new(42, "test");
            let encounters = gen.generate(&mut rng, 200);
            std_black_box(encounters)
        })
    });

    // Benchmark: Spacing constraint check (100 existing encounters)
    group.bench_function("spacing_check_100", |b| {
        let constraints = EncounterConstraints {
            bounds: (IVec2::ZERO, ivec2(200, 200)),
            min_spacing: 10.0,
            difficulty_range: (1.0, 5.0),
        };
        let gen = EncounterGenerator::new(constraints);
        let mut rng = SeedRng::new(42, "test");
        let encounters = gen.generate(&mut rng, 100);
        let positions: Vec<IVec2> = encounters.iter().map(|e| e.position).collect();

        b.iter(|| {
            let new_pos = ivec2(50, 50);
            let valid = positions.iter().all(|&p| {
                let dist = (new_pos - p).as_vec2().length();
                dist >= 10.0
            });
            std_black_box(valid)
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 4: Full Dungeon Pipeline
// ============================================================================

fn bench_full_dungeon(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_dungeon_pipeline");

    // Benchmark: Small dungeon (5 rooms + 10 encounters)
    group.bench_function("small_dungeon_5r_10e", |b| {
        let layout_gen = LayoutGenerator::new(ivec2(100, 100));
        let encounter_gen = EncounterGenerator::new(EncounterConstraints::default());

        b.iter(|| {
            let mut rng = SeedRng::new(42, "dungeon");
            let mut layout_rng = rng.fork("layout");
            let mut encounter_rng = rng.fork("encounters");

            let rooms = layout_gen.generate_rooms(&mut layout_rng, 5);
            let encounters = encounter_gen.generate(&mut encounter_rng, 10);

            std_black_box((rooms, encounters))
        })
    });

    // Benchmark: Medium dungeon (20 rooms + 50 encounters)
    group.bench_function("medium_dungeon_20r_50e", |b| {
        let layout_gen = LayoutGenerator::new(ivec2(150, 150));
        let encounter_gen = EncounterGenerator::new(EncounterConstraints {
            bounds: (IVec2::ZERO, ivec2(150, 150)),
            min_spacing: 8.0,
            difficulty_range: (1.0, 5.0),
        });

        b.iter(|| {
            let mut rng = SeedRng::new(42, "dungeon");
            let mut layout_rng = rng.fork("layout");
            let mut encounter_rng = rng.fork("encounters");

            let rooms = layout_gen.generate_rooms(&mut layout_rng, 20);
            let encounters = encounter_gen.generate(&mut encounter_rng, 50);

            std_black_box((rooms, encounters))
        })
    });

    // Benchmark: Large dungeon (50 rooms + 150 encounters)
    group.bench_function("large_dungeon_50r_150e", |b| {
        let layout_gen = LayoutGenerator::new(ivec2(250, 250));
        let encounter_gen = EncounterGenerator::new(EncounterConstraints {
            bounds: (IVec2::ZERO, ivec2(250, 250)),
            min_spacing: 6.0,
            difficulty_range: (1.0, 10.0),
        });

        b.iter(|| {
            let mut rng = SeedRng::new(42, "dungeon");
            let mut layout_rng = rng.fork("layout");
            let mut encounter_rng = rng.fork("encounters");

            let rooms = layout_gen.generate_rooms(&mut layout_rng, 50);
            let encounters = encounter_gen.generate(&mut encounter_rng, 150);

            std_black_box((rooms, encounters))
        })
    });

    // Benchmark: Huge dungeon (100 rooms + 300 encounters)
    group.bench_function("huge_dungeon_100r_300e", |b| {
        let layout_gen = LayoutGenerator::new(ivec2(400, 400));
        let encounter_gen = EncounterGenerator::new(EncounterConstraints {
            bounds: (IVec2::ZERO, ivec2(400, 400)),
            min_spacing: 5.0,
            difficulty_range: (1.0, 10.0),
        });

        b.iter(|| {
            let mut rng = SeedRng::new(42, "dungeon");
            let mut layout_rng = rng.fork("layout");
            let mut encounter_rng = rng.fork("encounters");

            let rooms = layout_gen.generate_rooms(&mut layout_rng, 100);
            let encounters = encounter_gen.generate(&mut encounter_rng, 300);

            std_black_box((rooms, encounters))
        })
    });

    group.finish();
}

// ============================================================================
// Benchmark 5: Scaling Analysis
// ============================================================================

fn bench_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("scaling");

    // Room generation scaling
    for room_count in [1, 5, 10, 20, 50, 100].iter() {
        let grid_size = ivec2(room_count * 30, room_count * 30);

        group.throughput(Throughput::Elements(*room_count as u64));
        group.bench_with_input(
            BenchmarkId::new("room_generation", format!("{}_rooms", room_count)),
            room_count,
            |b, &count| {
                let gen = LayoutGenerator::new(grid_size);

                b.iter(|| {
                    let mut rng = SeedRng::new(42, "test");
                    let rooms = gen.generate_rooms(&mut rng, count as u32);
                    std_black_box(rooms)
                })
            },
        );
    }

    // Encounter generation scaling
    for encounter_count in [10, 25, 50, 100, 200, 500].iter() {
        let grid_size = ivec2(encounter_count * 10, encounter_count * 10);

        group.throughput(Throughput::Elements(*encounter_count as u64));
        group.bench_with_input(
            BenchmarkId::new(
                "encounter_generation",
                format!("{}_encounters", encounter_count),
            ),
            encounter_count,
            |b, &count| {
                let constraints = EncounterConstraints {
                    bounds: (IVec2::ZERO, grid_size),
                    min_spacing: 5.0,
                    difficulty_range: (1.0, 10.0),
                };
                let gen = EncounterGenerator::new(constraints);

                b.iter(|| {
                    let mut rng = SeedRng::new(42, "test");
                    let encounters = gen.generate(&mut rng, count as u32);
                    std_black_box(encounters)
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_rng_operations,
    bench_room_generation,
    bench_encounter_generation,
    bench_full_dungeon,
    bench_scaling,
);
criterion_main!(benches);

