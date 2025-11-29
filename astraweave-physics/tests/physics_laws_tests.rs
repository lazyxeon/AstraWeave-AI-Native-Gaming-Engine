// Phase 0: Physics Law Verification Tests
//
// Objective: Prove the physics engine obeys fundamental laws of physics.
// This restores credibility after the audit found zero verification tests.

use astraweave_physics::{Layers, PhysicsWorld};
use glam::{Vec3, Vec4Swizzles};

const TIME_STEP: f32 = 1.0 / 60.0;

fn create_world() -> PhysicsWorld {
    // Zero gravity for Newton's laws tests (unless testing gravity itself)
    PhysicsWorld::new(Vec3::ZERO)
}

#[test]
fn test_newtons_first_law_inertia() {
    // Law: An object at rest stays at rest, and an object in motion stays in motion
    // with the same speed and in the same direction unless acted upon by an unbalanced force.
    let mut world = create_world();
    
    // 1. Object at rest
    let id1 = world.add_dynamic_box(Vec3::new(0.0, 0.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    
    // 2. Object in motion (initial velocity)
    let id2 = world.add_dynamic_box(Vec3::new(10.0, 0.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    world.set_velocity(id2, Vec3::new(1.0, 0.0, 0.0));
    
    // Step physics
    for _ in 0..60 {
        world.step();
    }
    
    // Verify object 1 is still at rest
    let pos1 = world.body_transform(id1).unwrap().w_axis.xyz();
    assert!(pos1.length() < 0.0001, "Object at rest moved: {:?}", pos1);
    
    // Verify object 2 moved at constant velocity
    let pos2 = world.body_transform(id2).unwrap().w_axis.xyz();
    let expected_pos = Vec3::new(10.0 + 1.0 * 60.0 * TIME_STEP, 0.0, 0.0);
    assert!((pos2 - expected_pos).length() < 0.001, "Object in motion drifted: {:?} vs {:?}", pos2, expected_pos);
}

#[test]
fn test_newtons_second_law_f_ma() {
    // Law: F = ma (Force = mass * acceleration)
    // Acceleration = F / m
    // Velocity = a * t
    // Position = 0.5 * a * t^2
    let mut world = create_world();
    
    let mass = 2.0;
    let force = Vec3::new(10.0, 0.0, 0.0);
    let id = world.add_dynamic_box(Vec3::ZERO, Vec3::ONE, mass, Layers::DEFAULT);
    
    // Apply constant force for 1 second (60 frames)
    // Rapier forces are persistent, so apply once.
    world.apply_force(id, force);
    
    for _ in 0..60 {
        world.step();
    }
    
    let expected_accel = force / mass; // 5.0
    let time = 60.0 * TIME_STEP; // 1.0
    
    // Check velocity: v = a * t
    let vel = world.get_velocity(id).unwrap();
    let expected_vel = expected_accel * time;
    
    // Allow small integration error
    assert!((vel - expected_vel).length() < 0.1, "Velocity mismatch: {:?} vs {:?}", vel, expected_vel);
}

#[test]
fn test_newtons_third_law_reaction() {
    // Law: For every action, there is an equal and opposite reaction.
    // When two bodies collide, they exert equal and opposite forces on each other.
    // This implies change in momentum is equal and opposite.
    let mut world = create_world();
    
    let mass = 1.0;
    // Body 1 moving right
    let id1 = world.add_dynamic_box(Vec3::new(-2.0, 0.0, 0.0), Vec3::ONE, mass, Layers::DEFAULT);
    world.set_velocity(id1, Vec3::new(5.0, 0.0, 0.0));
    
    // Body 2 stationary
    let id2 = world.add_dynamic_box(Vec3::new(2.0, 0.0, 0.0), Vec3::ONE, mass, Layers::DEFAULT);
    
    // Run until collision happens and settles
    for _ in 0..60 {
        world.step();
    }
    
    let v1 = world.get_velocity(id1).unwrap();
    let v2 = world.get_velocity(id2).unwrap();
    
    // Initial momentum: p1 = 5, p2 = 0 -> Total = 5
    // Final momentum: p1' + p2' should be 5
    // Also, impulse J on 1 should be -J on 2.
    // Delta v1 = v1 - 5
    // Delta v2 = v2 - 0
    // m * Delta v1 = - m * Delta v2
    // Delta v1 = - Delta v2
    
    let delta_v1 = v1 - Vec3::new(5.0, 0.0, 0.0);
    let delta_v2 = v2 - Vec3::ZERO;
    
    assert!((delta_v1 + delta_v2).length() < 0.001, "Reaction forces not equal/opposite: dv1={:?}, dv2={:?}", delta_v1, delta_v2);
}

#[test]
fn test_momentum_conservation() {
    // Law: Total momentum of an isolated system remains constant.
    let mut world = create_world();
    
    let m1 = 2.0;
    let m2 = 1.0;
    
    let id1 = world.add_dynamic_box(Vec3::new(-5.0, 0.0, 0.0), Vec3::ONE, m1, Layers::DEFAULT);
    world.set_velocity(id1, Vec3::new(4.0, 0.0, 0.0));
    
    let id2 = world.add_dynamic_box(Vec3::new(5.0, 0.0, 0.0), Vec3::ONE, m2, Layers::DEFAULT);
    world.set_velocity(id2, Vec3::new(-2.0, 0.0, 0.0));
    
    // Initial Momentum
    // P = m1*v1 + m2*v2 = 2*4 + 1*(-2) = 8 - 2 = 6
    let initial_momentum = Vec3::new(6.0, 0.0, 0.0);
    
    // Run collision
    for _ in 0..120 {
        world.step();
    }
    
    let v1 = world.get_velocity(id1).unwrap();
    let v2 = world.get_velocity(id2).unwrap();
    
    let final_momentum = v1 * m1 + v2 * m2;
    
    assert!((final_momentum - initial_momentum).length() < 0.001, "Momentum not conserved: {:?} vs {:?}", final_momentum, initial_momentum);
}

#[test]
fn test_energy_conservation() {
    // Law: Total energy (kinetic) is conserved in elastic collisions.
    // KE = 0.5 * m * v^2
    let mut world = create_world();
    
    let m1 = 1.0;
    let m2 = 1.0;
    
    let id1 = world.add_dynamic_box(Vec3::new(-3.0, 0.0, 0.0), Vec3::ONE, m1, Layers::DEFAULT);
    world.set_velocity(id1, Vec3::new(4.0, 0.0, 0.0));
    
    let id2 = world.add_dynamic_box(Vec3::new(3.0, 0.0, 0.0), Vec3::ONE, m2, Layers::DEFAULT);
    
    // Set restitution to 1.0 for elastic collision
    // We need to access colliders via handle
    if let Some(_h1) = world.handle_of(id1) {
        // The collider is attached to the body. We need to find the collider handle.
        // In add_dynamic_box, we insert collider with parent.
        // We can iterate colliders and find the one with parent h1.
        // Or just iterate all colliders since we only have 2 bodies.
        for (_, coll) in world.colliders.iter_mut() {
            coll.set_restitution(1.0);
            coll.set_friction(0.0); // Remove friction to avoid energy loss
        }
    }
    
    let initial_ke = 0.5 * m1 * 4.0 * 4.0 + 0.0; // 8.0
    
    // Run collision
    for _ in 0..60 {
        world.step();
    }
    
    let v1 = world.get_velocity(id1).unwrap();
    let v2 = world.get_velocity(id2).unwrap();
    
    let final_ke = 0.5 * m1 * v1.length_squared() + 0.5 * m2 * v2.length_squared();
    
    // Allow small numerical error
    assert!((final_ke - initial_ke).abs() < 0.01, "Energy not conserved: {} vs {}", final_ke, initial_ke);
}

#[test]
fn test_gravity_acceleration() {
    // Verify objects fall at exactly 9.81 m/s^2
    let gravity = Vec3::new(0.0, -9.81, 0.0);
    let mut world = PhysicsWorld::new(gravity);
    
    let id = world.add_dynamic_box(Vec3::new(0.0, 100.0, 0.0), Vec3::ONE, 1.0, Layers::DEFAULT);
    
    // Fall for 1 second
    for _ in 0..60 {
        world.step();
    }
    
    let vel = world.get_velocity(id).unwrap();
    let expected_vel = gravity * 1.0; // -9.81
    
    assert!((vel - expected_vel).length() < 0.1, "Gravity acceleration incorrect: {:?} vs {:?}", vel, expected_vel);
}

#[test]
fn test_wind_storage() {
    let mut world = create_world();
    let wind_dir = Vec3::new(1.0, 0.0, 0.0);
    let strength = 5.0;
    
    world.set_wind(wind_dir, strength);
    
    assert_eq!(world.wind, Vec3::new(5.0, 0.0, 0.0));
}

#[test]
fn test_climb_parameter() {
    let mut world = create_world();
    // Create ground
    world.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);
    
    // Create character
    let id = world.add_character(Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.5, 1.0, 0.5));
    
    // Step once to settle
    world.step();
    let start_y = world.body_transform(id).unwrap().w_axis.y;
    
    // Move with climb=true
    world.control_character(id, Vec3::ZERO, TIME_STEP, true);
    world.step();
    
    let end_y = world.body_transform(id).unwrap().w_axis.y;
    
    assert!(end_y > start_y, "Character did not climb: {} -> {}", start_y, end_y);
}
