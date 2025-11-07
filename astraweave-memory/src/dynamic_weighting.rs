//! Dynamic weight adjustment for behavior tree nodes based on player patterns.
//!
//! This module provides adaptive behavior tree weighting that evolves based on
//! detected player patterns and preferences. Weights are adjusted to favor actions
//! that align with the player's playstyle and have historically high effectiveness.

use crate::{
    EpisodeCategory, MemoryStorage, PatternDetector, PlaystylePattern, PreferenceProfile,
    ProfileBuilder,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Behavior tree node type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BehaviorNodeType {
    /// Combat-focused actions
    Combat,
    /// Support actions (healing, buffs)
    Support,
    /// Exploration actions
    Exploration,
    /// Social/dialogue actions
    Social,
    /// Puzzle-solving actions
    Analytical,
    /// Resource management actions
    Defensive,
}

impl BehaviorNodeType {
    /// Map node type to episode category
    pub fn to_category(&self) -> EpisodeCategory {
        match self {
            BehaviorNodeType::Combat => EpisodeCategory::Combat,
            BehaviorNodeType::Support => EpisodeCategory::Combat,
            BehaviorNodeType::Exploration => EpisodeCategory::Exploration,
            BehaviorNodeType::Social => EpisodeCategory::Social,
            BehaviorNodeType::Analytical => EpisodeCategory::Puzzle,
            BehaviorNodeType::Defensive => EpisodeCategory::Combat,
        }
    }

    /// Map playstyle pattern to preferred node types
    pub fn from_pattern(pattern: PlaystylePattern) -> Vec<BehaviorNodeType> {
        match pattern {
            PlaystylePattern::Aggressive => vec![BehaviorNodeType::Combat],
            PlaystylePattern::Cautious => {
                vec![BehaviorNodeType::Defensive, BehaviorNodeType::Support]
            }
            PlaystylePattern::Explorative => vec![BehaviorNodeType::Exploration],
            PlaystylePattern::Social => vec![BehaviorNodeType::Social],
            PlaystylePattern::Analytical => vec![BehaviorNodeType::Analytical],
            PlaystylePattern::Efficient => vec![
                BehaviorNodeType::Combat,
                BehaviorNodeType::Support,
                BehaviorNodeType::Analytical,
            ],
        }
    }
}

/// Weight configuration for a behavior tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeWeight {
    /// Current weight (0.0 to 1.0)
    pub weight: f32,
    /// Base weight before adaptations
    pub base_weight: f32,
    /// Pattern-based adjustment
    pub pattern_bonus: f32,
    /// Effectiveness-based adjustment
    pub effectiveness_bonus: f32,
    /// Number of times this weight has been updated
    pub update_count: usize,
}

impl NodeWeight {
    /// Create new node weight with base value
    pub fn new(base_weight: f32) -> Self {
        Self {
            weight: base_weight.clamp(0.0, 1.0),
            base_weight: base_weight.clamp(0.0, 1.0),
            pattern_bonus: 0.0,
            effectiveness_bonus: 0.0,
            update_count: 0,
        }
    }

    /// Calculate final weight
    pub fn calculate(&mut self) -> f32 {
        self.weight =
            (self.base_weight + self.pattern_bonus + self.effectiveness_bonus).clamp(0.0, 1.0);
        self.weight
    }

    /// Reset to base weight
    pub fn reset(&mut self) {
        self.weight = self.base_weight;
        self.pattern_bonus = 0.0;
        self.effectiveness_bonus = 0.0;
        self.update_count = 0;
    }
}

/// Adaptive behavior tree weight manager
pub struct AdaptiveWeightManager {
    /// Current node weights
    weights: HashMap<BehaviorNodeType, NodeWeight>,
    /// Pattern detector for analysis
    #[allow(dead_code)]
    detector: PatternDetector,
    /// Profile builder for player modeling
    builder: ProfileBuilder,
    /// Learning rate for weight updates
    learning_rate: f32,
    /// Maximum bonus from patterns
    max_pattern_bonus: f32,
    /// Maximum bonus from effectiveness
    max_effectiveness_bonus: f32,
}

impl AdaptiveWeightManager {
    /// Create new adaptive weight manager
    pub fn new() -> Self {
        let mut weights = HashMap::new();

        // Initialize all node types with neutral weights
        for node_type in [
            BehaviorNodeType::Combat,
            BehaviorNodeType::Support,
            BehaviorNodeType::Exploration,
            BehaviorNodeType::Social,
            BehaviorNodeType::Analytical,
            BehaviorNodeType::Defensive,
        ] {
            weights.insert(node_type, NodeWeight::new(0.5));
        }

        Self {
            weights,
            detector: PatternDetector::new(),
            builder: ProfileBuilder::new(),
            learning_rate: 0.1,
            max_pattern_bonus: 0.3,
            max_effectiveness_bonus: 0.2,
        }
    }

