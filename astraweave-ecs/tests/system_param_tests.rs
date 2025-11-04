//! Comprehensive tests for system parameter types (Query, Query2, Query2Mut)
//!
//! **Test Coverage**: system_param.rs (21/74 = 28.38% → target 80%+)
//!
//! **Test Categories**:
//! 1. Query<T> - Single-component read-only queries (8 tests)
//! 2. Query2<A, B> - Two-component read-only queries (7 tests)
//! 3. Query2Mut<A, B> - Two-component mutable queries (7 tests)
//! 4. Archetype iteration edge cases (5 tests)
//!
//! **Total**: 27 tests targeting +52pp coverage gain

use astraweave_ecs::{Component, Query, Query2, Query2Mut, World};

// Test components
#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Health {
    hp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Tag;

// ============================================================================
// Query<T> Tests - Single-component read-only queries
// ============================================================================

#[test]
fn test_query_single_component_empty_world() {
    let world = World::new();
    let query = Query::<Position>::new(&world);

    let results: Vec<_> = query.collect();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_query_single_component_one_entity() {
    let mut world = World::new();
    let entity = world.spawn();
    world.insert(entity, Position { x: 10.0, y: 20.0 });

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, entity);
    assert_eq!(results[0].1, &Position { x: 10.0, y: 20.0 });
}

#[test]
fn test_query_single_component_multiple_entities() {
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    let e3 = world.spawn();

    world.insert(e1, Position { x: 1.0, y: 2.0 });
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e3, Position { x: 5.0, y: 6.0 });

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    assert_eq!(results.len(), 3);

    // Verify all entities are present (order may vary by archetype)
    let entities: Vec<_> = results.iter().map(|(e, _)| *e).collect();
    assert!(entities.contains(&e1));
    assert!(entities.contains(&e2));
    assert!(entities.contains(&e3));
}

#[test]
fn test_query_single_component_filtered_entities() {
    // Create entities with different component combinations
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    let e3 = world.spawn();

    world.insert(e1, Position { x: 1.0, y: 2.0 });
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    // e3 has no Position

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    // Only e1 and e2 should be returned
    assert_eq!(results.len(), 2);
    let entities: Vec<_> = results.iter().map(|(e, _)| *e).collect();
    assert!(entities.contains(&e1));
    assert!(entities.contains(&e2));
    assert!(!entities.contains(&e3));
}

#[test]
fn test_query_single_component_different_archetypes() {
    // Create entities in different archetypes (different component combinations)
    let mut world = World::new();

    // Archetype 1: Position only
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });

    // Archetype 2: Position + Velocity
    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 0.5, y: 0.5 });

    // Archetype 3: Position + Health
    let e3 = world.spawn();
    world.insert(e3, Position { x: 5.0, y: 6.0 });
    world.insert(e3, Health { hp: 100 });

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    // All three entities should be returned (all have Position)
    assert_eq!(results.len(), 3);
    let entities: Vec<_> = results.iter().map(|(e, _)| *e).collect();
    assert!(entities.contains(&e1));
    assert!(entities.contains(&e2));
    assert!(entities.contains(&e3));
}

#[test]
fn test_query_single_component_immutability() {
    let mut world = World::new();
    let entity = world.spawn();
    world.insert(entity, Position { x: 10.0, y: 20.0 });

    let query = Query::<Position>::new(&world);

    // Iterate and verify positions are immutable
    for (_e, pos) in query {
        assert_eq!(pos.x, 10.0);
        assert_eq!(pos.y, 20.0);
        // Cannot mutate: pos.x = 15.0; // Would fail to compile
    }

    // Verify original data unchanged
    assert_eq!(
        world.get::<Position>(entity),
        Some(&Position { x: 10.0, y: 20.0 })
    );
}

#[test]
fn test_query_single_component_iteration_order_stable() {
    let mut world = World::new();
    let e1 = world.spawn();
    let e2 = world.spawn();
    let e3 = world.spawn();

    world.insert(e1, Position { x: 1.0, y: 2.0 });
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e3, Position { x: 5.0, y: 6.0 });

    // Run query twice and verify order is consistent
    let query1 = Query::<Position>::new(&world);
    let results1: Vec<_> = query1.collect();

    let query2 = Query::<Position>::new(&world);
    let results2: Vec<_> = query2.collect();

    assert_eq!(results1.len(), results2.len());
    for (r1, r2) in results1.iter().zip(results2.iter()) {
        assert_eq!(r1.0, r2.0); // Same entity order
        assert_eq!(r1.1, r2.1); // Same component values
    }
}

