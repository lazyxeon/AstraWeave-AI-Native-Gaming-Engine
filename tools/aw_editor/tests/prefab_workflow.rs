use astraweave_core::{IVec2, Team, World};
use aw_editor_lib::command::{
    spawn_prefab_with_undo, PrefabApplyOverridesCommand, PrefabRevertOverridesCommand,
    PrefabSpawnCommand, UndoStack,
};
use aw_editor_lib::prefab::{PrefabData, PrefabEntityData, PrefabManager};
use aw_editor_lib::EditorCommand;
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

#[test]
fn prefab_spawn_command_instantiates_and_undoes() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("crate.prefab.ron");
    sample_prefab()
        .save_to_file(&prefab_path)
        .expect("write prefab");

    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();

    let mut cmd = PrefabSpawnCommand::new(manager.clone(), prefab_path, (4, -2));
    cmd.execute(&mut world).expect("spawn prefab");

    let entities = world.entities();
    assert_eq!(entities.len(), 1, "spawn creates a root entity");
    let root = entities[0];
    let pose = world.pose(root).expect("spawned pose");
    assert_eq!(pose.pos, IVec2::new(4, -2));

    {
        let mgr = manager.lock().expect("manager lock");
        assert!(
            mgr.find_instance(root).is_some(),
            "prefab manager tracks the spawned instance"
        );
    }

    cmd.undo(&mut world).expect("undo spawn");
    let pose = world.pose(root).expect("pose after undo");
    assert_eq!(pose.pos, IVec2::new(-10000, -10000));
}

#[test]
fn prefab_apply_overrides_updates_prefab_file_and_is_undoable() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("crate.prefab.ron");
    sample_prefab()
        .save_to_file(&prefab_path)
        .expect("write prefab");

    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();

    let root = {
        let mut mgr = manager.lock().expect("manager lock");
        mgr.instantiate_prefab(&prefab_path, &mut world, (0, 0))
            .expect("instantiate prefab")
    };

    if let Some(pose) = world.pose_mut(root) {
        pose.pos = IVec2::new(12, -7);
    }

    let previous_prefab = PrefabData::load_from_file(&prefab_path).expect("load prefab");
    let mut stack = UndoStack::new(8);
    stack
        .execute(
            PrefabApplyOverridesCommand::new(
                manager.clone(),
                root,
                prefab_path.clone(),
                previous_prefab,
            ),
            &mut world,
        )
        .expect("apply overrides");

    let updated = PrefabData::load_from_file(&prefab_path).expect("updated prefab");
    assert_eq!(updated.entities[0].pos_x, 12);
    assert_eq!(updated.entities[0].pos_y, -7);

    stack.undo(&mut world).expect("undo apply");
    let restored = PrefabData::load_from_file(&prefab_path).expect("restored prefab");
    assert_eq!(restored.entities[0].pos_x, 0);
    assert_eq!(restored.entities[0].pos_y, 0);
}

#[test]
fn prefab_revert_overrides_restores_world_pose_and_supports_undo() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("crate.prefab.ron");
    sample_prefab()
        .save_to_file(&prefab_path)
        .expect("write prefab");

    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();

    let root = {
        let mut mgr = manager.lock().expect("manager lock");
        mgr.instantiate_prefab(&prefab_path, &mut world, (0, 0))
            .expect("instantiate prefab")
    };

    if let Some(pose) = world.pose_mut(root) {
        pose.pos = IVec2::new(5, 3);
    }

    let mutated_pose = world.pose(root).expect("mutated pose").pos;
    let snapshot = {
        let mgr = manager.lock().expect("manager lock");
        mgr.capture_snapshot(&world, root)
            .expect("snapshot available")
    };

    let mut cmd = PrefabRevertOverridesCommand::new(manager.clone(), root, snapshot);
    cmd.execute(&mut world).expect("revert overrides");
    let reverted = world.pose(root).expect("reverted pose");
    assert_eq!(reverted.pos, IVec2::new(0, 0));

    cmd.undo(&mut world).expect("undo revert");
    let restored = world.pose(root).expect("restored pose");
    assert_eq!(restored.pos, mutated_pose);
}

#[test]
fn spawn_prefab_helper_records_undo_entry() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("crate.prefab.ron");
    sample_prefab()
        .save_to_file(&prefab_path)
        .expect("write prefab");

    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();
    let mut undo_stack = UndoStack::new(8);

    let root = spawn_prefab_with_undo(
        manager.clone(),
        prefab_path.clone(),
        (7, 3),
        &mut world,
        &mut undo_stack,
    )
    .expect("spawn prefab with helper");

    assert!(undo_stack.can_undo(), "helper pushes undo entry");
    let pose = world.pose(root).expect("pose after spawn");
    assert_eq!(pose.pos, IVec2::new(7, 3));

    undo_stack.undo(&mut world).expect("undo drop");
    let reverted = world.pose(root).expect("pose after undo");
    assert_eq!(reverted.pos, IVec2::new(-10000, -10000));
}

#[test]
fn prefab_manager_tracks_override_from_snapshot() {
    let temp = tempdir().expect("temp dir");
    let prefab_path = temp.path().join("crate.prefab.ron");
    sample_prefab()
        .save_to_file(&prefab_path)
        .expect("write prefab");

    let manager = PrefabManager::shared(temp.path());
    let mut world = World::new();

    let root = {
        let mut mgr = manager.lock().expect("manager lock");
        mgr.instantiate_prefab(&prefab_path, &mut world, (0, 0))
            .expect("instantiate prefab")
    };

    if let Some(pose) = world.pose_mut(root) {
        pose.pos = IVec2::new(-4, 8);
    }

    let pose = world.pose(root);
    let health = world.health(root);

    {
        let mut mgr = manager.lock().expect("manager lock");
        mgr.track_override_snapshot(root, pose, health);
        let instance = mgr.find_instance(root).expect("prefab instance");
        assert!(instance.has_overrides(root));
    }
}
