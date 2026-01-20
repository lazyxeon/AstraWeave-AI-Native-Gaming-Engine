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

/// Level document validation issue
#[derive(Debug, Clone, PartialEq)]
pub struct LevelValidationIssue {
    pub message: String,
    pub is_error: bool,
}

impl LevelValidationIssue {
    /// Create a new error issue
    pub fn error(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_error: true,
        }
    }

    /// Create a new warning issue
    pub fn warning(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            is_error: false,
        }
    }
}

/// Level document statistics
#[derive(Debug, Clone, Default, PartialEq)]
pub struct LevelStats {
    pub obstacle_count: usize,
    pub npc_spawn_count: usize,
    pub total_npc_count: u32,
    pub fate_thread_count: usize,
    pub trigger_count: usize,
    pub biome_paint_count: usize,
    pub has_boss: bool,
}

impl LevelDoc {
    /// Validate the level document and return any issues found
    pub fn validate(&self) -> Vec<LevelValidationIssue> {
        let mut issues = Vec::new();

        // Check for empty title
        if self.title.trim().is_empty() {
            issues.push(LevelValidationIssue::warning("Level has no title"));
        }

        // Check for empty biome
        if self.biome.trim().is_empty() {
            issues.push(LevelValidationIssue::warning("Level has no biome specified"));
        }

        // Check for duplicate obstacle IDs
        let mut seen_ids = std::collections::HashSet::new();
        for obstacle in &self.obstacles {
            if !obstacle.id.is_empty() && !seen_ids.insert(&obstacle.id) {
                issues.push(LevelValidationIssue::error(format!(
                    "Duplicate obstacle ID: {}",
                    obstacle.id
                )));
            }
        }

        // Check for NPC spawns with zero count
        for (i, npc) in self.npcs.iter().enumerate() {
            if npc.count == 0 {
                issues.push(LevelValidationIssue::warning(format!(
                    "NPC spawn {} has zero count",
                    i
                )));
            }
            if npc.spawn.radius < 0.0 {
                issues.push(LevelValidationIssue::error(format!(
                    "NPC spawn {} has negative radius",
                    i
                )));
            }
        }

        // Check for fate threads with no triggers
        for thread in &self.fate_threads {
            if thread.triggers.is_empty() {
                issues.push(LevelValidationIssue::warning(format!(
                    "Fate thread '{}' has no triggers",
                    thread.name
                )));
            }
            if thread.ops.is_empty() {
                issues.push(LevelValidationIssue::warning(format!(
                    "Fate thread '{}' has no operations",
                    thread.name
                )));
            }
        }

        issues
    }

    /// Check if the level document is valid (no errors)
    pub fn is_valid(&self) -> bool {
        !self.validate().iter().any(|issue| issue.is_error)
    }

    /// Get statistics about the level document
    pub fn stats(&self) -> LevelStats {
        let total_npc_count: u32 = self.npcs.iter().map(|n| n.count).sum();
        let trigger_count: usize = self.fate_threads.iter().map(|t| t.triggers.len()).sum();
        let has_boss = !self.boss.director_budget_script.is_empty()
            || !self.boss.phase_script.is_empty();

        LevelStats {
            obstacle_count: self.obstacles.len(),
            npc_spawn_count: self.npcs.len(),
            total_npc_count,
            fate_thread_count: self.fate_threads.len(),
            trigger_count,
            biome_paint_count: self.biome_paints.len(),
            has_boss,
        }
    }

    /// Find obstacles by tag
    pub fn find_obstacles_by_tag(&self, tag: &str) -> Vec<&Obstacle> {
        self.obstacles
            .iter()
            .filter(|o| o.tags.iter().any(|t| t == tag))
            .collect()
    }