#[test]
fn test_query_single_component_large_count() {
    let mut world = World::new();
    let count = 1000;

    for i in 0..count {
        let entity = world.spawn();
        world.insert(
            entity,
            Position {
                x: i as f32,
                y: (i * 2) as f32,
            },
        );
    }

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    assert_eq!(results.len(), count);
}

// ============================================================================
// Query2<A, B> Tests - Two-component read-only queries
// ============================================================================

#[test]
fn test_query2_empty_world() {
    let world = World::new();
    let query = Query2::<Position, Velocity>::new(&world);

    let results: Vec<_> = query.collect();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_query2_one_entity_both_components() {
    let mut world = World::new();
    let entity = world.spawn();
    world.insert(entity, Position { x: 10.0, y: 20.0 });
    world.insert(entity, Velocity { x: 1.0, y: 2.0 });

    let query = Query2::<Position, Velocity>::new(&world);
    let results: Vec<_> = query.collect();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, entity);
    assert_eq!(results[0].1, &Position { x: 10.0, y: 20.0 });
    assert_eq!(results[0].2, &Velocity { x: 1.0, y: 2.0 });
}

#[test]
fn test_query2_filtered_by_second_component() {
    let mut world = World::new();

    // Entity with only Position (should NOT match)
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });

    // Entity with Position + Velocity (should match)
    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 0.5, y: 0.5 });

    // Entity with only Velocity (should NOT match)
    let e3 = world.spawn();
    world.insert(e3, Velocity { x: 1.0, y: 1.0 });

    let query = Query2::<Position, Velocity>::new(&world);
    let results: Vec<_> = query.collect();

    // Only e2 should match
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].0, e2);
}

#[test]
fn test_query2_multiple_entities_both_components() {
    let mut world = World::new();

    for i in 0..5 {
        let entity = world.spawn();
        world.insert(
            entity,
            Position {
                x: i as f32,
                y: (i * 2) as f32,
            },
        );
        world.insert(entity, Velocity { x: 0.1, y: 0.2 });
    }

    let query = Query2::<Position, Velocity>::new(&world);
    let results: Vec<_> = query.collect();

    assert_eq!(results.len(), 5);
}

#[test]
fn test_query2_different_archetypes() {
    let mut world = World::new();

    // Archetype 1: Position + Velocity
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });
    world.insert(e1, Velocity { x: 0.5, y: 0.5 });

    // Archetype 2: Position + Velocity + Health
    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 1.0, y: 1.0 });
    world.insert(e2, Health { hp: 100 });

    let query = Query2::<Position, Velocity>::new(&world);
    let results: Vec<_> = query.collect();

    // Both entities should match (both have Position + Velocity)
    assert_eq!(results.len(), 2);
    let entities: Vec<_> = results.iter().map(|(e, _, _)| *e).collect();
    assert!(entities.contains(&e1));
    assert!(entities.contains(&e2));
}

#[test]
fn test_query2_immutability() {
    let mut world = World::new();
    let entity = world.spawn();
    world.insert(entity, Position { x: 10.0, y: 20.0 });
    world.insert(entity, Velocity { x: 1.0, y: 2.0 });

    let query = Query2::<Position, Velocity>::new(&world);

    for (_e, pos, vel) in query {
        assert_eq!(pos.x, 10.0);
        assert_eq!(vel.x, 1.0);
        // Cannot mutate: pos.x = 15.0; // Would fail to compile
        // Cannot mutate: vel.x = 3.0;  // Would fail to compile
    }

    // Verify original data unchanged
    assert_eq!(
        world.get::<Position>(entity),
        Some(&Position { x: 10.0, y: 20.0 })
    );
    assert_eq!(
        world.get::<Velocity>(entity),
        Some(&Velocity { x: 1.0, y: 2.0 })
    );
}

#[test]
fn test_query2_large_count() {
    let mut world = World::new();
    let count = 500;

    for i in 0..count {
        let entity = world.spawn();
        world.insert(
            entity,
            Position {
                x: i as f32,
                y: (i * 2) as f32,
            },
        );
        world.insert(entity, Velocity { x: 0.1, y: 0.2 });
    }

    let query = Query2::<Position, Velocity>::new(&world);
    let results: Vec<_> = query.collect();

    assert_eq!(results.len(), count);
}

// ============================================================================
// Query2Mut<A, B> Tests - Two-component mutable queries
// ============================================================================

