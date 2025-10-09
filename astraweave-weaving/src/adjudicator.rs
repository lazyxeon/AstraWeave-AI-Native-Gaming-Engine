//! Weave adjudicator - enforces budget, cooldowns, and prioritizes intents

use crate::intents::WeaveIntent;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Configuration for the weave adjudicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeaveConfig {
    /// Budget per tick for weave actions
    pub budget_per_tick: u32,
    /// Cooldown durations (in ticks) for different intent types
    pub cooldowns: BTreeMap<String, u32>,
    /// Minimum priority to consider (filter out low-priority intents)
    pub min_priority: f32,
}

impl Default for WeaveConfig {
    fn default() -> Self {
        let mut cooldowns = BTreeMap::new();
        cooldowns.insert("aid_event".to_string(), 300); // 5 seconds at 60Hz
        cooldowns.insert("supply_drop_food".to_string(), 600);
        cooldowns.insert("supply_drop_water".to_string(), 600);
        cooldowns.insert("mediator".to_string(), 900);
        cooldowns.insert("scavenger_patrol".to_string(), 450);

        Self {
            budget_per_tick: 20,
            cooldowns,
            min_priority: 0.3,
        }
    }
}

impl WeaveConfig {
    /// Load configuration from TOML string
    pub fn from_toml(toml_str: &str) -> Result<Self> {
        toml::from_str(toml_str).context("Failed to parse weave config TOML")
    }

    /// Serialize configuration to TOML string
    pub fn to_toml(&self) -> Result<String> {
        toml::to_string_pretty(self).context("Failed to serialize weave config to TOML")
    }
}

/// Tracks cooldown state for weave intents
#[derive(Debug, Default)]
pub struct WeaveAdjudicator {
    config: WeaveConfig,
    /// Active cooldowns: key -> ticks remaining
    cooldowns: BTreeMap<String, u32>,
    /// Budget spent this tick
    budget_spent: u32,
}

impl WeaveAdjudicator {
    /// Create a new adjudicator with default config
    pub fn new() -> Self {
        Self::with_config(WeaveConfig::default())
    }

    /// Create a new adjudicator with custom config
    pub fn with_config(config: WeaveConfig) -> Self {
        Self {
            config,
            cooldowns: BTreeMap::new(),
            budget_spent: 0,
        }
    }

    /// Reset budget for a new tick
    pub fn begin_tick(&mut self) {
        self.budget_spent = 0;

        // Decrement all cooldowns
        for cooldown in self.cooldowns.values_mut() {
            *cooldown = cooldown.saturating_sub(1);
        }

        // Remove expired cooldowns
        self.cooldowns.retain(|_, ticks| *ticks > 0);
    }

    /// Check if an intent is on cooldown
    pub fn is_on_cooldown(&self, cooldown_key: &str) -> bool {
        self.cooldowns.contains_key(cooldown_key)
    }

    /// Get remaining cooldown ticks for a key
    pub fn cooldown_remaining(&self, cooldown_key: &str) -> u32 {
        self.cooldowns.get(cooldown_key).copied().unwrap_or(0)
    }

    /// Check if we have enough budget remaining
    pub fn has_budget(&self, cost: u32) -> bool {
        self.budget_spent + cost <= self.config.budget_per_tick
    }

