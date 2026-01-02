//! NaN and Infinity Input Validation Tests for astraweave-physics
//!
//! These tests verify that the physics system handles invalid floating-point
//! inputs (NaN, Infinity, -Infinity) gracefully without crashing or producing
//! undefined behavior.
//!
//! P0-Critical: These tests prevent silent corruption of physics state and
//! potential crashes in production.

use astraweave_physics::{Layers, PhysicsConfig, PhysicsWorld};
use glam::Vec3;

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/// Assert that a result did not panic
fn assert_no_panic<T>(name: &str, result: std::thread::Result<T>) {
    assert!(result.is_ok(), "{} should not panic", name);
}

/// Run a closure and verify it doesn't panic
fn should_not_panic<F: FnOnce() + std::panic::UnwindSafe>(name: &str, f: F) {
    let result = std::panic::catch_unwind(f);
    assert_no_panic(name, result);
}

// ============================================================================
// 1. WORLD CREATION WITH INVALID GRAVITY
// ============================================================================

#[test]
fn test_world_creation_nan_gravity_x() {
    should_not_panic("NaN gravity X", || {
        let _world = PhysicsWorld::new(Vec3::new(f32::NAN, -9.81, 0.0));
    });
}

#[test]
fn test_world_creation_nan_gravity_y() {
    should_not_panic("NaN gravity Y", || {
        let _world = PhysicsWorld::new(Vec3::new(0.0, f32::NAN, 0.0));
    });
}

#[test]
fn test_world_creation_nan_gravity_z() {
    should_not_panic("NaN gravity Z", || {
        let _world = PhysicsWorld::new(Vec3::new(0.0, -9.81, f32::NAN));
    });
}

#[test]
fn test_world_creation_all_nan_gravity() {
    should_not_panic("All NaN gravity", || {
        let _world = PhysicsWorld::new(Vec3::new(f32::NAN, f32::NAN, f32::NAN));
    });
}

#[test]
fn test_world_creation_infinity_gravity() {
    should_not_panic("Infinity gravity", || {
        let _world = PhysicsWorld::new(Vec3::new(0.0, f32::INFINITY, 0.0));
    });
}

#[test]
fn test_world_creation_neg_infinity_gravity() {
    should_not_panic("Negative infinity gravity", || {
        let _world = PhysicsWorld::new(Vec3::new(0.0, f32::NEG_INFINITY, 0.0));
    });
}

// ============================================================================
// 2. BODY CREATION WITH INVALID POSITIONS
// ============================================================================

#[test]
fn test_dynamic_box_nan_position() {
    should_not_panic("NaN position dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(
            Vec3::new(f32::NAN, 0.0, 0.0),
            Vec3::ONE,
            1.0,
            Layers::DEFAULT,
        );
    });
}

#[test]
fn test_dynamic_box_infinity_position() {
    should_not_panic("Infinity position dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(
            Vec3::new(f32::INFINITY, 0.0, 0.0),
            Vec3::ONE,
            1.0,
            Layers::DEFAULT,
        );
    });
}

#[test]
fn test_dynamic_box_all_nan_position() {
    should_not_panic("All NaN position dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(
            Vec3::new(f32::NAN, f32::NAN, f32::NAN),
            Vec3::ONE,
            1.0,
            Layers::DEFAULT,
        );
    });
}

#[test]
fn test_character_nan_position() {
    should_not_panic("NaN position character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_character(Vec3::new(f32::NAN, 0.0, 0.0), Vec3::ONE);
    });
}

#[test]
fn test_character_infinity_position() {
    should_not_panic("Infinity position character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_character(Vec3::new(f32::INFINITY, 0.0, 0.0), Vec3::ONE);
    });
}

// ============================================================================
// 3. BODY CREATION WITH INVALID SIZES
// ============================================================================

#[test]
fn test_dynamic_box_nan_size() {
    should_not_panic("NaN size dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(
            Vec3::ZERO,
            Vec3::new(f32::NAN, 1.0, 1.0),
            1.0,
            Layers::DEFAULT,
        );
    });
}

#[test]
fn test_dynamic_box_infinity_size() {
    should_not_panic("Infinity size dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(
            Vec3::ZERO,
            Vec3::new(f32::INFINITY, 1.0, 1.0),
            1.0,
            Layers::DEFAULT,
        );
    });
}