#[test]
fn test_query2mut_empty_world() {
    let mut world = World::new();
    let query = Query2Mut::<Position, Velocity>::new(&mut world);

    let results: Vec<_> = query.collect();
    assert_eq!(results.len(), 0);
}

#[test]
fn test_query2mut_one_entity_mutation() {
    let mut world = World::new();
    let entity = world.spawn();
    world.insert(entity, Position { x: 10.0, y: 20.0 });
    world.insert(entity, Velocity { x: 1.0, y: 2.0 });

    // Mutate position using velocity
    {
        let query = Query2Mut::<Position, Velocity>::new(&mut world);

        for (_e, pos, vel) in query {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }

    // Verify mutation applied
    let pos = world.get::<Position>(entity).unwrap();
    assert_eq!(pos.x, 11.0);
    assert_eq!(pos.y, 22.0);
}

#[test]
fn test_query2mut_multiple_entities_mutation() {
    let mut world = World::new();
    let mut entities = Vec::new();

    for i in 0..5 {
        let entity = world.spawn();
        world.insert(
            entity,
            Position {
                x: i as f32,
                y: (i * 2) as f32,
            },
        );
        world.insert(entity, Velocity { x: 1.0, y: 1.0 });
        entities.push(entity);
    }

    // Apply velocity to all positions
    {
        let query = Query2Mut::<Position, Velocity>::new(&mut world);

        for (_e, pos, vel) in query {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }

    // Verify all positions updated
    for (i, &entity) in entities.iter().enumerate() {
        let pos = world.get::<Position>(entity).unwrap();
        assert_eq!(pos.x, i as f32 + 1.0);
        assert_eq!(pos.y, (i * 2) as f32 + 1.0);
    }
}

#[test]
fn test_query2mut_filtered_by_second_component() {
    let mut world = World::new();

    // Entity with only Position (should NOT match)
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });

    // Entity with Position + Velocity (should match)
    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 0.5, y: 0.5 });

    // Mutate
    {
        let query = Query2Mut::<Position, Velocity>::new(&mut world);

        for (_e, pos, vel) in query {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }

    // e1 should be unchanged (no Velocity)
    assert_eq!(
        world.get::<Position>(e1),
        Some(&Position { x: 1.0, y: 2.0 })
    );

    // e2 should be updated
    assert_eq!(
        world.get::<Position>(e2),
        Some(&Position { x: 3.5, y: 4.5 })
    );
}

#[test]
fn test_query2mut_different_archetypes() {
    let mut world = World::new();

    // Archetype 1: Position + Velocity
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });
    world.insert(e1, Velocity { x: 0.5, y: 0.5 });

    // Archetype 2: Position + Velocity + Health
    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 1.0, y: 1.0 });
    world.insert(e2, Health { hp: 100 });

    // Mutate
    {
        let query = Query2Mut::<Position, Velocity>::new(&mut world);

        for (_e, pos, vel) in query {
            pos.x += vel.x;
            pos.y += vel.y;
        }
    }

    // Both should be updated
    assert_eq!(
        world.get::<Position>(e1),
        Some(&Position { x: 1.5, y: 2.5 })
    );
    assert_eq!(
        world.get::<Position>(e2),
        Some(&Position { x: 4.0, y: 5.0 })
    );
}

#[test]
fn test_query2mut_second_component_immutable() {
    let mut world = World::new();
    let entity = world.spawn();
    world.insert(entity, Position { x: 10.0, y: 20.0 });
    world.insert(entity, Velocity { x: 1.0, y: 2.0 });

    {
        let query = Query2Mut::<Position, Velocity>::new(&mut world);

        for (_e, pos, vel) in query {
            // Can mutate first component (A)
            pos.x += 5.0;

            // Cannot mutate second component (B) - it's immutable
            // vel.x += 1.0; // Would fail to compile

            // But can read it
            assert_eq!(vel.x, 1.0);
        }
    }

    // Verify only Position changed
    assert_eq!(
        world.get::<Position>(entity),
        Some(&Position { x: 15.0, y: 20.0 })
    );
    assert_eq!(
        world.get::<Velocity>(entity),
        Some(&Velocity { x: 1.0, y: 2.0 })
    );
}

