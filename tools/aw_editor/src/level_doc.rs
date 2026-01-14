//! Level document types for editor serialization
//!
//! These types represent the TOML-serializable level format used by the editor.

use serde::{Deserialize, Serialize};

/// Main level document
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct LevelDoc {
    pub title: String,
    pub biome: String,
    pub seed: u64,
    pub sky: Sky,
    pub biome_paints: Vec<BiomePaint>,
    pub obstacles: Vec<Obstacle>,
    pub npcs: Vec<NpcSpawn>,
    pub fate_threads: Vec<FateThread>,
    pub boss: BossCfg,
}

/// Sky configuration
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Sky {
    pub time_of_day: String,
    pub weather: String,
}

/// Biome paint brush types
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum BiomePaint {
    #[serde(rename = "grass_dense")]
    GrassDense { area: Circle },
    #[serde(rename = "moss_path")]
    MossPath { polyline: Vec<[i32; 2]> },
}

/// Circle area definition
#[derive(Clone, Serialize, Deserialize)]
pub struct Circle {
    pub cx: i32,
    pub cz: i32,
    pub radius: i32,
}

/// Obstacle definition
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Obstacle {
    pub id: String,
    pub pos: [f32; 3],
    pub yaw: f32,
    pub tags: Vec<String>,
}

/// NPC spawn point
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct NpcSpawn {
    pub archetype: String,
    pub count: u32,
    pub spawn: Spawn,
    pub behavior: String,
}

/// Spawn area definition
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct Spawn {
    pub pos: [f32; 3],
    pub radius: f32,
}

/// Fate thread (trigger-based event)
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct FateThread {
    pub name: String,
    pub triggers: Vec<Trigger>,
    pub ops: Vec<DirectorOp>,
}

/// Trigger types
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub enum Trigger {
    #[serde(rename = "enter_area")]
    EnterArea { center: [f32; 3], radius: f32 },
}

impl Default for Trigger {
    fn default() -> Self {
        Trigger::EnterArea {
            center: [0.0, 0.0, 0.0],
            radius: 5.0,
        }
    }
}

/// Director operation types
#[derive(Clone, Serialize, Deserialize)]
#[serde(tag = "op")]
pub enum DirectorOp {
    Fortify {
        area: FortRegion,
    },
    Collapse {
        area: FortRegion,
    },
    SpawnWave {
        archetype: String,
        count: u32,
        scatter: f32,
    },
}

impl Default for DirectorOp {
    fn default() -> Self {
        DirectorOp::SpawnWave {
            archetype: "enemy".to_string(),
            count: 1,
            scatter: 2.0,
        }
    }
}

/// Fortify region definition
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct FortRegion {
    pub cx: i32,
    pub cz: i32,
    pub r: i32,
}

/// Boss encounter configuration
#[derive(Clone, Serialize, Deserialize, Default)]
pub struct BossCfg {
    pub director_budget_script: String,
    pub phase_script: String,
}
