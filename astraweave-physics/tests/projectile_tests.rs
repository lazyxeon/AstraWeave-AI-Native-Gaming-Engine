//! Integration tests for the Projectile System (Phase 2)

use astraweave_physics::{
    projectile::{
        ExplosionConfig, FalloffCurve, ProjectileConfig, ProjectileKind, ProjectileManager,
    },
    Layers, PhysicsWorld,
};
use glam::Vec3;

#[test]
fn test_projectile_with_physics_raycast() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let mut manager = ProjectileManager::new();

    // Create a wall at X=10
    let _wall = world.add_static_trimesh(
        &[
            Vec3::new(10.0, -10.0, -10.0),
            Vec3::new(10.0, -10.0, 10.0),
            Vec3::new(10.0, 10.0, -10.0),
            Vec3::new(10.0, 10.0, 10.0),
        ],
        &[[0, 1, 2], [1, 3, 2]],
        Layers::DEFAULT,
    );

    // Step physics to update query pipeline
    world.step();

    // Spawn projectile moving toward wall
    let config = ProjectileConfig {
        kind: ProjectileKind::Kinematic,
        position: Vec3::new(0.0, 0.0, 0.0),
        velocity: Vec3::new(20.0, 0.0, 0.0),
        gravity_scale: 0.0,
        ..Default::default()
    };
    let _id = manager.spawn(config);

    // Update until we hit something or timeout
    for _ in 0..120 {
        // Create raycast closure fresh each iteration to avoid borrow issues
        let raycast = |origin: Vec3, dir: Vec3, max: f32| world.raycast(origin, dir, max);
        manager.update(1.0 / 60.0, raycast);
        world.step();
    }

    // Check hits
    let hits = manager.drain_hits();
    assert!(!hits.is_empty(), "Projectile should have hit the wall");
    assert!(
        (hits[0].position.x - 10.0).abs() < 0.5,
        "Hit should be near X=10, got {}",
        hits[0].position.x
    );
}

#[test]
fn test_hitscan_with_physics() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let mut manager = ProjectileManager::new();

    // Create a target box
    let _target = world.add_dynamic_box(Vec3::new(5.0, 0.0, 0.0), Vec3::splat(0.5), 1.0, Layers::DEFAULT);

    // Step to update query pipeline
    world.step();

    // Perform hitscan
    let raycast = |origin: Vec3, dir: Vec3, max: f32| world.raycast(origin, dir, max);
    let hit = manager.hitscan(Vec3::ZERO, Vec3::X, 100.0, raycast);

    assert!(hit.is_some(), "Hitscan should hit the target");
    let hit = hit.unwrap();
    assert!(hit.body_id.is_some(), "Hit should have body ID");
    assert!(
        (hit.distance - 4.5).abs() < 0.1,
        "Distance should be ~4.5 (5 - 0.5 box half), got {}",
        hit.distance
    );
}

#[test]
fn test_explosion_affects_dynamic_bodies() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    // Create ground
    let _ground = world.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);

    // Create dynamic boxes around explosion center
    let box1 = world.add_dynamic_box(Vec3::new(3.0, 1.0, 0.0), Vec3::splat(0.5), 1.0, Layers::DEFAULT);
    let box2 = world.add_dynamic_box(Vec3::new(0.0, 1.0, 3.0), Vec3::splat(0.5), 1.0, Layers::DEFAULT);
    let _static_box = world.add_static_trimesh(
        &[
            Vec3::new(-3.0, 0.0, 0.0),
            Vec3::new(-3.0, 2.0, 0.0),
            Vec3::new(-3.0, 0.0, 1.0),
        ],
        &[[0, 1, 2]],
        Layers::DEFAULT,
    );

    // Apply explosion
    let affected = world.apply_radial_impulse(
        Vec3::new(0.0, 1.0, 0.0),
        10.0,
        100.0,
        FalloffCurve::Linear,
        0.3,
    );

    assert_eq!(affected, 2, "Should affect 2 dynamic bodies");

    // Step physics
    for _ in 0..30 {
        world.step();
    }

    // Verify boxes moved away from explosion
    let pos1 = world.body_transform(box1).unwrap().w_axis;
    let pos2 = world.body_transform(box2).unwrap().w_axis;

    assert!(pos1.x > 3.5, "Box1 should have moved away in +X");
    assert!(pos2.z > 3.5, "Box2 should have moved away in +Z");
}

