use astraweave_core::{Entity, IVec2, Team, World};
use aw_editor_lib::command::{
    EditAmmoCommand, EditHealthCommand, EditTeamCommand, MoveEntityCommand, RotateEntityCommand,
    ScaleEntityCommand, UndoStack,
};
use aw_editor_lib::component_ui::{ComponentRegistry, ComponentType};
use aw_editor_lib::scene_serialization::{load_scene, save_scene, SceneData};
use std::env;
use std::fs;

#[test]
fn test_full_editor_workflow_with_undo_and_save() {
    let mut world = World::new();
    let entity1 = world.spawn("Player", IVec2::new(10, 20), Team { id: 0 }, 100, 30);
    let entity2 = world.spawn("Enemy", IVec2::new(50, 60), Team { id: 2 }, 50, 15);

    let mut undo_stack = UndoStack::new(100);

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

    undo_stack.push_executed(EditHealthCommand::new(entity2, 50, 25));
    assert_eq!(world.health(entity2).unwrap().hp, 25);

    let temp_dir = env::temp_dir();
    let scene_path = temp_dir.join("integration_test_scene.ron");
    save_scene(&world, &scene_path).unwrap();
    assert!(scene_path.exists());

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

    fs::remove_file(&scene_path).unwrap();
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

    let temp_dir = env::temp_dir();
    let scene_path = temp_dir.join("undo_test_scene.ron");

    let scene = SceneData::from_world(&world);
    scene.save_to_file(&scene_path).unwrap();

    let mut loaded_world = load_scene(&scene_path).unwrap();
    let mut undo_stack = UndoStack::new(100);

    undo_stack
        .execute(
            MoveEntityCommand::new(entity, IVec2::new(10, 10), IVec2::new(20, 20)),
            &mut loaded_world,
        )
        .unwrap();

    assert_eq!(loaded_world.pose(entity).unwrap().pos, IVec2::new(20, 20));

    undo_stack.undo(&mut loaded_world).unwrap();
    assert_eq!(loaded_world.pose(entity).unwrap().pos, IVec2::new(10, 10));

    fs::remove_file(&scene_path).unwrap();
}

#[test]
fn test_component_edits_with_undo_stack() {
    let mut world = World::new();
    let entity = world.spawn("TestEntity", IVec2::new(0, 0), Team { id: 0 }, 100, 30);

    let mut undo_stack = UndoStack::new(100);

    undo_stack.push_executed(EditHealthCommand::new(entity, 100, 75));
    undo_stack.push_executed(EditTeamCommand::new(entity, Team { id: 0 }, Team { id: 1 }));
    undo_stack.push_executed(EditAmmoCommand::new(entity, 30, 20));

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

    let temp_dir = env::temp_dir();
    let scene_path = temp_dir.join("complex_scene_test.ron");
    save_scene(&world, &scene_path).unwrap();

    let loaded_world = load_scene(&scene_path).unwrap();

    assert_eq!(loaded_world.entities().len(), 2);
    assert_eq!(loaded_world.obstacles.len(), 3);
    assert!(loaded_world.obstacle(IVec2::new(5, 5)));
    assert!(loaded_world.obstacle(IVec2::new(6, 6)));
    assert!(loaded_world.obstacle(IVec2::new(7, 7)));

    fs::remove_file(&scene_path).unwrap();
}

