//! Property-based tests for ECS using proptest
//!
//! These tests validate ECS invariants under arbitrary sequences of operations.
//! Property-based testing generates random test cases and shrinks failing cases
//! to minimal examples, helping find edge cases that manual tests might miss.
//!
//! **Phase 4.1 Focus**: Demonstrate proptest integration with simplified tests
//! that match the actual World API. Phase 4.2 will expand with more comprehensive
//! test coverage once this foundation is proven.
//!
//! # Miri Compatibility
//!
//! Property tests are skipped under Miri because:
//! 1. Miri runs ~100× slower than native execution
//! 2. Proptest generates many test cases (256 by default)
//! 3. Total time: 256 × 100 = 7+ hours per property test!
//!
//! The non-property tests in other modules exercise the same unsafe code paths
//! and are validated by Miri. Property tests are validated in normal test runs.

#![cfg(not(miri))]

use crate::*;
use proptest::prelude::*;

// ============================================================================
// Test Components
// ============================================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PropPosition {
    x: i32,
    y: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PropVelocity {
    dx: i32,
    dy: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PropHealth {
    hp: u32,
}

// ============================================================================
// Property Tests: Basic Entity Operations
// ============================================================================

proptest! {
        /// Property: Entity count equals number of spawns minus number of despawns
    #[test]
    fn prop_entity_count_invariant(spawn_count in 1usize..100, despawn_ratio in 0.0f32..1.0f32) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities
        for _ in 0..spawn_count {
            let entity = world.spawn();
            entities.push(entity);
        }

        // Despawn some entities
        let despawn_count = (spawn_count as f32 * despawn_ratio) as usize;
        let mut actually_despawned = 0;
        for &entity in entities.iter().take(despawn_count) {
            if world.despawn(entity) {
                actually_despawned += 1;
            }
        }

        // Invariant: entity_count() should match expected alive count
        let expected_alive = spawn_count - actually_despawned;
        let actual_alive = world.entity_count();
        prop_assert_eq!(actual_alive, expected_alive);
    }

    /// Property: Spawned entities always have unique, valid IDs
    #[test]
    fn prop_entity_ids_unique(spawn_count in 1usize..500) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities
        for _ in 0..spawn_count {
            let entity = world.spawn();
            entities.push(entity);
        }

        // All entities should have unique IDs
        for (i, &entity1) in entities.iter().enumerate() {
            for (j, &entity2) in entities.iter().enumerate() {
                if i != j {
                    prop_assert_ne!(entity1.id(), entity2.id(),
                        "Entities at indices {} and {} have duplicate IDs: {:?} vs {:?}",
                        i, j, entity1, entity2);
                }
            }
        }
    }

    /// Property: Component insertion is idempotent
    #[test]
    fn prop_component_insertion_idempotent(
        spawn_count in 1usize..50,
        insert_count in 1usize..10,
        x in any::<i32>(),
        y in any::<i32>()
    ) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities
        for _ in 0..spawn_count {
            entities.push(world.spawn());
        }

        // Insert same component multiple times
        for &entity in &entities {
            for _ in 0..insert_count {
                world.insert(entity, PropPosition { x, y });
            }
        }

        // Component should only exist once with the last inserted value
        for &entity in &entities {
            let position = world.get::<PropPosition>(entity);
            prop_assert!(position.is_some(), "Entity {:?} missing PropPosition after {} inserts", entity, insert_count);
            prop_assert_eq!(position.copied(), Some(PropPosition { x, y }));
        }

        // Count should equal number of entities (not inserts)
        prop_assert_eq!(world.count::<PropPosition>(), spawn_count);
    }

    /// Property: Despawned entities are no longer alive
    #[test]
    fn prop_despawned_entities_invalid(spawn_count in 1usize..100) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities with Position component
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            entities.push(entity);
        }

        // Despawn half of the entities
        let mut despawned = Vec::new();
        for &entity in entities.iter().take(spawn_count / 2) {
            if world.despawn(entity) {
                despawned.push(entity);
            }
        }

        // Component count should reflect only alive entities
        let expected_count = spawn_count - despawned.len();
        let actual_count = world.count::<PropPosition>();
        prop_assert_eq!(actual_count, expected_count);

        // Despawned entities should not be alive
        for entity in despawned {
            prop_assert!(!world.is_alive(entity), "Entity {:?} should not be alive after despawn", entity);
            prop_assert!(world.get::<PropPosition>(entity).is_none(), "Entity {:?} should not have PropPosition after despawn", entity);
        }
    }

    /// Property: Component removal does not affect other entities
    #[test]
    fn prop_component_removal_isolation(spawn_count in 2usize..50) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities with Position and Velocity
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            world.insert(entity, PropVelocity { dx: i as i32 * 2, dy: i as i32 * 3 });
            entities.push((entity, i));
        }

        // Remove Position from first entity
        let (first_entity, first_i) = entities[0];
        world.remove::<PropPosition>(first_entity);

        // First entity should have Velocity but not Position
        prop_assert!(world.get::<PropPosition>(first_entity).is_none());
        prop_assert_eq!(world.get::<PropVelocity>(first_entity).copied(),
            Some(PropVelocity { dx: first_i as i32 * 2, dy: first_i as i32 * 3 }));

        // Other entities should still have both components
        for &(entity, i) in entities.iter().skip(1) {
            prop_assert_eq!(world.get::<PropPosition>(entity).copied(),
                Some(PropPosition { x: i as i32, y: i as i32 }));
            prop_assert_eq!(world.get::<PropVelocity>(entity).copied(),
                Some(PropVelocity { dx: i as i32 * 2, dy: i as i32 * 3 }));
        }
    }

    /// Property: Large entity counts don't cause panics
    #[test]
    fn prop_large_entity_count_stable(spawn_count in 1000usize..5000) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn large number of entities
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            entities.push(entity);
        }

        // Verify count
        prop_assert_eq!(world.count::<PropPosition>(), spawn_count);
        prop_assert_eq!(world.entity_count(), spawn_count);

        // Despawn all entities
        for entity in entities {
            prop_assert!(world.despawn(entity), "Despawn should succeed for valid entity {:?}", entity);
        }

        // World should be empty
        prop_assert_eq!(world.count::<PropPosition>(), 0);
        prop_assert_eq!(world.entity_count(), 0);
    }

    /// Property: Interleaved spawn/despawn maintains consistency
    #[test]
    fn prop_interleaved_spawn_despawn(ops in prop::collection::vec(any::<bool>(), 0..200)) {
        let mut world = World::new();
        let mut entities = Vec::new();
        let mut expected_count = 0;

        for spawn in ops {
            if spawn {
                // Spawn entity
                let entity = world.spawn();
                world.insert(entity, PropPosition { x: 0, y: 0 });
                entities.push(entity);
                expected_count += 1;
            } else if !entities.is_empty() {
                // Despawn random entity
                let idx = entities.len() / 2;
                let entity = entities.swap_remove(idx);
                if world.despawn(entity) {
                    expected_count -= 1;
                }
            }
        }

        // Verify entity count
        prop_assert_eq!(world.entity_count(), expected_count);
        prop_assert_eq!(world.count::<PropPosition>(), expected_count);
    }

    /// Property: Entity is_alive check is consistent with operations
    #[test]
    fn prop_is_alive_consistent(spawn_count in 1usize..100) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities
        for _ in 0..spawn_count {
            entities.push(world.spawn());
        }

        // All spawned entities should be alive
        for &entity in &entities {
            prop_assert!(world.is_alive(entity), "Spawned entity {:?} should be alive", entity);
        }

        // Despawn half
        for &entity in entities.iter().take(spawn_count / 2) {
            world.despawn(entity);
        }

        // First half should be dead, second half alive
        for (i, &entity) in entities.iter().enumerate() {
            if i < spawn_count / 2 {
                prop_assert!(!world.is_alive(entity), "Despawned entity {:?} should not be alive", entity);
            } else {
                prop_assert!(world.is_alive(entity), "Entity {:?} should still be alive", entity);
            }
        }
    }

    /// Property: Component has() check is consistent with get()
    #[test]
    fn prop_has_consistent_with_get(spawn_count in 1usize..100) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities with Position
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            entities.push(entity);
        }

        // has() should match get().is_some()
        for &entity in &entities {
            let has_position = world.has::<PropPosition>(entity);
            let get_position = world.get::<PropPosition>(entity).is_some();
            prop_assert_eq!(has_position, get_position,
                "has() and get().is_some() should agree for entity {:?}", entity);
        }
    }

    /// Property: entities_with() returns correct entities
    #[test]
    fn prop_entities_with_accurate(
        pos_only in 1usize..30,
        vel_only in 1usize..30,
        both in 1usize..30,
    ) {
        let mut world = World::new();

        // Spawn entities with Position only
        for i in 0..pos_only {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
        }

        // Spawn entities with Velocity only
        for i in 0..vel_only {
            let entity = world.spawn();
            world.insert(entity, PropVelocity { dx: i as i32, dy: i as i32 });
        }

        // Spawn entities with both
        for i in 0..both {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            world.insert(entity, PropVelocity { dx: i as i32, dy: i as i32 });
        }

        // entities_with::<Position>() should return pos_only + both
        let pos_entities = world.entities_with::<PropPosition>();
        prop_assert_eq!(pos_entities.len(), pos_only + both);

        // entities_with::<Velocity>() should return vel_only + both
        let vel_entities = world.entities_with::<PropVelocity>();
        prop_assert_eq!(vel_entities.len(), vel_only + both);

        // Verify all returned entities actually have the component
        for entity in pos_entities {
            prop_assert!(world.has::<PropPosition>(entity),
                "Entity {:?} from entities_with::<Position>() should have Position", entity);
        }

        for entity in vel_entities {
            prop_assert!(world.has::<PropVelocity>(entity),
                "Entity {:?} from entities_with::<Velocity>() should have Velocity", entity);
        }
    }
}

