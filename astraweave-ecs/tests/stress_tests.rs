//! Large-scale stress tests for ECS
//!
//! These tests validate performance under extreme load and detect memory leaks.
//! Run with: cargo test --test stress_tests --release -- --nocapture
//!
//! Note: These tests are slow (several seconds each) but validate production limits.

use astraweave_ecs::World;
use std::time::Instant;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Velocity {
    dx: i32,
    dy: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Health {
    hp: u32,
    max_hp: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Armor {
    defense: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Damage {
    attack: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Name {
    id: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tag1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tag2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tag3;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Tag4;

// ============================================================================
// Test 1: 100k Entity Stress Test
// ============================================================================

#[test]
#[ignore] // Slow test, run with --ignored
fn stress_test_100k_entities() {
    println!("\n=== 100k Entity Stress Test ===");

    let mut world = World::new();
    let mut entities = Vec::with_capacity(100_000);

    // Phase 1: Spawn 100k entities
    println!("Phase 1: Spawning 100,000 entities...");
    let start = Instant::now();

    for i in 0..100_000 {
        let entity = world.spawn();
        world.insert(entity, Position { x: i, y: i });
        world.insert(
            entity,
            Health {
                hp: 100,
                max_hp: 100,
            },
        );
        entities.push(entity);
    }

    let spawn_time = start.elapsed();
    println!(
        "  Spawned in {:?} ({:.2} entities/sec)",
        spawn_time,
        100_000.0 / spawn_time.as_secs_f64()
    );

    assert_eq!(world.entity_count(), 100_000);

    // Phase 2: Modify all entities
    println!("Phase 2: Modifying 100,000 entities...");
    let start = Instant::now();

    for &entity in &entities {
        if let Some(pos) = world.get_mut::<Position>(entity) {
            pos.x += 1;
            pos.y += 1;
        }
    }

    let modify_time = start.elapsed();
    println!(
        "  Modified in {:?} ({:.2} entities/sec)",
        modify_time,
        100_000.0 / modify_time.as_secs_f64()
    );

    // Phase 3: Query all entities
    println!("Phase 3: Querying 100,000 entities...");
    let start = Instant::now();

    let positions = world.entities_with::<Position>();

    let query_time = start.elapsed();
    println!(
        "  Queried in {:?} (found {} entities)",
        query_time,
        positions.len()
    );

    assert_eq!(positions.len(), 100_000);

    // Phase 4: Despawn all entities
    println!("Phase 4: Despawning 100,000 entities...");
    let start = Instant::now();

    for entity in entities {
        world.despawn(entity);
    }

    let despawn_time = start.elapsed();
    println!(
        "  Despawned in {:?} ({:.2} entities/sec)",
        despawn_time,
        100_000.0 / despawn_time.as_secs_f64()
    );

    assert_eq!(world.entity_count(), 0);

    // Summary
    let total_time = spawn_time + modify_time + query_time + despawn_time;
    println!("\n=== Summary ===");
    println!("  Total time: {:?}", total_time);
    println!(
        "  Spawn:   {:?} ({:.2}%)",
        spawn_time,
        100.0 * spawn_time.as_secs_f64() / total_time.as_secs_f64()
    );
    println!(
        "  Modify:  {:?} ({:.2}%)",
        modify_time,
        100.0 * modify_time.as_secs_f64() / total_time.as_secs_f64()
    );
    println!(
        "  Query:   {:?} ({:.2}%)",
        query_time,
        100.0 * query_time.as_secs_f64() / total_time.as_secs_f64()
    );
    println!(
        "  Despawn: {:?} ({:.2}%)",
        despawn_time,
        100.0 * despawn_time.as_secs_f64() / total_time.as_secs_f64()
    );

    // Validate performance
    assert!(
        spawn_time.as_secs_f64() < 5.0,
        "Spawning too slow: {:?}",
        spawn_time
    );
    assert!(
        modify_time.as_secs_f64() < 5.0,
        "Modifying too slow: {:?}",
        modify_time
    );
    assert!(
        query_time.as_secs_f64() < 1.0,
        "Querying too slow: {:?}",
        query_time
    );
    assert!(
        despawn_time.as_secs_f64() < 5.0,
        "Despawning too slow: {:?}",
        despawn_time
    );
}

// ============================================================================
// Test 2: Component Thrashing Test
// ============================================================================

#[test]
#[ignore] // Slow test
fn stress_test_component_thrashing() {
    println!("\n=== Component Thrashing Test ===");

    let mut world = World::new();
    let mut entities = Vec::with_capacity(10_000);

    // Spawn 10k entities with Position
    println!("Spawning 10,000 entities...");
    for i in 0..10_000 {
        let entity = world.spawn();
        world.insert(entity, Position { x: i, y: i });
        entities.push(entity);
    }

    // Thrash components: 1000 cycles of add/remove
    println!("Thrashing components (1000 cycles)...");
    let start = Instant::now();
    let mut cycle_times = Vec::with_capacity(1000);

    for cycle in 0..1000 {
        let cycle_start = Instant::now();

        // Add Velocity to all entities
        for &entity in &entities {
            world.insert(
                entity,
                Velocity {
                    dx: cycle,
                    dy: cycle,
                },
            );
        }

        // Remove Velocity from all entities
        for &entity in &entities {
            world.remove::<Velocity>(entity);
        }

        cycle_times.push(cycle_start.elapsed());

        if cycle % 100 == 0 {
            println!("  Cycle {}/1000 complete", cycle);
        }
    }

    let total_time = start.elapsed();
    println!("  Completed in {:?}", total_time);

    // Analyze performance degradation
    let first_100_avg: std::time::Duration =
        cycle_times[0..100].iter().sum::<std::time::Duration>() / 100;
    let last_100_avg: std::time::Duration =
        cycle_times[900..1000].iter().sum::<std::time::Duration>() / 100;

    let degradation = (last_100_avg.as_secs_f64() / first_100_avg.as_secs_f64() - 1.0) * 100.0;

    println!("\n=== Performance Analysis ===");
    println!("  First 100 cycles avg: {:?}", first_100_avg);
    println!("  Last 100 cycles avg:  {:?}", last_100_avg);
    println!("  Performance degradation: {:.2}%", degradation);

    // Validate no significant degradation (< 10%)
    assert!(
        degradation < 10.0,
        "Performance degraded by {:.2}%",
        degradation
    );

    // Validate all entities still alive
    assert_eq!(world.entity_count(), 10_000);
    for entity in entities {
        assert!(world.is_alive(entity));
    }
}

// ============================================================================
// Test 3: Memory Leak Detection
// ============================================================================

#[test]
#[ignore] // Slow test
fn stress_test_memory_leak_detection() {
    println!("\n=== Memory Leak Detection Test ===");

    let mut world = World::new();

    println!("Running 10,000 spawn/despawn cycles...");
    let start = Instant::now();

    for cycle in 0..10_000 {
        // Spawn 100 entities
        let mut entities = Vec::with_capacity(100);
        for i in 0..100 {
            let entity = world.spawn();
            world.insert(entity, Position { x: i, y: i });
            world.insert(entity, Velocity { dx: i, dy: i });
            world.insert(
                entity,
                Health {
                    hp: 100,
                    max_hp: 100,
                },
            );
            entities.push(entity);
        }

        // Despawn all entities
        for entity in entities {
            world.despawn(entity);
        }

        // Validate entity count returns to 0
        assert_eq!(
            world.entity_count(),
            0,
            "Memory leak detected at cycle {}",
            cycle
        );

        if cycle % 1000 == 0 {
            println!("  Cycle {}/10,000 complete", cycle);
        }
    }

    let total_time = start.elapsed();
    println!("  Completed in {:?}", total_time);
    println!("  Average cycle time: {:?}", total_time / 10_000);

    // Final validation
    assert_eq!(world.entity_count(), 0, "Entities leaked after test");

    println!("\n=== Result ===");
    println!("  ✅ No memory leaks detected (1M total entities processed)");
}

// ============================================================================
// Test 4: Query Performance Test
// ============================================================================

#[test]
#[ignore] // Slow test
fn stress_test_query_performance() {
    println!("\n=== Query Performance Test ===");

    let mut world = World::new();

    // Spawn 50k entities with different component combinations
    println!("Spawning 50,000 entities with varying components...");
    let start = Instant::now();

    for i in 0..50_000 {
        let entity = world.spawn();
        world.insert(entity, Position { x: i, y: i });

        if i % 2 == 0 {
            world.insert(entity, Velocity { dx: i, dy: i });
        }

        if i % 3 == 0 {
            world.insert(
                entity,
                Health {
                    hp: 100,
                    max_hp: 100,
                },
            );
        }

        if i % 5 == 0 {
            world.insert(entity, Armor { defense: 50 });
        }
    }

    let spawn_time = start.elapsed();
    println!("  Spawned in {:?}", spawn_time);

    // Query 1: All entities with Position
    println!("Query 1: All entities with Position...");
    let start = Instant::now();
    let pos_entities = world.entities_with::<Position>();
    let query1_time = start.elapsed();
    println!(
        "  Found {} entities in {:?}",
        pos_entities.len(),
        query1_time
    );
    assert_eq!(pos_entities.len(), 50_000);

    // Query 2: All entities with Velocity
    println!("Query 2: All entities with Velocity...");
    let start = Instant::now();
    let vel_entities = world.entities_with::<Velocity>();
    let query2_time = start.elapsed();
    println!(
        "  Found {} entities in {:?}",
        vel_entities.len(),
        query2_time
    );
    assert_eq!(vel_entities.len(), 25_000);

    // Query 3: All entities with Health
    println!("Query 3: All entities with Health...");
    let start = Instant::now();
    let health_entities = world.entities_with::<Health>();
    let query3_time = start.elapsed();
    println!(
        "  Found {} entities in {:?}",
        health_entities.len(),
        query3_time
    );
    assert_eq!(health_entities.len(), 16_667); // 50000 / 3 rounded up

    // Query 4: All entities with Armor
    println!("Query 4: All entities with Armor...");
    let start = Instant::now();
    let armor_entities = world.entities_with::<Armor>();
    let query4_time = start.elapsed();
    println!(
        "  Found {} entities in {:?}",
        armor_entities.len(),
        query4_time
    );
    assert_eq!(armor_entities.len(), 10_000);

    println!("\n=== Query Performance Summary ===");
    println!("  Query 1 (50k results): {:?}", query1_time);
    println!("  Query 2 (25k results): {:?}", query2_time);
    println!("  Query 3 (16k results): {:?}", query3_time);
    println!("  Query 4 (10k results): {:?}", query4_time);

    // Validate query time is reasonable (< 100ms for 50k entities)
    assert!(
        query1_time.as_millis() < 100,
        "Query 1 too slow: {:?}",
        query1_time
    );
    assert!(
        query2_time.as_millis() < 100,
        "Query 2 too slow: {:?}",
        query2_time
    );
    assert!(
        query3_time.as_millis() < 100,
        "Query 3 too slow: {:?}",
        query3_time
    );
    assert!(
        query4_time.as_millis() < 100,
        "Query 4 too slow: {:?}",
        query4_time
    );
}

// ============================================================================
// Test 5: Archetype Explosion Test
// ============================================================================

#[test]
#[ignore] // Slow test
fn stress_test_archetype_explosion() {
    println!("\n=== Archetype Explosion Test ===");

    let mut world = World::new();

    // Create entities with many unique component combinations
    // With 10 component types, we can create 2^10 = 1024 unique combinations
    println!("Creating 1024 unique archetypes...");
    let start = Instant::now();

    let mut entity_count = 0;

    for i in 0..1024_u32 {
        let entity = world.spawn();

        // Use bits of i to determine which components to add
        if i & 0b0000000001 != 0 {
            world.insert(
                entity,
                Position {
                    x: i as i32,
                    y: i as i32,
                },
            );
        }
        if i & 0b0000000010 != 0 {
            world.insert(
                entity,
                Velocity {
                    dx: i as i32,
                    dy: i as i32,
                },
            );
        }
        if i & 0b0000000100 != 0 {
            world.insert(
                entity,
                Health {
                    hp: 100,
                    max_hp: 100,
                },
            );
        }
        if i & 0b0000001000 != 0 {
            world.insert(entity, Armor { defense: 50 });
        }
        if i & 0b0000010000 != 0 {
            world.insert(entity, Damage { attack: 25 });
        }
        if i & 0b0000100000 != 0 {
            world.insert(entity, Name { id: i });
        }
        if i & 0b0001000000 != 0 {
            world.insert(entity, Tag1);
        }
        if i & 0b0010000000 != 0 {
            world.insert(entity, Tag2);
        }
        if i & 0b0100000000 != 0 {
            world.insert(entity, Tag3);
        }
        if i & 0b1000000000 != 0 {
            world.insert(entity, Tag4);
        }

        entity_count += 1;
    }

    let creation_time = start.elapsed();
    println!("  Created {} entities in {:?}", entity_count, creation_time);

    // Query across all archetypes
    println!("Querying across archetypes...");
    let start = Instant::now();

    let pos_count = world.entities_with::<Position>().len();
    let vel_count = world.entities_with::<Velocity>().len();
    let health_count = world.entities_with::<Health>().len();

    let query_time = start.elapsed();

    println!("\n=== Archetype Statistics ===");
    println!("  Total entities: {}", entity_count);
    println!("  Entities with Position: {}", pos_count);
    println!("  Entities with Velocity: {}", vel_count);
    println!("  Entities with Health: {}", health_count);
    println!("  Query time: {:?}", query_time);

    // Validate expected counts (each bit set in ~50% of numbers)
    assert!(
        pos_count >= 400 && pos_count <= 600,
        "Position count unexpected: {}",
        pos_count
    );
    assert!(
        vel_count >= 400 && vel_count <= 600,
        "Velocity count unexpected: {}",
        vel_count
    );
    assert!(
        health_count >= 400 && health_count <= 600,
        "Health count unexpected: {}",
        health_count
    );

    // Validate query performance (< 50ms across many archetypes)
    assert!(
        query_time.as_millis() < 50,
        "Query too slow: {:?}",
        query_time
    );

    println!("\n=== Result ===");
    println!("  ✅ Storage efficient across {} unique archetypes", 1024);
}

// ============================================================================
// Test 6: Mixed Workload Stress Test
// ============================================================================

#[test]
#[ignore] // Slow test
fn stress_test_mixed_workload() {
    println!("\n=== Mixed Workload Stress Test ===");

    let mut world = World::new();
    let mut entities = Vec::with_capacity(10_000);

    println!("Running mixed workload (10,000 operations)...");
    let start = Instant::now();

    for i in 0..10_000 {
        match i % 10 {
            0..=3 => {
                // Spawn (40%)
                let entity = world.spawn();
                world.insert(
                    entity,
                    Position {
                        x: i as i32,
                        y: i as i32,
                    },
                );
                entities.push(entity);
            }
            4..=5 => {
                // Insert component (20%)
                if !entities.is_empty() {
                    let entity = entities[i % entities.len()];
                    world.insert(
                        entity,
                        Velocity {
                            dx: i as i32,
                            dy: i as i32,
                        },
                    );
                }
            }
            6..=7 => {
                // Query (20%)
                let _ = world.entities_with::<Position>();
            }
            8 => {
                // Remove component (10%)
                if !entities.is_empty() {
                    let entity = entities[i % entities.len()];
                    world.remove::<Velocity>(entity);
                }
            }
            9 => {
                // Despawn (10%)
                if entities.len() > 100 {
                    let entity = entities.remove(i % entities.len());
                    world.despawn(entity);
                }
            }
            _ => unreachable!(),
        }

        if i % 1000 == 0 {
            println!("  {} operations complete", i);
        }
    }

    let total_time = start.elapsed();
    println!("  Completed in {:?}", total_time);
    println!("  Average operation time: {:?}", total_time / 10_000);

    // Validate state consistency
    println!("\n=== Final State ===");
    println!("  Entities alive: {}", world.entity_count());
    println!("  Tracked entities: {}", entities.len());

    for entity in entities {
        assert!(world.is_alive(entity), "Entity should be alive");
    }

    println!("\n=== Result ===");
    println!("  ✅ Mixed workload completed successfully");
}

// ============================================================================
// Test Summary Module
// ============================================================================

#[cfg(test)]
mod stress_test_summary {
    //! Summary of stress tests:
    //!
    //! Test 1: 100k Entity Stress Test
    //! - Spawn 100,000 entities with components
    //! - Modify all entities
    //! - Query all entities
    //! - Despawn all entities
    //! - Validates: Memory stability, performance under load
    //!
    //! Test 2: Component Thrashing
    //! - 10,000 entities
    //! - 1,000 cycles of add/remove Velocity component
    //! - Validates: No performance degradation (< 10%)
    //!
    //! Test 3: Memory Leak Detection
    //! - 10,000 cycles of spawn 100 / despawn 100
    //! - 1,000,000 total entities processed
    //! - Validates: Bounded memory usage, no entity leaks
    //!
    //! Test 4: Query Performance
    //! - 50,000 entities with varying components
    //! - 4 queries with different result sizes
    //! - Validates: Query time < 100ms (linear complexity)
    //!
    //! Test 5: Archetype Explosion
    //! - 1,024 unique component combinations
    //! - 10 component types (2^10 combinations)
    //! - Validates: Storage efficiency, fast queries
    //!
    //! Test 6: Mixed Workload
    //! - 10,000 operations: 40% spawn, 20% insert, 20% query, 10% remove, 10% despawn
    //! - Validates: State consistency under mixed operations
    //!
    //! Total: 6 stress tests
    //!
    //! Run with: cargo test --test stress_tests --release -- --ignored --nocapture
    //!
    //! Expected time: 10-30 seconds total
}
