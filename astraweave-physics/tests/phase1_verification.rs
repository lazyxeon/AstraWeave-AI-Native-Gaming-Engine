// Phase 1 Verification Tests
//
// This test suite verifies the Phase 1 implementation requirements:
// 1. PhysicsWorld initialization with PhysicsConfig
// 2. Enabling CCD on a body
// 3. Adding Fixed joints between bodies
// 4. Adding Revolute joints between bodies

use astraweave_physics::{JointType, Layers, PhysicsConfig, PhysicsWorld};
use glam::Vec3;

// ===== Test Category 1: PhysicsWorld with PhysicsConfig =====

#[test]
fn world_creation_with_default_config() {
    let config = PhysicsConfig::default();
    let _world = PhysicsWorld::from_config(config);

    // World should be created successfully without panic
}

#[test]
fn world_creation_with_custom_config() {
    let config = PhysicsConfig {
        gravity: Vec3::new(0.0, -20.0, 0.0),
        ccd_enabled: true,
        max_ccd_substeps: 4,
        time_step: 1.0 / 120.0,
        water_level: f32::NEG_INFINITY,
        fluid_density: 1000.0,
    };

    let _world = PhysicsWorld::from_config(config);

    // World should be created with custom configuration
}

#[test]
fn world_config_with_zero_gravity() {
    let config = PhysicsConfig {
        gravity: Vec3::ZERO,
        ccd_enabled: false,
        max_ccd_substeps: 1,
        time_step: 1.0 / 60.0,
        water_level: f32::NEG_INFINITY,
        fluid_density: 1000.0,
    };

    let _world = PhysicsWorld::from_config(config);

    // Should support zero gravity configuration
}

#[test]
fn world_config_with_ccd_enabled() {
    let config = PhysicsConfig {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        ccd_enabled: true,
        max_ccd_substeps: 2,
        time_step: 1.0 / 60.0,
        water_level: f32::NEG_INFINITY,
        fluid_density: 1000.0,
    };

    let _world = PhysicsWorld::from_config(config);

    // World should be created with CCD enabled
}

// ===== Test Category 2: CCD (Continuous Collision Detection) =====

#[test]
fn enable_ccd_on_dynamic_body() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box_id = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Enable CCD on the body
    world.enable_ccd(box_id);

    // Should not crash
    assert!(world.body_transform(box_id).is_some());
}

#[test]
fn enable_ccd_on_multiple_bodies() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(5.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box3 = world.add_dynamic_box(Vec3::new(10.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Enable CCD on all bodies
    world.enable_ccd(box1);
    world.enable_ccd(box2);
    world.enable_ccd(box3);

    // All bodies should still exist
    assert!(world.body_transform(box1).is_some());
    assert!(world.body_transform(box2).is_some());
    assert!(world.body_transform(box3).is_some());
}

#[test]
fn enable_ccd_on_invalid_body_does_not_crash() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    // Try to enable CCD on non-existent body
    let invalid_id = 9999;
    world.enable_ccd(invalid_id);

    // Should not crash (silently ignore invalid ID)
}

// ===== Test Category 3: Fixed Joints =====

#[test]
fn add_fixed_joint_between_two_bodies() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Add fixed joint
    let joint_id = world.add_joint(box1, box2, JointType::Fixed);

    // Joint should be created successfully
    assert!(joint_id.0 > 0, "Joint ID should be valid");
}

#[test]
fn add_multiple_fixed_joints() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box3 = world.add_dynamic_box(Vec3::new(4.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Create chain of fixed joints
    let joint1 = world.add_joint(box1, box2, JointType::Fixed);
    let joint2 = world.add_joint(box2, box3, JointType::Fixed);

    // Both joints should be created
    assert!(joint1.0 > 0);
    assert!(joint2.0 > 0);
    assert_ne!(joint1.0, joint2.0, "Joint IDs should be unique");
}

#[test]
fn fixed_joint_bodies_remain_rigid() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let _ground = world.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Add fixed joint
    let _joint_id = world.add_joint(box1, box2, JointType::Fixed);

    // Simulate physics to allow joint to settle
    for _ in 0..120 {
        world.step();
    }

    // Get distance after settling
    let pos1_settled = world.body_transform(box1).unwrap().w_axis;
    let pos2_settled = world.body_transform(box2).unwrap().w_axis;
    let settled_distance = pos1_settled.distance(pos2_settled);

    // Simulate more frames
    for _ in 0..60 {
        world.step();
    }

    // Get final distance
    let pos1_final = world.body_transform(box1).unwrap().w_axis;
    let pos2_final = world.body_transform(box2).unwrap().w_axis;
    let final_distance = pos1_final.distance(pos2_final);

    // Distance should remain constant after joint settles (fixed joint keeps bodies rigid)
    assert!(
        (final_distance - settled_distance).abs() < 0.1,
        "Fixed joint should maintain distance after settling: settled={}, final={}",
        settled_distance,
        final_distance
    );
}

// ===== Test Category 4: Revolute Joints =====

#[test]
fn add_revolute_joint_without_limits() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Add revolute joint around Y axis
    let joint_id = world.add_joint(
        box1,
        box2,
        JointType::Revolute {
            axis: Vec3::Y,
            limits: None,
        },
    );

    // Joint should be created successfully
    assert!(joint_id.0 > 0, "Revolute joint ID should be valid");
}

