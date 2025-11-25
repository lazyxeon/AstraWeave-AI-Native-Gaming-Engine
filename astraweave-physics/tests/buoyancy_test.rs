use astraweave_physics::*;
use glam::Vec3;

#[test]
fn buoyancy_prevents_indefinite_sinking() {
    // Create world with water level at y=0
    let mut config = PhysicsConfig::default();
    config.water_level = 0.0;
    config.fluid_density = 1000.0; // Water density in kg/m³

    let mut world = PhysicsWorld::from_config(config);

    // Add ground plane below water
    let _ground = world.create_ground_plane(Vec3::new(10.0, 0.5, 10.0), 0.9);

    // Add a dynamic body (mass 1.0 kg) starting at y=2.0 (above water)
    let body = world.add_dynamic_box(
        Vec3::new(0.0, 2.0, 0.0),
        Vec3::new(0.1, 0.1, 0.1),
        1.0,
        Layers::DEFAULT,
    );

    // Add buoyancy: volume 0.2 m³
    // This displaces 0.2 m³ of water = 200 kg of water
    // Buoyancy force = 200 kg * 9.81 m/s² ≈ 1962 N (upward)
    // Weight of body = 1 kg * 9.81 m/s² ≈ 9.81 N (downward)
    // Net force when submerged ≈ 1952 N upward >> body should be pushed up strongly
    world.add_buoyancy(body, 0.2, 5.0);

    // Run simulation
    let mut min_y = f32::MAX;
    let mut max_y = f32::MIN;

    for _ in 0..300 {
        world.step();

        if let Some(transform) = world.body_transform(body) {
            let y = transform.w_axis.y;
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }

    // Body should fall initially, hit the water, and be pushed back up
    // It should NOT sink indefinitely deep
    // Since buoyancy >> weight, the body should stay near or above water level
    assert!(
        min_y > -2.0,
        "Body sank too deep (min_y={}), buoyancy not working properly",
        min_y
    );

    // The body should have moved (fallen and then pushed up)
    assert!(
        max_y - min_y > 0.1,
        "Body didn't move enough (range={}), physics may not be running",
        max_y - min_y
    );
}

#[test]
fn buoyancy_only_applies_below_water() {
    // Create world with water level at y=0
    let mut config = PhysicsConfig::default();
    config.water_level = 0.0;
    config.fluid_density = 1000.0;

    let mut world = PhysicsWorld::from_config(config);

    // Add a dynamic body well above water
    let body = world.add_dynamic_box(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(0.1, 0.1, 0.1),
        1.0,
        Layers::DEFAULT,
    );

    // Add huge buoyancy (should push body up if applied)
    world.add_buoyancy(body, 10.0, 5.0);

    // Step once
    world.step();

    // Get velocity
    if let Some(handle) = world.handle_of(body) {
        if let Some(rb) = world.bodies.get(handle) {
            let vel = rb.linvel();

            // Body should be falling (negative y velocity) because it's above water
            // Buoyancy should NOT be applied
            assert!(
                vel.y < -0.01,
                "Body above water should be falling, got vel.y={}",
                vel.y
            );
        }
    }
}
