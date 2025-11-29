//! Integration tests for ragdoll system with PhysicsWorld
//!
//! Tests ragdoll creation, physics simulation, impulse propagation,
//! and state management.

use astraweave_physics::{
    ragdoll::{RagdollBuilder, RagdollConfig, RagdollPresets, BoneShape, RagdollState},
    PhysicsWorld,
};
use glam::Vec3;

/// Test that a simple ragdoll can be created and simulated
#[test]
fn test_ragdoll_simulation_basic() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    // Create ground
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    // Create simple ragdoll
    let mut builder = RagdollBuilder::new(RagdollConfig::default());
    builder.add_bone("body", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.3 }, 5.0);
    builder.add_bone(
        "head",
        Some("body"),
        Vec3::new(0.0, 0.5, 0.0),
        BoneShape::Sphere { radius: 0.15 },
        1.0,
    );

    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 5.0, 0.0));

    // Simulate for a bit
    for _ in 0..60 {
        physics.step();
    }

    // Ragdoll should have fallen
    let com = ragdoll.center_of_mass(&physics);
    assert!(com.y < 5.0, "Ragdoll should have fallen, COM at y={}", com.y);
}

/// Test humanoid ragdoll preset with full simulation
#[test]
fn test_humanoid_ragdoll_simulation() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 3.0, 0.0));

    // Should have all 12 bones
    assert_eq!(ragdoll.bone_bodies.len(), 12);

    let initial_com = ragdoll.center_of_mass(&physics);

    // Simulate
    for _ in 0..120 {
        physics.step();
    }

    // Check that ragdoll has fallen (COM should be lower than initial)
    let final_com = ragdoll.center_of_mass(&physics);
    assert!(
        final_com.y < initial_com.y,
        "Ragdoll should have fallen, initial y={}, final y={}",
        initial_com.y,
        final_com.y
    );
}

/// Test quadruped ragdoll preset
#[test]
fn test_quadruped_ragdoll_simulation() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    let mut builder = RagdollPresets::quadruped(RagdollConfig::default());
    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 2.0, 0.0));

    // Should have all 7 bones
    assert_eq!(ragdoll.bone_bodies.len(), 7);

    // Simulate
    for _ in 0..60 {
        physics.step();
    }

    // Check stability (not exploding)
    let com = ragdoll.center_of_mass(&physics);
    assert!(com.y.is_finite(), "COM should be finite (not NaN)");
}

/// Test applying impulse to specific bone
#[test]
fn test_ragdoll_impulse_to_bone() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 2.0, 0.0));

    // Apply strong impulse to head
    let impulse = Vec3::new(100.0, 0.0, 0.0);
    let success = ragdoll.apply_impulse_to_bone(&mut physics, "head", impulse);
    assert!(success, "Should successfully apply impulse to head");

    // Step physics
    physics.step();

    // Head should be moving
    if let Some(head_body) = ragdoll.get_bone_body("head") {
        if let Some(vel) = physics.get_velocity(head_body) {
            assert!(vel.x > 0.0, "Head should be moving right after impulse");
        }
    }
}

/// Test impulse propagation through joint chain
#[test]
fn test_ragdoll_impulse_propagation() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 2.0, 0.0));

    // Apply impulse to spine with propagation
    let impulse = Vec3::new(50.0, 50.0, 0.0);
    ragdoll.apply_impulse_with_propagation(&mut physics, "spine", impulse, 0.5);

    physics.step();

    // Multiple bodies should be affected
    let spine_vel = ragdoll
        .get_bone_body("spine")
        .and_then(|id| physics.get_velocity(id))
        .unwrap_or(Vec3::ZERO);

    let pelvis_vel = ragdoll
        .get_bone_body("pelvis")
        .and_then(|id| physics.get_velocity(id))
        .unwrap_or(Vec3::ZERO);

    // Both should have some velocity
    assert!(spine_vel.length() > 0.1, "Spine should be moving");
    assert!(pelvis_vel.length() > 0.1, "Pelvis should be moving (propagated)");
}

/// Test ragdoll doesn't explode (stability test)
#[test]
fn test_ragdoll_stability() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    let config = RagdollConfig {
        enable_ccd: true,
        ..Default::default()
    };

    let mut builder = RagdollPresets::humanoid(config);
    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 5.0, 0.0));

    // Simulate for extended time
    for _ in 0..300 {
        physics.step();
    }

    // Check all bones are still valid (not NaN, not exploded)
    for body_id in ragdoll.all_bodies() {
        if let Some(transform) = physics.body_transform(body_id) {
            let pos = Vec3::new(transform.w_axis.x, transform.w_axis.y, transform.w_axis.z);
            assert!(pos.is_finite(), "Body position should be finite");
            assert!(pos.length() < 100.0, "Body should not have exploded far away");
        }
    }
}

/// Test ragdoll state tracking
#[test]
fn test_ragdoll_state() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let mut builder = RagdollBuilder::new(RagdollConfig::default());
    builder.add_bone("root", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.2 }, 1.0);

    let ragdoll = builder.build(&mut physics, Vec3::ZERO);

    // Should start active
    assert_eq!(ragdoll.state, RagdollState::Active);
}