#[test]
fn test_projectile_trajectory_prediction() {
    use astraweave_physics::projectile::predict_trajectory;

    let points = predict_trajectory(
        Vec3::new(0.0, 10.0, 0.0),
        Vec3::new(10.0, 10.0, 0.0),
        Vec3::new(0.0, -9.81, 0.0),
        0.0,
        0.1,
        50, // More points to see full arc
    );

    // Should create a parabolic arc
    assert_eq!(points.len(), 50);

    // Peak should be somewhere in the middle
    let max_y = points.iter().map(|p| p.y).reduce(f32::max).unwrap();
    assert!(max_y > 10.0, "Trajectory should go up first");

    // With enough time, gravity should bring it back down
    // After 5 seconds (50 points * 0.1s), it should definitely be lower
    assert!(points[49].y < max_y, "End should be lower than peak");

    // Should move forward
    assert!(points[49].x > points[0].x, "Should move in +X direction");
}

#[test]
fn test_explosion_falloff_curves() {
    let manager = ProjectileManager::new();

    // Test all falloff curves
    let curves = [
        (FalloffCurve::Linear, 0.5),      // At half radius: 0.5
        (FalloffCurve::Quadratic, 0.75),  // At half radius: 1 - 0.5^2 = 0.75
        (FalloffCurve::Constant, 1.0),    // At half radius: 1.0
    ];

    for (curve, expected) in curves {
        let config = ExplosionConfig {
            center: Vec3::ZERO,
            radius: 10.0,
            force: 1000.0,
            falloff: curve,
            upward_bias: 0.0,
        };

        let bodies = vec![(1, Vec3::new(5.0, 0.0, 0.0))]; // At half radius
        let results = manager.calculate_explosion(&config, bodies);

        assert_eq!(results.len(), 1);
        assert!(
            (results[0].falloff_multiplier - expected).abs() < 0.01,
            "{:?}: Expected {}, got {}",
            curve,
            expected,
            results[0].falloff_multiplier
        );
    }
}

#[test]
fn test_projectile_with_wind() {
    let mut manager = ProjectileManager::new();
    manager.gravity = Vec3::ZERO;
    manager.wind = Vec3::new(10.0, 0.0, 0.0); // Strong wind in +X

    let config = ProjectileConfig {
        position: Vec3::ZERO,
        velocity: Vec3::new(0.0, 0.0, 10.0), // Moving +Z
        gravity_scale: 0.0,
        ..Default::default()
    };
    let id = manager.spawn(config);

    let raycast = |_: Vec3, _: Vec3, _: f32| None;

    // Simulate 1 second
    for _ in 0..60 {
        manager.update(1.0 / 60.0, raycast);
    }

    let proj = manager.get(id).unwrap();
    // Should have drifted in wind direction
    assert!(proj.position.x > 1.0, "Wind should push projectile in +X");
    assert!((proj.position.z - 10.0).abs() < 1.0, "Should still move ~10 in +Z");
}

#[test]
fn test_projectile_bouncing() {
    let mut manager = ProjectileManager::new();
    manager.gravity = Vec3::ZERO;

    let config = ProjectileConfig {
        position: Vec3::ZERO,
        velocity: Vec3::new(10.0, 0.0, 0.0),
        gravity_scale: 0.0,
        max_bounces: 5,
        restitution: 0.9,
        ..Default::default()
    };
    let id = manager.spawn(config);

    // Simulate bouncing off walls at X=5 and X=-5
    let raycast = |origin: Vec3, dir: Vec3, max: f32| {
        if origin.x < 5.0 && dir.x > 0.0 {
            let dist = 5.0 - origin.x;
            if dist < max && dist > 0.0 {
                return Some((Vec3::new(5.0, 0.0, 0.0), Vec3::new(-1.0, 0.0, 0.0), None, dist));
            }
        }
        if origin.x > -5.0 && dir.x < 0.0 {
            let dist = origin.x + 5.0;
            if dist < max && dist > 0.0 {
                return Some((Vec3::new(-5.0, 0.0, 0.0), Vec3::new(1.0, 0.0, 0.0), None, dist));
            }
        }
        None
    };

    // Simulate enough time for multiple bounces
    for _ in 0..300 {
        manager.update(1.0 / 60.0, raycast);
    }

    let hits = manager.drain_hits();
    assert!(hits.len() >= 3, "Should have bounced at least 3 times, got {}", hits.len());
}

#[test]
fn test_projectile_lifetime_despawn() {
    let mut manager = ProjectileManager::new();

    let config = ProjectileConfig {
        max_lifetime: 0.1, // Very short lifetime
        ..Default::default()
    };
    let id = manager.spawn(config);

    let raycast = |_: Vec3, _: Vec3, _: f32| None;

    // Should exist initially
    assert!(manager.get(id).is_some());

    // Simulate past lifetime
    for _ in 0..20 {
        manager.update(1.0 / 60.0, raycast);
    }

    // Should be despawned
    assert!(manager.get(id).is_none(), "Projectile should despawn after lifetime");
}