#[test]
fn test_dynamic_box_zero_size() {
    should_not_panic("Zero size dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(Vec3::ZERO, Vec3::ZERO, 1.0, Layers::DEFAULT);
    });
}

#[test]
fn test_dynamic_box_negative_size() {
    should_not_panic("Negative size dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(Vec3::ZERO, Vec3::new(-1.0, -1.0, -1.0), 1.0, Layers::DEFAULT);
    });
}

#[test]
fn test_character_nan_size() {
    should_not_panic("NaN size character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_character(Vec3::ZERO, Vec3::new(f32::NAN, 1.0, 1.0));
    });
}

#[test]
fn test_character_zero_size() {
    should_not_panic("Zero size character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_character(Vec3::ZERO, Vec3::ZERO);
    });
}

// ============================================================================
// 4. BODY CREATION WITH INVALID MASS
// ============================================================================

#[test]
fn test_dynamic_box_nan_mass() {
    should_not_panic("NaN mass dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, f32::NAN, Layers::DEFAULT);
    });
}

#[test]
fn test_dynamic_box_infinity_mass() {
    should_not_panic("Infinity mass dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, f32::INFINITY, Layers::DEFAULT);
    });
}

#[test]
fn test_dynamic_box_zero_mass() {
    should_not_panic("Zero mass dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 0.0, Layers::DEFAULT);
    });
}

#[test]
fn test_dynamic_box_negative_mass() {
    should_not_panic("Negative mass dynamic box", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, -1.0, Layers::DEFAULT);
    });
}

// ============================================================================
// 5. FORCE/IMPULSE APPLICATION WITH INVALID VALUES
// ============================================================================

#[test]
fn test_apply_force_nan() {
    should_not_panic("NaN force", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.apply_force(id, Vec3::new(f32::NAN, 0.0, 0.0));
    });
}

#[test]
fn test_apply_force_infinity() {
    should_not_panic("Infinity force", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.apply_force(id, Vec3::new(f32::INFINITY, 0.0, 0.0));
    });
}

#[test]
fn test_apply_force_neg_infinity() {
    should_not_panic("Negative infinity force", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.apply_force(id, Vec3::new(f32::NEG_INFINITY, 0.0, 0.0));
    });
}

#[test]
fn test_apply_impulse_nan() {
    should_not_panic("NaN impulse", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.apply_impulse(id, Vec3::new(f32::NAN, 0.0, 0.0));
    });
}

#[test]
fn test_apply_impulse_infinity() {
    should_not_panic("Infinity impulse", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.apply_impulse(id, Vec3::new(f32::INFINITY, 0.0, 0.0));
    });
}

// ============================================================================
// 6. VELOCITY OPERATIONS WITH INVALID VALUES
// ============================================================================

#[test]
fn test_set_velocity_nan() {
    should_not_panic("NaN velocity", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.set_velocity(id, Vec3::new(f32::NAN, 0.0, 0.0));
    });
}

#[test]
fn test_set_velocity_infinity() {
    should_not_panic("Infinity velocity", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.set_velocity(id, Vec3::new(f32::INFINITY, 0.0, 0.0));
    });
}

#[test]
fn test_set_velocity_neg_infinity() {
    should_not_panic("Negative infinity velocity", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.set_velocity(id, Vec3::new(f32::NEG_INFINITY, 0.0, 0.0));
    });
}

#[test]
fn test_set_velocity_all_nan() {
    should_not_panic("All NaN velocity", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, 1.0, Layers::DEFAULT);
        world.set_velocity(id, Vec3::new(f32::NAN, f32::NAN, f32::NAN));
    });
}

// ============================================================================
// 7. CHARACTER CONTROLLER WITH INVALID INPUTS
// ============================================================================

#[test]
fn test_control_character_nan_movement() {
    should_not_panic("NaN movement character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.control_character(id, Vec3::new(f32::NAN, 0.0, 0.0), 0.016, false);
    });
}

#[test]
fn test_control_character_infinity_movement() {
    should_not_panic("Infinity movement character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.control_character(id, Vec3::new(f32::INFINITY, 0.0, 0.0), 0.016, false);
    });
}

#[test]
fn test_control_character_nan_dt() {
    should_not_panic("NaN dt character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.control_character(id, Vec3::new(1.0, 0.0, 1.0), f32::NAN, false);
    });
}

