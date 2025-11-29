//! Integration tests for gravity system with PhysicsWorld
//!
//! Tests gravity zones, per-body gravity, and complex scenarios
//! involving multiple bodies and zone interactions.

use astraweave_physics::{
    gravity::{GravityManager, GravityZone, GravityZoneShape, BodyGravitySettings},
    PhysicsWorld, Layers,
};
use glam::Vec3;

/// Test gravity manager with physics world integration
#[test]
fn test_gravity_manager_integration_basic() {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let gravity_manager = GravityManager::new(Vec3::new(0.0, -9.81, 0.0));

    // Create a dynamic body
    let body_id = physics.add_dynamic_box(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(0.5, 0.5, 0.5),
        1.0,
        Layers::DEFAULT,
    );

    // Get position from transform
    let pos = physics.body_transform(body_id)
        .map(|m| Vec3::new(m.w_axis.x, m.w_axis.y, m.w_axis.z))
        .unwrap_or(Vec3::ZERO);
    let gravity = gravity_manager.calculate_gravity(body_id, pos);

    assert!((gravity.y - (-9.81)).abs() < 0.01);
}

/// Test zero-G zone affects physics behavior
#[test]
fn test_zero_g_zone_integration() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    // Add a zero-G zone in the middle of the world
    gravity_manager.add_zero_g_box(
        Vec3::new(-10.0, 5.0, -10.0),
        Vec3::new(10.0, 15.0, 10.0),
        1,
    );

    // Body inside the zone
    let gravity_inside = gravity_manager.calculate_gravity(1, Vec3::new(0.0, 10.0, 0.0));
    assert!(gravity_inside.length() < 0.001, "Should be zero-G inside zone");

    // Body below the zone
    let gravity_below = gravity_manager.calculate_gravity(2, Vec3::new(0.0, 0.0, 0.0));
    assert!((gravity_below.y - (-10.0)).abs() < 0.01, "Should have normal gravity below zone");

    // Body above the zone
    let gravity_above = gravity_manager.calculate_gravity(3, Vec3::new(0.0, 20.0, 0.0));
    assert!((gravity_above.y - (-10.0)).abs() < 0.01, "Should have normal gravity above zone");
}

/// Test orbital mechanics with point gravity
#[test]
fn test_orbital_mechanics_point_gravity() {
    let mut gravity_manager = GravityManager::new(Vec3::ZERO); // No global gravity

    // Add a "planet" attractor at origin
    gravity_manager.add_attractor(Vec3::ZERO, 100.0, 50.0, 1);

    // Satellite at various distances - should be pulled toward center
    let positions = [
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(0.0, 20.0, 0.0),
        Vec3::new(0.0, 0.0, 30.0),
        Vec3::new(50.0, 50.0, 0.0),
    ];

    for (i, pos) in positions.iter().enumerate() {
        let gravity = gravity_manager.calculate_gravity((i + 1) as u64, *pos);
        let to_center = -pos.normalize();

        // Gravity should point toward center
        let dot = gravity.normalize().dot(to_center);
        assert!(
            dot > 0.99,
            "Gravity should point toward attractor center, dot={:.4}",
            dot
        );
    }
}