    /// Find NPCs by archetype
    pub fn find_npcs_by_archetype(&self, archetype: &str) -> Vec<&NpcSpawn> {
        self.npcs
            .iter()
            .filter(|n| n.archetype == archetype)
            .collect()
    }
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
        let mut doc = LevelDoc {
            title: "Test Level".to_string(),
            biome: "Forest".to_string(),
            seed: 12345,
            sky: Sky {
                time_of_day: "dawn".to_string(),
                weather: "fog".to_string(),
            },
            ..Default::default()
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

    #[test]
    fn test_validation_issue_error() {
        let issue = LevelValidationIssue::error("Test error");
        assert!(issue.is_error);
        assert_eq!(issue.message, "Test error");
    }

    #[test]
    fn test_validation_issue_warning() {
        let issue = LevelValidationIssue::warning("Test warning");
        assert!(!issue.is_error);
        assert_eq!(issue.message, "Test warning");
    }

    #[test]
    fn test_validate_empty_level_has_warnings() {
        let doc = LevelDoc::default();
        let issues = doc.validate();
        
        // Should have warnings for empty title and biome
        assert!(issues.iter().any(|i| i.message.contains("title")));
        assert!(issues.iter().any(|i| i.message.contains("biome")));
        // All should be warnings, not errors
        assert!(issues.iter().all(|i| !i.is_error));
    }

    #[test]
    fn test_validate_duplicate_obstacle_ids() {
        let mut doc = LevelDoc {
            title: "Test".to_string(),
            biome: "Forest".to_string(),
            ..Default::default()
        };
        doc.obstacles.push(Obstacle { id: "rock".to_string(), ..Default::default() });
        doc.obstacles.push(Obstacle { id: "rock".to_string(), ..Default::default() });

        let issues = doc.validate();
        let dup_error = issues.iter().find(|i| i.message.contains("Duplicate"));
        assert!(dup_error.is_some());
        assert!(dup_error.unwrap().is_error);
    }

    #[test]
    fn test_validate_zero_npc_count_warning() {
        let mut doc = LevelDoc {
            title: "Test".to_string(),
            biome: "Forest".to_string(),
            ..Default::default()
        };
        doc.npcs.push(NpcSpawn {
            archetype: "Guard".to_string(),
            count: 0,
            spawn: Spawn::default(),
            behavior: "Patrol".to_string(),
        });

        let issues = doc.validate();
        assert!(issues.iter().any(|i| i.message.contains("zero count")));
    }

    #[test]
    fn test_validate_negative_spawn_radius_error() {
        let mut doc = LevelDoc {
            title: "Test".to_string(),
            biome: "Forest".to_string(),
            ..Default::default()
        };
        doc.npcs.push(NpcSpawn {
            archetype: "Guard".to_string(),
            count: 5,
            spawn: Spawn { pos: [0.0, 0.0, 0.0], radius: -5.0 },
            behavior: "Patrol".to_string(),
        });

        let issues = doc.validate();
        let error = issues.iter().find(|i| i.message.contains("negative radius"));
        assert!(error.is_some());
        assert!(error.unwrap().is_error);
    }

    #[test]
    fn test_validate_fate_thread_no_triggers() {
        let mut doc = LevelDoc {
            title: "Test".to_string(),
            biome: "Forest".to_string(),
            ..Default::default()
        };
        doc.fate_threads.push(FateThread {
            name: "EmptyThread".to_string(),
            triggers: vec![],
            ops: vec![DirectorOp::default()],
        });

        let issues = doc.validate();
        assert!(issues.iter().any(|i| i.message.contains("no triggers")));
    }

    #[test]
    fn test_is_valid_with_errors() {
        let mut doc = LevelDoc::default();
        doc.obstacles.push(Obstacle { id: "dup".to_string(), ..Default::default() });
        doc.obstacles.push(Obstacle { id: "dup".to_string(), ..Default::default() });

        assert!(!doc.is_valid());
    }

    #[test]
    fn test_is_valid_with_only_warnings() {
        // Empty doc has warnings but no errors
        let doc = LevelDoc::default();
        assert!(doc.is_valid());
    }

    #[test]
    fn test_stats_empty_level() {
        let doc = LevelDoc::default();
        let stats = doc.stats();

        assert_eq!(stats.obstacle_count, 0);
        assert_eq!(stats.npc_spawn_count, 0);
        assert_eq!(stats.total_npc_count, 0);
        assert_eq!(stats.fate_thread_count, 0);
        assert_eq!(stats.trigger_count, 0);
        assert_eq!(stats.biome_paint_count, 0);
        assert!(!stats.has_boss);
    }

    #[test]
    fn test_stats_populated_level() {
        let mut doc = LevelDoc::default();
        doc.obstacles.push(Obstacle::default());
        doc.obstacles.push(Obstacle::default());
        doc.npcs.push(NpcSpawn { count: 5, ..Default::default() });
        doc.npcs.push(NpcSpawn { count: 3, ..Default::default() });
        doc.fate_threads.push(FateThread {
            name: "Thread1".to_string(),
            triggers: vec![Trigger::default(), Trigger::default()],
            ops: vec![],
        });
        doc.biome_paints.push(BiomePaint::GrassDense {
            area: Circle { cx: 0, cz: 0, radius: 10 },
        });
        doc.boss.director_budget_script = "boss_script".to_string();

        let stats = doc.stats();
        assert_eq!(stats.obstacle_count, 2);
        assert_eq!(stats.npc_spawn_count, 2);
        assert_eq!(stats.total_npc_count, 8);
        assert_eq!(stats.fate_thread_count, 1);
        assert_eq!(stats.trigger_count, 2);
        assert_eq!(stats.biome_paint_count, 1);
        assert!(stats.has_boss);
    }

    #[test]
    fn test_find_obstacles_by_tag() {
        let mut doc = LevelDoc::default();
        doc.obstacles.push(Obstacle { 
            id: "rock1".to_string(), 
            tags: vec!["solid".to_string(), "natural".to_string()],
            ..Default::default() 
        });
        doc.obstacles.push(Obstacle { 
            id: "rock2".to_string(), 
            tags: vec!["solid".to_string()],
            ..Default::default() 
        });
        doc.obstacles.push(Obstacle { 
            id: "tree".to_string(), 
            tags: vec!["natural".to_string()],
            ..Default::default() 
        });

        let solid = doc.find_obstacles_by_tag("solid");
        assert_eq!(solid.len(), 2);
        
        let natural = doc.find_obstacles_by_tag("natural");
        assert_eq!(natural.len(), 2);
        
        let empty = doc.find_obstacles_by_tag("nonexistent");
        assert!(empty.is_empty());
    }

    #[test]
    fn test_find_npcs_by_archetype() {
        let mut doc = LevelDoc::default();
        doc.npcs.push(NpcSpawn { archetype: "Guard".to_string(), count: 2, ..Default::default() });
        doc.npcs.push(NpcSpawn { archetype: "Archer".to_string(), count: 3, ..Default::default() });
        doc.npcs.push(NpcSpawn { archetype: "Guard".to_string(), count: 1, ..Default::default() });

        let guards = doc.find_npcs_by_archetype("Guard");
        assert_eq!(guards.len(), 2);
        
        let archers = doc.find_npcs_by_archetype("Archer");
        assert_eq!(archers.len(), 1);
        
        let empty = doc.find_npcs_by_archetype("Mage");
        assert!(empty.is_empty());
    }
}