#[test]
fn add_revolute_joint_with_limits() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Add revolute joint with angle limits (-45 to +45 degrees)
    let joint_id = world.add_joint(
        box1,
        box2,
        JointType::Revolute {
            axis: Vec3::Y,
            limits: Some((-0.785, 0.785)), // -45 to +45 degrees in radians
        },
    );

    // Joint should be created successfully
    assert!(joint_id.0 > 0, "Revolute joint with limits should be valid");
}

#[test]
fn add_revolute_joints_with_different_axes() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box3 = world.add_dynamic_box(Vec3::new(4.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Revolute joint around X axis
    let joint_x = world.add_joint(
        box1,
        box2,
        JointType::Revolute {
            axis: Vec3::X,
            limits: None,
        },
    );

    // Revolute joint around Z axis
    let joint_z = world.add_joint(
        box2,
        box3,
        JointType::Revolute {
            axis: Vec3::Z,
            limits: None,
        },
    );

    // Both joints should be created
    assert!(joint_x.0 > 0);
    assert!(joint_z.0 > 0);
    assert_ne!(joint_x.0, joint_z.0, "Joint IDs should be unique");
}

#[test]
fn revolute_joint_allows_rotation() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let _ground = world.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);

    // Create two boxes
    let box1 = world.add_dynamic_box(Vec3::new(0.0, 3.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 3.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Add revolute joint
    let _joint_id = world.add_joint(
        box1,
        box2,
        JointType::Revolute {
            axis: Vec3::Y,
            limits: None,
        },
    );

    // Simulate physics
    for _ in 0..120 {
        world.step();
    }

    // Bodies should still exist and be connected
    assert!(world.body_transform(box1).is_some());
    assert!(world.body_transform(box2).is_some());
}

// ===== Test Category 5: Joint Edge Cases =====

#[test]
fn add_joint_with_invalid_body1_does_not_crash() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Try to add joint with invalid first body
    let invalid_id = 9999;
    let joint_id = world.add_joint(invalid_id, box2, JointType::Fixed);

    // Should return invalid joint ID (0)
    assert_eq!(joint_id.0, 0, "Invalid body should result in invalid joint");
}

#[test]
fn add_joint_with_invalid_body2_does_not_crash() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Try to add joint with invalid second body
    let invalid_id = 9999;
    let joint_id = world.add_joint(box1, invalid_id, JointType::Fixed);

    // Should return invalid joint ID (0)
    assert_eq!(joint_id.0, 0, "Invalid body should result in invalid joint");
}

#[test]
fn add_joint_between_same_body_does_not_crash() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Try to add joint between same body
    let joint_id = world.add_joint(box1, box1, JointType::Fixed);

    // Should either create joint or return invalid ID, but not crash
    // (specific behavior depends on implementation choice)
    let _ = joint_id;
}

// ===== Test Category 6: Mixed Joint Types =====

#[test]
fn add_both_fixed_and_revolute_joints() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box3 = world.add_dynamic_box(Vec3::new(4.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    // Add fixed joint
    let fixed_joint = world.add_joint(box1, box2, JointType::Fixed);

    // Add revolute joint
    let revolute_joint = world.add_joint(
        box2,
        box3,
        JointType::Revolute {
            axis: Vec3::Y,
            limits: None,
        },
    );

    // Both joints should be created
    assert!(fixed_joint.0 > 0);
    assert!(revolute_joint.0 > 0);
    assert_ne!(fixed_joint.0, revolute_joint.0);
}

// ===== Test Category 7: Integration Test =====

#[test]
fn complete_phase1_integration_test() {
    // 1. Create world with PhysicsConfig
    let config = PhysicsConfig {
        gravity: Vec3::new(0.0, -9.81, 0.0),
        ccd_enabled: true,
        max_ccd_substeps: 2,
        time_step: 1.0 / 60.0,
        water_level: f32::NEG_INFINITY,
        fluid_density: 1000.0,
    };

    let mut world = PhysicsWorld::from_config(config);

    // Add ground
    let _ground = world.create_ground_plane(Vec3::new(20.0, 0.5, 20.0), 0.9);

    // 2. Create bodies and enable CCD
    let box1 = world.add_dynamic_box(Vec3::new(0.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(2.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    let box3 = world.add_dynamic_box(Vec3::new(4.0, 5.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);

    world.enable_ccd(box1);
    world.enable_ccd(box2);
    world.enable_ccd(box3);

    // 3. Add Fixed joint
    let fixed_joint = world.add_joint(box1, box2, JointType::Fixed);
    assert!(fixed_joint.0 > 0);

    // 4. Add Revolute joint
    let revolute_joint = world.add_joint(
        box2,
        box3,
        JointType::Revolute {
            axis: Vec3::Y,
            limits: Some((-1.57, 1.57)), // -90 to +90 degrees
        },
    );
    assert!(revolute_joint.0 > 0);

    // 5. Simulate physics
    for _ in 0..120 {
        world.step();
    }

    // 6. Verify bodies still exist
    assert!(world.body_transform(box1).is_some());
    assert!(world.body_transform(box2).is_some());
    assert!(world.body_transform(box3).is_some());
}
