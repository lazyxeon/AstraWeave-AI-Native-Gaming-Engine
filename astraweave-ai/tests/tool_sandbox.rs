// Unit test for Tool Sandbox error taxonomy and validation
use astraweave_ai::tool_sandbox::*;
use astraweave_core::{IVec2, WorldSnapshot};
use rapier3d::prelude::*;

#[test]
fn test_tool_error_taxonomy() {
    let err = ToolError::Cooldown;
    assert_eq!(format!("{:?}", err), "Cooldown");
}

#[test]
fn test_validate_tool_action_stub() {
    let world = WorldSnapshot::default();
    let context = ValidationContext::new();
    let result = validate_tool_action(
        1,
        ToolVerb::MoveTo,
        &world,
        &context,
        Some(IVec2 { x: 1, y: 0 }),
    );
    assert!(result.is_ok());
}

#[test]
fn test_throw_respects_cooldown() {
    let mut world = WorldSnapshot::default();
    world.me.cooldowns.insert("throw".to_string(), 3.0);
    let context = ValidationContext::new();
    let res = validate_tool_action(
        1,
        ToolVerb::Throw,
        &world,
        &context,
        Some(IVec2 { x: 1, y: 0 }),
    );
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert!(format!("{}", e).contains("cooldown"));
}

#[test]
fn test_cover_fire_requires_ammo() {
    let mut world = WorldSnapshot::default();
    world.me.ammo = 0;
    let context = ValidationContext::new();
    let res = validate_tool_action(
        1,
        ToolVerb::CoverFire,
        &world,
        &context,
        Some(IVec2 { x: 1, y: 0 }),
    );
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert!(format!("{}", e).contains("ammo"));
}

#[test]
fn test_move_to_blocked_by_locked_door() {
    use rapier3d::prelude::*;
    
    let world = WorldSnapshot::default();
    
    // Create physics world with a static collider at (5, 0, 0) representing a locked door
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();
    
    let rigid_body = RigidBodyBuilder::fixed()
        .translation(vector![5.0, 0.0, 0.0])
        .build();
    let body_handle = rigid_body_set.insert(rigid_body);
    
    let collider = ColliderBuilder::cuboid(0.5, 1.0, 0.5).build();
    collider_set.insert_with_parent(collider, body_handle, &mut rigid_body_set);
    
    let physics_pipeline = PhysicsPipeline::new();
    
    let context = ValidationContext::new().with_physics(
        &physics_pipeline,
        &rigid_body_set,
        &collider_set,
    );
    
    // Try to move to the locked door position
    let res = validate_tool_action(
        1,
        ToolVerb::MoveTo,
        &world,
        &context,
        Some(IVec2 { x: 5, y: 0 }),
    );
    
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert!(format!("{}", e).contains("physics collision") || format!("{}", e).contains("blocked"));
}

#[test]
fn test_move_to_succeeds_with_unlocked_door() {
    let world = WorldSnapshot::default();
    
    // No physics colliders - representing an unlocked/open door
    let rigid_body_set = RigidBodySet::new();
    let collider_set = ColliderSet::new();
    let physics_pipeline = PhysicsPipeline::new();
    
    let context = ValidationContext::new().with_physics(
        &physics_pipeline,
        &rigid_body_set,
        &collider_set,
    );
    
    // Try to move to a position with no colliders
    let res = validate_tool_action(
        1,
        ToolVerb::MoveTo,
        &world,
        &context,
        Some(IVec2 { x: 5, y: 0 }),
    );
    
    // Should succeed since there are no colliders blocking the path
    assert!(res.is_ok());
}

#[test]
fn test_move_to_far_from_locked_door() {
    use rapier3d::prelude::*;
    
    let world = WorldSnapshot::default();
    
    // Place a static collider at (10, 0, 10) representing a locked door
    let mut rigid_body_set = RigidBodySet::new();
    let mut collider_set = ColliderSet::new();
    
    let rigid_body = RigidBodyBuilder::fixed()
        .translation(vector![10.0, 0.0, 10.0])
        .build();
    let body_handle = rigid_body_set.insert(rigid_body);
    
    let collider = ColliderBuilder::cuboid(0.5, 1.0, 0.5).build();
    collider_set.insert_with_parent(collider, body_handle, &mut rigid_body_set);
    
    let physics_pipeline = PhysicsPipeline::new();
    
    let context = ValidationContext::new().with_physics(
        &physics_pipeline,
        &rigid_body_set,
        &collider_set,
    );
    
    // Try to move to a position far from the locked door
    let res = validate_tool_action(
        1,
        ToolVerb::MoveTo,
        &world,
        &context,
        Some(IVec2 { x: 1, y: 1 }),
    );
    
    // Should succeed since we're not moving to the locked door position
    assert!(res.is_ok());
}
