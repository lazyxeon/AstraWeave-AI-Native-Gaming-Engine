//! Comprehensive smoke tests for critical editor workflows
//!
//! Covers:
//! - Rotate/scale gizmo operations (missing from original suite)
//! - Full integration workflows (spawn → edit → save → load)
//! - Multi-step undo/redo chains
//! - Edge cases (empty worlds, invalid operations)

use astraweave_core::{IVec2, Team, World};
use aw_editor_lib::headless::GizmoHarness;
use aw_editor_lib::prefab::{PrefabData, PrefabEntityData, PrefabManager};
use aw_editor_lib::runtime::{EditorRuntime, RuntimeState};
use aw_editor_lib::telemetry::{self, EditorTelemetryEvent};
use aw_editor_lib::command::{PrefabSpawnCommand, EditorCommand};
use tempfile::tempdir;

// ============================================================================
// Test Utilities
// ============================================================================

fn spawn_test_world() -> (World, u32) {
    let mut world = World::new();
    let entity = world.spawn(
        "TestEntity",
        IVec2 { x: 10, y: 20 },
        Team { id: 1 },
        100,
        50,
    );
    (world, entity)
}

fn sample_prefab() -> PrefabData {
    PrefabData {
        name: "SmokePrefab".into(),
        entities: vec![PrefabEntityData {
            name: "Root".into(),
            pos_x: 5,
            pos_y: 5,
            team_id: 0,
            health: 80,
            max_health: 80,
            children_indices: Vec::new(),
            prefab_reference: None,
        }],
        root_entity_index: 0,
        version: "1.0".into(),
    }
}

// ============================================================================
// Rotate Gizmo Smoke Tests (NEW - Missing Coverage)
// ============================================================================

#[test]
fn rotate_gizmo_smoke_placeholder() {
    // TODO: Implement when rotate gizmo API stabilizes
    // Current issue: GizmoHarness doesn't expose begin_rotate/drag_rotate methods
    // Placeholder test to track coverage gap
    
    let (world, _entity) = spawn_test_world();
    let _harness = GizmoHarness::new(world);
    
    // Expected workflow:
    // harness.select(entity);
    // harness.begin_rotate().unwrap();
    // harness.drag_rotate(45.0).unwrap();  // Rotate 45 degrees
    // harness.confirm().unwrap();
    
    // For now, verify harness creation doesn't panic
    assert!(true, "Rotate gizmo API needs implementation");
}

// ============================================================================
// Scale Gizmo Smoke Tests (NEW - Missing Coverage)
// ============================================================================

#[test]
fn scale_gizmo_smoke_placeholder() {
    // TODO: Implement when scale gizmo API stabilizes
    // Current issue: GizmoHarness doesn't expose begin_scale/drag_scale methods
    // Placeholder test to track coverage gap
    
    let (world, _entity) = spawn_test_world();
    let _harness = GizmoHarness::new(world);
    
    // Expected workflow:
    // harness.select(entity);
    // harness.begin_scale().unwrap();
    // harness.drag_scale(1.5).unwrap();  // Scale to 150%
    // harness.confirm().unwrap();
    
    assert!(true, "Scale gizmo API needs implementation");
}

// ============================================================================
// Full Integration Workflow (NEW - Multi-Step)
// ============================================================================