    /// Adjudicate a list of proposed intents, returning approved intents
    pub fn adjudicate(&mut self, mut intents: Vec<WeaveIntent>) -> Vec<WeaveIntent> {
        // Filter by minimum priority
        intents.retain(|intent| intent.priority >= self.config.min_priority);

        // Sort by priority (descending), then by cost (ascending) for stable ordering
        intents.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.cost.cmp(&b.cost))
                .then_with(|| a.kind.cmp(&b.kind))
        });

        let mut approved = Vec::new();

        for intent in intents {
            // Check cooldown
            if !intent.cooldown_key.is_empty() && self.is_on_cooldown(&intent.cooldown_key) {
                continue;
            }

            // Check budget
            if !self.has_budget(intent.cost) {
                continue; // Budget exhausted, skip remaining (they're lower priority)
            }

            // Approve and spend budget
            self.budget_spent += intent.cost;

            // Start cooldown if specified
            if !intent.cooldown_key.is_empty() {
                let cooldown_duration = self
                    .config
                    .cooldowns
                    .get(&intent.cooldown_key)
                    .copied()
                    .unwrap_or(300); // Default 5 seconds
                self.cooldowns
                    .insert(intent.cooldown_key.clone(), cooldown_duration);
            }

            approved.push(intent);
        }

        approved
    }

    /// Get current config
    pub fn config(&self) -> &WeaveConfig {
        &self.config
    }

    /// Get current budget spent this tick
    pub fn budget_spent(&self) -> u32 {
        self.budget_spent
    }

    /// Get remaining budget for this tick
    pub fn budget_remaining(&self) -> u32 {
        self.config
            .budget_per_tick
            .saturating_sub(self.budget_spent)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_intent(kind: &str, priority: f32, cost: u32, cooldown: &str) -> WeaveIntent {
        WeaveIntent::new(kind)
            .with_priority(priority)
            .with_cost(cost)
            .with_cooldown(cooldown)
    }

    #[test]
    fn test_budget_enforcement() {
        let mut adjudicator = WeaveAdjudicator::new();
        adjudicator.begin_tick();

        let intents = vec![
            make_intent("intent_a", 0.9, 15, ""),
            make_intent("intent_b", 0.8, 10, ""),
        ];

        let approved = adjudicator.adjudicate(intents);

        // Should approve both (15 + 10 = 25 > 20 budget, so only first approved)
        // Wait, budget is 20, so 15 + 10 = 25 exceeds it
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].kind, "intent_a");
        assert_eq!(adjudicator.budget_spent(), 15);
    }

    #[test]
    fn test_cooldown_enforcement() {
        let mut adjudicator = WeaveAdjudicator::new();
        adjudicator.begin_tick();

        let intents = vec![make_intent("spawn_aid", 0.9, 10, "aid_event")];

        // First approval should succeed
        let approved = adjudicator.adjudicate(intents.clone());
        assert_eq!(approved.len(), 1);
        assert!(adjudicator.is_on_cooldown("aid_event"));

        // Second approval should fail (on cooldown)
        adjudicator.begin_tick();
        let approved = adjudicator.adjudicate(intents);
        assert_eq!(approved.len(), 0);
    }

    #[test]
    fn test_cooldown_expiration() {
        let mut config = WeaveConfig::default();
        config.cooldowns.insert("test_cooldown".to_string(), 3);

        let mut adjudicator = WeaveAdjudicator::with_config(config);
        adjudicator.begin_tick();

        let intents = vec![make_intent("test", 0.9, 5, "test_cooldown")];

        // Activate cooldown
        adjudicator.adjudicate(intents.clone());
        assert_eq!(adjudicator.cooldown_remaining("test_cooldown"), 3);

        // Tick 1: still on cooldown
        adjudicator.begin_tick();
        assert_eq!(adjudicator.cooldown_remaining("test_cooldown"), 2);

        // Tick 2: still on cooldown
        adjudicator.begin_tick();
        assert_eq!(adjudicator.cooldown_remaining("test_cooldown"), 1);

        // Tick 3: cooldown expires
        adjudicator.begin_tick();
        assert!(!adjudicator.is_on_cooldown("test_cooldown"));

        // Now should approve again
        let approved = adjudicator.adjudicate(intents);
        assert_eq!(approved.len(), 1);
    }

    #[test]
    fn test_priority_sorting() {
        let mut adjudicator = WeaveAdjudicator::new();
        adjudicator.begin_tick();

        let intents = vec![
            make_intent("low", 0.5, 5, ""),
            make_intent("high", 0.9, 5, ""),
            make_intent("medium", 0.7, 5, ""),
        ];

        let approved = adjudicator.adjudicate(intents);

        // Should approve high, medium, low (up to budget)
        assert_eq!(approved[0].kind, "high");
        assert_eq!(approved[1].kind, "medium");
        assert_eq!(approved[2].kind, "low");
    }

    #[test]
    fn test_min_priority_filter() {
        let mut adjudicator = WeaveAdjudicator::new();
        adjudicator.begin_tick();

        let intents = vec![
            make_intent("too_low", 0.2, 5, ""),
            make_intent("ok", 0.5, 5, ""),
        ];

        let approved = adjudicator.adjudicate(intents);

        // Only "ok" should pass (min_priority is 0.3)
        assert_eq!(approved.len(), 1);
        assert_eq!(approved[0].kind, "ok");
    }

    #[test]
    fn test_config_toml() {
        let config = WeaveConfig::default();
        let toml_str = config.to_toml().unwrap();
        let parsed = WeaveConfig::from_toml(&toml_str).unwrap();

        assert_eq!(parsed.budget_per_tick, config.budget_per_tick);
        assert_eq!(parsed.min_priority, config.min_priority);
        assert_eq!(parsed.cooldowns.len(), config.cooldowns.len());
    }

    #[test]
    fn test_budget_reset_per_tick() {
        let mut adjudicator = WeaveAdjudicator::new();

        // Tick 1: spend all budget
        adjudicator.begin_tick();
        let intents = vec![make_intent("expensive", 0.9, 20, "")];
        adjudicator.adjudicate(intents);
        assert_eq!(adjudicator.budget_remaining(), 0);

        // Tick 2: budget should reset
        adjudicator.begin_tick();
        assert_eq!(adjudicator.budget_remaining(), 20);
    }

    #[test]
    fn test_deterministic_tie_breaking() {
        let mut adjudicator = WeaveAdjudicator::new();
        adjudicator.begin_tick();

        // Same priority, different costs and kinds
        let intents = vec![
            make_intent("zebra", 0.8, 10, ""),
            make_intent("alpha", 0.8, 5, ""),
            make_intent("beta", 0.8, 5, ""),
        ];

        let approved = adjudicator.adjudicate(intents);

        // Should sort by: priority (same), then cost (5 < 10), then kind (alpha < beta < zebra)
        assert_eq!(approved[0].kind, "alpha");
        assert_eq!(approved[1].kind, "beta");
    }
}
