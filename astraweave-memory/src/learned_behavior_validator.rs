//! Learned behavior validation through simulation and safety checks.
//!
//! This module provides a sandbox environment for testing proposed companion
//! behaviors before execution. Validates that learned behaviors align with
//! player preferences and don't introduce unsafe or inappropriate actions.

use crate::{MemoryStorage, PreferenceProfile, ProfileBuilder};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation result for a proposed action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the action is valid
    pub valid: bool,
    /// Confidence in the validation (0.0 to 1.0)
    pub confidence: f32,
    /// Predicted player satisfaction (0.0 to 1.0)
    pub predicted_satisfaction: f32,
    /// Reasons for validation outcome
    pub reasons: Vec<String>,
    /// Suggested alternatives if invalid
    pub alternatives: Vec<String>,
}

impl ValidationResult {
    /// Create a valid result
    pub fn valid(confidence: f32, satisfaction: f32, reason: &str) -> Self {
        Self {
            valid: true,
            confidence: confidence.clamp(0.0, 1.0),
            predicted_satisfaction: satisfaction.clamp(0.0, 1.0),
            reasons: vec![reason.to_string()],
            alternatives: Vec::new(),
        }
    }

    /// Create an invalid result with alternatives
    pub fn invalid(reason: &str, alternatives: Vec<String>) -> Self {
        Self {
            valid: false,
            confidence: 0.9, // High confidence in rejection
            predicted_satisfaction: 0.0,
            reasons: vec![reason.to_string()],
            alternatives,
        }
    }

    /// Create uncertain result
    pub fn uncertain(confidence: f32, reason: &str) -> Self {
        Self {
            valid: false,
            confidence: confidence.clamp(0.0, 1.0),
            predicted_satisfaction: 0.5,
            reasons: vec![reason.to_string()],
            alternatives: Vec::new(),
        }
    }
}

/// Safety rule for behavior validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyRule {
    /// Rule identifier
    pub id: String,
    /// Rule description
    pub description: String,
    /// Minimum satisfaction threshold (0.0 to 1.0)
    pub min_satisfaction: f32,
    /// Whether this rule is strict (blocks on violation)
    pub strict: bool,
}

impl SafetyRule {
    /// Create new safety rule
    pub fn new(id: &str, description: &str, min_satisfaction: f32, strict: bool) -> Self {
        Self {
            id: id.to_string(),
            description: description.to_string(),
            min_satisfaction: min_satisfaction.clamp(0.0, 1.0),
            strict,
        }
    }
}

/// Behavior validation engine
pub struct BehaviorValidator {
    /// Profile builder for player modeling
    builder: ProfileBuilder,
    /// Safety rules
    safety_rules: Vec<SafetyRule>,
    /// Minimum confidence threshold for validation
    min_confidence: f32,
    /// Minimum predicted satisfaction threshold
    min_satisfaction: f32,
    /// Cache of validated actions
    validation_cache: HashMap<String, ValidationResult>,
}

