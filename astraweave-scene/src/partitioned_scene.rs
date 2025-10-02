//! Partitioned Scene Loading
//!
//! This module provides integration between the Scene type and WorldPartition system.

use crate::world_partition::{GridConfig, WorldPartition};
use crate::streaming::{StreamingConfig, WorldPartitionManager};
use crate::Scene;
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A scene that supports world partitioning
pub struct PartitionedScene {
    pub scene: Scene,
    pub partition: Arc<RwLock<WorldPartition>>,
    pub manager: WorldPartitionManager,
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
        }
    }

    /// Create with default configurations
    pub fn new_default() -> Self {
        Self::new(GridConfig::default(), StreamingConfig::default())
    }

    /// Update streaming based on camera position
    pub async fn update_streaming(&mut self, camera_position: glam::Vec3) -> Result<()> {
        self.manager.update(camera_position).await
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

    #[tokio::test]
    async fn test_partitioned_scene_creation() {
        let scene = PartitionedScene::new_default();
        assert_eq!(scene.metrics().active_cells, 0);
    }

    #[tokio::test]
    async fn test_partitioned_scene_update() {
        let mut scene = PartitionedScene::new_default();
        let camera_pos = glam::Vec3::new(0.0, 0.0, 0.0);
        
        scene.update_streaming(camera_pos).await.unwrap();
        
        // Should have loaded some cells
        assert!(scene.metrics().active_cells > 0);
    }

    #[test]
    fn test_scene_partition_ext() {
        let _scene = Scene::load_partitioned(
            GridConfig::default(),
            StreamingConfig::default(),
        );
    }
}