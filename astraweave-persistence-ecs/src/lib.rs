//! ECS Persistence Integration for AstraWeave
//!
//! This crate provides ECS plugins and systems for save/load functionality,
//! integrating the aw-save persistence system with the AstraWeave ECS.

use anyhow::Result;
use astraweave_ecs::{App, Plugin, World};
use aw_save::{SaveBundleV2, SaveManager, WorldState, SAVE_SCHEMA_VERSION};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use time::OffsetDateTime;
use uuid::Uuid;

/// Save/Load manager component (stored in ECS)
pub struct CPersistenceManager {
    save_manager: SaveManager,
    current_player: String,
}

/// Save metadata for tracking game state
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SaveMetadata {
    pub player_id: String,
    pub slot: u8,
    pub save_id: Uuid,
    pub created_at: OffsetDateTime,
    pub world_tick: u64,
    pub world_hash: u64,
}

/// Replay state component (stored in ECS)
#[derive(Clone)]
pub struct CReplayState {
    pub is_replaying: bool,
    pub current_tick: u64,
    pub total_ticks: u64,
    pub events: Vec<ReplayEvent>,
}

/// Individual replay event
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub tick: u64,
    pub event_type: String,
    pub data: Vec<u8>,
}

/// ECS Plugin for persistence functionality
#[allow(dead_code)]
pub struct PersistencePlugin {
    save_directory: PathBuf,
}

impl PersistencePlugin {
    pub fn new(save_directory: PathBuf) -> Self {
        Self { save_directory }
    }
}

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        // Add persistence systems
        app.add_system("post_simulation", auto_save_system);
        app.add_system("pre_simulation", replay_system);
    }
}

/// System that automatically saves game state at regular intervals
fn auto_save_system(_world: &mut World) {
    // TODO: Query for persistence manager and save at intervals
    // This would serialize the current ECS state and save it
}

/// System that handles replay functionality
fn replay_system(world: &mut World) {
    // First collect all entities that need updating
    let mut entities_to_update = Vec::new();

    {
        let mut q = astraweave_ecs::Query::<CReplayState>::new(world);
        while let Some((entity, replay)) = q.next() {
            if replay.is_replaying {
                entities_to_update.push(entity);
            }
        }
    }

    // Now update each entity
    for entity in entities_to_update {
        if let Some(replay) = world.get_mut::<CReplayState>(entity) {
            if replay.current_tick < replay.total_ticks {
                // Apply next replay event
                // TODO: Implement replay event application
                replay.current_tick += 1;
            } else {
                // Replay finished
                replay.is_replaying = false;
            }
        }
    }
}

impl CPersistenceManager {
    /// Set the current player for save operations
    pub fn set_player(&mut self, player_id: &str) {
        self.current_player = player_id.to_string();
    }

    /// Save the current game state to a slot
    pub fn save_game(
        &self,
        slot: u8,
        world_tick: u64,
        world_hash: u64,
        ecs_blob: Vec<u8>,
    ) -> Result<PathBuf> {
        // Create companion profiles from ECS data
        let companions = Vec::new(); // TODO: Query ECS for companion data

        // Create inventory from ECS data
        let inventory = aw_save::PlayerInventory {
            credits: 1000,     // TODO: Get from ECS
            items: Vec::new(), // TODO: Get from ECS
        };

        // Create metadata
        let mut meta = HashMap::new();
        meta.insert(
            "engine_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        let bundle = SaveBundleV2 {
            schema: SAVE_SCHEMA_VERSION,
            save_id: Uuid::new_v4(),
            created_at: OffsetDateTime::now_utc(),
            player_id: self.current_player.clone(),
            slot,
            world: WorldState {
                tick: world_tick,
                ecs_blob,
                state_hash: world_hash,
            },
            companions,
            inventory,
            meta,
        };

        self.save_manager.save(&self.current_player, slot, bundle)
    }

    /// Load game state from a slot
    pub fn load_game(&self, slot: u8) -> Result<(SaveBundleV2, PathBuf)> {
        self.save_manager
            .load_latest_slot(&self.current_player, slot)
    }

    /// Start replay from a saved game
    pub fn start_replay(&self, slot: u8) -> Result<CReplayState> {
        let (bundle, _) = self.load_game(slot)?;

        Ok(CReplayState {
            is_replaying: true,
            current_tick: 0,
            total_ticks: bundle.world.tick,
            events: Vec::new(), // TODO: Load replay events from save data
        })
    }

    /// List all saves for the current player
    pub fn list_saves(&self) -> Result<Vec<aw_save::SaveMeta>> {
        self.save_manager.list_saves(&self.current_player)
    }

    /// Migrate an old save file to the latest version
    pub fn migrate_save(&self, path: &std::path::Path, resave: bool) -> Result<SaveBundleV2> {
        self.save_manager.migrate_file_to_latest(path, resave)
    }
}

/// Serialize ECS world state for saving
pub fn serialize_ecs_world(// TODO: Add ECS world queries here
) -> Result<Vec<u8>> {
    // TODO: Implement ECS world serialization
    // This should collect all relevant components and serialize them
    // For now, return empty blob
    Ok(Vec::new())
}

/// Deserialize and restore ECS world state from save
pub fn deserialize_ecs_world(
    ecs_blob: &[u8],
    // TODO: Add ECS world mutation parameters here
) -> Result<()> {
    // TODO: Implement ECS world deserialization
    // This should restore all components from the serialized data
    if ecs_blob.is_empty() {
        // No data to restore
        return Ok(());
    }

    // TODO: Deserialize and apply to ECS world
    Ok(())
}

/// Calculate a hash of the current ECS world state for integrity checking
pub fn calculate_world_hash(// TODO: Add ECS world queries here
) -> u64 {
    // TODO: Implement world state hashing
    // This should hash all relevant world state for integrity checking
    0 // Placeholder
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn persistence_manager_creation() {
        let temp_dir = tempdir().unwrap();
        let save_manager = SaveManager::new(temp_dir.path());
        let persistence = CPersistenceManager {
            save_manager,
            current_player: "test_player".to_string(),
        };

        assert_eq!(persistence.current_player, "test_player");
    }

    #[test]
    fn replay_state_initialization() {
        let replay = CReplayState {
            is_replaying: false,
            current_tick: 0,
            total_ticks: 100,
            events: Vec::new(),
        };

        assert!(!replay.is_replaying);
        assert_eq!(replay.current_tick, 0);
        assert_eq!(replay.total_ticks, 100);
        assert!(replay.events.is_empty());
    }

    #[test]
    fn serialize_empty_world() {
        // Test serialization of empty world
        let blob = serialize_ecs_world().unwrap();
        assert!(blob.is_empty());
    }

    #[test]
    fn deserialize_empty_world() {
        // Test deserialization of empty world
        let blob = Vec::new();
        deserialize_ecs_world(&blob).unwrap();
    }
}
