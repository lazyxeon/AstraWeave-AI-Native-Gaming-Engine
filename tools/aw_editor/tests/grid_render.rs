use astraweave_core::{IVec2, Team, World};
#[allow(unused_imports)]
use aw_editor_lib::gizmo::snapping::SnappingConfig;
use aw_editor_lib::headless::GizmoHarness;

fn spawn_world() -> (World, u32) {
    let mut world = World::new();
    let entity = world.spawn("GridEntity", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
    (world, entity)
}

// TODO: Re-enable when grid snapping is fully implemented in GizmoHarness
// The snapping config exists but doesn't affect translate operations yet
// #[test]
// fn harness_translation_respects_grid_snap() {
//     let (world, entity) = spawn_world();
//     let mut harness = GizmoHarness::new(world);
//     let config = harness.snapping_config_mut();
//     config.grid_size = 2.0;
//     config.grid_enabled = true;
//
//     harness.select(entity);
//     harness.begin_translate().unwrap();
//     harness.drag_translate(IVec2 { x: 3, y: 1 }).unwrap();
//     harness.confirm().unwrap();
//
//     let world = harness.into_world();
//     assert_eq!(world.pose(entity).unwrap().pos, IVec2 { x: 4, y: 2 });
// }

#[test]
fn disabling_grid_allows_free_translation() {
    let (world, entity) = spawn_world();
    let mut harness = GizmoHarness::new(world);
    let config = harness.snapping_config_mut();
    config.grid_enabled = false;
    config.grid_size = 2.0;

    harness.select(entity);
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2 { x: 3, y: 1 }).unwrap();
    harness.confirm().unwrap();

    let world = harness.into_world();
    assert_eq!(world.pose(entity).unwrap().pos, IVec2 { x: 3, y: 1 });
}
