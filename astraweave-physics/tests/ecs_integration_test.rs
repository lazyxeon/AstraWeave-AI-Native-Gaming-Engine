#[cfg(feature = "ecs")]
mod ecs_integration {
    use astraweave_ecs::App;
    use astraweave_physics::{PhysicsBodyComponent, PhysicsPlugin, PhysicsWorld};
    use astraweave_scene::Transform;
    use glam::Vec3;

    #[test]
    fn test_physics_ecs_integration_gravity() {
        // Setup App with PhysicsPlugin
        let mut app = App::new();
        app = app.add_plugin(PhysicsPlugin);

        // Create an entity with Transform and PhysicsBody
        let entity = app.world.spawn();
        app.world.insert(entity, Transform::default());

        // Add a dynamic box to physics world and link to entity
        let body_id = {
            let physics_world = app.world.get_resource_mut::<PhysicsWorld>().unwrap();
            physics_world.add_dynamic_box(
                Vec3::new(0.0, 10.0, 0.0),
                Vec3::new(0.5, 0.5, 0.5),
                1.0,
                astraweave_physics::Layers::DEFAULT,
            )
        };

        app.world.insert(entity, PhysicsBodyComponent(body_id));

        // Get initial position
        let initial_y = app.world.get::<Transform>(entity).unwrap().translation.y;

        // Run 100 physics steps using run_fixed
        app = app.run_fixed(100);

        // Get final position
        let final_transform = app.world.get::<Transform>(entity).unwrap();
        let final_y = final_transform.translation.y;

        // Assert that gravity has affected the body (y position decreased)
        assert!(
            final_y < initial_y,
            "Expected y position to decrease due to gravity. Initial: {}, Final: {}",
            initial_y,
            final_y
        );

        // Assert that the body has fallen significantly
        assert!(
            (initial_y - final_y) > 1.0,
            "Expected body to fall at least 1.0 units. Actual drop: {}",
            initial_y - final_y
        );
    }
}
