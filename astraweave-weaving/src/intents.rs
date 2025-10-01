//! Intent proposers - generate actions based on detected patterns

use std::collections::BTreeMap;

/// A weave intent proposed by the system
#[derive(Debug, Clone)]
pub struct WeaveIntent {
    /// Type of intent (e.g., "spawn_aid_event", "suggest_craft")
    pub kind: String,
    /// Priority for adjudication (higher = more urgent)
    pub priority: f32,
    /// Budget cost to execute
    pub cost: u32,
    /// Cooldown key for rate limiting
    pub cooldown_key: String,
    /// Intent-specific payload data
    pub payload: BTreeMap<String, String>,
}

impl WeaveIntent {
    pub fn new(kind: impl Into<String>) -> Self {
        Self {
            kind: kind.into(),
            priority: 0.5,
            cost: 1,
            cooldown_key: String::new(),
            payload: BTreeMap::new(),
        }
    }

    pub fn with_priority(mut self, priority: f32) -> Self {
        self.priority = priority;
        self
    }

    pub fn with_cost(mut self, cost: u32) -> Self {
        self.cost = cost;
        self
    }

    pub fn with_cooldown(mut self, cooldown_key: impl Into<String>) -> Self {
        self.cooldown_key = cooldown_key.into();
        self
    }

    pub fn with_payload(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.payload.insert(key.into(), value.into());
        self
    }
}

/// Trait for intent proposers
pub trait IntentProposer: Send + Sync {
    /// Propose intents based on detected patterns
    fn propose(&self, patterns: &BTreeMap<String, f32>, seed: u64) -> Vec<WeaveIntent>;

    /// Get the name of this proposer
    fn name(&self) -> &str;
}

/// Proposer for aid events (healers, supplies) when health is low
pub struct AidEventProposer {
    pub strength_threshold: f32,
}

impl IntentProposer for AidEventProposer {
    fn propose(&self, patterns: &BTreeMap<String, f32>, _seed: u64) -> Vec<WeaveIntent> {
        if let Some(&strength) = patterns.get("low_health_cluster") {
            if strength >= self.strength_threshold {
                return vec![WeaveIntent::new("spawn_aid_event")
                    .with_priority(strength)
                    .with_cost(10)
                    .with_cooldown("aid_event")
                    .with_payload("event_type", "wandering_healer")];
            }
        }
        Vec::new()
    }

    fn name(&self) -> &str {
        "aid_event"
    }
}

/// Proposer for supply drops when resources are scarce
pub struct SupplyDropProposer {
    pub strength_threshold: f32,
}

impl IntentProposer for SupplyDropProposer {
    fn propose(&self, patterns: &BTreeMap<String, f32>, _seed: u64) -> Vec<WeaveIntent> {
        let mut intents = Vec::new();

        for (pattern_id, strength) in patterns {
            if pattern_id.starts_with("resource_scarce_") && *strength >= self.strength_threshold {
                let resource = pattern_id.strip_prefix("resource_scarce_").unwrap();
                intents.push(
                    WeaveIntent::new("spawn_supply_drop")
                        .with_priority(*strength)
                        .with_cost(8)
                        .with_cooldown(format!("supply_drop_{}", resource))
                        .with_payload("resource_type", resource),
                );
            }
        }

        intents
    }

    fn name(&self) -> &str {
        "supply_drop"
    }
}

/// Proposer for faction mediators when conflicts escalate
pub struct MediatorProposer {
    pub strength_threshold: f32,
}

impl IntentProposer for MediatorProposer {
    fn propose(&self, patterns: &BTreeMap<String, f32>, _seed: u64) -> Vec<WeaveIntent> {
        let mut intents = Vec::new();

        for (pattern_id, strength) in patterns {
            if pattern_id.starts_with("faction_conflict_") && *strength >= self.strength_threshold {
                let faction = pattern_id.strip_prefix("faction_conflict_").unwrap();
                intents.push(
                    WeaveIntent::new("spawn_mediator")
                        .with_priority(*strength)
                        .with_cost(15)
                        .with_cooldown(format!("mediator_{}", faction))
                        .with_payload("faction", faction),
                );
            }
        }

        intents
    }

    fn name(&self) -> &str {
        "mediator"
    }
}