    /// Create with custom parameters
    pub fn with_params(
        learning_rate: f32,
        max_pattern_bonus: f32,
        max_effectiveness_bonus: f32,
    ) -> Self {
        let mut manager = Self::new();
        manager.learning_rate = learning_rate;
        manager.max_pattern_bonus = max_pattern_bonus;
        manager.max_effectiveness_bonus = max_effectiveness_bonus;
        manager
    }

    /// Update weights based on player profile
    pub fn update_from_profile(&mut self, storage: &MemoryStorage) -> Result<()> {
        // Build player profile
        let profile = self
            .builder
            .build_profile(storage)
            .context("Failed to build player profile")?;

        // Update weights based on dominant patterns
        self.apply_pattern_bonuses(&profile);

        // Update weights based on category effectiveness
        self.apply_effectiveness_bonuses(&profile);

        // Recalculate final weights
        for weight in self.weights.values_mut() {
            weight.calculate();
            weight.update_count += 1;
        }

        Ok(())
    }

    /// Apply bonuses based on detected patterns
    fn apply_pattern_bonuses(&mut self, profile: &PreferenceProfile) {
        // Clear existing pattern bonuses
        for weight in self.weights.values_mut() {
            weight.pattern_bonus = 0.0;
        }

        // Apply bonuses for each dominant pattern
        for pattern_strength in &profile.dominant_patterns {
            let preferred_nodes = BehaviorNodeType::from_pattern(pattern_strength.pattern);
            let bonus_per_node = (pattern_strength.confidence * self.max_pattern_bonus)
                / preferred_nodes.len() as f32;

            for node_type in preferred_nodes {
                if let Some(weight) = self.weights.get_mut(&node_type) {
                    weight.pattern_bonus += bonus_per_node;
                    weight.pattern_bonus = weight.pattern_bonus.min(self.max_pattern_bonus);
                }
            }
        }
    }

    /// Apply bonuses based on category effectiveness
    fn apply_effectiveness_bonuses(&mut self, profile: &PreferenceProfile) {
        // Calculate average preference score
        let avg_preference: f32 = if !profile.preferred_categories.is_empty() {
            profile.preferred_categories.values().sum::<f32>()
                / profile.preferred_categories.len() as f32
        } else {
            0.5
        };

        // Apply effectiveness bonuses
        for (node_type, weight) in self.weights.iter_mut() {
            let category = node_type.to_category();

            if let Some(&preference) = profile.preferred_categories.get(&category) {
                // Bonus is proportional to how much this category exceeds average
                let relative_preference = (preference - avg_preference).max(0.0);
                weight.effectiveness_bonus =
                    (relative_preference * self.max_effectiveness_bonus * 2.0)
                        .min(self.max_effectiveness_bonus);
            } else {
                weight.effectiveness_bonus = 0.0;
            }
        }
    }

    /// Get weight for a specific node type
    pub fn get_weight(&self, node_type: BehaviorNodeType) -> f32 {
        self.weights
            .get(&node_type)
            .map(|w| w.weight)
            .unwrap_or(0.5)
    }

    /// Get all current weights
    pub fn get_all_weights(&self) -> HashMap<BehaviorNodeType, f32> {
        self.weights.iter().map(|(k, v)| (*k, v.weight)).collect()
    }

    /// Reset all weights to base values
    pub fn reset_weights(&mut self) {
        for weight in self.weights.values_mut() {
            weight.reset();
        }
    }

    /// Get weight details for debugging
    pub fn get_weight_details(&self, node_type: BehaviorNodeType) -> Option<&NodeWeight> {
        self.weights.get(&node_type)
    }

    /// Set base weight for a node type
    pub fn set_base_weight(&mut self, node_type: BehaviorNodeType, base_weight: f32) {
        if let Some(weight) = self.weights.get_mut(&node_type) {
            weight.base_weight = base_weight.clamp(0.0, 1.0);
            weight.calculate();
        }
    }

    /// Get learning rate
    pub fn learning_rate(&self) -> f32 {
        self.learning_rate
    }

    /// Get total update count across all weights
    pub fn total_updates(&self) -> usize {
        self.weights.values().map(|w| w.update_count).sum()
    }
}

