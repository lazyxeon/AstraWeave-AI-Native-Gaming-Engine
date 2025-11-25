use astraweave_physics::*;
use glam::Vec3;

#[test]
fn test_debug_lines_not_empty() {
    let mut world = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));

    // Create a simple dynamic body with a collider
    let _body_id = world.add_dynamic_box(
        Vec3::new(0.0, 5.0, 0.0),
        Vec3::new(1.0, 1.0, 1.0),
        1.0,
        Layers::DEFAULT,
    );

    // Get debug lines
    let lines = world.get_debug_lines();

    // Assert that we have some debug lines
    assert!(
        !lines.is_empty(),
        "Debug lines should not be empty when there are bodies in the world"
    );
}
