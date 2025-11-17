use astraweave_core::{IVec2, Team, World};
use aw_editor::headless::GizmoHarness;

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

#[test]
fn drag_commit_creates_single_undo_entry() {
    let (mut harness, entity) = harness_with_entity();
    harness.select(entity);
    harness.begin_translate().unwrap();

    harness.drag_translate(IVec2 { x: 2, y: 0 }).unwrap();
    harness.drag_translate(IVec2 { x: 1, y: -1 }).unwrap();
    harness.confirm().unwrap();

    assert_eq!(
        harness.undo_depth(),
        1,
        "only one undo entry should be recorded"
    );

    harness.undo_last().unwrap();
    assert_eq!(
        harness.world().pose(entity).unwrap().pos,
        IVec2 { x: 0, y: 0 }
    );

    harness.redo_last().unwrap();
    assert_eq!(
        harness.world().pose(entity).unwrap().pos,
        IVec2 { x: 3, y: -1 }
    );
}

#[test]
fn cancel_does_not_push_transaction() {
    let (mut harness, entity) = harness_with_entity();
    harness.select(entity);
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2 { x: -4, y: 7 }).unwrap();
    harness.cancel().unwrap();

    assert_eq!(
        harness.undo_depth(),
        0,
        "cancelled drags should not create undo entries"
    );
    assert_eq!(
        harness.world().pose(entity).unwrap().pos,
        IVec2 { x: 0, y: 0 }
    );
}
