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
