//! Prefab workflow tests
//!
//! NOTE: Many tests are commented out because they depend on PrefabSpawnCommand,
//! PrefabApplyOverridesCommand, and PrefabRevertOverridesCommand which are not yet
//! implemented in the command module. These should be re-enabled when the prefab
//! command infrastructure is complete.

use astraweave_core::{IVec2, World};
use aw_editor_lib::prefab::{PrefabData, PrefabEntityData, PrefabManager};
use tempfile::tempdir;

fn sample_prefab() -> PrefabData {
    PrefabData {
        name: "TestPrefab".into(),
        entities: vec![PrefabEntityData {
            name: "CrateRoot".into(),
            pos_x: 0,
            pos_y: 0,
            team_id: 0,
            health: 100,
            max_health: 100,
            children_indices: Vec::new(),
            prefab_reference: None,
        }],
        root_entity_index: 0,
        version: "1.0".into(),
    }
}

// TODO: Re-enable when PrefabSpawnCommand is implemented
// #[test]
// fn prefab_spawn_command_instantiates_and_undoes() { ... }

// TODO: Re-enable when PrefabApplyOverridesCommand is implemented
// #[test]
// fn prefab_apply_overrides_updates_prefab_file_and_is_undoable() { ... }

/// Test that prefab data can be saved and loaded correctly
#[test]
fn prefab_data_roundtrip() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("test.prefab.ron");
    
    let original = sample_prefab();
    original.save_to_file(&prefab_path).expect("save prefab");
    
    let loaded = PrefabData::load_from_file(&prefab_path).expect("load prefab");
    assert_eq!(loaded.name, "TestPrefab");
    assert_eq!(loaded.entities.len(), 1);
    assert_eq!(loaded.entities[0].name, "CrateRoot");
}

/// Test that prefab manager can instantiate prefabs directly
#[test]
fn prefab_manager_instantiates_prefab() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("crate.prefab.ron");
    sample_prefab()
        .save_to_file(&prefab_path)
        .expect("write prefab");

    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();

    let root = {
        let mut mgr = manager.lock().expect("manager lock");
        mgr.instantiate_prefab(&prefab_path, &mut world, (4, -2))
            .expect("instantiate prefab")
    };

    let entities = world.entities();
    assert_eq!(entities.len(), 1, "instantiate creates a root entity");
    let pose = world.pose(root).expect("spawned pose");
    assert_eq!(pose.pos, IVec2::new(4, -2));

    {
        let mgr = manager.lock().expect("manager lock");
        assert!(
            mgr.find_instance(root).is_some(),
            "prefab manager tracks the spawned instance"
        );
    }
}

// TODO: Re-enable when capture_snapshot is implemented in PrefabManager
// #[test]
// fn prefab_revert_overrides_restores_world_pose_and_supports_undo() {
//     let temp = tempdir().expect("temp dir");
//     let prefab_path = temp.path().join("crate.prefab.ron");
//     sample_prefab()
//         .save_to_file(&prefab_path)
//         .expect("write prefab");
//
//     let manager = PrefabManager::shared(temp.path());
//     let mut world = World::new();
//
//     let root = {
//         let mut mgr = manager.lock().expect("manager lock");
//         mgr.instantiate_prefab(&prefab_path, &mut world, (0, 0))
//             .expect("instantiate prefab")
//     };
//
//     if let Some(pose) = world.pose_mut(root) {
//         pose.pos = IVec2::new(5, 3);
//     }
//
//     let mutated_pose = world.pose(root).expect("mutated pose").pos;
//     let snapshot = {
//         let mgr = manager.lock().expect("manager lock");
//         mgr.capture_snapshot(&world, root)
//             .expect("snapshot available")
//     };
//
//     let mut cmd = PrefabRevertOverridesCommand::new(manager.clone(), root, snapshot);
//     cmd.execute(&mut world).expect("revert overrides");
//     let reverted = world.pose(root).expect("reverted pose");
//     assert_eq!(reverted.pos, IVec2::new(0, 0));
//
//     cmd.undo(&mut world).expect("undo revert");
//     let restored = world.pose(root).expect("restored pose");
//     assert_eq!(restored.pos, mutated_pose);
// }

// TODO: Re-enable when spawn_prefab_with_undo is implemented
// #[test]
// fn spawn_prefab_helper_records_undo_entry() {
//     let temp = tempdir().expect("temp dir");
//     let prefab_path = temp.path().join("crate.prefab.ron");
//     sample_prefab()
//         .save_to_file(&prefab_path)
//         .expect("write prefab");
//
//     let manager = PrefabManager::shared(temp.path());
//     let mut world = World::new();
//     let mut undo_stack = UndoStack::new(8);
//
//     let root = spawn_prefab_with_undo(
//         manager.clone(),
//         prefab_path.clone(),
//         (7, 3),
//         &mut world,
//         &mut undo_stack,
//     )
//     .expect("spawn prefab with helper");
//
//     assert!(undo_stack.can_undo(), "helper pushes undo entry");
//     let pose = world.pose(root).expect("pose after spawn");
//     assert_eq!(pose.pos, IVec2::new(7, 3));
//
//     undo_stack.undo(&mut world).expect("undo drop");
//     let reverted = world.pose(root).expect("pose after undo");
//     assert_eq!(reverted.pos, IVec2::new(-10000, -10000));
// }

// TODO: Re-enable when track_override_snapshot is implemented in PrefabManager
// #[test]
// fn prefab_manager_tracks_override_from_snapshot() {
//     let temp = tempdir().expect("temp dir");
//     let prefab_path = temp.path().join("crate.prefab.ron");
//     sample_prefab()
//         .save_to_file(&prefab_path)
//         .expect("write prefab");
//
//     let manager = PrefabManager::shared(temp.path());
//     let mut world = World::new();
//
//     let root = {
//         let mut mgr = manager.lock().expect("manager lock");
//         mgr.instantiate_prefab(&prefab_path, &mut world, (0, 0))
//             .expect("instantiate prefab")
//     };
//
//     if let Some(pose) = world.pose_mut(root) {
//         pose.pos = IVec2::new(-4, 8);
//     }
//
//     let pose = world.pose(root);
//     let health = world.health(root);
//
//     {
//         let mut mgr = manager.lock().expect("manager lock");
//         mgr.track_override_snapshot(root, pose, health);
//         let instance = mgr.find_instance(root).expect("prefab instance");
//         assert!(instance.has_overrides(root));
//     }
// }
