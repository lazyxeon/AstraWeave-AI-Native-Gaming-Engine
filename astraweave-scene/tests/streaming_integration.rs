//! Integration tests for World Partition async streaming

use astraweave_asset::cell_loader::{save_cell_to_ron, AssetKind, AssetRef, CellData, EntityData};
use astraweave_scene::partitioned_scene::{PartitionedScene, SceneEvent};
use astraweave_scene::streaming::{StreamingConfig, WorldPartitionManager};
use astraweave_scene::world_partition::{GridConfig, GridCoord, WorldPartition};
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Helper to create a test cell file
async fn create_test_cell_file(path: &Path, coord: [i32; 3]) -> anyhow::Result<()> {
    let mut cell_data = CellData::new(coord);

    // Add test entities
    cell_data.add_entity(
        EntityData::new([coord[0] as f32 * 100.0, 0.0, coord[2] as f32 * 100.0])
            .with_name(format!("entity_cell_{}_{}_{}", coord[0], coord[1], coord[2]))
            .with_mesh("models/cube.glb")
            .with_material(0),
    );

    cell_data.add_entity(
        EntityData::new([
            coord[0] as f32 * 100.0 + 10.0,
            0.0,
            coord[2] as f32 * 100.0 + 10.0,
        ])
        .with_name(format!(
            "entity2_cell_{}_{}_{}",
            coord[0], coord[1], coord[2]
        ))
        .with_mesh("models/sphere.glb")
        .with_material(1),
    );

    // Add test assets
    cell_data.add_asset(AssetRef::new("models/cube.glb", AssetKind::Mesh));
    cell_data.add_asset(AssetRef::new("models/sphere.glb", AssetKind::Mesh));
    cell_data.add_asset(AssetRef::new("textures/wood.png", AssetKind::Texture));

    // Save to file
    save_cell_to_ron(path, &cell_data).await?;

    Ok(())
}

#[tokio::test]
async fn test_async_cell_loading() {
    // Create test directory
    let test_dir = Path::new("target/test_assets/cells");
    fs::create_dir_all(test_dir).await.ok();

    // Create test cell file
    let coord = GridCoord::new(0, 0, 0);
    let cell_path = test_dir.join("0_0_0.ron");
    create_test_cell_file(&cell_path, [0, 0, 0]).await.unwrap();

    // Create streaming manager
    let config = GridConfig {
        cell_size: 100.0,
        world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
    };
    let partition = Arc::new(RwLock::new(WorldPartition::new(config)));
    let mut manager = WorldPartitionManager::new(partition, StreamingConfig::default());

    // Force load the cell
    manager.force_load_cell(coord).await.unwrap();

    // Verify cell is active
    assert!(manager.is_cell_active(coord));

    // Cleanup
    fs::remove_file(cell_path).await.ok();
}

#[tokio::test]
async fn test_memory_budget_enforcement() {
    // Create test directory
    let test_dir = Path::new("target/test_assets/cells");
    fs::create_dir_all(test_dir).await.ok();

    // Create multiple cells
    for x in 0..5 {
        for z in 0..5 {
            let cell_path = test_dir.join(format!("{}_0_{}.ron", x, z));
            create_test_cell_file(&cell_path, [x, 0, z]).await.unwrap();
        }
    }

    // Create streaming manager with tight memory budget
    let config = GridConfig {
        cell_size: 100.0,
        world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
    };
    let streaming_config = StreamingConfig {
        max_active_cells: 5, // Limit active cells
        lru_cache_size: 2,
        streaming_radius: 300.0,
        max_concurrent_loads: 2,
    };
    let partition = Arc::new(RwLock::new(WorldPartition::new(config)));
    let mut manager = WorldPartitionManager::new(partition, streaming_config);

    // Update near origin (should load cells 0,0 and neighbors)
    let camera_pos = glam::Vec3::new(0.0, 0.0, 0.0);
    manager.update(camera_pos).await.unwrap();

    // Check that active cells are within limit
    let metrics = manager.metrics();
    assert!(metrics.active_cells <= 5);

    // Cleanup
    for x in 0..5 {
        for z in 0..5 {
            let cell_path = test_dir.join(format!("{}_0_{}.ron", x, z));
            fs::remove_file(cell_path).await.ok();
        }
    }
}

#[tokio::test]
async fn test_partitioned_scene_entity_tracking() {
    let mut scene = PartitionedScene::new_default();

    // Create test cell data
    let coord = GridCoord::new(0, 0, 0);
    let mut cell_data = CellData::new([0, 0, 0]);
    cell_data.add_entity(EntityData::new([1.0, 0.0, 1.0]).with_name("test_entity_1"));
    cell_data.add_entity(EntityData::new([2.0, 0.0, 2.0]).with_name("test_entity_2"));

    // Load cell
    scene.on_cell_loaded(coord, cell_data);

    // Verify entities are tracked
    let entities = scene.query_entities_in_cell(coord).unwrap();
    assert_eq!(entities.len(), 2);

    // Verify events
    let events = scene.drain_events();
    let spawned_count = events
        .iter()
        .filter(|e| matches!(e, SceneEvent::EntitySpawned(_, _)))
        .count();
    assert_eq!(spawned_count, 2);

    // Unload cell
    scene.on_cell_unloaded(coord);
    assert!(scene.query_entities_in_cell(coord).is_none());

    // Verify despawn events
    let events = scene.drain_events();
    let despawned_count = events
        .iter()
        .filter(|e| matches!(e, SceneEvent::EntityDespawned(_, _)))
        .count();
    assert_eq!(despawned_count, 2);
}