impl BehaviorValidator {
    /// Create new behavior validator with default rules
    pub fn new() -> Self {
        let safety_rules = vec![
            SafetyRule::new(
                "min_satisfaction",
                "Action must have predicted player satisfaction above threshold",
                0.4,
                true,
            ),
            SafetyRule::new(
                "profile_alignment",
                "Action should align with detected player patterns",
                0.5,
                false,
            ),
            SafetyRule::new(
                "historical_effectiveness",
                "Similar actions should have positive historical effectiveness",
                0.6,
                false,
            ),
        ];

        Self {
            builder: ProfileBuilder::new(),
            safety_rules,
            min_confidence: 0.6,
            min_satisfaction: 0.4,
            validation_cache: HashMap::new(),
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(min_confidence: f32, min_satisfaction: f32) -> Self {
        let mut validator = Self::new();
        validator.min_confidence = min_confidence.clamp(0.0, 1.0);
        validator.min_satisfaction = min_satisfaction.clamp(0.0, 1.0);
        validator
    }

    /// Add custom safety rule
    pub fn add_safety_rule(&mut self, rule: SafetyRule) {
        self.safety_rules.push(rule);
    }

    /// Validate a proposed action
    pub fn validate_action(
        &mut self,
        action_type: &str,
        context: &str,
        storage: &MemoryStorage,
    ) -> Result<ValidationResult> {
        // Check cache first
        let cache_key = format!("{}:{}", action_type, context);
        if let Some(cached) = self.validation_cache.get(&cache_key) {
            return Ok(cached.clone());
        }

        // Build player profile
        let profile = self
            .builder
            .build_profile(storage)
            .context("Failed to build player profile")?;

        // Validate against rules
        let result = self.validate_with_profile(action_type, context, &profile);

        // Cache result
        self.validation_cache.insert(cache_key, result.clone());

        Ok(result)
    }

    /// Validate action with existing profile
    fn validate_with_profile(
        &self,
        action_type: &str,
        _context: &str,
        profile: &PreferenceProfile,
    ) -> ValidationResult {
        let mut reasons = Vec::new();
        let mut rule_violations = Vec::new();

        // Check if we have enough data
        if profile.episode_count < 5 {
            return ValidationResult::uncertain(
                0.3,
                "Insufficient episode data for confident validation",
            );
        }

        // Predict satisfaction
        let predicted_satisfaction = self.builder.predict_satisfaction(profile, action_type);

        // Check satisfaction threshold
        if predicted_satisfaction < self.min_satisfaction {
            rule_violations.push("min_satisfaction");
            reasons.push(format!(
                "Predicted satisfaction {:.2} below threshold {:.2}",
                predicted_satisfaction, self.min_satisfaction
            ));
        }

        // Check profile alignment
        let has_optimal_response = profile.optimal_responses.contains_key(action_type);
        if !has_optimal_response {
            rule_violations.push("profile_alignment");
            reasons.push(format!(
                "Action '{}' not found in optimal responses",
                action_type
            ));
        }

        // Check historical effectiveness
        if let Some(pref) = profile.optimal_responses.get(action_type) {
            if pref.avg_effectiveness < 0.6 {
                rule_violations.push("historical_effectiveness");
                reasons.push(format!(
                    "Historical effectiveness {:.2} below threshold",
                    pref.avg_effectiveness
                ));
            }
        }

        // Determine if strict rules were violated
        let strict_violation = rule_violations.iter().any(|rule_id| {
            self.safety_rules
                .iter()
                .any(|r| r.id == *rule_id && r.strict)
        });

        if strict_violation {
            // Find alternatives
            let alternatives = self.suggest_alternatives(profile);
            return ValidationResult::invalid(&reasons.join("; "), alternatives);
        }

        // Calculate confidence based on data quality
        let confidence = self.calculate_confidence(profile, &rule_violations);

        if reasons.is_empty() {
            ValidationResult::valid(
                confidence,
                predicted_satisfaction,
                "Action passes all validation checks",
            )
        } else {
            // Non-strict violations - valid but with warnings
            let mut result = ValidationResult::valid(
                confidence,
                predicted_satisfaction,
                "Action passes with minor concerns",
            );
            result.reasons.extend(reasons);
            result
        }
    }

    /// Calculate validation confidence
    fn calculate_confidence(&self, profile: &PreferenceProfile, violations: &[&str]) -> f32 {
        // Base confidence from learning confidence
        let mut confidence = profile.learning_confidence;

        // Reduce for each violation
        let violation_penalty = violations.len() as f32 * 0.1;
        confidence -= violation_penalty;

        // Bonus for convergence
        if profile.converged {
            confidence += 0.1;
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Suggest alternative actions
    fn suggest_alternatives(&self, profile: &PreferenceProfile) -> Vec<String> {
        profile
            .optimal_responses
            .iter()
            .filter(|(_, pref)| pref.positive_response_rate > 0.6 && pref.avg_effectiveness > 0.6)
            .take(3)
            .map(|(action, _)| action.clone())
            .collect()
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
    }

    /// Get validation statistics
    pub fn get_stats(&self) -> ValidationStats {
        let total_cached = self.validation_cache.len();
        let valid_count = self.validation_cache.values().filter(|v| v.valid).count();

        ValidationStats {
            total_validations: total_cached,
            valid_count,
            invalid_count: total_cached - valid_count,
            cache_size: total_cached,
        }
    }

    /// Simulate batch validation
    pub fn validate_batch(
        &mut self,
        actions: &[(String, String)], // (action_type, context)
        storage: &MemoryStorage,
    ) -> Result<Vec<ValidationResult>> {
        actions
            .iter()
            .map(|(action, context)| self.validate_action(action, context, storage))
            .collect()
    }
}

impl Default for BehaviorValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStats {
    /// Total validations performed
    pub total_validations: usize,
    /// Number of valid actions
    pub valid_count: usize,
    /// Number of invalid actions
    pub invalid_count: usize,
    /// Current cache size
    pub cache_size: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::{ActionResult, CompanionResponse, EpisodeCategory, EpisodeOutcome, Observation};

    fn create_test_episode(
        id: &str,
        category: EpisodeCategory,
        companion_action: &str,
        effectiveness: f32,
        satisfaction: f32,
    ) -> Episode {
        let mut episode = Episode::new(id.to_string(), category);
        episode.outcome = Some(EpisodeOutcome {
            success_rating: satisfaction,
            player_satisfaction: satisfaction,
            companion_effectiveness: effectiveness,
            duration_ms: 10000,
            damage_dealt: 300.0,
            damage_taken: 50.0,
            resources_used: 100.0,
            failure_count: 0,
        });

        episode.add_observation(Observation::new(
            0,
            None,
            Some(CompanionResponse {
                action_type: companion_action.to_string(),
                result: ActionResult::Success,
                effectiveness,
            }),
            serde_json::json!({"player_health": 100}),
        ));

        episode
    }

    #[test]
    fn test_validation_result_creation() {
        let valid = ValidationResult::valid(0.9, 0.85, "Test passed");
        assert!(valid.valid);
        assert_eq!(valid.confidence, 0.9);
        assert_eq!(valid.predicted_satisfaction, 0.85);

        let invalid = ValidationResult::invalid("Test failed", vec!["alt1".to_string()]);
        assert!(!invalid.valid);
        assert_eq!(invalid.alternatives.len(), 1);
    }

    #[test]
    fn test_safety_rule_creation() {
        let rule = SafetyRule::new("test_rule", "Test description", 0.5, true);
        assert_eq!(rule.id, "test_rule");
        assert_eq!(rule.min_satisfaction, 0.5);
        assert!(rule.strict);
    }

    #[test]
    fn test_validator_creation() {
        let validator = BehaviorValidator::new();
        assert_eq!(validator.safety_rules.len(), 3);
        assert_eq!(validator.min_confidence, 0.6);
        assert_eq!(validator.min_satisfaction, 0.4);
    }

    #[test]
    fn test_validator_with_thresholds() {
        let validator = BehaviorValidator::with_thresholds(0.8, 0.6);
        assert_eq!(validator.min_confidence, 0.8);
        assert_eq!(validator.min_satisfaction, 0.6);
    }

    #[test]
    fn test_add_safety_rule() {
        let mut validator = BehaviorValidator::new();
        let initial_count = validator.safety_rules.len();

        validator.add_safety_rule(SafetyRule::new(
            "custom_rule",
            "Custom test rule",
            0.7,
            false,
        ));

        assert_eq!(validator.safety_rules.len(), initial_count + 1);
    }

    #[test]
    fn test_validation_with_sufficient_data() {
        let mut storage = MemoryStorage::in_memory().unwrap();
        let mut validator = BehaviorValidator::new();

        // Store episodes with effective healing actions
        for i in 0..10 {
            let episode = create_test_episode(
                &format!("ep_{}", i),
                EpisodeCategory::Combat,
                "healing_spell",
                0.85,
                0.8,
            );
            storage.store_memory(&episode.to_memory().unwrap()).unwrap();
        }

        let result = validator
            .validate_action("healing_spell", "combat", &storage)
            .unwrap();
        assert!(
            result.valid,
            "Validation should pass with good historical data"
        );
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_validation_with_insufficient_data() {
        let storage = MemoryStorage::in_memory().unwrap();
        let mut validator = BehaviorValidator::new();

        let result = validator
            .validate_action("unknown_action", "test", &storage)
            .unwrap();
        assert!(!result.valid, "Should be uncertain with no data");
        assert!(result.confidence < 0.5);
        assert!(result.reasons[0].contains("Insufficient"));
    }

    #[test]
    fn test_validation_caching() {
        let mut storage = MemoryStorage::in_memory().unwrap();
        let mut validator = BehaviorValidator::new();

        for i in 0..10 {
            let episode = create_test_episode(
                &format!("ep_{}", i),
                EpisodeCategory::Combat,
                "attack",
                0.8,
                0.75,
            );
            storage.store_memory(&episode.to_memory().unwrap()).unwrap();
        }

        // First validation
        let result1 = validator
            .validate_action("attack", "combat", &storage)
            .unwrap();

        // Second validation (should hit cache)
        let result2 = validator
            .validate_action("attack", "combat", &storage)
            .unwrap();

        assert_eq!(result1.valid, result2.valid);
        assert_eq!(result1.confidence, result2.confidence);

        let stats = validator.get_stats();
        assert_eq!(stats.cache_size, 1);
    }

    #[test]
    fn test_clear_cache() {
        let mut storage = MemoryStorage::in_memory().unwrap();
        let mut validator = BehaviorValidator::new();

        for i in 0..10 {
            let episode = create_test_episode(
                &format!("ep_{}", i),
                EpisodeCategory::Combat,
                "buff",
                0.7,
                0.7,
            );
            storage.store_memory(&episode.to_memory().unwrap()).unwrap();
        }

        validator.validate_action("buff", "test", &storage).unwrap();
        assert_eq!(validator.get_stats().cache_size, 1);

        validator.clear_cache();
        assert_eq!(validator.get_stats().cache_size, 0);
    }

    #[test]
    fn test_batch_validation() {
        let mut storage = MemoryStorage::in_memory().unwrap();
        let mut validator = BehaviorValidator::new();

        for i in 0..15 {
            let episode = create_test_episode(
                &format!("ep_{}", i),
                EpisodeCategory::Combat,
                if i % 2 == 0 { "heal" } else { "attack" },
                0.8,
                0.75,
            );
            storage.store_memory(&episode.to_memory().unwrap()).unwrap();
        }

        let actions = vec![
            ("heal".to_string(), "combat".to_string()),
            ("attack".to_string(), "combat".to_string()),
        ];

        let results = validator.validate_batch(&actions, &storage).unwrap();
        assert_eq!(results.len(), 2);
    }
}