// ============================================================================
// Property Tests: Archetype Transitions
// ============================================================================

proptest! {
        /// Property: Adding a component triggers archetype migration
    #[test]
    fn prop_archetype_migration_on_add(spawn_count in 1usize..50) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities with Position only
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            entities.push(entity);
        }

        // Add Velocity to all entities (triggers archetype migration)
        for (i, &entity) in entities.iter().enumerate() {
            world.insert(entity, PropVelocity { dx: i as i32, dy: i as i32 });
        }

        // All entities should still be alive
        for &entity in &entities {
            prop_assert!(world.is_alive(entity), "Entity {:?} should still be alive after archetype migration", entity);
        }

        // All entities should have both components
        for (i, &entity) in entities.iter().enumerate() {
            let pos = world.get::<PropPosition>(entity);
            let vel = world.get::<PropVelocity>(entity);
            prop_assert!(pos.is_some(), "Entity {:?} missing Position after migration", entity);
            prop_assert!(vel.is_some(), "Entity {:?} missing Velocity after migration", entity);
            prop_assert_eq!(pos.copied(), Some(PropPosition { x: i as i32, y: i as i32 }));
            prop_assert_eq!(vel.copied(), Some(PropVelocity { dx: i as i32, dy: i as i32 }));
        }

        // Component counts should match
        prop_assert_eq!(world.count::<PropPosition>(), spawn_count);
        prop_assert_eq!(world.count::<PropVelocity>(), spawn_count);
    }

    /// Property: Removing a component triggers archetype migration
    #[test]
    fn prop_archetype_migration_on_remove(spawn_count in 2usize..50) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities with both Position and Velocity
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            world.insert(entity, PropVelocity { dx: i as i32, dy: i as i32 });
            entities.push(entity);
        }

        // Remove Velocity from all entities (triggers archetype migration)
        for &entity in &entities {
            world.remove::<PropVelocity>(entity);
        }

        // All entities should still be alive
        for &entity in &entities {
            prop_assert!(world.is_alive(entity), "Entity {:?} should still be alive after archetype migration", entity);
        }

        // All entities should have Position but not Velocity
        for (i, &entity) in entities.iter().enumerate() {
            let pos = world.get::<PropPosition>(entity);
            let vel = world.get::<PropVelocity>(entity);
            prop_assert!(pos.is_some(), "Entity {:?} missing Position after migration", entity);
            prop_assert!(vel.is_none(), "Entity {:?} should not have Velocity after removal", entity);
            prop_assert_eq!(pos.copied(), Some(PropPosition { x: i as i32, y: i as i32 }));
        }

        // Component counts should match
        prop_assert_eq!(world.count::<PropPosition>(), spawn_count);
        prop_assert_eq!(world.count::<PropVelocity>(), 0);
    }

    /// Property: Component data preserved during archetype transitions
    #[test]
    fn prop_component_data_preserved_during_transition(
        x in any::<i32>(),
        y in any::<i32>(),
        dx in any::<i32>(),
        dy in any::<i32>(),
        hp in 1u32..1000,
    ) {
        let mut world = World::new();
        let entity = world.spawn();

        // Add Position
        world.insert(entity, PropPosition { x, y });
        prop_assert_eq!(world.get::<PropPosition>(entity).copied(), Some(PropPosition { x, y }));

        // Add Velocity (archetype transition)
        world.insert(entity, PropVelocity { dx, dy });
        prop_assert_eq!(world.get::<PropPosition>(entity).copied(), Some(PropPosition { x, y }));
        prop_assert_eq!(world.get::<PropVelocity>(entity).copied(), Some(PropVelocity { dx, dy }));

        // Add Health (another archetype transition)
        world.insert(entity, PropHealth { hp });
        prop_assert_eq!(world.get::<PropPosition>(entity).copied(), Some(PropPosition { x, y }));
        prop_assert_eq!(world.get::<PropVelocity>(entity).copied(), Some(PropVelocity { dx, dy }));
        prop_assert_eq!(world.get::<PropHealth>(entity).copied(), Some(PropHealth { hp }));

        // Remove Velocity (archetype transition)
        world.remove::<PropVelocity>(entity);
        prop_assert_eq!(world.get::<PropPosition>(entity).copied(), Some(PropPosition { x, y }));
        prop_assert!(world.get::<PropVelocity>(entity).is_none());
        prop_assert_eq!(world.get::<PropHealth>(entity).copied(), Some(PropHealth { hp }));
    }

    /// Property: Multiple archetype transitions don't corrupt entity state
    #[test]
    fn prop_multiple_transitions_stable(transition_count in 1usize..20) {
        let mut world = World::new();
        let entity = world.spawn();
        let value = 42;

        world.insert(entity, PropPosition { x: value, y: value });

        // Perform multiple add/remove cycles
        for _ in 0..transition_count {
            world.insert(entity, PropVelocity { dx: value, dy: value });
            prop_assert!(world.is_alive(entity));
            prop_assert_eq!(world.get::<PropPosition>(entity).map(|p| p.x), Some(value));

            world.remove::<PropVelocity>(entity);
            prop_assert!(world.is_alive(entity));
            prop_assert_eq!(world.get::<PropPosition>(entity).map(|p| p.x), Some(value));
        }

        // Final state should be consistent
        prop_assert!(world.is_alive(entity));
        prop_assert_eq!(world.get::<PropPosition>(entity).copied(), Some(PropPosition { x: value, y: value }));
        prop_assert!(world.get::<PropVelocity>(entity).is_none());
    }
}