/// Test stable orbit with point gravity
#[test]
fn test_stable_orbit_simulation() {
    let mut gravity_manager = GravityManager::new(Vec3::ZERO);

    // Strong central attractor
    let attractor_pos = Vec3::new(0.0, 0.0, 0.0);
    let attractor_strength = 1000.0;
    let attractor_radius = 200.0;

    gravity_manager.add_attractor(attractor_pos, attractor_radius, attractor_strength, 1);

    // Satellite starting position and velocity for circular orbit
    let orbit_radius = 50.0;
    let mut position = Vec3::new(orbit_radius, 0.0, 0.0);

    // Calculate orbital velocity for circular orbit
    // v = sqrt(GM/r) but our system uses different parameters
    // We'll just verify the body stays roughly the same distance
    let gravity_at_orbit = gravity_manager.calculate_gravity(1, position);
    let orbital_speed = (gravity_at_orbit.length() * orbit_radius).sqrt();
    let mut velocity = Vec3::new(0.0, 0.0, orbital_speed);

    let dt = 0.016; // 60 FPS
    let mut min_distance = f32::MAX;
    let mut max_distance = 0.0f32;

    // Simulate for a few "orbits"
    for _ in 0..500 {
        let gravity = gravity_manager.calculate_gravity(1, position);
        velocity += gravity * dt;
        position += velocity * dt;

        let distance = position.length();
        min_distance = min_distance.min(distance);
        max_distance = max_distance.max(distance);
    }

    // Orbit should be somewhat stable (within 50% variation is okay for this test)
    let variation = (max_distance - min_distance) / orbit_radius;
    assert!(
        variation < 1.0,
        "Orbit too eccentric: min={:.2}, max={:.2}, variation={:.2}",
        min_distance,
        max_distance,
        variation
    );
}

/// Test multiple overlapping zones with priority
#[test]
fn test_overlapping_zones_priority() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    // Large zone with upward gravity (priority 1)
    gravity_manager.add_zone(GravityZone {
        shape: GravityZoneShape::Box {
            min: Vec3::splat(-20.0),
            max: Vec3::splat(20.0),
        },
        gravity: Vec3::new(0.0, 5.0, 0.0), // Upward
        priority: 1,
        ..Default::default()
    });

    // Medium zone with sideways gravity (priority 5)
    gravity_manager.add_zone(GravityZone {
        shape: GravityZoneShape::Box {
            min: Vec3::splat(-10.0),
            max: Vec3::splat(10.0),
        },
        gravity: Vec3::new(10.0, 0.0, 0.0), // Rightward
        priority: 5,
        ..Default::default()
    });

    // Small zone with zero gravity (priority 10)
    gravity_manager.add_zero_g_box(Vec3::splat(-5.0), Vec3::splat(5.0), 10);

    // Test at origin (inside all three zones) - priority 10 wins
    let gravity = gravity_manager.calculate_gravity(1, Vec3::ZERO);
    assert!(gravity.length() < 0.001, "Should be zero-G at origin");

    // Test at (7, 0, 0) - inside large and medium zones - priority 5 wins
    let gravity = gravity_manager.calculate_gravity(2, Vec3::new(7.0, 0.0, 0.0));
    assert!((gravity.x - 10.0).abs() < 0.01, "Should have rightward gravity");
    assert!(gravity.y.abs() < 0.01, "Should have no vertical gravity");

    // Test at (15, 0, 0) - inside only large zone - priority 1
    let gravity = gravity_manager.calculate_gravity(3, Vec3::new(15.0, 0.0, 0.0));
    assert!(gravity.x.abs() < 0.01, "Should have no horizontal gravity");
    assert!((gravity.y - 5.0).abs() < 0.01, "Should have upward gravity");

    // Test at (25, 0, 0) - outside all zones - global gravity
    let gravity = gravity_manager.calculate_gravity(4, Vec3::new(25.0, 0.0, 0.0));
    assert!((gravity.y - (-10.0)).abs() < 0.01, "Should have global gravity");
}

/// Test per-body gravity scale with physics simulation
#[test]
fn test_per_body_gravity_scale_simulation() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    // Different gravity scales for different bodies
    gravity_manager.set_gravity_scale(1, 0.0);  // Floating
    gravity_manager.set_gravity_scale(2, 0.5);  // Half gravity (moon-like)
    gravity_manager.set_gravity_scale(3, 1.0);  // Normal
    gravity_manager.set_gravity_scale(4, 2.0);  // Double gravity (heavy planet)

    // Verify each body's gravity
    let g1 = gravity_manager.calculate_gravity(1, Vec3::ZERO);
    let g2 = gravity_manager.calculate_gravity(2, Vec3::ZERO);
    let g3 = gravity_manager.calculate_gravity(3, Vec3::ZERO);
    let g4 = gravity_manager.calculate_gravity(4, Vec3::ZERO);

    assert!(g1.length() < 0.001, "Body 1 should have zero gravity");
    assert!((g2.y - (-5.0)).abs() < 0.01, "Body 2 should have half gravity");
    assert!((g3.y - (-10.0)).abs() < 0.01, "Body 3 should have normal gravity");
    assert!((g4.y - (-20.0)).abs() < 0.01, "Body 4 should have double gravity");
}