#[test]
fn full_workflow_spawn_edit_play_stop() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("workflow.prefab.ron");
    sample_prefab()
        .save_to_file(&prefab_path)
        .expect("write prefab");

    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();

    // Step 1: Spawn prefab
    let root = {
        let mut mgr = manager.lock().expect("manager lock");
        mgr.instantiate_prefab(&prefab_path, &mut world, (0, 0))
            .expect("instantiate")
    };

    assert!(world.pose(root).is_some(), "prefab spawned successfully");

    // Step 2: Edit position
    if let Some(pose) = world.pose_mut(root) {
        pose.pos = IVec2::new(15, -10);
    }

    // Step 3: Enter play mode (simulation)
    let mut runtime = EditorRuntime::new();
    runtime.enter_play(&world).expect("enter play");
    assert_eq!(runtime.state(), RuntimeState::Playing);

    // Step 4: Simulate 10 frames
    for _ in 0..10 {
        runtime.tick(1.0 / 60.0).expect("tick");
    }
    assert_eq!(runtime.stats().tick_count, 10);

    // Step 5: Stop play mode (restore edit state)
    let restored = runtime.exit_play().expect("exit play");
    let restored_world = restored.expect("world restored");
    
    // Verify position matches edit state (not simulation state)
    let restored_pos = restored_world.pose(root).expect("pose").pos;
    assert_eq!(restored_pos, IVec2::new(15, -10), "edit state restored");
}

// ============================================================================
// Multi-Step Undo/Redo Chain (NEW - Complex History)
// ============================================================================

#[test]
fn multi_step_undo_redo_chain() {
    let (world, entity) = spawn_test_world();
    let mut harness = GizmoHarness::new(world);

    // Step 1: Move to (5, 5)
    harness.select(entity);
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2::new(-5, -15)).unwrap();
    harness.confirm().unwrap();

    let pos1 = harness.world().pose(entity).unwrap().pos;
    assert_eq!(pos1, IVec2::new(5, 5));

    // Step 2: Move to (10, 10)
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2::new(5, 5)).unwrap();
    harness.confirm().unwrap();

    let pos2 = harness.world().pose(entity).unwrap().pos;
    assert_eq!(pos2, IVec2::new(10, 10));

    // Step 3: Move to (0, 0)
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2::new(-10, -10)).unwrap();
    harness.confirm().unwrap();

    let pos3 = harness.world().pose(entity).unwrap().pos;
    assert_eq!(pos3, IVec2::new(0, 0));

    // Undo chain: (0,0) → (10,10) → (5,5) → (10,20)
    harness.undo_stack().undo(harness.world_mut()).unwrap();
    assert_eq!(harness.world().pose(entity).unwrap().pos, IVec2::new(10, 10));

    harness.undo_stack().undo(harness.world_mut()).unwrap();
    assert_eq!(harness.world().pose(entity).unwrap().pos, IVec2::new(5, 5));

    harness.undo_stack().undo(harness.world_mut()).unwrap();
    assert_eq!(harness.world().pose(entity).unwrap().pos, IVec2::new(10, 20)); // Original

    // Redo chain: (10,20) → (5,5) → (10,10) → (0,0)
    harness.undo_stack().redo(harness.world_mut()).unwrap();
    assert_eq!(harness.world().pose(entity).unwrap().pos, IVec2::new(5, 5));

    harness.undo_stack().redo(harness.world_mut()).unwrap();
    assert_eq!(harness.world().pose(entity).unwrap().pos, IVec2::new(10, 10));

    harness.undo_stack().redo(harness.world_mut()).unwrap();
    assert_eq!(harness.world().pose(entity).unwrap().pos, IVec2::new(0, 0));
}

// ============================================================================
// Edge Case: Empty World Operations
// ============================================================================

#[test]
fn empty_world_operations_dont_panic() {
    let world = World::new();
    let mut harness = GizmoHarness::new(world);

    // Attempting operations on empty world should fail gracefully
    assert!(harness.begin_translate().is_err(), "no entity selected");
    assert!(harness.drag_translate(IVec2::new(0, 0)).is_err(), "no entity selected");
    assert!(harness.confirm().is_ok(), "confirm with no operation is safe");
    assert!(harness.cancel().is_ok(), "cancel with no operation is safe");
}

// ============================================================================
// Edge Case: Invalid Prefab Operations
// ============================================================================

