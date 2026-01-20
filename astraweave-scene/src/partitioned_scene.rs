//! Partitioned Scene Loading
//!
//! This module provides integration between the Scene type and WorldPartition system.

use crate::streaming::{StreamingConfig, StreamingEvent, WorldPartitionManager};
use crate::world_partition::{Entity, GridConfig, GridCoord, WorldPartition};
use crate::Scene;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

// EntityId type alias
type EntityId = Entity;

/// Entity-to-cell mapping for spatial queries
#[derive(Debug, Clone)]
pub struct CellEntities {
    pub cell: GridCoord,
    pub entities: Vec<EntityId>,
}

impl CellEntities {
    pub fn new(cell: GridCoord) -> Self {
        Self {
            cell,
            entities: Vec::new(),
        }
    }

    pub fn add_entity(&mut self, entity: EntityId) {
        if !self.entities.contains(&entity) {
            self.entities.push(entity);
        }
    }

    pub fn remove_entity(&mut self, entity: EntityId) {
        self.entities.retain(|&e| e != entity);
    }
}

/// Events emitted by partitioned scene
#[derive(Debug, Clone)]
pub enum SceneEvent {
    CellLoaded(GridCoord),
    CellUnloaded(GridCoord),
    EntitySpawned(EntityId, GridCoord),
    EntityMoved(EntityId, GridCoord, GridCoord), // entity, old_cell, new_cell
    EntityDespawned(EntityId, GridCoord),
}

/// A scene that supports world partitioning
pub struct PartitionedScene {
    pub scene: Scene,
    pub partition: Arc<RwLock<WorldPartition>>,
    pub manager: WorldPartitionManager,
    /// Map of cell coordinates to entities within those cells
    pub cell_entities: HashMap<GridCoord, CellEntities>,
    /// Map of entity IDs to their current cell
    pub entity_cells: HashMap<EntityId, GridCoord>,
    /// Scene events queue
    pub events: Vec<SceneEvent>,
}

impl PartitionedScene {
    /// Create a new partitioned scene
    pub fn new(grid_config: GridConfig, streaming_config: StreamingConfig) -> Self {
        let scene = Scene::new();
        let partition = Arc::new(RwLock::new(WorldPartition::new(grid_config)));
        let manager = WorldPartitionManager::new(Arc::clone(&partition), streaming_config);

        Self {
            scene,
            partition,
            manager,
            cell_entities: HashMap::new(),
            entity_cells: HashMap::new(),
            events: Vec::new(),
        }
    }

    /// Create with default configurations
    pub fn new_default() -> Self {
        Self::new(GridConfig::default(), StreamingConfig::default())
    }

    /// Update streaming based on camera position
    pub async fn update_streaming(&mut self, camera_position: glam::Vec3) -> Result<()> {
        // Add event listener to capture streaming events
        let events_clone = Arc::new(RwLock::new(Vec::new()));
        let events_for_listener = Arc::clone(&events_clone);

        self.manager.add_event_listener(move |event| {
            let events = Arc::clone(&events_for_listener);
            tokio::spawn(async move {
                let mut events = events.write().await;
                match event {
                    StreamingEvent::CellLoaded(coord) => {
                        events.push(SceneEvent::CellLoaded(coord));
                    }
                    StreamingEvent::CellUnloaded(coord) => {
                        events.push(SceneEvent::CellUnloaded(coord));
                    }
                    _ => {}
                }
            });
        });

        // Update streaming
        self.manager.update(camera_position).await?;

        // Collect events
        let captured_events = events_clone.read().await;
        self.events.extend(captured_events.iter().cloned());

        Ok(())
    }

    /// Handle cell loaded event - spawn entities from cell data
    pub fn on_cell_loaded(
        &mut self,
        coord: GridCoord,
        cell_data: astraweave_asset::cell_loader::CellData,
    ) {
        let mut cell_entities = CellEntities::new(coord);

        // Create entities from cell data
        // Note: In full ECS integration, this would use ECS World to spawn entities
        // For now, we use placeholder entity IDs
        for (idx, _entity_data) in cell_data.entities.iter().enumerate() {
            // Generate entity ID (in real ECS, this would come from world.spawn())
            let entity_id = ((coord.x as u64) << 40) | ((coord.y as u64) << 20) | idx as u64;

            // Add entity to cell
            cell_entities.add_entity(entity_id);
            self.entity_cells.insert(entity_id, coord);

            // Emit event
            self.events
                .push(SceneEvent::EntitySpawned(entity_id, coord));

            // TODO: In full implementation, add components to ECS:
            // - CTransformLocal from entity_data.position/rotation/scale
            // - CMesh if entity_data.mesh is Some
            // - CMaterial if entity_data.material is Some
        }

        // Store cell entities mapping
        self.cell_entities.insert(coord, cell_entities);

        // Emit cell loaded event
        self.events.push(SceneEvent::CellLoaded(coord));
    }

    /// Handle cell unloaded event - despawn entities
    pub fn on_cell_unloaded(&mut self, coord: GridCoord) {
        if let Some(cell_entities) = self.cell_entities.remove(&coord) {
            // Despawn all entities in the cell
            for entity_id in cell_entities.entities {
                self.entity_cells.remove(&entity_id);
                self.events
                    .push(SceneEvent::EntityDespawned(entity_id, coord));

                // TODO: In full implementation, despawn from ECS:
                // world.despawn(entity_id);
            }
        }

        // Emit cell unloaded event
        self.events.push(SceneEvent::CellUnloaded(coord));
    }

    /// Query entities in a specific cell
    pub fn query_entities_in_cell(&self, coord: GridCoord) -> Option<&Vec<EntityId>> {
        self.cell_entities.get(&coord).map(|ce| &ce.entities)
    }

