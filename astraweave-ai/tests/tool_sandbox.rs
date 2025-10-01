// Unit test for Tool Sandbox error taxonomy and validation
use astraweave_ai::tool_sandbox::*;
use astraweave_core::{IVec2, WorldSnapshot};

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
