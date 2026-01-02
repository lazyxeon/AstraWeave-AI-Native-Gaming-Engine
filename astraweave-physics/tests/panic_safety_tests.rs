//! Panic safety tests for Physics subsystem.
//!
//! P0-Critical: Ensures physics handles edge cases gracefully without panicking.
//! Tests cover invalid body operations, zero-size colliders, edge cases, and stale handles.

use astraweave_physics::*;
use glam::Vec3;
use std::panic;

/// Helper to verify a closure doesn't panic
fn should_not_panic<F: FnOnce() + panic::UnwindSafe>(name: &str, f: F) {
    let result = panic::catch_unwind(f);
    assert!(
        result.is_ok(),
        "{} should not panic on invalid input",
        name
    );
}

// ============================================================================
// Invalid BodyId operations
// ============================================================================

#[test]
fn test_apply_force_invalid_body_id_no_panic() {
    should_not_panic("apply_force with invalid body_id", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Apply force to nonexistent body (ID that was never created)
        world.apply_force(99999, Vec3::new(100.0, 0.0, 0.0));
    });
}

#[test]
fn test_apply_impulse_invalid_body_id_no_panic() {
    should_not_panic("apply_impulse with invalid body_id", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Apply impulse to nonexistent body
        world.apply_impulse(99999, Vec3::new(100.0, 0.0, 0.0));
    });
}

#[test]
fn test_get_velocity_invalid_body_id_no_panic() {
    should_not_panic("get_velocity with invalid body_id", || {
        let world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Get velocity of nonexistent body - should return None, not panic
        let vel = world.get_velocity(99999);
        assert!(vel.is_none(), "Should return None for invalid body");
    });
}

#[test]
fn test_set_velocity_invalid_body_id_no_panic() {
    should_not_panic("set_velocity with invalid body_id", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Set velocity on nonexistent body
        world.set_velocity(99999, Vec3::new(10.0, 0.0, 0.0));
    });
}

#[test]
fn test_body_transform_invalid_body_id_no_panic() {
    should_not_panic("body_transform with invalid body_id", || {
        let world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Get transform of nonexistent body
        let transform = world.body_transform(99999);
        assert!(transform.is_none(), "Should return None for invalid body");
    });
}

#[test]
fn test_set_body_position_invalid_body_id_no_panic() {
    should_not_panic("set_body_position with invalid body_id", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Set position on nonexistent body
        world.set_body_position(99999, Vec3::new(100.0, 0.0, 0.0));
    });
}

#[test]
fn test_handle_of_invalid_body_id_no_panic() {
    should_not_panic("handle_of with invalid body_id", || {
        let world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Get handle of nonexistent body
        let handle = world.handle_of(99999);
        assert!(handle.is_none(), "Should return None for invalid body");
    });
}

#[test]
fn test_break_destructible_invalid_body_id_no_panic() {
    should_not_panic("break_destructible with invalid body_id", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Break nonexistent destructible - should fail gracefully
        world.break_destructible(99999);
    });
}

// ============================================================================
// Zero-size and edge case body creation tests
// ============================================================================

#[test]
fn test_create_ground_plane_zero_size_no_panic() {
    should_not_panic("create ground plane with zero size", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create ground plane with zero dimensions
        let result = world.create_ground_plane(Vec3::ZERO, 0.5);
        let _ = result;
    });
}

#[test]
fn test_add_dynamic_box_zero_size_no_panic() {
    should_not_panic("add dynamic box with zero half extents", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create box with zero dimensions
        let result = world.add_dynamic_box(Vec3::ZERO, Vec3::ZERO, 1.0, Layers::DEFAULT);
        let _ = result;
    });
}

#[test]
fn test_add_character_zero_size_no_panic() {
    should_not_panic("add character with zero size", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create character with zero dimensions
        let result = world.add_character(Vec3::ZERO, Vec3::ZERO);
        let _ = result;
    });
}

// ============================================================================
// Negative value tests
// ============================================================================

#[test]
fn test_add_dynamic_box_negative_half_extents_no_panic() {
    should_not_panic("add dynamic box with negative half extents", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create box with negative dimensions
        let result = world.add_dynamic_box(Vec3::ZERO, Vec3::new(-1.0, -1.0, -1.0), 1.0, Layers::DEFAULT);
        let _ = result;
    });
}

#[test]
fn test_add_character_negative_size_no_panic() {
    should_not_panic("add character with negative size", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create character with negative dimensions
        let result = world.add_character(Vec3::ZERO, Vec3::new(-1.0, -1.0, -1.0));
        let _ = result;
    });
}