/// Test ragdoll is_at_rest detection
#[test]
fn test_ragdoll_at_rest_detection() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    let mut builder = RagdollBuilder::new(RagdollConfig::default());
    builder.add_bone("body", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.3 }, 2.0);

    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 0.5, 0.0));

    // Simulate until settled
    for _ in 0..500 {
        physics.step();
    }

    // Should be at rest after settling
    let at_rest = ragdoll.is_at_rest(&physics, 0.5);
    assert!(at_rest, "Ragdoll should be at rest after settling");
}

/// Test multiple ragdolls in same world
#[test]
fn test_multiple_ragdolls() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    let mut builder = RagdollPresets::humanoid(RagdollConfig::default());

    // Create multiple ragdolls at different positions
    let ragdoll1 = builder.build(&mut physics, Vec3::new(-5.0, 3.0, 0.0));
    let ragdoll2 = builder.build(&mut physics, Vec3::new(0.0, 3.0, 0.0));
    let ragdoll3 = builder.build(&mut physics, Vec3::new(5.0, 3.0, 0.0));

    // Different IDs
    assert_ne!(ragdoll1.id, ragdoll2.id);
    assert_ne!(ragdoll2.id, ragdoll3.id);

    // Different body IDs (no collision)
    let r1_root = ragdoll1.root_body().unwrap();
    let r2_root = ragdoll2.root_body().unwrap();
    let r3_root = ragdoll3.root_body().unwrap();
    assert_ne!(r1_root, r2_root);
    assert_ne!(r2_root, r3_root);

    // All three ragdolls should have 12 bones each
    assert_eq!(ragdoll1.bone_bodies.len(), 12);
    assert_eq!(ragdoll2.bone_bodies.len(), 12);
    assert_eq!(ragdoll3.bone_bodies.len(), 12);

    // Total 36 dynamic bodies + ground should be in world
    // This validates they were all created independently
}

/// Test ragdoll with custom mass scale
#[test]
fn test_ragdoll_mass_scale() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    // Heavy ragdoll (mass_scale = 3.0)
    let heavy_config = RagdollConfig {
        mass_scale: 3.0,
        ..Default::default()
    };

    // Light ragdoll (mass_scale = 0.5)
    let light_config = RagdollConfig {
        mass_scale: 0.5,
        ..Default::default()
    };

    let mut heavy_builder = RagdollBuilder::new(heavy_config);
    heavy_builder.add_bone("body", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.3 }, 10.0);
    let _heavy_ragdoll = heavy_builder.build(&mut physics, Vec3::new(-5.0, 2.0, 0.0));

    let mut light_builder = RagdollBuilder::new(light_config);
    light_builder.add_bone("body", None, Vec3::ZERO, BoneShape::Sphere { radius: 0.3 }, 10.0);
    let _light_ragdoll = light_builder.build(&mut physics, Vec3::new(5.0, 2.0, 0.0));

    // Both ragdolls should have been created successfully
    // The mass difference would be observable in simulation response to forces
}

/// Test hit reaction (shooting a ragdoll)
#[test]
fn test_ragdoll_hit_reaction() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 1.5, 0.0));

    // Record initial chest position
    let chest_body = ragdoll.get_bone_body("chest").unwrap();
    let initial_pos = physics
        .body_transform(chest_body)
        .map(|t| Vec3::new(t.w_axis.x, t.w_axis.y, t.w_axis.z))
        .unwrap();

    // "Shoot" the chest with a strong impulse
    let bullet_impulse = Vec3::new(0.0, 0.0, 200.0); // Forward impact
    ragdoll.apply_impulse_with_propagation(&mut physics, "chest", bullet_impulse, 0.3);

    // Simulate hit reaction
    for _ in 0..30 {
        physics.step();
    }

    // Chest should have moved backward
    let final_pos = physics
        .body_transform(chest_body)
        .map(|t| Vec3::new(t.w_axis.x, t.w_axis.y, t.w_axis.z))
        .unwrap();

    let displacement = final_pos - initial_pos;
    assert!(
        displacement.z > 0.1,
        "Chest should have moved backward from impact, z displacement: {}",
        displacement.z
    );
}

/// Test joint limits prevent unrealistic poses
#[test]
fn test_ragdoll_joint_limits() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    physics.create_ground_plane(Vec3::new(50.0, 0.5, 50.0), 0.8);

    let mut builder = RagdollPresets::humanoid(RagdollConfig::default());
    let ragdoll = builder.build(&mut physics, Vec3::new(0.0, 5.0, 0.0));

    // Apply extreme force to test joints hold
    for bone_name in ["upper_arm_l", "upper_arm_r", "upper_leg_l", "upper_leg_r"] {
        if let Some(body_id) = ragdoll.get_bone_body(bone_name) {
            physics.apply_impulse(body_id, Vec3::new(500.0, 500.0, 500.0));
        }
    }

    // Simulate
    for _ in 0..100 {
        physics.step();
    }

    // Check ragdoll didn't explode
    let com = ragdoll.center_of_mass(&physics);
    assert!(
        com.length() < 50.0,
        "Ragdoll should not have exploded, COM at {:?}",
        com
    );
}