// ============================================================================
// Property Tests: Multi-Component Operations
// ============================================================================

proptest! {
        /// Property: Adding multiple components in sequence preserves all data
    #[test]
    fn prop_multi_component_add_preserves_data(
        pos_x in any::<i32>(),
        pos_y in any::<i32>(),
        vel_dx in any::<i32>(),
        vel_dy in any::<i32>(),
        hp in 1u32..1000,
    ) {
        let mut world = World::new();
        let entity = world.spawn();

        // Add components one by one
        world.insert(entity, PropPosition { x: pos_x, y: pos_y });
        world.insert(entity, PropVelocity { dx: vel_dx, dy: vel_dy });
        world.insert(entity, PropHealth { hp });

        // All components should be present
        prop_assert_eq!(world.get::<PropPosition>(entity).copied(), Some(PropPosition { x: pos_x, y: pos_y }));
        prop_assert_eq!(world.get::<PropVelocity>(entity).copied(), Some(PropVelocity { dx: vel_dx, dy: vel_dy }));
        prop_assert_eq!(world.get::<PropHealth>(entity).copied(), Some(PropHealth { hp }));
    }

    /// Property: Removing one component doesn't affect others
    #[test]
    fn prop_remove_one_preserves_others(
        spawn_count in 1usize..30,
    ) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities with all three components
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            world.insert(entity, PropVelocity { dx: i as i32 * 2, dy: i as i32 * 3 });
            world.insert(entity, PropHealth { hp: (i as u32 + 1) * 10 });
            entities.push(entity);
        }

        // Remove Velocity from all entities
        for &entity in &entities {
            world.remove::<PropVelocity>(entity);
        }

        // Position and Health should still be present
        for (i, &entity) in entities.iter().enumerate() {
            prop_assert_eq!(world.get::<PropPosition>(entity).copied(), Some(PropPosition { x: i as i32, y: i as i32 }));
            prop_assert!(world.get::<PropVelocity>(entity).is_none());
            prop_assert_eq!(world.get::<PropHealth>(entity).copied(), Some(PropHealth { hp: (i as u32 + 1) * 10 }));
        }
    }

    /// Property: Component combinations create correct archetypes
    #[test]
    fn prop_component_combinations_distinct(spawn_count in 1usize..20) {
        let mut world = World::new();

        // Create entities with different component combinations
        let mut pos_only = Vec::new();
        let mut vel_only = Vec::new();
        let mut pos_vel = Vec::new();
        let mut all_three = Vec::new();

        for i in 0..spawn_count {
            // Position only
            let e1 = world.spawn();
            world.insert(e1, PropPosition { x: i as i32, y: i as i32 });
            pos_only.push(e1);

            // Velocity only
            let e2 = world.spawn();
            world.insert(e2, PropVelocity { dx: i as i32, dy: i as i32 });
            vel_only.push(e2);

            // Position + Velocity
            let e3 = world.spawn();
            world.insert(e3, PropPosition { x: i as i32, y: i as i32 });
            world.insert(e3, PropVelocity { dx: i as i32, dy: i as i32 });
            pos_vel.push(e3);

            // All three
            let e4 = world.spawn();
            world.insert(e4, PropPosition { x: i as i32, y: i as i32 });
            world.insert(e4, PropVelocity { dx: i as i32, dy: i as i32 });
            world.insert(e4, PropHealth { hp: (i as u32 + 1) * 10 });
            all_three.push(e4);
        }

        // Verify component counts
        prop_assert_eq!(world.count::<PropPosition>(), spawn_count * 3); // pos_only + pos_vel + all_three
        prop_assert_eq!(world.count::<PropVelocity>(), spawn_count * 3); // vel_only + pos_vel + all_three
        prop_assert_eq!(world.count::<PropHealth>(), spawn_count); // all_three only

        // Verify each entity has correct components
        for &e in &pos_only {
            prop_assert!(world.has::<PropPosition>(e));
            prop_assert!(!world.has::<PropVelocity>(e));
            prop_assert!(!world.has::<PropHealth>(e));
        }

        for &e in &vel_only {
            prop_assert!(!world.has::<PropPosition>(e));
            prop_assert!(world.has::<PropVelocity>(e));
            prop_assert!(!world.has::<PropHealth>(e));
        }

        for &e in &pos_vel {
            prop_assert!(world.has::<PropPosition>(e));
            prop_assert!(world.has::<PropVelocity>(e));
            prop_assert!(!world.has::<PropHealth>(e));
        }

        for &e in &all_three {
            prop_assert!(world.has::<PropPosition>(e));
            prop_assert!(world.has::<PropVelocity>(e));
            prop_assert!(world.has::<PropHealth>(e));
        }
    }
}