#[tokio::test]
async fn test_streaming_with_camera_movement() {
    // Create test directory
    let test_dir = Path::new("target/test_assets/cells");
    fs::create_dir_all(test_dir).await.ok();

    // Create grid of cells
    for x in -2..=2 {
        for z in -2..=2 {
            let cell_path = test_dir.join(format!("{}_0_{}.ron", x, z));
            create_test_cell_file(&cell_path, [x, 0, z]).await.unwrap();
        }
    }

    // Create partitioned scene
    let config = GridConfig {
        cell_size: 100.0,
        world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
    };
    let streaming_config = StreamingConfig {
        max_active_cells: 9,
        lru_cache_size: 5,
        streaming_radius: 250.0, // ~2.5 cells
        max_concurrent_loads: 4,
    };
    let mut scene = PartitionedScene::new(config, streaming_config);

    // Update at origin
    scene.update_streaming(glam::Vec3::ZERO).await.unwrap();
    let initial_cells = scene.metrics().active_cells;
    assert!(initial_cells > 0);

    // Move camera to new position
    let new_pos = glam::Vec3::new(500.0, 0.0, 500.0); // ~5 cells away
    scene.update_streaming(new_pos).await.unwrap();

    // Should have loaded new cells and unloaded far cells
    let after_move_cells = scene.metrics().active_cells;
    assert!(after_move_cells > 0);
    assert!(after_move_cells <= 9); // Within limit

    // Cleanup
    for x in -2..=2 {
        for z in -2..=2 {
            let cell_path = test_dir.join(format!("{}_0_{}.ron", x, z));
            fs::remove_file(cell_path).await.ok();
        }
    }
}

#[tokio::test]
async fn test_entity_cell_migration() {
    let mut scene = PartitionedScene::new_default();

    // Spawn entity in cell (0,0,0)
    let entity_id = 42;
    let initial_coord = GridCoord::new(0, 0, 0);
    scene.move_entity_to_cell(entity_id, initial_coord);

    assert_eq!(scene.get_entity_cell(entity_id), Some(initial_coord));
    assert_eq!(
        scene.query_entities_in_cell(initial_coord).unwrap().len(),
        1
    );

    // Move entity to new cell (1,0,1)
    let new_coord = GridCoord::new(1, 0, 1);
    scene.move_entity_to_cell(entity_id, new_coord);

    // Verify entity moved
    assert_eq!(scene.get_entity_cell(entity_id), Some(new_coord));
    assert_eq!(scene.query_entities_in_cell(new_coord).unwrap().len(), 1);
    assert_eq!(
        scene.query_entities_in_cell(initial_coord).unwrap().len(),
        0
    );

    // Verify move event
    let events = scene.drain_events();
    let move_event = events
        .iter()
        .find(|e| matches!(e, SceneEvent::EntityMoved(_, _, _)));
    assert!(move_event.is_some());
}

#[tokio::test]
async fn test_lru_cache_functionality() {
    let config = GridConfig::default();
    let streaming_config = StreamingConfig {
        max_active_cells: 3,
        lru_cache_size: 2,
        streaming_radius: 200.0,
        max_concurrent_loads: 2,
    };
    let partition = Arc::new(RwLock::new(WorldPartition::new(config)));
    let mut manager = WorldPartitionManager::new(partition, streaming_config);

    // Load a cell
    let coord1 = GridCoord::new(0, 0, 0);
    manager.force_load_cell(coord1).await.unwrap();
    assert!(manager.is_cell_active(coord1));

    // Unload the cell (should go to LRU cache)
    manager.force_unload_cell(coord1).await.unwrap();
    assert!(!manager.is_cell_active(coord1));

    // Reload the cell (should be fast from LRU cache)
    manager.force_load_cell(coord1).await.unwrap();
    assert!(manager.is_cell_active(coord1));

    // Verify metrics
    let metrics = manager.metrics();
    assert_eq!(metrics.total_loads, 2); // Original load + reload
}

#[tokio::test]
async fn test_performance_no_stalls() {
    // Create test directory
    let test_dir = Path::new("target/test_assets/cells");
    fs::create_dir_all(test_dir).await.ok();

    // Create cells
    for x in 0..10 {
        for z in 0..10 {
            let cell_path = test_dir.join(format!("{}_0_{}.ron", x, z));
            create_test_cell_file(&cell_path, [x, 0, z]).await.unwrap();
        }
    }

    let config = GridConfig {
        cell_size: 100.0,
        world_bounds: (-1000.0, 1000.0, -1000.0, 1000.0),
    };
    let mut scene = PartitionedScene::new(config, StreamingConfig::default());

    // Measure update time (should be <100ms per acceptance criteria)
    let start = std::time::Instant::now();
    scene.update_streaming(glam::Vec3::ZERO).await.unwrap();
    let duration = start.elapsed();

    println!("Streaming update took: {:?}", duration);
    assert!(
        duration.as_millis() < 100,
        "Streaming update took {}ms, expected <100ms",
        duration.as_millis()
    );

    // Cleanup
    for x in 0..10 {
        for z in 0..10 {
            let cell_path = test_dir.join(format!("{}_0_{}.ron", x, z));
            fs::remove_file(cell_path).await.ok();
        }
    }
}