#[test]
fn test_query2mut_large_count() {
    let mut world = World::new();
    let count = 1000;

    for i in 0..count {
        let entity = world.spawn();
        world.insert(
            entity,
            Position {
                x: i as f32,
                y: (i * 2) as f32,
            },
        );
        world.insert(entity, Velocity { x: 1.0, y: 1.0 });
    }

    // Apply velocity to all positions
    {
        let query = Query2Mut::<Position, Velocity>::new(&mut world);
        let mut update_count = 0;

        for (_e, pos, vel) in query {
            pos.x += vel.x;
            pos.y += vel.y;
            update_count += 1;
        }

        assert_eq!(update_count, count);
    }
}

// ============================================================================
// Archetype Iteration Edge Cases
// ============================================================================

#[test]
fn test_query_empty_archetype_iteration() {
    // Test that queries handle empty archetypes gracefully
    let mut world = World::new();

    // Create entity with Position, then remove it (leaves empty archetype)
    let entity = world.spawn();
    world.insert(entity, Position { x: 1.0, y: 2.0 });
    world.remove::<Position>(entity);

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    assert_eq!(results.len(), 0);
}

#[test]
fn test_query_archetype_idx_wraparound() {
    // Test that archetype_idx advances correctly across multiple archetypes
    let mut world = World::new();

    // Create 3 archetypes with different component combinations
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });

    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 0.5, y: 0.5 });

    let e3 = world.spawn();
    world.insert(e3, Position { x: 5.0, y: 6.0 });
    world.insert(e3, Health { hp: 100 });

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    // Should iterate through all 3 archetypes
    assert_eq!(results.len(), 3);
}

#[test]
fn test_query_entity_idx_reset_between_archetypes() {
    // Test that entity_idx resets to 0 when moving to next archetype
    let mut world = World::new();

    // Archetype 1: 2 entities with Position only
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });

    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });

    // Archetype 2: 2 entities with Position + Velocity
    let e3 = world.spawn();
    world.insert(e3, Position { x: 5.0, y: 6.0 });
    world.insert(e3, Velocity { x: 0.5, y: 0.5 });

    let e4 = world.spawn();
    world.insert(e4, Position { x: 7.0, y: 8.0 });
    world.insert(e4, Velocity { x: 1.0, y: 1.0 });

    let query = Query::<Position>::new(&world);
    let results: Vec<_> = query.collect();

    // All 4 entities should be returned
    assert_eq!(results.len(), 4);
}

#[test]
fn test_query2_archetype_filtering() {
    // Test that Query2 only returns entities from archetypes with both components
    let mut world = World::new();

    // Archetype 1: Position only (should NOT match)
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });

    // Archetype 2: Position + Velocity (should match)
    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 0.5, y: 0.5 });

    // Archetype 3: Position + Health (should NOT match)
    let e3 = world.spawn();
    world.insert(e3, Position { x: 5.0, y: 6.0 });
    world.insert(e3, Health { hp: 100 });

    // Archetype 4: Position + Velocity + Health (should match)
    let e4 = world.spawn();
    world.insert(e4, Position { x: 7.0, y: 8.0 });
    world.insert(e4, Velocity { x: 1.0, y: 1.0 });
    world.insert(e4, Health { hp: 50 });

    let query = Query2::<Position, Velocity>::new(&world);
    let results: Vec<_> = query.collect();

    // Only e2 and e4 should match (both have Position + Velocity)
    assert_eq!(results.len(), 2);
    let entities: Vec<_> = results.iter().map(|(e, _, _)| *e).collect();
    assert!(entities.contains(&e2));
    assert!(entities.contains(&e4));
    assert!(!entities.contains(&e1));
    assert!(!entities.contains(&e3));
}

#[test]
fn test_query2mut_archetype_filtering() {
    // Test that Query2Mut only returns entities from archetypes with both components
    let mut world = World::new();

    // Archetype 1: Position only (should NOT match)
    let e1 = world.spawn();
    world.insert(e1, Position { x: 1.0, y: 2.0 });

    // Archetype 2: Position + Velocity (should match)
    let e2 = world.spawn();
    world.insert(e2, Position { x: 3.0, y: 4.0 });
    world.insert(e2, Velocity { x: 0.5, y: 0.5 });

    {
        let query = Query2Mut::<Position, Velocity>::new(&mut world);

        for (_e, pos, vel) in query {
            pos.x += vel.x * 2.0;
        }
    }

    // e1 should be unchanged (no Velocity)
    assert_eq!(
        world.get::<Position>(e1),
        Some(&Position { x: 1.0, y: 2.0 })
    );

    // e2 should be updated
    assert_eq!(
        world.get::<Position>(e2),
        Some(&Position { x: 4.0, y: 4.0 })
    );
}