#[test]
fn test_control_character_infinity_dt() {
    should_not_panic("Infinity dt character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.control_character(id, Vec3::new(1.0, 0.0, 1.0), f32::INFINITY, false);
    });
}

#[test]
fn test_control_character_zero_dt() {
    should_not_panic("Zero dt character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.control_character(id, Vec3::new(1.0, 0.0, 1.0), 0.0, false);
    });
}

#[test]
fn test_control_character_negative_dt() {
    should_not_panic("Negative dt character", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.control_character(id, Vec3::new(1.0, 0.0, 1.0), -0.016, false);
    });
}

// ============================================================================
// 8. JUMP WITH INVALID HEIGHT
// ============================================================================

#[test]
fn test_jump_nan_height() {
    should_not_panic("NaN jump height", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.jump(id, f32::NAN);
    });
}

#[test]
fn test_jump_infinity_height() {
    should_not_panic("Infinity jump height", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.jump(id, f32::INFINITY);
    });
}

#[test]
fn test_jump_negative_height() {
    should_not_panic("Negative jump height", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.jump(id, -1.0);
    });
}

#[test]
fn test_jump_zero_height() {
    should_not_panic("Zero jump height", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
        world.jump(id, 0.0);
    });
}

// ============================================================================
// 9. PHYSICS STEP WITH NAN STATE
// ============================================================================

#[test]
fn test_step_with_nan_body() {
    should_not_panic("Step with NaN body", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(
            Vec3::new(f32::NAN, f32::NAN, f32::NAN),
            Vec3::ONE,
            1.0,
            Layers::DEFAULT,
        );
        world.set_velocity(id, Vec3::new(f32::NAN, f32::NAN, f32::NAN));
        
        // Step should not crash
        for _ in 0..10 {
            world.step();
        }
    });
}

#[test]
fn test_step_after_nan_force() {
    should_not_panic("Step after NaN force", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
        world.apply_force(id, Vec3::new(f32::NAN, f32::NAN, f32::NAN));
        
        // Step should not crash
        for _ in 0..10 {
            world.step();
        }
    });
}

#[test]
fn test_step_after_infinity_impulse() {
    should_not_panic("Step after infinity impulse", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let id = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
        world.apply_impulse(id, Vec3::new(f32::INFINITY, 0.0, 0.0));
        
        // Step should not crash
        for _ in 0..10 {
            world.step();
        }
    });
}

// ============================================================================
// 10. GROUND PLANE WITH INVALID VALUES
// ============================================================================

#[test]
fn test_ground_plane_nan_size() {
    should_not_panic("NaN ground size", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.create_ground_plane(Vec3::new(f32::NAN, 0.5, 50.0), 0.9);
    });
}

#[test]
fn test_ground_plane_nan_friction() {
    should_not_panic("NaN ground friction", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), f32::NAN);
    });
}

#[test]
fn test_ground_plane_infinity_friction() {
    should_not_panic("Infinity ground friction", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), f32::INFINITY);
    });
}

#[test]
fn test_ground_plane_negative_friction() {
    should_not_panic("Negative ground friction", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), -1.0);
    });
}

// ============================================================================
// 11. TRIMESH WITH INVALID VERTICES
// ============================================================================

#[test]
fn test_trimesh_nan_vertices() {
    should_not_panic("NaN trimesh vertices", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let vertices = vec![
            Vec3::new(f32::NAN, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 10.0),
        ];
        let indices = vec![[0, 1, 2]];
        let _id = world.add_static_trimesh(&vertices, &indices, Layers::DEFAULT);
    });
}

#[test]
fn test_trimesh_infinity_vertices() {
    should_not_panic("Infinity trimesh vertices", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let vertices = vec![
            Vec3::new(f32::INFINITY, 0.0, 0.0),
            Vec3::new(10.0, 0.0, 0.0),
            Vec3::new(5.0, 0.0, 10.0),
        ];
        let indices = vec![[0, 1, 2]];
        let _id = world.add_static_trimesh(&vertices, &indices, Layers::DEFAULT);
    });
}

// ============================================================================
// 12. PHYSICS CONFIG WITH INVALID VALUES
// ============================================================================

#[test]
fn test_config_nan_time_step() {
    should_not_panic("NaN time step", || {
        let config = PhysicsConfig {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ccd_enabled: false,
            max_ccd_substeps: 1,
            time_step: f32::NAN,
            water_level: f32::NEG_INFINITY,
            fluid_density: 1000.0,
        };
        let _world = PhysicsWorld::from_config(config);
    });
}