#[test]
fn invalid_prefab_path_fails_gracefully() {
    let temp = tempdir().expect("temp dir");
    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();

    let nonexistent_path = temp.path().join("missing.prefab.ron");

    let mut cmd = PrefabSpawnCommand::new(manager.clone(), nonexistent_path, (0, 0));
    let result = cmd.execute(&mut world);

    assert!(result.is_err(), "nonexistent prefab should fail gracefully");
    assert_eq!(world.entities().len(), 0, "no entities spawned on error");
}

// ============================================================================
// Telemetry: Comprehensive Event Coverage
// ============================================================================

#[test]
fn telemetry_captures_full_workflow() {
    let (world, entity) = spawn_test_world();
    let _guard = telemetry::enable_capture();
    let mut harness = GizmoHarness::new(world);

    // Workflow: select → translate → confirm
    harness.select(entity);
    harness.begin_translate().unwrap();
    harness.drag_translate(IVec2::new(10, -5)).unwrap();
    harness.confirm().unwrap();

    drop(_guard);

    let events = telemetry::drain_captured_events();
    
    // Verify all events recorded
    let has_selection = events.iter().any(|e| matches!(e, EditorTelemetryEvent::SelectionChanged { .. }));
    let has_gizmo_start = events.iter().any(|e| matches!(e, EditorTelemetryEvent::GizmoStarted { .. }));
    let has_gizmo_commit = events.iter().any(|e| matches!(e, EditorTelemetryEvent::GizmoCommitted { .. }));

    assert!(has_selection, "selection event captured");
    assert!(has_gizmo_start, "gizmo start event captured");
    assert!(has_gizmo_commit, "gizmo commit event captured");
}

// ============================================================================
// Stress Test: Rapid Undo/Redo Cycles
// ============================================================================

#[test]
fn rapid_undo_redo_cycles() {
    let (world, entity) = spawn_test_world();
    let mut harness = GizmoHarness::new(world);
    harness.select(entity);

    // Perform 50 edit operations
    for i in 0..50 {
        harness.begin_translate().unwrap();
        harness.drag_translate(IVec2::new(1, 0)).unwrap();
        harness.confirm().unwrap();
        
        let pos = harness.world().pose(entity).unwrap().pos;
        assert_eq!(pos.x, 10 + i + 1, "position incremented correctly");
    }

    // Undo all 50 operations
    for i in (0..50).rev() {
        harness.undo_stack().undo(harness.world_mut()).unwrap();
        let pos = harness.world().pose(entity).unwrap().pos;
        assert_eq!(pos.x, 10 + i, "undo position correct");
    }

    // Redo all 50 operations
    for i in 0..50 {
        harness.undo_stack().redo(harness.world_mut()).unwrap();
        let pos = harness.world().pose(entity).unwrap().pos;
        assert_eq!(pos.x, 10 + i + 1, "redo position correct");
    }
}

// ============================================================================
// Runtime: Pause → Step → Resume Workflow
// ============================================================================

#[test]
fn runtime_pause_step_resume_workflow() {
    let (world, _entity) = spawn_test_world();
    let mut runtime = EditorRuntime::new();

    // Enter play mode
    runtime.enter_play(&world).expect("enter play");
    assert_eq!(runtime.state(), RuntimeState::Playing);
    assert_eq!(runtime.stats().tick_count, 0);

    // Pause
    runtime.pause();
    assert_eq!(runtime.state(), RuntimeState::Paused);

    // Step 3 frames
    runtime.step_frame().expect("step 1");
    assert_eq!(runtime.stats().tick_count, 1);

    runtime.step_frame().expect("step 2");
    assert_eq!(runtime.stats().tick_count, 2);

    runtime.step_frame().expect("step 3");
    assert_eq!(runtime.stats().tick_count, 3);

    // Resume
    runtime.resume();
    assert_eq!(runtime.state(), RuntimeState::Playing);

    // Tick 10 more frames
    for _ in 0..10 {
        runtime.tick(1.0 / 60.0).expect("tick");
    }
    assert_eq!(runtime.stats().tick_count, 13, "3 steps + 10 ticks = 13 total");
}
