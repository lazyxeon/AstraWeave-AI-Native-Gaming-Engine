use astraweave_core::{Entity, IVec2, Team, World};
use aw_editor::command::{
    EditAmmoCommand, EditHealthCommand, EditTeamCommand, MoveEntityCommand, RotateEntityCommand,
    ScaleEntityCommand, UndoStack,
};
use aw_editor::component_ui::{ComponentRegistry, ComponentType};
use aw_editor::scene_serialization::{load_scene, save_scene, SceneData};
use std::fs;
use std::path::{Path, PathBuf};

fn integration_scene_path(file_name: &str) -> PathBuf {
    let relative = PathBuf::from(format!("test_scenes/integration/{file_name}"));
    let absolute = Path::new("content").join(&relative);
    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    relative
}

fn integration_scene_exists(path: &Path) -> bool {
    Path::new("content").join(path).exists()
}

fn remove_integration_scene(path: &Path) {
    let _ = fs::remove_file(Path::new("content").join(path));
}

#[test]
fn test_full_editor_workflow_with_undo_and_save() {
    let mut world = World::new();
    let entity1 = world.spawn("Player", IVec2::new(10, 20), Team { id: 0 }, 100, 30);
    let entity2 = world.spawn("Enemy", IVec2::new(50, 60), Team { id: 2 }, 50, 15);

    let mut undo_stack = UndoStack::new(100);
    undo_stack.set_auto_merge(false);

    undo_stack
        .execute(
            MoveEntityCommand::new(entity1, IVec2::new(10, 20), IVec2::new(15, 25)),
            &mut world,
        )
        .unwrap();
    assert_eq!(world.pose(entity1).unwrap().pos, IVec2::new(15, 25));

    undo_stack
        .execute(
            RotateEntityCommand::new(entity1, (0.0, 0.0, 0.0), (0.0, 1.57, 0.0)),
            &mut world,
        )
        .unwrap();
    assert!((world.pose(entity1).unwrap().rotation - 1.57).abs() < 0.01);

    undo_stack
        .execute(EditHealthCommand::new(entity2, 50, 25), &mut world)
        .unwrap();
    assert_eq!(world.health(entity2).unwrap().hp, 25);

    let scene_path = integration_scene_path("integration_test_scene.ron");
    save_scene(&world, &scene_path).unwrap();
    assert!(integration_scene_exists(&scene_path));

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.health(entity2).unwrap().hp, 50);

    undo_stack.undo(&mut world).unwrap();
    assert!((world.pose(entity1).unwrap().rotation - 0.0).abs() < 0.01);

    undo_stack.redo(&mut world).unwrap();
    assert!((world.pose(entity1).unwrap().rotation - 1.57).abs() < 0.01);

    let loaded_world = load_scene(&scene_path).unwrap();
    assert_eq!(loaded_world.entities().len(), 2);
    assert_eq!(loaded_world.pose(entity1).unwrap().pos, IVec2::new(15, 25));
    assert!((loaded_world.pose(entity1).unwrap().rotation - 1.57).abs() < 0.01);

    remove_integration_scene(&scene_path);
}

#[test]
fn test_component_inspector_workflow() {
    let mut world = World::new();
    let entity = world.spawn("TestEntity", IVec2::new(0, 0), Team { id: 0 }, 100, 30);

    let registry = ComponentRegistry::new();
    let components = registry.get_entity_components(&world, entity);

    assert_eq!(components.len(), 4);
    assert!(components.contains(&ComponentType::Pose));
    assert!(components.contains(&ComponentType::Health));
    assert!(components.contains(&ComponentType::Team));
    assert!(components.contains(&ComponentType::Ammo));

    for component_type in components {
        assert!(component_type.has_component(&world, entity));
    }

    if let Some(health) = world.health_mut(entity) {
        health.hp = 50;
    }
    assert_eq!(world.health(entity).unwrap().hp, 50);

    if let Some(team) = world.team_mut(entity) {
        team.id = 2;
    }
    assert_eq!(world.team(entity).unwrap().id, 2);
}

#[test]
fn test_undo_redo_with_multiple_entity_types() {
    let mut world = World::new();
    let player = world.spawn("Player", IVec2::new(0, 0), Team { id: 0 }, 100, 30);
    let enemy1 = world.spawn("Enemy1", IVec2::new(10, 10), Team { id: 2 }, 50, 15);
    let enemy2 = world.spawn("Enemy2", IVec2::new(20, 20), Team { id: 2 }, 50, 15);

    let mut undo_stack = UndoStack::new(100);

    undo_stack
        .execute(
            MoveEntityCommand::new(player, IVec2::new(0, 0), IVec2::new(5, 5)),
            &mut world,
        )
        .unwrap();

    undo_stack
        .execute(
            MoveEntityCommand::new(enemy1, IVec2::new(10, 10), IVec2::new(15, 15)),
            &mut world,
        )
        .unwrap();

    undo_stack
        .execute(ScaleEntityCommand::new(enemy2, 1.0, 2.0), &mut world)
        .unwrap();

    assert_eq!(world.pose(player).unwrap().pos, IVec2::new(5, 5));
    assert_eq!(world.pose(enemy1).unwrap().pos, IVec2::new(15, 15));
    assert!((world.pose(enemy2).unwrap().scale - 2.0).abs() < 0.01);

    undo_stack.undo(&mut world).unwrap();
    assert!((world.pose(enemy2).unwrap().scale - 1.0).abs() < 0.01);

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.pose(enemy1).unwrap().pos, IVec2::new(10, 10));

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.pose(player).unwrap().pos, IVec2::new(0, 0));

    undo_stack.redo(&mut world).unwrap();
    undo_stack.redo(&mut world).unwrap();
    undo_stack.redo(&mut world).unwrap();

    assert_eq!(world.pose(player).unwrap().pos, IVec2::new(5, 5));
    assert_eq!(world.pose(enemy1).unwrap().pos, IVec2::new(15, 15));
    assert!((world.pose(enemy2).unwrap().scale - 2.0).abs() < 0.01);
}