#[test]
fn test_negative_mass_no_panic() {
    should_not_panic("create body with negative mass", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create box with negative mass (invalid but should not panic)
        let result = world.add_dynamic_box(Vec3::ZERO, Vec3::splat(1.0), -10.0, Layers::DEFAULT);
        let _ = result;
    });
}

#[test]
fn test_zero_mass_no_panic() {
    should_not_panic("create body with zero mass", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create box with zero mass
        let result = world.add_dynamic_box(Vec3::ZERO, Vec3::splat(1.0), 0.0, Layers::DEFAULT);
        let _ = result;
    });
}

// ============================================================================
// Double removal tests (using break_destructible)
// ============================================================================

#[test]
fn test_double_break_destructible_no_panic() {
    should_not_panic("double break_destructible", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create and break a destructible body
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        
        // First break should work
        world.break_destructible(id);
        
        // Second break should fail gracefully, not panic
        world.break_destructible(id);
    });
}

#[test]
fn test_triple_break_destructible_no_panic() {
    should_not_panic("triple break_destructible", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        
        world.break_destructible(id);
        world.break_destructible(id);
        world.break_destructible(id);
    });
}

// ============================================================================
// Operations on removed bodies (using break_destructible)
// ============================================================================

#[test]
fn test_apply_force_broken_body_no_panic() {
    should_not_panic("apply_force to broken body", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        world.break_destructible(id);
        
        // Apply force to broken body
        world.apply_force(id, Vec3::new(100.0, 0.0, 0.0));
    });
}

#[test]
fn test_apply_impulse_broken_body_no_panic() {
    should_not_panic("apply_impulse to broken body", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        world.break_destructible(id);
        
        // Apply impulse to broken body
        world.apply_impulse(id, Vec3::new(100.0, 0.0, 0.0));
    });
}

#[test]
fn test_get_velocity_broken_body_no_panic() {
    should_not_panic("get_velocity on broken body", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        world.break_destructible(id);
        
        // Get velocity of broken body
        let vel = world.get_velocity(id);
        assert!(vel.is_none(), "Should return None for broken body");
    });
}

#[test]
fn test_set_velocity_broken_body_no_panic() {
    should_not_panic("set_velocity on broken body", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        world.break_destructible(id);
        
        // Set velocity on broken body
        world.set_velocity(id, Vec3::new(10.0, 0.0, 0.0));
    });
}

#[test]
fn test_body_transform_broken_body_no_panic() {
    should_not_panic("body_transform on broken body", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        world.break_destructible(id);
        
        // Get transform of broken body
        let transform = world.body_transform(id);
        assert!(transform.is_none(), "Should return None for broken body");
    });
}

#[test]
fn test_set_body_position_broken_body_no_panic() {
    should_not_panic("set_body_position on broken body", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_destructible_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, 100.0, 50.0);
        world.break_destructible(id);
        
        // Set position on broken body
        world.set_body_position(id, Vec3::new(100.0, 0.0, 0.0));
    });
}

// ============================================================================
// Physics step with edge cases
// ============================================================================

#[test]
fn test_step_empty_world_no_panic() {
    should_not_panic("step empty world", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Step with no bodies
        for _ in 0..100 {
            world.step();
        }
    });
}

#[test]
fn test_step_after_all_bodies_removed_no_panic() {
    should_not_panic("step after all bodies removed", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create bodies
        let ids: Vec<_> = (0..10)
            .map(|i| world.add_dynamic_box(Vec3::new(i as f32, 0.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT))
            .collect();
        
        // Step once
        world.step();
        
        // Remove all bodies
        for id in ids {
            world.break_destructible(id);
        }
        
        // Step again
        for _ in 0..100 {
            world.step();
        }
    });
}

#[test]
fn test_step_with_settling_bodies_no_panic() {
    should_not_panic("step with settling bodies", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create ground and box that should settle
        world.create_ground_plane(Vec3::new(100.0, 0.1, 100.0), 0.5);
        world.add_dynamic_box(Vec3::new(0.0, 10.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        
        // Step many times to let box settle
        for _ in 0..1000 {
            world.step();
        }
    });
}

// ============================================================================
// Character controller edge cases
// ============================================================================

#[test]
fn test_control_character_invalid_id_no_panic() {
    should_not_panic("control_character with invalid id", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Control nonexistent character
        world.control_character(99999, Vec3::new(1.0, 0.0, 0.0), 0.016, false);
    });
}

#[test]
fn test_control_character_broken_id_no_panic() {
    should_not_panic("control_character with broken character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create and break character
        let id = world.add_character(Vec3::ZERO, Vec3::new(0.5, 0.9, 0.5));
        world.break_destructible(id);
        
        // Control broken character
        world.control_character(id, Vec3::new(1.0, 0.0, 0.0), 0.016, false);
    });
}