// ============================================================================
// Property Tests: Edge Cases
// ============================================================================

proptest! {
        /// Property: Operations on NULL entity fail gracefully
    #[test]
    fn prop_null_entity_operations_safe(_dummy in 0..100u32) {
        let mut world = World::new();
        let null = Entity::null();

        // All operations on NULL should be safe (no panic)
        prop_assert!(!world.is_alive(null));
        prop_assert!(!world.has::<PropPosition>(null));
        prop_assert!(world.get::<PropPosition>(null).is_none());
        prop_assert!(!world.despawn(null));
        prop_assert!(!world.remove::<PropPosition>(null));
    }

    /// Property: Operations on despawned entities fail gracefully
    #[test]
    fn prop_stale_entity_operations_safe(spawn_count in 1usize..50) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn and despawn entities
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            entities.push(entity);
        }

        for &entity in &entities {
            world.despawn(entity);
        }

        // All operations on stale entities should be safe (no panic)
        for &entity in &entities {
            prop_assert!(!world.is_alive(entity));
            prop_assert!(!world.has::<PropPosition>(entity));
            prop_assert!(world.get::<PropPosition>(entity).is_none());
            prop_assert!(!world.despawn(entity)); // Double despawn fails
            prop_assert!(!world.remove::<PropPosition>(entity));
        }
    }

    /// Property: Empty world operations are safe
    #[test]
    fn prop_empty_world_operations_safe(_dummy in 0..100u32) {
        let world = World::new();

        // All query operations on empty world should be safe
        prop_assert_eq!(world.entity_count(), 0);
        prop_assert_eq!(world.count::<PropPosition>(), 0);
        prop_assert_eq!(world.count::<PropVelocity>(), 0);
        prop_assert_eq!(world.count::<PropHealth>(), 0);
        prop_assert_eq!(world.entities_with::<PropPosition>().len(), 0);
        prop_assert_eq!(world.entities_with::<PropVelocity>().len(), 0);
    }

    /// Property: Entity recycling maintains generation
    #[test]
    fn prop_entity_recycling_safe(cycle_count in 1usize..20) {
        let mut world = World::new();
        let mut seen_ids = std::collections::HashSet::new();

        for _ in 0..cycle_count {
            // Spawn entity
            let entity = world.spawn();
            let id = entity.id();

            // Entity ID should be unique in this cycle
            if seen_ids.contains(&id) {
                // If ID is reused, generation MUST have increased
                // (This property is hard to test directly without internal access)
                prop_assert!(world.is_alive(entity));
            }
            seen_ids.insert(id);

            world.insert(entity, PropPosition { x: 42, y: 42 });
            prop_assert!(world.is_alive(entity));

            // Despawn entity
            prop_assert!(world.despawn(entity));
            prop_assert!(!world.is_alive(entity));
        }
    }

    /// Property: Mixed valid and invalid entity operations
    #[test]
    fn prop_mixed_valid_invalid_entities(valid_count in 1usize..30, invalid_count in 1usize..30) {
        let mut world = World::new();
        let mut valid_entities = Vec::new();
        let mut invalid_entities = Vec::new();

        // Create valid entities
        for i in 0..valid_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            valid_entities.push(entity);
        }

        // Create and despawn invalid entities
        for _ in 0..invalid_count {
            let entity = world.spawn();
            world.despawn(entity);
            invalid_entities.push(entity);
        }

        // Operations on valid entities should succeed
        for &entity in &valid_entities {
            prop_assert!(world.is_alive(entity));
            prop_assert!(world.has::<PropPosition>(entity));
        }

        // Operations on invalid entities should fail gracefully
        for &entity in &invalid_entities {
            prop_assert!(!world.is_alive(entity));
            prop_assert!(!world.has::<PropPosition>(entity));
        }

        // Component count should only include valid entities
        prop_assert_eq!(world.count::<PropPosition>(), valid_count);
    }
}