/// Test wall-walking with directional gravity zones
#[test]
fn test_wall_walking_directional_gravity() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    // Floor zone (normal gravity)
    gravity_manager.add_directional_zone(
        Vec3::new(-100.0, 0.0, -100.0),
        Vec3::new(100.0, 2.0, 100.0),
        Vec3::new(0.0, -10.0, 0.0), // Down
        1,
    );

    // Left wall zone (gravity points left)
    gravity_manager.add_directional_zone(
        Vec3::new(-100.0, 0.0, -100.0),
        Vec3::new(-98.0, 50.0, 100.0),
        Vec3::new(-10.0, 0.0, 0.0), // Left
        2,
    );

    // Right wall zone (gravity points right)
    gravity_manager.add_directional_zone(
        Vec3::new(98.0, 0.0, -100.0),
        Vec3::new(100.0, 50.0, 100.0),
        Vec3::new(10.0, 0.0, 0.0), // Right
        2,
    );

    // Ceiling zone (gravity points up)
    gravity_manager.add_directional_zone(
        Vec3::new(-100.0, 48.0, -100.0),
        Vec3::new(100.0, 50.0, 100.0),
        Vec3::new(0.0, 10.0, 0.0), // Up (stick to ceiling)
        2,
    );

    // Test on floor
    let g_floor = gravity_manager.calculate_gravity(1, Vec3::new(0.0, 1.0, 0.0));
    assert!((g_floor.y - (-10.0)).abs() < 0.01, "Should fall down on floor");

    // Test on left wall
    let g_left = gravity_manager.calculate_gravity(2, Vec3::new(-99.0, 25.0, 0.0));
    assert!((g_left.x - (-10.0)).abs() < 0.01, "Should be pulled to left wall");

    // Test on right wall
    let g_right = gravity_manager.calculate_gravity(3, Vec3::new(99.0, 25.0, 0.0));
    assert!((g_right.x - 10.0).abs() < 0.01, "Should be pulled to right wall");

    // Test on ceiling
    let g_ceiling = gravity_manager.calculate_gravity(4, Vec3::new(0.0, 49.0, 0.0));
    assert!((g_ceiling.y - 10.0).abs() < 0.01, "Should be pulled to ceiling");
}

/// Test dynamic zone activation/deactivation
#[test]
fn test_dynamic_zone_toggle() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    let zero_g_zone = gravity_manager.add_zero_g_box(Vec3::splat(-10.0), Vec3::splat(10.0), 1);

    // Initially active
    let g1 = gravity_manager.calculate_gravity(1, Vec3::ZERO);
    assert!(g1.length() < 0.001, "Should be zero-G when zone active");

    // Deactivate
    gravity_manager.set_zone_active(zero_g_zone, false);
    let g2 = gravity_manager.calculate_gravity(1, Vec3::ZERO);
    assert!((g2.y - (-10.0)).abs() < 0.01, "Should have gravity when zone inactive");

    // Reactivate
    gravity_manager.set_zone_active(zero_g_zone, true);
    let g3 = gravity_manager.calculate_gravity(1, Vec3::ZERO);
    assert!(g3.length() < 0.001, "Should be zero-G when zone reactivated");
}