#[test]
fn test_jump_invalid_id_no_panic() {
    should_not_panic("jump with invalid id", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Try to jump nonexistent character
        world.jump(99999, 5.0);
    });
}

#[test]
fn test_jump_broken_id_no_panic() {
    should_not_panic("jump on broken character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        let id = world.add_character(Vec3::ZERO, Vec3::new(0.5, 0.9, 0.5));
        world.break_destructible(id);
        
        // Try to jump broken character
        world.jump(id, 5.0);
    });
}

// ============================================================================
// Raycast edge cases
// ============================================================================

#[test]
fn test_raycast_empty_world_no_panic() {
    should_not_panic("raycast in empty world", || {
        let world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Raycast in world with no bodies
        let hit = world.raycast(Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0), 100.0);
        assert!(hit.is_none(), "Should return None in empty world");
    });
}

#[test]
fn test_raycast_zero_direction_no_panic() {
    should_not_panic("raycast with zero direction", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world.create_ground_plane(Vec3::new(100.0, 0.1, 100.0), 0.5);
        
        // Raycast with zero direction vector
        let hit = world.raycast(Vec3::ZERO, Vec3::ZERO, 100.0);
        let _ = hit;
    });
}

#[test]
fn test_raycast_zero_distance_no_panic() {
    should_not_panic("raycast with zero distance", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world.create_ground_plane(Vec3::new(100.0, 0.1, 100.0), 0.5);
        
        // Raycast with zero max distance
        let hit = world.raycast(Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0), 0.0);
        let _ = hit;
    });
}

#[test]
fn test_raycast_negative_distance_no_panic() {
    should_not_panic("raycast with negative distance", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        world.create_ground_plane(Vec3::new(100.0, 0.1, 100.0), 0.5);
        
        // Raycast with negative max distance
        let hit = world.raycast(Vec3::ZERO, Vec3::new(0.0, -1.0, 0.0), -100.0);
        let _ = hit;
    });
}

// ============================================================================
// Extreme value tests
// ============================================================================

