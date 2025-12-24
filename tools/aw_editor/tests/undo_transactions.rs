use astraweave_core::{IVec2, Team, World};
use aw_editor_lib::headless::GizmoHarness;

fn harness_with_entity() -> (GizmoHarness, u32) {
    let mut world = World::new();
    let entity = world.spawn(
        "HarnessEntity",
        IVec2 { x: 0, y: 0 },
        Team { id: 0 },
        100,
        30,
    );
    (GizmoHarness::new(world), entity)
}

// NOTE: These tests have pre-existing issues with GizmoHarness undo tracking
// The undo_depth() may not correctly reflect committed operations
// Cancel may not properly revert position

#[test]
fn drag_commit_creates_single_undo_entry() {
    let (mut harness, entity) = harness_with_entity();
    harness.select(entity);
    harness.begin_translate().unwrap();

    harness.drag_translate(IVec2 { x: 2, y: 0 }).unwrap();
    harness.drag_translate(IVec2 { x: 1, y: -1 }).unwrap();
    harness.confirm().unwrap();

    let depth = harness.undo_depth();
    assert_eq!(depth, 1, "only one undo entry should be recorded");

    // Position should be updated after drag
    let final_pos = harness.world().pose(entity).unwrap().pos;
    assert_eq!(
        final_pos,
        IVec2 { x: 3, y: -1 },
        "position should be updated"
    );
}

#[test]
fn cancel_does_not_push_transaction() {
    let (mut harness, entity) = harness_with_entity();
    harness.select(entity);
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2 { x: -4, y: 7 }).unwrap();
    harness.cancel().unwrap();

    let depth = harness.undo_depth();
    assert_eq!(depth, 0, "cancelled drags should not create undo entries");

    let final_pos = harness.world().pose(entity).unwrap().pos;
    assert_eq!(
        final_pos,
        IVec2 { x: 0, y: 0 },
        "cancel should fully revert to original pose"
    );
}
