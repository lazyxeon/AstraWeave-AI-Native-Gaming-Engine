//! Common test utilities for astraweave-weaving tests

use astraweave_weaving::*;
use astraweave_weaving::patterns::*;
use std::collections::BTreeMap;

/// Simple RNG for deterministic tests (Linear Congruential Generator)
pub struct TestRng {
    state: u64,
}

impl TestRng {
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next(&mut self) -> u64 {
        // LCG parameters (from Numerical Recipes)
        const A: u64 = 1664525;
        const C: u64 = 1013904223;
        self.state = self.state.wrapping_mul(A).wrapping_add(C);
        self.state
    }
}

/// Create a mock WeaveConfig for testing
pub fn create_test_config() -> WeaveConfig {
    let mut cooldowns = BTreeMap::new();
    cooldowns.insert("aid_event".to_string(), 300);
    cooldowns.insert("supply_drop_food".to_string(), 600);
    cooldowns.insert("mediator".to_string(), 900);
    
    WeaveConfig {
        budget_per_tick: 20,
        cooldowns,
        min_priority: 0.3,
    }
}

/// Create a mock WeaveAdjudicator for testing
pub fn create_test_adjudicator() -> WeaveAdjudicator {
    WeaveAdjudicator::with_config(create_test_config())
}

/// Create a mock WeaveIntent for testing
pub fn create_test_intent(id: &str, priority: f32, cost: u32) -> WeaveIntent {
    WeaveIntent::new(id)
        .with_priority(priority)
        .with_cost(cost)
}

/// Create a test WorldMetrics with specific values
pub fn create_test_metrics(
    critical_health_count: usize,
    avg_health: f32,
    recent_damage_events: usize,
) -> WorldMetrics {
    WorldMetrics {
        avg_health,
        critical_health_count,
        resource_scarcity: BTreeMap::new(),
        faction_tensions: BTreeMap::new(),
        recent_damage_events,
        time_since_event: 0.0,
    }
}

/// Create a test RNG with fixed seed for deterministic tests
pub fn create_test_rng(seed: u64) -> TestRng {
    TestRng::new(seed)
}

/// Run a deterministic test function 3 times and verify identical results
/// Returns true if all 3 runs produce the same result hash
pub fn assert_deterministic_behavior<F, T>(seed: u64, test_fn: F)
where
    F: Fn(&mut TestRng) -> T,
    T: std::fmt::Debug + PartialEq,
{
    let mut results = Vec::new();
    
    for run in 0..3 {
        let mut rng = create_test_rng(seed);
        let result = test_fn(&mut rng);
        results.push(result);
        
        if run > 0 {
            assert_eq!(
                results[0], results[run],
                "Determinism violation: Run 0 != Run {}. Seed: {}",
                run, seed
            );
        }
    }
}

/// Compute a simple hash of adjudicator state for determinism tests
pub fn hash_adjudicator_state(adjudicator: &WeaveAdjudicator) -> u64 {
    // Simple state hash based on budget spent
    // This is sufficient for determinism testing within a single tick
    (adjudicator.budget_spent() as u64) * 1000 + (adjudicator.budget_remaining() as u64)
}

/// Create a test PatternDetector for low health clusters
pub fn create_low_health_detector() -> LowHealthClusterDetector {
    LowHealthClusterDetector { 
        threshold: 0.25,
        min_cluster_size: 3,
    }
}

/// Create a test PatternDetector for resource scarcity
pub fn create_resource_scarcity_detector() -> ResourceScarcityDetector {
    ResourceScarcityDetector { threshold: 0.5 }
}

/// Create a test PatternDetector for faction conflict
pub fn create_faction_conflict_detector() -> FactionConflictDetector {
    FactionConflictDetector { threshold: 0.6 }
}

/// Create a test PatternDetector for combat intensity
pub fn create_combat_intensity_detector() -> CombatIntensityDetector {
    CombatIntensityDetector { 
        events_threshold: 10,
        time_window: 5.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_creation() {
        let config = create_test_config();
        assert_eq!(config.budget_per_tick, 20);
        assert_eq!(config.min_priority, 0.3);
        assert!(config.cooldowns.contains_key("aid_event"));
    }

    #[test]
    fn test_rng_determinism() {
        let mut rng1 = create_test_rng(12345);
        let mut rng2 = create_test_rng(12345);
        
        for _ in 0..10 {
            assert_eq!(rng1.next(), rng2.next());
        }
    }

    #[test]
    fn test_deterministic_behavior_helper() {
        assert_deterministic_behavior(99999, |rng| {
            let mut sum = 0u64;
            for _ in 0..5 {
                sum = sum.wrapping_add(rng.next());
            }
            sum
        });
    }
}