#[test]
fn test_extreme_position_no_panic() {
    should_not_panic("create body at extreme position", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create bodies at extreme positions
        let _ = world.add_dynamic_box(Vec3::new(f32::MAX / 2.0, 0.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        let _ = world.add_dynamic_box(Vec3::new(f32::MIN / 2.0, 0.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        let _ = world.add_dynamic_box(Vec3::new(0.0, f32::MAX / 2.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        let _ = world.add_dynamic_box(Vec3::new(0.0, f32::MIN / 2.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
    });
}

#[test]
fn test_extreme_force_no_panic() {
    should_not_panic("apply extreme force", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        
        // Apply extremely large forces
        world.apply_force(id, Vec3::new(f32::MAX / 2.0, 0.0, 0.0));
        world.step();
        
        world.apply_force(id, Vec3::new(f32::MIN / 2.0, 0.0, 0.0));
        world.step();
    });
}

#[test]
fn test_extreme_velocity_no_panic() {
    should_not_panic("set extreme velocity", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        
        // Set extremely high velocities
        world.set_velocity(id, Vec3::new(f32::MAX / 2.0, 0.0, 0.0));
        world.step();
        
        world.set_velocity(id, Vec3::new(f32::MIN / 2.0, 0.0, 0.0));
        world.step();
    });
}

#[test]
fn test_extreme_mass_no_panic() {
    should_not_panic("create body with extreme mass", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Very small mass
        let _ = world.add_dynamic_box(Vec3::ZERO, Vec3::splat(1.0), f32::MIN_POSITIVE, Layers::DEFAULT);
        
        // Very large mass
        let _ = world.add_dynamic_box(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(1.0), f32::MAX / 2.0, Layers::DEFAULT);
    });
}

// ============================================================================
// Stress tests
// ============================================================================

#[test]
fn test_rapid_create_remove_cycle_no_panic() {
    should_not_panic("rapid create/remove cycle", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        for _ in 0..1000 {
            let id = world.add_dynamic_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, Layers::DEFAULT);
            world.step();
            world.break_destructible(id);
        }
    });
}

#[test]
fn test_many_bodies_same_position_no_panic() {
    should_not_panic("many bodies at same position", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create many overlapping bodies (potential collision explosion)
        for _ in 0..100 {
            world.add_dynamic_box(Vec3::ZERO, Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        }
        
        // Step to let physics resolve
        for _ in 0..10 {
            world.step();
        }
    });
}

#[test]
fn test_operations_interleaved_with_step_no_panic() {
    should_not_panic("operations interleaved with step", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        for i in 0..100 {
            // Create some bodies
            let id = world.add_dynamic_box(Vec3::new(i as f32, 0.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
            
            // Step
            world.step();
            
            // Apply forces
            world.apply_force(id, Vec3::new(0.0, 100.0, 0.0));
            
            // Step
            world.step();
            
            // Remove sometimes
            if i % 2 == 0 {
                world.break_destructible(id);
            }
            
            // Step
            world.step();
        }
    });
}

// ============================================================================
// PhysicsConfig edge cases
// ============================================================================

#[test]
fn test_config_with_zero_timestep_no_panic() {
    should_not_panic("PhysicsConfig with zero timestep", || {
        let config = PhysicsConfig {
            time_step: 0.0,
            ..Default::default()
        };
        let mut world = PhysicsWorld::from_config(config);
        
        // Try to step with zero timestep
        world.step();
    });
}

#[test]
fn test_config_with_negative_timestep_no_panic() {
    should_not_panic("PhysicsConfig with negative timestep", || {
        let config = PhysicsConfig {
            time_step: -0.016,
            ..Default::default()
        };
        let mut world = PhysicsWorld::from_config(config);
        
        // Try to step with negative timestep
        world.step();
    });
}

#[test]
fn test_config_with_zero_gravity_no_panic() {
    should_not_panic("PhysicsConfig with zero gravity", || {
        let config = PhysicsConfig {
            gravity: Vec3::ZERO,
            ..Default::default()
        };
        let mut world = PhysicsWorld::from_config(config);
        let id = world.add_dynamic_box(Vec3::new(0.0, 10.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        
        // Step with zero gravity
        for _ in 0..100 {
            world.step();
        }
        
        let _ = world.body_transform(id);
    });
}

#[test]
fn test_config_with_extreme_gravity_no_panic() {
    should_not_panic("PhysicsConfig with extreme gravity", || {
        let config = PhysicsConfig {
            gravity: Vec3::new(0.0, -1000000.0, 0.0),
            ..Default::default()
        };
        let mut world = PhysicsWorld::from_config(config);
        let _ = world.add_dynamic_box(Vec3::new(0.0, 10.0, 0.0), Vec3::splat(1.0), 1.0, Layers::DEFAULT);
        
        // Step with extreme gravity
        for _ in 0..10 {
            world.step();
        }
    });
}

// ============================================================================
// Spatial hash edge cases
// ============================================================================

#[test]
fn test_spatial_hash_empty_no_panic() {
    should_not_panic("spatial hash empty query", || {
        let hash = SpatialHash::<u32>::new(1.0);
        
        // Query empty spatial hash
        let results = hash.query(AABB {
            min: Vec3::new(-10.0, -10.0, -10.0),
            max: Vec3::new(10.0, 10.0, 10.0),
        });
        assert!(results.is_empty());
    });
}

#[test]
fn test_spatial_hash_tiny_cell_size_no_panic() {
    should_not_panic("spatial hash with tiny cell size", || {
        // Very small but valid cell size
        let hash = SpatialHash::<u32>::new(0.001);
        let _ = hash;
    });
}

#[test]
fn test_spatial_hash_large_cell_size_no_panic() {
    should_not_panic("spatial hash with large cell size", || {
        // Very large cell size
        let hash = SpatialHash::<u32>::new(10000.0);
        let _ = hash;
    });
}

#[test]
fn test_spatial_hash_insert_query_no_panic() {
    should_not_panic("spatial hash insert and query", || {
        let mut hash = SpatialHash::<u32>::new(10.0);
        
        // Insert several items
        for i in 0..100 {
            let aabb = AABB {
                min: Vec3::new(i as f32 * 5.0, 0.0, 0.0),
                max: Vec3::new(i as f32 * 5.0 + 2.0, 2.0, 2.0),
            };
            hash.insert(i, aabb);
        }
        
        // Query various ranges
        let results = hash.query(AABB {
            min: Vec3::new(-100.0, -100.0, -100.0),
            max: Vec3::new(100.0, 100.0, 100.0),
        });
        let _ = results;
    });
}

#[test]
fn test_spatial_hash_inverted_aabb_no_panic() {
    should_not_panic("spatial hash with inverted AABB (min > max)", || {
        let mut hash = SpatialHash::<u32>::new(1.0);
        
        // Insert with inverted AABB
        let aabb = AABB {
            min: Vec3::new(10.0, 10.0, 10.0),
            max: Vec3::new(-10.0, -10.0, -10.0),
        };
        hash.insert(1, aabb);
        
        // Query with inverted AABB
        let _ = hash.query(aabb);
    });
}