/// Proposer for scavenger patrols during high combat
pub struct ScavengerPatrolProposer {
    pub strength_threshold: f32,
}

impl IntentProposer for ScavengerPatrolProposer {
    fn propose(&self, patterns: &BTreeMap<String, f32>, seed: u64) -> Vec<WeaveIntent> {
        if let Some(&strength) = patterns.get("high_combat_intensity") {
            if strength >= self.strength_threshold {
                // Use seed for deterministic variation
                let patrol_type = if seed % 2 == 0 {
                    "looters"
                } else {
                    "scavengers"
                };

                return vec![WeaveIntent::new("spawn_patrol")
                    .with_priority(strength * 0.8) // Lower priority than aid
                    .with_cost(12)
                    .with_cooldown("scavenger_patrol")
                    .with_payload("patrol_type", patrol_type)];
            }
        }
        Vec::new()
    }

    fn name(&self) -> &str {
        "scavenger_patrol"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aid_event_proposal() {
        let proposer = AidEventProposer {
            strength_threshold: 0.5,
        };

        let mut patterns = BTreeMap::new();
        patterns.insert("low_health_cluster".to_string(), 0.8);

        let intents = proposer.propose(&patterns, 42);
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].kind, "spawn_aid_event");
        assert_eq!(intents[0].priority, 0.8);
        assert_eq!(intents[0].cost, 10);
        assert_eq!(intents[0].cooldown_key, "aid_event");
    }

    #[test]
    fn test_aid_event_below_threshold() {
        let proposer = AidEventProposer {
            strength_threshold: 0.5,
        };

        let mut patterns = BTreeMap::new();
        patterns.insert("low_health_cluster".to_string(), 0.3);

        let intents = proposer.propose(&patterns, 42);
        assert_eq!(intents.len(), 0);
    }

    #[test]
    fn test_supply_drop_proposal() {
        let proposer = SupplyDropProposer {
            strength_threshold: 0.5,
        };

        let mut patterns = BTreeMap::new();
        patterns.insert("resource_scarce_food".to_string(), 0.8);
        patterns.insert("resource_scarce_water".to_string(), 0.3);

        let intents = proposer.propose(&patterns, 42);
        assert_eq!(intents.len(), 1); // Only food above threshold
        assert_eq!(intents[0].kind, "spawn_supply_drop");
        assert_eq!(intents[0].payload.get("resource_type").unwrap(), "food");
    }

    #[test]
    fn test_mediator_proposal() {
        let proposer = MediatorProposer {
            strength_threshold: 0.6,
        };

        let mut patterns = BTreeMap::new();
        patterns.insert("faction_conflict_red".to_string(), 0.9);

        let intents = proposer.propose(&patterns, 42);
        assert_eq!(intents.len(), 1);
        assert_eq!(intents[0].kind, "spawn_mediator");
        assert_eq!(intents[0].payload.get("faction").unwrap(), "red");
    }

    #[test]
    fn test_scavenger_patrol_deterministic() {
        let proposer = ScavengerPatrolProposer {
            strength_threshold: 0.5,
        };

        let mut patterns = BTreeMap::new();
        patterns.insert("high_combat_intensity".to_string(), 0.8);

        // Same seed should produce same patrol type
        let intents1 = proposer.propose(&patterns, 42);
        let intents2 = proposer.propose(&patterns, 42);
        assert_eq!(intents1[0].payload, intents2[0].payload);

        // Different seed should produce different type
        let intents3 = proposer.propose(&patterns, 43);
        assert_ne!(
            intents1[0].payload.get("patrol_type"),
            intents3[0].payload.get("patrol_type")
        );
    }

    #[test]
    fn test_multiple_proposers() {
        let proposers: Vec<Box<dyn IntentProposer>> = vec![
            Box::new(AidEventProposer {
                strength_threshold: 0.5,
            }),
            Box::new(SupplyDropProposer {
                strength_threshold: 0.5,
            }),
        ];

        let mut patterns = BTreeMap::new();
        patterns.insert("low_health_cluster".to_string(), 0.8);
        patterns.insert("resource_scarce_food".to_string(), 0.7);

        let mut all_intents = Vec::new();
        for proposer in &proposers {
            all_intents.extend(proposer.propose(&patterns, 42));
        }

        assert_eq!(all_intents.len(), 2);
    }
}
