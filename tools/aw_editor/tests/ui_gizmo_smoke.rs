use astraweave_core::{IVec2, Team, World};
use aw_editor::headless::GizmoHarness;
use aw_editor::interaction::GizmoMeasurement;
use aw_editor::telemetry::{self, EditorTelemetryEvent};

fn spawn_test_world() -> (World, u32) {
    let mut world = World::new();
    let entity = world.spawn(
        "HarnessEntity",
        IVec2 { x: 0, y: 0 },
        Team { id: 0 },
        100,
        30,
    );
    (world, entity)
}

#[test]
fn translate_drag_records_commit_event() {
    let (world, entity) = spawn_test_world();
    let _guard = telemetry::enable_capture();
    let mut harness = GizmoHarness::new(world);
    harness.select(entity);
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2 { x: 5, y: -2 }).unwrap();
    harness.confirm().unwrap();

    assert_eq!(
        harness.undo_depth(),
        1,
        "gizmo commit should record one undo"
    );

    let world = harness.into_world();
    assert_eq!(world.pose(entity).unwrap().pos, IVec2 { x: 5, y: -2 });

    let events = telemetry::drain_captured_events();
    assert!(events.iter().any(|event| matches!(event, EditorTelemetryEvent::GizmoStarted { entity: id, .. } if *id == entity as u32)));
    assert!(events.iter().any(|event| matches!(event,
        EditorTelemetryEvent::GizmoCommitted {
            measurement: GizmoMeasurement::Translate { from, to }, ..
        } if *from == IVec2 { x: 0, y: 0 } && *to == IVec2 { x: 5, y: -2 }
    )));
}

#[test]
fn cancel_reverts_world_and_emits_event() {
    let (world, entity) = spawn_test_world();
    let _guard = telemetry::enable_capture();
    let mut harness = GizmoHarness::new(world);
    harness.select(entity);
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2 { x: -3, y: 4 }).unwrap();
    harness.cancel().unwrap();

    assert_eq!(
        harness.undo_depth(),
        0,
        "cancelled drags must not record undo"
    );

    let world = harness.into_world();
    assert_eq!(world.pose(entity).unwrap().pos, IVec2 { x: 0, y: 0 });

    let events = telemetry::drain_captured_events();
    assert!(events.iter().any(|event| matches!(event, EditorTelemetryEvent::GizmoCancelled { entity: id, .. } if *id == entity as u32)));
}