#[test]
fn test_undo_stack_branching_preserves_state() {
    let mut world = World::new();
    let entity = world.spawn("Player", IVec2::new(0, 0), Team { id: 0 }, 100, 30);

    let mut undo_stack = UndoStack::new(100);

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

// ============================================================================
// Prefab Override System Tests (Week 5)
// ============================================================================

#[test]
fn test_apply_clears_overrides() {
    use aw_editor_lib::prefab::{PrefabData, PrefabInstance};
    use std::collections::HashMap;

    let mut world = World::new();
    let entity = world.spawn("TestEntity", IVec2::new(10, 20), Team { id: 0 }, 100, 30);

    // Create a simple prefab
    let prefab_data = PrefabData::from_entity(&world, entity, "TestPrefab".to_string()).unwrap();
    let temp_dir = env::temp_dir();
    let prefab_path = temp_dir.join("test_apply_clears_overrides.prefab.ron");
    prefab_data.save_to_file(&prefab_path).unwrap();

    // Create prefab instance with entity mapping
    let mut entity_mapping = HashMap::new();
    entity_mapping.insert(0, entity);
    
    let mut instance = PrefabInstance {
        source: prefab_path.clone(),
        root_entity: entity,
        entity_mapping,
        overrides: HashMap::new(),
    };

    // Track override (modify position)
    instance.track_override(entity, &world);
    assert!(instance.has_overrides(entity), "Entity should have overrides after tracking");
    assert_eq!(instance.overrides.len(), 1, "Should have 1 entity with overrides");

    // Modify entity further
    if let Some(pose) = world.pose_mut(entity) {
        pose.pos.x = 50;
        pose.pos.y = 60;
    }
    instance.track_override(entity, &world);

    // Apply to prefab should clear overrides
    instance.apply_to_prefab(&world).unwrap();
    assert!(!instance.has_overrides(entity), "Overrides should be cleared after apply");
    assert_eq!(instance.overrides.len(), 0, "Override map should be empty after apply");

    // Cleanup
    fs::remove_file(&prefab_path).unwrap();
}

#[test]
fn test_revert_restores_all_components() {
    use aw_editor_lib::prefab::{PrefabData, PrefabInstance};
    use std::collections::HashMap;

    let mut world = World::new();
    let entity = world.spawn("TestEntity", IVec2::new(10, 20), Team { id: 0 }, 100, 30);

    // Save original state
    let original_pos = world.pose(entity).unwrap().pos;
    let original_health = world.health(entity).unwrap().hp;

    // Create prefab
    let prefab_data = PrefabData::from_entity(&world, entity, "TestPrefab".to_string()).unwrap();
    let temp_dir = env::temp_dir();
    let prefab_path = temp_dir.join("test_revert_restores_all.prefab.ron");
    prefab_data.save_to_file(&prefab_path).unwrap();

    // Create prefab instance
    let mut entity_mapping = HashMap::new();
    entity_mapping.insert(0, entity);
    
    let mut instance = PrefabInstance {
        source: prefab_path.clone(),
        root_entity: entity,
        entity_mapping,
        overrides: HashMap::new(),
    };

    // Modify BOTH position and health
    if let Some(pose) = world.pose_mut(entity) {
        pose.pos.x = 999;
        pose.pos.y = 888;
    }
    if let Some(health) = world.health_mut(entity) {
        health.hp = 1;
    }

    // Track overrides
    instance.track_override(entity, &world);
    assert!(instance.has_overrides(entity));

    // Verify modifications applied
    assert_eq!(world.pose(entity).unwrap().pos.x, 999);
    assert_eq!(world.pose(entity).unwrap().pos.y, 888);
    assert_eq!(world.health(entity).unwrap().hp, 1);

    // Revert should restore ALL components
    instance.revert_to_prefab(&mut world).unwrap();
    
    // Verify position restored
    assert_eq!(world.pose(entity).unwrap().pos.x, original_pos.x, "Position X should be restored");
    assert_eq!(world.pose(entity).unwrap().pos.y, original_pos.y, "Position Y should be restored");
    
    // Verify health restored
    assert_eq!(world.health(entity).unwrap().hp, original_health, "Health should be restored");
    
    // Verify overrides cleared
    assert!(!instance.has_overrides(entity), "Overrides should be cleared after revert");
    assert_eq!(instance.overrides.len(), 0);

    // Cleanup
    fs::remove_file(&prefab_path).unwrap();
}

#[test]
fn test_apply_revert_workflow() {
    use aw_editor_lib::prefab::{PrefabData, PrefabInstance};
    use std::collections::HashMap;

    let mut world = World::new();
    let entity = world.spawn("TestEntity", IVec2::new(100, 200), Team { id: 0 }, 75, 30);

    let original_pos = world.pose(entity).unwrap().pos;
    let original_health = world.health(entity).unwrap().hp;

    // Create prefab
    let prefab_data = PrefabData::from_entity(&world, entity, "TestPrefab".to_string()).unwrap();
    let temp_dir = env::temp_dir();
    let prefab_path = temp_dir.join("test_apply_revert_workflow.prefab.ron");
    prefab_data.save_to_file(&prefab_path).unwrap();

    // Create prefab instance
    let mut entity_mapping = HashMap::new();
    entity_mapping.insert(0, entity);
    
    let mut instance = PrefabInstance {
        source: prefab_path.clone(),
        root_entity: entity,
        entity_mapping,
        overrides: HashMap::new(),
    };

    // Step 1: Modify entity
    if let Some(pose) = world.pose_mut(entity) {
        pose.pos.x = 150;
        pose.pos.y = 250;
    }
    instance.track_override(entity, &world);
    assert!(instance.has_overrides(entity));

    // Step 2: Apply changes to prefab (makes them permanent)
    instance.apply_to_prefab(&world).unwrap();
    assert!(!instance.has_overrides(entity), "Overrides cleared after apply");

    // Step 3: Modify again
    if let Some(pose) = world.pose_mut(entity) {
        pose.pos.x = 500;
        pose.pos.y = 600;
    }
    instance.track_override(entity, &world);
    assert!(instance.has_overrides(entity));

    // Step 4: Revert should restore to APPLIED state (150, 250), NOT original (100, 200)
    instance.revert_to_prefab(&mut world).unwrap();
    assert_eq!(world.pose(entity).unwrap().pos.x, 150, "Should revert to applied state");
    assert_eq!(world.pose(entity).unwrap().pos.y, 250, "Should revert to applied state");
    assert!(!instance.has_overrides(entity), "Overrides cleared after revert");

    // Cleanup
    fs::remove_file(&prefab_path).unwrap();
}

#[test]
fn test_bulk_operations_apply_all() {
    use aw_editor_lib::prefab::{PrefabData, PrefabEntityData};
    use std::collections::HashMap;
    use std::path::PathBuf;

    let mut world = World::new();
    let entity1 = world.spawn("Entity1", IVec2::new(10, 20), Team { id: 0 }, 100, 30);
    let entity2 = world.spawn("Entity2", IVec2::new(30, 40), Team { id: 0 }, 80, 20);

    // Create prefab with 2 entities
    let prefab_data = PrefabData {
        name: "MultiEntityPrefab".to_string(),
        entities: vec![
            PrefabEntityData {
                name: "Entity1".to_string(),
                pos_x: 10,
                pos_y: 20,
                team_id: 0,
                health: 100,
                max_health: 100,
                children_indices: vec![],
                prefab_reference: None,
            },
            PrefabEntityData {
                name: "Entity2".to_string(),
                pos_x: 30,
                pos_y: 40,
                team_id: 0,
                health: 80,
                max_health: 80,
                children_indices: vec![],
                prefab_reference: None,
            },
        ],
        root_entity_index: 0,
        version: "1.0".to_string(),
    };

    let temp_dir = env::temp_dir();
    let prefab_path = temp_dir.join("test_bulk_apply_all.prefab.ron");
    prefab_data.save_to_file(&prefab_path).unwrap();

    // Create prefab instance
    let mut entity_mapping = HashMap::new();
    entity_mapping.insert(0, entity1);
    entity_mapping.insert(1, entity2);

    let mut instance = aw_editor_lib::prefab::PrefabInstance {
        source: prefab_path.clone(),
        root_entity: entity1,
        entity_mapping,
        overrides: HashMap::new(),
    };

    // Modify both entities
    if let Some(pose) = world.pose_mut(entity1) {
        pose.pos.x = 100;
        pose.pos.y = 200;
    }
    if let Some(pose) = world.pose_mut(entity2) {
        pose.pos.x = 300;
        pose.pos.y = 400;
    }

    instance.track_override(entity1, &world);
    instance.track_override(entity2, &world);
    assert_eq!(instance.overrides.len(), 2);

    // Apply all should save both and clear overrides
    instance.apply_all_to_prefab(&world).unwrap();
    assert_eq!(instance.overrides.len(), 0, "All overrides should be cleared");

    // Verify file was updated
    let loaded_data = PrefabData::load_from_file(&prefab_path).unwrap();
    assert_eq!(loaded_data.entities[0].pos_x, 100);
    assert_eq!(loaded_data.entities[0].pos_y, 200);
    assert_eq!(loaded_data.entities[1].pos_x, 300);
    assert_eq!(loaded_data.entities[1].pos_y, 400);

    // Cleanup
    fs::remove_file(&prefab_path).unwrap();
}

#[test]
fn test_bulk_operations_revert_all() {
    use aw_editor_lib::prefab::{PrefabData, PrefabEntityData};
    use std::collections::HashMap;

    let mut world = World::new();
    let entity1 = world.spawn("Entity1", IVec2::new(10, 20), Team { id: 0 }, 100, 30);
    let entity2 = world.spawn("Entity2", IVec2::new(30, 40), Team { id: 0 }, 80, 20);

    // Create prefab
    let prefab_data = PrefabData {
        name: "MultiEntityPrefab".to_string(),
        entities: vec![
            PrefabEntityData {
                name: "Entity1".to_string(),
                pos_x: 10,
                pos_y: 20,
                team_id: 0,
                health: 100,
                max_health: 100,
                children_indices: vec![],
                prefab_reference: None,
            },
            PrefabEntityData {
                name: "Entity2".to_string(),
                pos_x: 30,
                pos_y: 40,
                team_id: 0,
                health: 80,
                max_health: 80,
                children_indices: vec![],
                prefab_reference: None,
            },
        ],
        root_entity_index: 0,
        version: "1.0".to_string(),
    };

    let temp_dir = env::temp_dir();
    let prefab_path = temp_dir.join("test_bulk_revert_all.prefab.ron");
    prefab_data.save_to_file(&prefab_path).unwrap();

    // Create prefab instance
    let mut entity_mapping = HashMap::new();
    entity_mapping.insert(0, entity1);
    entity_mapping.insert(1, entity2);

    let mut instance = aw_editor_lib::prefab::PrefabInstance {
        source: prefab_path.clone(),
        root_entity: entity1,
        entity_mapping,
        overrides: HashMap::new(),
    };

    // Modify both entities
    if let Some(pose) = world.pose_mut(entity1) {
        pose.pos.x = 999;
        pose.pos.y = 888;
    }
    if let Some(health) = world.health_mut(entity1) {
        health.hp = 1;
    }
    if let Some(pose) = world.pose_mut(entity2) {
        pose.pos.x = 777;
        pose.pos.y = 666;
    }
    if let Some(health) = world.health_mut(entity2) {
        health.hp = 2;
    }

    instance.track_override(entity1, &world);
    instance.track_override(entity2, &world);
    assert_eq!(instance.overrides.len(), 2);

    // Revert all should restore both entities
    instance.revert_all_to_prefab(&mut world).unwrap();
    
    // Verify entity1 restored
    assert_eq!(world.pose(entity1).unwrap().pos.x, 10);
    assert_eq!(world.pose(entity1).unwrap().pos.y, 20);
    assert_eq!(world.health(entity1).unwrap().hp, 100);
    
    // Verify entity2 restored
    assert_eq!(world.pose(entity2).unwrap().pos.x, 30);
    assert_eq!(world.pose(entity2).unwrap().pos.y, 40);
    assert_eq!(world.health(entity2).unwrap().hp, 80);
    
    // Verify overrides cleared
    assert_eq!(instance.overrides.len(), 0);

    // Cleanup
    fs::remove_file(&prefab_path).unwrap();
}
