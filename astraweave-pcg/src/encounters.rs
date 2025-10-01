//! Encounter generation with constraints

use crate::SeedRng;
use glam::IVec2;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Type of encounter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EncounterKind {
    /// Combat encounter with enemy types and count
    Combat {
        enemy_types: Vec<String>,
        count: u32,
    },
    /// Loot encounter with items
    Loot { items: Vec<String> },
    /// Ambient event (NPC, cutscene, etc.)
    Ambient { event_id: String },
}

/// An encounter placed in the world
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Encounter {
    pub kind: EncounterKind,
    pub position: IVec2,
    pub difficulty: f32,
    pub metadata: BTreeMap<String, String>,
}

/// Constraints for encounter generation
#[derive(Debug, Clone)]
pub struct EncounterConstraints {
    /// Minimum and maximum position bounds
    pub bounds: (IVec2, IVec2),
    /// Minimum spacing between encounters
    pub min_spacing: f32,
    /// Difficulty range
    pub difficulty_range: (f32, f32),
}

impl Default for EncounterConstraints {
    fn default() -> Self {
        Self {
            bounds: (IVec2::ZERO, IVec2::new(100, 100)),
            min_spacing: 10.0,
            difficulty_range: (1.0, 5.0),
        }
    }
}

/// Generator for encounters
pub struct EncounterGenerator {
    pub constraints: EncounterConstraints,
}

impl EncounterGenerator {
    pub fn new(constraints: EncounterConstraints) -> Self {
        Self { constraints }
    }

    /// Generate a set of encounters
    pub fn generate(&self, rng: &mut SeedRng, count: u32) -> Vec<Encounter> {
        let mut encounters = Vec::new();
        let mut positions = Vec::new();

        let max_attempts = count * 10; // Prevent infinite loops
        let mut attempts = 0;

        while encounters.len() < count as usize && attempts < max_attempts {
            attempts += 1;

            // Generate position with spacing constraint
            let pos = self.generate_position(rng, &positions);

            // Check spacing
            if self.check_spacing(&pos, &positions) {
                positions.push(pos);

                // Generate encounter type
                let kind = self.generate_kind(rng);
                let difficulty = rng.gen_range(
                    self.constraints.difficulty_range.0..=self.constraints.difficulty_range.1,
                );

                encounters.push(Encounter {
                    kind,
                    position: pos,
                    difficulty,
                    metadata: BTreeMap::new(),
                });
            }
        }

        encounters
    }

    fn generate_position(&self, rng: &mut SeedRng, _existing: &[IVec2]) -> IVec2 {
        let x = rng.gen_range(self.constraints.bounds.0.x..=self.constraints.bounds.1.x);
        let y = rng.gen_range(self.constraints.bounds.0.y..=self.constraints.bounds.1.y);
        IVec2::new(x, y)
    }

    fn check_spacing(&self, pos: &IVec2, existing: &[IVec2]) -> bool {
        for other in existing {
            let dist = (*pos - *other).as_vec2().length();
            if dist < self.constraints.min_spacing {
                return false;
            }
        }
        true
    }

    fn generate_kind(&self, rng: &mut SeedRng) -> EncounterKind {
        match rng.gen_range(0..3) {
            0 => EncounterKind::Combat {
                enemy_types: vec![rng
                    .choose(&["goblin", "orc", "skeleton", "wolf"])
                    .unwrap_or(&"goblin")
                    .to_string()],
                count: rng.gen_range(1..=3),
            },
            1 => EncounterKind::Loot {
                items: vec![rng
                    .choose(&["health_potion", "mana_potion", "gold", "weapon"])
                    .unwrap_or(&"health_potion")
                    .to_string()],
            },
            _ => EncounterKind::Ambient {
                event_id: rng
                    .choose(&["merchant", "quest_giver", "ambient_npc"])
                    .unwrap_or(&"merchant")
                    .to_string(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deterministic_generation() {
        let constraints = EncounterConstraints::default();
        let gen = EncounterGenerator::new(constraints);

        let mut rng1 = SeedRng::new(42, "test");
        let mut rng2 = SeedRng::new(42, "test");

        let encounters1 = gen.generate(&mut rng1, 10);
        let encounters2 = gen.generate(&mut rng2, 10);

        assert_eq!(encounters1.len(), encounters2.len());

        for (e1, e2) in encounters1.iter().zip(encounters2.iter()) {
            assert_eq!(e1.position, e2.position);
            assert!((e1.difficulty - e2.difficulty).abs() < 0.001);
        }
    }

    #[test]
    fn test_spacing_constraint() {
        let constraints = EncounterConstraints {
            bounds: (IVec2::ZERO, IVec2::new(50, 50)),
            min_spacing: 10.0,
            difficulty_range: (1.0, 5.0),
        };
        let gen = EncounterGenerator::new(constraints);

        let mut rng = SeedRng::new(42, "test");
        let encounters = gen.generate(&mut rng, 10);

        // Check all pairs for spacing
        for i in 0..encounters.len() {
            for j in (i + 1)..encounters.len() {
                let dist = (encounters[i].position - encounters[j].position)
                    .as_vec2()
                    .length();
                assert!(
                    dist >= 10.0,
                    "Encounters {} and {} too close: {}",
                    i,
                    j,
                    dist
                );
            }
        }
    }

    #[test]
    fn test_bounds_constraint() {
        let constraints = EncounterConstraints {
            bounds: (IVec2::new(10, 10), IVec2::new(20, 20)),
            min_spacing: 2.0,
            difficulty_range: (1.0, 5.0),
        };
        let gen = EncounterGenerator::new(constraints);

        let mut rng = SeedRng::new(42, "test");
        let encounters = gen.generate(&mut rng, 10);

        for enc in &encounters {
            assert!(enc.position.x >= 10 && enc.position.x <= 20);
            assert!(enc.position.y >= 10 && enc.position.y <= 20);
        }
    }

    #[test]
    fn test_difficulty_range() {
        let constraints = EncounterConstraints {
            bounds: (IVec2::ZERO, IVec2::new(100, 100)),
            min_spacing: 10.0,
            difficulty_range: (2.0, 4.0),
        };
        let gen = EncounterGenerator::new(constraints);

        let mut rng = SeedRng::new(42, "test");
        let encounters = gen.generate(&mut rng, 10);

        for enc in &encounters {
            assert!(
                enc.difficulty >= 2.0 && enc.difficulty <= 4.0,
                "Difficulty out of range: {}",
                enc.difficulty
            );
        }
    }
}