    /// Query entities in multiple cells (e.g., within radius)
    pub fn query_entities_in_cells(&self, coords: &[GridCoord]) -> Vec<EntityId> {
        let mut entities = Vec::new();
        for coord in coords {
            if let Some(cell_entities) = self.cell_entities.get(coord) {
                entities.extend_from_slice(&cell_entities.entities);
            }
        }
        entities
    }

    /// Get cell for an entity
    pub fn get_entity_cell(&self, entity: EntityId) -> Option<GridCoord> {
        self.entity_cells.get(&entity).copied()
    }

    /// Move an entity to a different cell (when position changes)
    pub fn move_entity_to_cell(&mut self, entity: EntityId, new_coord: GridCoord) {
        // Get old cell
        let old_coord = match self.entity_cells.get(&entity).copied() {
            Some(coord) => coord,
            None => {
                // Entity not tracked yet, just add to new cell
                self.entity_cells.insert(entity, new_coord);
                self.cell_entities
                    .entry(new_coord)
                    .or_insert_with(|| CellEntities::new(new_coord))
                    .add_entity(entity);
                self.events
                    .push(SceneEvent::EntitySpawned(entity, new_coord));
                return;
            }
        };

        // Same cell, no action needed
        if old_coord == new_coord {
            return;
        }

        // Remove from old cell
        if let Some(old_cell_entities) = self.cell_entities.get_mut(&old_coord) {
            old_cell_entities.remove_entity(entity);
        }

        // Add to new cell
        self.cell_entities
            .entry(new_coord)
            .or_insert_with(|| CellEntities::new(new_coord))
            .add_entity(entity);

        // Update entity->cell mapping
        self.entity_cells.insert(entity, new_coord);

        // Emit event
        self.events
            .push(SceneEvent::EntityMoved(entity, old_coord, new_coord));
    }

    /// Drain pending events
    pub fn drain_events(&mut self) -> Vec<SceneEvent> {
        std::mem::take(&mut self.events)
    }

    /// Get streaming metrics
    pub fn metrics(&self) -> &crate::streaming::StreamingMetrics {
        self.manager.metrics()
    }
}

/// Extension trait for Scene to support partitioned loading
pub trait ScenePartitionExt {
    /// Load a scene with partitioning enabled
    fn load_partitioned(
        grid_config: GridConfig,
        streaming_config: StreamingConfig,
    ) -> PartitionedScene;
}

impl ScenePartitionExt for Scene {
    fn load_partitioned(
        grid_config: GridConfig,
        streaming_config: StreamingConfig,
    ) -> PartitionedScene {
        PartitionedScene::new(grid_config, streaming_config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_asset::cell_loader::{CellData, EntityData};

    #[test]
    fn test_cell_entities_add_remove_dedup() {
        let cell = GridCoord::new(0, 0, 0);
        let mut ce = CellEntities::new(cell);
        assert_eq!(ce.cell, cell);
        assert!(ce.entities.is_empty());

        ce.add_entity(42);
        ce.add_entity(42);
        assert_eq!(ce.entities, vec![42]);

        ce.add_entity(7);
        assert_eq!(ce.entities.len(), 2);

        ce.remove_entity(42);
        assert_eq!(ce.entities, vec![7]);
    }

    #[test]
    fn test_on_cell_loaded_and_unloaded_emits_events_and_tracks_entities() {
        let mut ps = PartitionedScene::new_default();
        let coord = GridCoord::new(1, 0, 2);

        let mut cell_data = CellData::new([coord.x, coord.y, coord.z]);
        cell_data.add_entity(EntityData::new([1.0, 2.0, 3.0]).with_name("a"));
        cell_data.add_entity(EntityData::new([4.0, 5.0, 6.0]).with_name("b"));

        ps.on_cell_loaded(coord, cell_data);

        // 2 entity spawned events + 1 cell loaded event.
        assert_eq!(ps.events.len(), 3);
        assert!(ps.cell_entities.contains_key(&coord));
        assert_eq!(ps.query_entities_in_cell(coord).unwrap().len(), 2);

        let spawned_entities = ps.query_entities_in_cell(coord).unwrap().clone();
        for e in &spawned_entities {
            assert_eq!(ps.get_entity_cell(*e), Some(coord));
        }

        ps.on_cell_unloaded(coord);

        // 2 despawned events + 1 cell unloaded event.
        assert_eq!(ps.events.len(), 6);
        assert!(!ps.cell_entities.contains_key(&coord));
        for e in &spawned_entities {
            assert!(ps.get_entity_cell(*e).is_none());
        }
    }

    #[test]
    fn test_move_entity_to_cell_tracks_spawn_move_and_noop_same_cell() {
        let mut ps = PartitionedScene::new_default();
        let a = GridCoord::new(0, 0, 0);
        let b = GridCoord::new(2, 0, 3);

        // Untracked entity -> treated as spawn.
        ps.move_entity_to_cell(100, a);
        assert_eq!(ps.get_entity_cell(100), Some(a));
        assert!(matches!(ps.events.last(), Some(SceneEvent::EntitySpawned(100, _))));

        let events_before = ps.events.len();
        // Same cell -> no-op.
        ps.move_entity_to_cell(100, a);
        assert_eq!(ps.events.len(), events_before);

        // Move to a new cell.
        ps.move_entity_to_cell(100, b);
        assert_eq!(ps.get_entity_cell(100), Some(b));
        assert!(matches!(ps.events.last(), Some(SceneEvent::EntityMoved(100, _, _))));
    }

    #[test]
    fn test_scene_partition_ext_load_partitioned_constructs() {
        let ps = Scene::load_partitioned(GridConfig::default(), StreamingConfig::default());
        assert!(ps.events.is_empty());
    }
}