#[test]
fn test_scene_save_load_preserves_undo_capability() {
    let mut world = World::new();
    let entity = world.spawn("Player", IVec2::new(10, 10), Team { id: 0 }, 100, 30);

    if let Some(pose) = world.pose_mut(entity) {
        pose.rotation = 1.57;
        pose.scale = 2.0;
    }

    let scene_path = integration_scene_path("undo_test_scene.ron");

    let scene = SceneData::from_world(&world);
    scene.save_to_file(&scene_path).unwrap();

    let mut loaded_world = load_scene(&scene_path).unwrap();
    let mut undo_stack = UndoStack::new(100);
    undo_stack.set_auto_merge(false);

    undo_stack
        .execute(
            MoveEntityCommand::new(entity, IVec2::new(10, 10), IVec2::new(20, 20)),
            &mut loaded_world,
        )
        .unwrap();

    assert_eq!(loaded_world.pose(entity).unwrap().pos, IVec2::new(20, 20));

    undo_stack.undo(&mut loaded_world).unwrap();
    assert_eq!(loaded_world.pose(entity).unwrap().pos, IVec2::new(10, 10));

    remove_integration_scene(&scene_path);
}

#[test]
fn test_component_edits_with_undo_stack() {
    let mut world = World::new();
    let entity = world.spawn("TestEntity", IVec2::new(0, 0), Team { id: 0 }, 100, 30);

    let mut undo_stack = UndoStack::new(100);

    undo_stack
        .execute(EditHealthCommand::new(entity, 100, 75), &mut world)
        .unwrap();
    undo_stack
        .execute(
            EditTeamCommand::new(entity, Team { id: 0 }, Team { id: 1 }),
            &mut world,
        )
        .unwrap();
    undo_stack
        .execute(EditAmmoCommand::new(entity, 30, 20), &mut world)
        .unwrap();

    assert_eq!(world.health(entity).unwrap().hp, 75);
    assert_eq!(world.team(entity).unwrap().id, 1);
    assert_eq!(world.ammo(entity).unwrap().rounds, 20);

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.ammo(entity).unwrap().rounds, 30);

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.team(entity).unwrap().id, 0);

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.health(entity).unwrap().hp, 100);

    undo_stack.redo(&mut world).unwrap();
    undo_stack.redo(&mut world).unwrap();
    undo_stack.redo(&mut world).unwrap();

    assert_eq!(world.health(entity).unwrap().hp, 75);
    assert_eq!(world.team(entity).unwrap().id, 1);
    assert_eq!(world.ammo(entity).unwrap().rounds, 20);
}

#[test]
fn test_complex_scene_with_obstacles_and_undo() {
    let mut world = World::new();
    let entity1 = world.spawn("E1", IVec2::new(0, 0), Team { id: 0 }, 100, 30);
    let entity2 = world.spawn("E2", IVec2::new(10, 10), Team { id: 1 }, 50, 15);

    world.obstacles.insert((5, 5));
    world.obstacles.insert((6, 6));
    world.obstacles.insert((7, 7));

    let mut undo_stack = UndoStack::new(100);
    undo_stack
        .execute(
            MoveEntityCommand::new(entity1, IVec2::new(0, 0), IVec2::new(1, 1)),
            &mut world,
        )
        .unwrap();

    let scene_path = integration_scene_path("complex_scene_test.ron");
    save_scene(&world, &scene_path).unwrap();

    let loaded_world = load_scene(&scene_path).unwrap();

    assert_eq!(loaded_world.entities().len(), 2);
    assert_eq!(loaded_world.obstacles.len(), 3);
    assert!(loaded_world.obstacle(IVec2::new(5, 5)));
    assert!(loaded_world.obstacle(IVec2::new(6, 6)));
    assert!(loaded_world.obstacle(IVec2::new(7, 7)));

    remove_integration_scene(&scene_path);
}

#[test]
fn test_undo_stack_branching_preserves_state() {
    let mut world = World::new();
    let entity = world.spawn("Player", IVec2::new(0, 0), Team { id: 0 }, 100, 30);

    let mut undo_stack = UndoStack::new(100);
    undo_stack.set_auto_merge(false);

    undo_stack
        .execute(
            MoveEntityCommand::new(entity, IVec2::new(0, 0), IVec2::new(10, 10)),
            &mut world,
        )
        .unwrap();

    undo_stack
        .execute(
            MoveEntityCommand::new(entity, IVec2::new(10, 10), IVec2::new(20, 20)),
            &mut world,
        )
        .unwrap();

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(10, 10));

    undo_stack
        .execute(
            MoveEntityCommand::new(entity, IVec2::new(10, 10), IVec2::new(30, 30)),
            &mut world,
        )
        .unwrap();

    assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(30, 30));
    assert!(!undo_stack.can_redo());

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(10, 10));

    undo_stack.undo(&mut world).unwrap();
    assert_eq!(world.pose(entity).unwrap().pos, IVec2::new(0, 0));
}