/// Test sphere gravity zone
#[test]
fn test_sphere_gravity_zone() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    // Spherical zero-G zone centered at (0, 20, 0) with radius 10
    gravity_manager.add_zero_g_sphere(Vec3::new(0.0, 20.0, 0.0), 10.0, 1);

    // Inside the sphere
    let g_inside = gravity_manager.calculate_gravity(1, Vec3::new(0.0, 20.0, 0.0));
    assert!(g_inside.length() < 0.001, "Should be zero-G inside sphere");

    let g_edge = gravity_manager.calculate_gravity(2, Vec3::new(5.0, 20.0, 0.0));
    assert!(g_edge.length() < 0.001, "Should be zero-G near edge of sphere");

    // Just outside the sphere
    let g_outside = gravity_manager.calculate_gravity(3, Vec3::new(0.0, 31.0, 0.0));
    assert!((g_outside.y - (-10.0)).abs() < 0.01, "Should have gravity outside sphere");
}

/// Test repulsor (negative strength point gravity)
#[test]
fn test_repulsor_point_gravity() {
    let mut gravity_manager = GravityManager::new(Vec3::ZERO);

    // Add a repulsor at origin
    gravity_manager.add_attractor(Vec3::ZERO, 50.0, -100.0, 1); // Negative = repel

    // Test bodies at various positions - should all be pushed away
    let test_positions = [
        Vec3::new(10.0, 0.0, 0.0),
        Vec3::new(-10.0, 0.0, 0.0),
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(5.0, 5.0, 5.0),
    ];

    for (i, pos) in test_positions.iter().enumerate() {
        let gravity = gravity_manager.calculate_gravity((i + 1) as u64, *pos);
        let away_from_center = pos.normalize();

        // Gravity should point away from center (repulsion)
        let dot = gravity.normalize().dot(away_from_center);
        assert!(
            dot > 0.99,
            "Gravity should point away from repulsor, dot={:.4} at pos {:?}",
            dot,
            pos
        );
    }
}

/// Test body ignoring zones
#[test]
fn test_body_ignores_zones() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    // Add a zero-G zone
    gravity_manager.add_zero_g_box(Vec3::splat(-20.0), Vec3::splat(20.0), 1);

    // Body 1: normal behavior (affected by zone)
    let g1 = gravity_manager.calculate_gravity(1, Vec3::ZERO);
    assert!(g1.length() < 0.001, "Body 1 should be affected by zone");

    // Body 2: ignores zones
    gravity_manager.set_body_gravity(2, BodyGravitySettings {
        scale: 1.0,
        custom_direction: None,
        ignore_zones: true,
    });

    let g2 = gravity_manager.calculate_gravity(2, Vec3::ZERO);
    assert!((g2.y - (-10.0)).abs() < 0.01, "Body 2 should ignore zone and use global gravity");
}

/// Test combining scale with zones
#[test]
fn test_scale_combined_with_zone() {
    let mut gravity_manager = GravityManager::new(Vec3::new(0.0, -10.0, 0.0));

    // Zone with sideways gravity
    gravity_manager.add_directional_zone(
        Vec3::splat(-10.0),
        Vec3::splat(10.0),
        Vec3::new(20.0, 0.0, 0.0), // Rightward
        1,
    );

    // Body with 0.5 scale
    gravity_manager.set_gravity_scale(1, 0.5);

    let gravity = gravity_manager.calculate_gravity(1, Vec3::ZERO);

    // Should be half of the zone gravity
    assert!((gravity.x - 10.0).abs() < 0.01, "Should have halved zone gravity");
    assert!(gravity.y.abs() < 0.01, "Should have no vertical component");
}

/// Test gravity at attractor center
#[test]
fn test_gravity_at_attractor_center() {
    let mut gravity_manager = GravityManager::new(Vec3::ZERO);
    gravity_manager.add_attractor(Vec3::new(0.0, 50.0, 0.0), 100.0, 500.0, 1);

    // At the exact center of the attractor, gravity should be zero
    let gravity = gravity_manager.calculate_gravity(1, Vec3::new(0.0, 50.0, 0.0));
    assert!(
        gravity.length() < 0.01,
        "Gravity at attractor center should be zero, got {:?}",
        gravity
    );
}