// ============================================================================
// Property Tests: Query Invariants
// ============================================================================

proptest! {
        /// Property: count() consistent across operations
    #[test]
    fn prop_count_consistent_across_operations(
        spawn_count in 1usize..50,
        remove_count in 0usize..25,
    ) {
        let mut world = World::new();
        let mut entities = Vec::new();

        // Spawn entities with Position
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            entities.push(entity);
        }

        prop_assert_eq!(world.count::<PropPosition>(), spawn_count);

        // Remove Position from some entities
        let actual_remove = remove_count.min(entities.len());
        for &entity in entities.iter().take(actual_remove) {
            world.remove::<PropPosition>(entity);
        }

        prop_assert_eq!(world.count::<PropPosition>(), spawn_count - actual_remove);

        // Despawn some entities
        let despawn_count = (entities.len() / 2).min(entities.len());
        for &entity in entities.iter().take(despawn_count) {
            world.despawn(entity);
        }

        // Count should reflect only alive entities with Position
        let expected = entities.iter().skip(despawn_count).take_while(|&&e| world.is_alive(e)).filter(|&&e| world.has::<PropPosition>(e)).count();
        prop_assert_eq!(world.count::<PropPosition>(), expected);
    }

    /// Property: entities_with() returns only entities with component
    #[test]
    fn prop_entities_with_returns_correct_entities(
        with_pos in 1usize..30,
        without_pos in 1usize..30,
    ) {
        let mut world = World::new();
        let mut has_position = Vec::new();
        let mut no_position = Vec::new();

        // Entities with Position
        for i in 0..with_pos {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
            has_position.push(entity);
        }

        // Entities without Position
        for i in 0..without_pos {
            let entity = world.spawn();
            world.insert(entity, PropVelocity { dx: i as i32, dy: i as i32 });
            no_position.push(entity);
        }

        let result = world.entities_with::<PropPosition>();

        // Result should contain all entities with Position
        for &entity in &has_position {
            prop_assert!(result.contains(&entity), "entities_with should contain entity {:?}", entity);
        }

        // Result should NOT contain entities without Position
        for &entity in &no_position {
            prop_assert!(!result.contains(&entity), "entities_with should not contain entity {:?}", entity);
        }

        // Result length should match count
        prop_assert_eq!(result.len(), with_pos);
    }

    /// Property: Query results deterministic for same world state
    #[test]
    fn prop_query_deterministic(spawn_count in 1usize..50) {
        let mut world = World::new();

        // Create deterministic world state
        for i in 0..spawn_count {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
        }

        // Query multiple times
        let result1 = world.entities_with::<PropPosition>();
        let result2 = world.entities_with::<PropPosition>();
        let result3 = world.entities_with::<PropPosition>();

        // All results should be identical
        prop_assert_eq!(&result1, &result2);
        prop_assert_eq!(&result2, &result3);

        // Count should also be consistent
        let count1 = world.count::<PropPosition>();
        let count2 = world.count::<PropPosition>();
        prop_assert_eq!(count1, count2);
        prop_assert_eq!(count1, spawn_count);
    }

    /// Property: has() consistent with entities_with()
    #[test]
    fn prop_has_consistent_with_entities_with(
        with_comp in 1usize..30,
        without_comp in 1usize..30,
    ) {
        let mut world = World::new();

        // Entities with Position
        for i in 0..with_comp {
            let entity = world.spawn();
            world.insert(entity, PropPosition { x: i as i32, y: i as i32 });
        }

        // Entities without Position
        for _ in 0..without_comp {
            world.spawn();
        }

        let entities_with_pos = world.entities_with::<PropPosition>();

        // For every entity in entities_with, has() should return true
        for entity in entities_with_pos {
            prop_assert!(world.has::<PropPosition>(entity), "has() should return true for entity {:?} from entities_with()", entity);
        }
    }
}

// ============================================================================
// Configuration Tests
// ============================================================================

#[cfg(test)]
mod config_tests {
    use super::*;

    #[test]
    fn test_proptest_config() {
        // Verify proptest is configured correctly
        let config = ProptestConfig::default();
        assert!(config.cases > 0);
        println!("Proptest config: {} cases per property", config.cases);
    }

    #[test]
    fn test_proptest_basic() {
        // Simple sanity check that proptest works
        proptest!(|(x in 0..100i32, y in 0..100i32)| {
            prop_assert!((0..100).contains(&x));
            prop_assert!((0..100).contains(&y));
        });
    }

    #[test]
    fn test_components_defined() {
        // Verify test components are properly defined
        let pos = PropPosition { x: 42, y: 100 };
        let vel = PropVelocity { dx: 1, dy: 2 };
        let hp = PropHealth { hp: 100 };

        assert_eq!(pos.x, 42);
        assert_eq!(vel.dx, 1);
        assert_eq!(hp.hp, 100);
    }
}
