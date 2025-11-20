// SPDX-License-Identifier: MIT
//! High-level ECS operation benchmarks
//!
//! Baseline performance metrics for regression detection.
//! Created: Week 6 Day 4 (October 24, 2025)
//!
//! # Benchmarks
//!
//! 1. **Entity Spawn/Despawn** (1k, 10k entities)
//! 2. **Component Add/Remove** (1k, 10k operations)
//! 3. **Component Iteration** (1k, 10k entities)
//! 4. **Archetype Transitions** (100 entities × 10 transitions)
//!
//! # Performance Targets (60 FPS = 16.67ms frame budget)
//!
//! - Entity spawn: <1 µs/entity (10k spawns < 10ms)
//! - Component iteration: <100 ns/entity (10k queries < 1ms)
//! - Archetype transition: <2 µs/transition

use astraweave_ecs::World;
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::hint::black_box;

// Test components
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Health {
    current: i32,
    max: i32,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Armor {
    value: i32,
}

// === Benchmark 1: Entity Spawn ===

fn bench_entity_spawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_spawn");

    for count in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("empty", count), &count, |b, &count| {
            b.iter_batched(
                || World::new(),
                |mut world| {
                    for _ in 0..count {
                        black_box(world.spawn());
                    }
                    black_box(world);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("with_position", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || World::new(),
                    |mut world| {
                        for i in 0..count {
                            let entity = world.spawn();
                            world.insert(
                                entity,
                                Position {
                                    x: i as f32,
                                    y: i as f32 * 2.0,
                                    z: i as f32 * 3.0,
                                },
                            );
                        }
                        black_box(world);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("with_position_velocity", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || World::new(),
                    |mut world| {
                        for i in 0..count {
                            let entity = world.spawn();
                            world.insert(
                                entity,
                                Position {
                                    x: i as f32,
                                    y: i as f32 * 2.0,
                                    z: i as f32 * 3.0,
                                },
                            );
                            world.insert(
                                entity,
                                Velocity {
                                    x: 1.0,
                                    y: 0.0,
                                    z: 0.0,
                                },
                            );
                        }
                        black_box(world);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// === Benchmark 2: Entity Despawn ===

fn bench_entity_despawn(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_despawn");

    for count in [100, 1000, 10000] {
        group.bench_with_input(BenchmarkId::new("empty", count), &count, |b, &count| {
            b.iter_batched(
                || {
                    let mut world = World::new();
                    let entities: Vec<_> = (0..count).map(|_| world.spawn()).collect();
                    (world, entities)
                },
                |(mut world, entities)| {
                    for entity in entities {
                        black_box(world.despawn(entity));
                    }
                    black_box(world);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        group.bench_with_input(
            BenchmarkId::new("with_components", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut world = World::new();
                        let entities: Vec<_> = (0..count)
                            .map(|i| {
                                let entity = world.spawn();
                                world.insert(
                                    entity,
                                    Position {
                                        x: i as f32,
                                        y: 0.0,
                                        z: 0.0,
                                    },
                                );
                                world.insert(
                                    entity,
                                    Velocity {
                                        x: 1.0,
                                        y: 0.0,
                                        z: 0.0,
                                    },
                                );
                                entity
                            })
                            .collect();
                        (world, entities)
                    },
                    |(mut world, entities)| {
                        for entity in entities {
                            black_box(world.despawn(entity));
                        }
                        black_box(world);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// === Benchmark 3: Component Add ===

fn bench_component_add(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_add");

    for count in [100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("single_component", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut world = World::new();
                        let entities: Vec<_> = (0..count).map(|_| world.spawn()).collect();
                        (world, entities)
                    },
                    |(mut world, entities)| {
                        for (i, entity) in entities.iter().enumerate() {
                            world.insert(
                                *entity,
                                Position {
                                    x: i as f32,
                                    y: 0.0,
                                    z: 0.0,
                                },
                            );
                        }
                        black_box(world);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("multiple_components", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut world = World::new();
                        let entities: Vec<_> = (0..count).map(|_| world.spawn()).collect();
                        (world, entities)
                    },
                    |(mut world, entities)| {
                        for (i, entity) in entities.iter().enumerate() {
                            world.insert(
                                *entity,
                                Position {
                                    x: i as f32,
                                    y: 0.0,
                                    z: 0.0,
                                },
                            );
                            world.insert(
                                *entity,
                                Velocity {
                                    x: 1.0,
                                    y: 0.0,
                                    z: 0.0,
                                },
                            );
                            world.insert(
                                *entity,
                                Health {
                                    current: 100,
                                    max: 100,
                                },
                            );
                        }
                        black_box(world);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// === Benchmark 4: Component Remove ===

fn bench_component_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_remove");

    for count in [100, 1000, 10000] {
        group.bench_with_input(
            BenchmarkId::new("single_component", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut world = World::new();
                        let entities: Vec<_> = (0..count)
                            .map(|i| {
                                let entity = world.spawn();
                                world.insert(
                                    entity,
                                    Position {
                                        x: i as f32,
                                        y: 0.0,
                                        z: 0.0,
                                    },
                                );
                                entity
                            })
                            .collect();
                        (world, entities)
                    },
                    |(mut world, entities)| {
                        for entity in entities {
                            black_box(world.remove::<Position>(entity));
                        }
                        black_box(world);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );

        group.bench_with_input(
            BenchmarkId::new("multiple_components", count),
            &count,
            |b, &count| {
                b.iter_batched(
                    || {
                        let mut world = World::new();
                        let entities: Vec<_> = (0..count)
                            .map(|i| {
                                let entity = world.spawn();
                                world.insert(
                                    entity,
                                    Position {
                                        x: i as f32,
                                        y: 0.0,
                                        z: 0.0,
                                    },
                                );
                                world.insert(
                                    entity,
                                    Velocity {
                                        x: 1.0,
                                        y: 0.0,
                                        z: 0.0,
                                    },
                                );
                                world.insert(
                                    entity,
                                    Health {
                                        current: 100,
                                        max: 100,
                                    },
                                );
                                entity
                            })
                            .collect();
                        (world, entities)
                    },
                    |(mut world, entities)| {
                        for entity in entities {
                            black_box(world.remove::<Position>(entity));
                            black_box(world.remove::<Velocity>(entity));
                            black_box(world.remove::<Health>(entity));
                        }
                        black_box(world);
                    },
                    criterion::BatchSize::SmallInput,
                );
            },
        );
    }

    group.finish();
}

// === Benchmark 5: Component Iteration (using each_mut) ===

fn bench_component_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_iteration");

    for count in [100, 1000, 10000] {
        // Setup world with Position components
        let mut world = World::new();
        for i in 0..count {
            let entity = world.spawn();
            world.insert(
                entity,
                Position {
                    x: i as f32,
                    y: i as f32 * 2.0,
                    z: i as f32 * 3.0,
                },
            );
        }

        group.bench_with_input(BenchmarkId::new("position_write", count), &count, |b, _| {
            b.iter(|| {
                world.each_mut(|_entity, pos: &mut Position| {
                    pos.x += 1.0;
                    pos.y += 1.0;
                    pos.z += 1.0;
                });
            });
        });
    }

    group.finish();
}

// === Benchmark 7: Archetype Transitions ===

fn bench_archetype_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("archetype_transitions");

    // Test with 100 entities doing 10 transitions each
    let entity_count = 100;
    let transition_count = 10;

    group.bench_function("add_remove_cycle", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entities: Vec<_> = (0..entity_count)
                    .map(|i| {
                        let entity = world.spawn();
                        world.insert(
                            entity,
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                        );
                        entity
                    })
                    .collect();
                (world, entities)
            },
            |(mut world, entities)| {
                for _ in 0..transition_count {
                    // Add Velocity (transition to [Position, Velocity] archetype)
                    for &entity in &entities {
                        world.insert(
                            entity,
                            Velocity {
                                x: 1.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        );
                    }

                    // Remove Velocity (transition back to [Position] archetype)
                    for &entity in &entities {
                        world.remove::<Velocity>(entity);
                    }
                }
                black_box(world);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.bench_function("multi_component_transitions", |b| {
        b.iter_batched(
            || {
                let mut world = World::new();
                let entities: Vec<_> = (0..entity_count)
                    .map(|i| {
                        let entity = world.spawn();
                        world.insert(
                            entity,
                            Position {
                                x: i as f32,
                                y: 0.0,
                                z: 0.0,
                            },
                        );
                        entity
                    })
                    .collect();
                (world, entities)
            },
            |(mut world, entities)| {
                for _ in 0..transition_count {
                    // Transition 1: Add Velocity
                    for &entity in &entities {
                        world.insert(
                            entity,
                            Velocity {
                                x: 1.0,
                                y: 0.0,
                                z: 0.0,
                            },
                        );
                    }

                    // Transition 2: Add Health
                    for &entity in &entities {
                        world.insert(
                            entity,
                            Health {
                                current: 100,
                                max: 100,
                            },
                        );
                    }

                    // Transition 3: Add Armor
                    for &entity in &entities {
                        world.insert(entity, Armor { value: 50 });
                    }

                    // Transition 4: Remove Armor
                    for &entity in &entities {
                        world.remove::<Armor>(entity);
                    }

                    // Transition 5: Remove Health
                    for &entity in &entities {
                        world.remove::<Health>(entity);
                    }

                    // Transition 6: Remove Velocity (back to starting archetype)
                    for &entity in &entities {
                        world.remove::<Velocity>(entity);
                    }
                }
                black_box(world);
            },
            criterion::BatchSize::SmallInput,
        );
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_entity_spawn,
    bench_entity_despawn,
    bench_component_add,
    bench_component_remove,
    bench_component_iteration,
    bench_archetype_transitions,
);
criterion_main!(benches);