impl Default for AdaptiveWeightManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::{EpisodeOutcome, Observation};

    fn create_test_episode(id: &str, category: EpisodeCategory, quality: f32) -> Episode {
        let mut episode = Episode::new(id.to_string(), category);
        episode.outcome = Some(EpisodeOutcome {
            success_rating: quality,
            player_satisfaction: quality,
            companion_effectiveness: 0.8,
            duration_ms: 10000,
            damage_dealt: 400.0,
            damage_taken: 60.0,
            resources_used: 100.0,
            failure_count: 0,
        });

        episode.add_observation(Observation::new(
            0,
            None,
            None,
            serde_json::json!({"player_health": 100}),
        ));

        episode
    }

    #[test]
    fn test_node_weight_creation() {
        let weight = NodeWeight::new(0.7);
        assert_eq!(weight.weight, 0.7);
        assert_eq!(weight.base_weight, 0.7);
        assert_eq!(weight.pattern_bonus, 0.0);
        assert_eq!(weight.effectiveness_bonus, 0.0);
        assert_eq!(weight.update_count, 0);
    }

    #[test]
    fn test_node_weight_calculation() {
        let mut weight = NodeWeight::new(0.5);
        weight.pattern_bonus = 0.2;
        weight.effectiveness_bonus = 0.1;

        let final_weight = weight.calculate();
        assert_eq!(final_weight, 0.8);
        assert_eq!(weight.weight, 0.8);
    }

    #[test]
    fn test_node_weight_clamping() {
        let mut weight = NodeWeight::new(0.5);
        weight.pattern_bonus = 0.4;
        weight.effectiveness_bonus = 0.3;

        let final_weight = weight.calculate();
        assert_eq!(final_weight, 1.0); // Clamped to max
    }

    #[test]
    fn test_node_weight_reset() {
        let mut weight = NodeWeight::new(0.5);
        weight.pattern_bonus = 0.2;
        weight.effectiveness_bonus = 0.1;
        weight.update_count = 5;
        weight.calculate();

        weight.reset();
        assert_eq!(weight.weight, 0.5);
        assert_eq!(weight.pattern_bonus, 0.0);
        assert_eq!(weight.effectiveness_bonus, 0.0);
        assert_eq!(weight.update_count, 0);
    }

    #[test]
    fn test_manager_creation() {
        let manager = AdaptiveWeightManager::new();

        // All node types should have neutral weights
        assert_eq!(manager.get_weight(BehaviorNodeType::Combat), 0.5);
        assert_eq!(manager.get_weight(BehaviorNodeType::Support), 0.5);
        assert_eq!(manager.get_weight(BehaviorNodeType::Exploration), 0.5);
        assert_eq!(manager.learning_rate, 0.1);
    }

    #[test]
    fn test_pattern_to_node_mapping() {
        let aggressive = BehaviorNodeType::from_pattern(PlaystylePattern::Aggressive);
        assert_eq!(aggressive, vec![BehaviorNodeType::Combat]);

        let cautious = BehaviorNodeType::from_pattern(PlaystylePattern::Cautious);
        assert!(cautious.contains(&BehaviorNodeType::Defensive));
        assert!(cautious.contains(&BehaviorNodeType::Support));
    }

    #[test]
    fn test_weight_adaptation() {
        let mut storage = MemoryStorage::in_memory().unwrap();
        let mut manager = AdaptiveWeightManager::new();

        // Store combat episodes with high quality
        for i in 0..10 {
            let episode =
                create_test_episode(&format!("combat_{}", i), EpisodeCategory::Combat, 0.9);
            storage.store_memory(&episode.to_memory().unwrap()).unwrap();
        }

        // Update weights from profile
        manager.update_from_profile(&storage).unwrap();

        // Combat weight should increase (pattern: Aggressive or Efficient)
        let combat_weight = manager.get_weight(BehaviorNodeType::Combat);
        assert!(
            combat_weight > 0.5,
            "Combat weight should increase: {}",
            combat_weight
        );

        // Total updates should reflect the operation
        assert!(manager.total_updates() > 0);
    }

    #[test]
    fn test_reset_weights() {
        let mut manager = AdaptiveWeightManager::new();
        manager.set_base_weight(BehaviorNodeType::Combat, 0.8);

        let mut storage = MemoryStorage::in_memory().unwrap();
        for i in 0..10 {
            let episode = create_test_episode(&format!("ep_{}", i), EpisodeCategory::Combat, 0.9);
            storage.store_memory(&episode.to_memory().unwrap()).unwrap();
        }

        manager.update_from_profile(&storage).unwrap();
        let _weight_after_update = manager.get_weight(BehaviorNodeType::Combat);

        manager.reset_weights();
        assert_eq!(manager.get_weight(BehaviorNodeType::Combat), 0.8); // Back to base
        assert!(manager.total_updates() == 0);
    }
}
