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

#[cfg(test)]
mod tests {
    use super::*;
    use toml;

    #[test]
    fn test_level_doc_defaults() {
        let doc = LevelDoc::default();
        assert_eq!(doc.title, "");
        assert_eq!(doc.biome, "");
        assert_eq!(doc.seed, 0);
        assert!(doc.biome_paints.is_empty());
    }

    #[test]
    fn test_serialization_roundtrip() {
        let mut doc = LevelDoc::default();
        doc.title = "Test Level".to_string();
        doc.biome = "Forest".to_string();
        doc.seed = 12345;
        doc.sky = Sky {
            time_of_day: "dawn".to_string(),
            weather: "fog".to_string(),
        };
        doc.obstacles.push(Obstacle {
            id: "rock_1".to_string(),
            pos: [1.0, 2.0, 3.0],
            yaw: 45.0,
            tags: vec!["solid".to_string()],
        });

        // TOML serialization
        let toml_str = toml::to_string(&doc).expect("serialize");
        let loaded: LevelDoc = toml::from_str(&toml_str).expect("deserialize");

        assert_eq!(loaded.title, "Test Level");
        assert_eq!(loaded.biome, "Forest");
        assert_eq!(loaded.seed, 12345);
        assert_eq!(loaded.sky.time_of_day, "dawn");
        assert_eq!(loaded.obstacles.len(), 1);
        assert_eq!(loaded.obstacles[0].id, "rock_1");
    }

    #[test]
    fn test_enum_serialization() {
        let paint = BiomePaint::GrassDense {
            area: Circle { cx: 10, cz: 20, radius: 5 },
        };
        let toml_paint = toml::to_string(&paint).expect("serialize paint");
        assert!(toml_paint.contains("grass_dense"));
        assert!(toml_paint.contains("radius = 5"));

        let trigger = Trigger::EnterArea { center: [1.0, 2.0, 3.0], radius: 10.0 };
        let toml_trigger = toml::to_string(&trigger).expect("serialize trigger");
        assert!(toml_trigger.contains("enter_area"));
    }

    #[test]
    fn test_npcs_structure() {
        let npc = NpcSpawn {
            archetype: "Guard".to_string(),
            count: 3,
            spawn: Spawn { pos: [0.0, 0.0, 0.0], radius: 10.0 },
            behavior: "Patrol".to_string(),
        };

        let mut doc = LevelDoc::default();
        doc.npcs.push(npc);

        assert_eq!(doc.npcs[0].archetype, "Guard");
        assert_eq!(doc.npcs[0].count, 3);
    }
}
