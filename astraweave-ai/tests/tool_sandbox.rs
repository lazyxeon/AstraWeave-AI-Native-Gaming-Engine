// Unit test for Tool Sandbox error taxonomy and validation
use astraweave_ai::tool_sandbox::*;
use astraweave_core::WorldSnapshot;

#[test]
fn test_tool_error_taxonomy() {
    let err = ToolError::Cooldown;
    assert_eq!(format!("{:?}", err), "Cooldown");
}

#[test]
fn test_validate_tool_action_stub() {
    let world = WorldSnapshot::default();
    let result = validate_tool_action(1, ToolVerb::MoveTo, &world);
    assert!(result.is_ok());
}

#[test]
fn test_throw_respects_cooldown() {
    let mut world = WorldSnapshot::default();
    world.me.cooldowns.insert(crate::cooldowns::CooldownKey::from("throw:smoke"), 3.0);
    let res = validate_tool_action(1, ToolVerb::Throw, &world);
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert!(format!("{}", e).contains("cooldown"));
}

#[test]
fn test_cover_fire_requires_ammo() {
    let mut world = WorldSnapshot::default();
    world.me.ammo = 0;
    let res = validate_tool_action(1, ToolVerb::CoverFire, &world);
    assert!(res.is_err());
    let e = res.unwrap_err();
    assert!(format!("{}", e).contains("ammo"));
}