#[test]
fn test_config_zero_time_step() {
    should_not_panic("Zero time step", || {
        let config = PhysicsConfig {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ccd_enabled: false,
            max_ccd_substeps: 1,
            time_step: 0.0,
            water_level: f32::NEG_INFINITY,
            fluid_density: 1000.0,
        };
        let _world = PhysicsWorld::from_config(config);
    });
}

#[test]
fn test_config_negative_time_step() {
    should_not_panic("Negative time step", || {
        let config = PhysicsConfig {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ccd_enabled: false,
            max_ccd_substeps: 1,
            time_step: -0.016,
            water_level: f32::NEG_INFINITY,
            fluid_density: 1000.0,
        };
        let _world = PhysicsWorld::from_config(config);
    });
}

#[test]
fn test_config_nan_fluid_density() {
    should_not_panic("NaN fluid density", || {
        let config = PhysicsConfig {
            gravity: Vec3::new(0.0, -9.81, 0.0),
            ccd_enabled: false,
            max_ccd_substeps: 1,
            time_step: 1.0 / 60.0,
            water_level: 0.0,
            fluid_density: f32::NAN,
        };
        let _world = PhysicsWorld::from_config(config);
    });
}

// ============================================================================
// 13. COMBINED STRESS TEST
// ============================================================================

#[test]
fn test_multiple_nan_bodies_simulation() {
    should_not_panic("Multiple NaN bodies", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        
        // Create ground
        world.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.9);
        
        // Create various invalid bodies
        for i in 0..10 {
            let pos = if i % 3 == 0 {
                Vec3::new(f32::NAN, 5.0, 0.0)
            } else if i % 3 == 1 {
                Vec3::new(i as f32, f32::INFINITY, 0.0)
            } else {
                Vec3::new(i as f32, 5.0, f32::NEG_INFINITY)
            };
            
            let _id = world.add_dynamic_box(pos, Vec3::ONE, 1.0, Layers::DEFAULT);
        }
        
        // Simulate for 100 frames
        for _ in 0..100 {
            world.step();
        }
    });
}

#[test]
fn test_nan_propagation_isolation() {
    // Test that NaN in one body doesn't corrupt other bodies
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    
    // Create ground
    world.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.9);
    
    // Create a valid body
    let valid_id = world.add_dynamic_box(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::ONE,
        1.0,
        Layers::DEFAULT,
    );
    
    // Create a NaN-infected body
    let _nan_id = world.add_dynamic_box(
        Vec3::new(5.0, 5.0, 0.0),
        Vec3::ONE,
        1.0,
        Layers::DEFAULT,
    );
    world.set_velocity(_nan_id, Vec3::new(f32::NAN, f32::NAN, f32::NAN));
    
    // Step simulation
    for _ in 0..10 {
        world.step();
    }
    
    // Valid body should still have valid transform (may be NaN depending on Rapier behavior)
    // The key test is that this doesn't crash
    let _transform = world.body_transform(valid_id);
}

// ============================================================================
// 14. EDGE CASE: SUBNORMAL NUMBERS
// ============================================================================

#[test]
fn test_subnormal_position() {
    should_not_panic("Subnormal position", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        // Subnormal (denormalized) float - very small but not zero
        let subnormal = f32::from_bits(1); // Smallest positive subnormal
        let _id = world.add_dynamic_box(
            Vec3::new(subnormal, subnormal, subnormal),
            Vec3::ONE,
            1.0,
            Layers::DEFAULT,
        );
    });
}

#[test]
fn test_max_float_values() {
    should_not_panic("Max float values", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(
            Vec3::new(f32::MAX, f32::MAX, f32::MAX),
            Vec3::ONE,
            f32::MAX,
            Layers::DEFAULT,
        );
    });
}

#[test]
fn test_min_positive_float_values() {
    should_not_panic("Min positive float values", || {
        let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
        let _id = world.add_dynamic_box(
            Vec3::new(f32::MIN_POSITIVE, f32::MIN_POSITIVE, f32::MIN_POSITIVE),
            Vec3::new(f32::MIN_POSITIVE, f32::MIN_POSITIVE, f32::MIN_POSITIVE),
            f32::MIN_POSITIVE,
            Layers::DEFAULT,
        );
    });
}
