//! Player preference profiles derived from behavioral patterns.
//!
//! Builds comprehensive models of player preferences, optimal companion
//! responses, and learning confidence based on episode history analysis.

use crate::{
    EpisodeCategory, GameEpisode, MemoryStorage, MemoryType, PatternDetector, PatternStrength,
    PlaystylePattern,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Player preference profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceProfile {
    /// Dominant playstyle patterns
    pub dominant_patterns: Vec<PatternStrength>,
    /// Preferred episode categories
    pub preferred_categories: HashMap<EpisodeCategory, f32>,
    /// Optimal companion actions by context
    pub optimal_responses: HashMap<String, CompanionActionPreference>,
    /// Overall learning confidence (0.0 - 1.0)
    pub learning_confidence: f32,
    /// Number of episodes analyzed
    pub episode_count: usize,
    /// Profile convergence status
    pub converged: bool,
}

/// Companion action preference in specific context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompanionActionPreference {
    /// Action type (e.g., "support_spell", "defensive_buff")
    pub action_type: String,
    /// How often player responds positively to this action
    pub positive_response_rate: f32,
    /// Average effectiveness rating
    pub avg_effectiveness: f32,
    /// Number of occurrences
    pub sample_count: usize,
}

/// Profile builder and analyzer
pub struct ProfileBuilder {
    /// Pattern detector for analysis
    detector: PatternDetector,
    /// Convergence threshold (profile stability)
    convergence_threshold: f32,
    /// Minimum episodes for convergence
    convergence_min_episodes: usize,
}

impl ProfileBuilder {
    /// Create new profile builder
    pub fn new() -> Self {
        Self {
            detector: PatternDetector::new(),
            convergence_threshold: 0.55,
            convergence_min_episodes: 15,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(convergence_threshold: f32, convergence_min_episodes: usize) -> Self {
        Self {
            detector: PatternDetector::new(),
            convergence_threshold,
            convergence_min_episodes,
        }
    }

    /// Build preference profile from storage
    pub fn build_profile(&self, storage: &MemoryStorage) -> Result<PreferenceProfile> {
        let episode_ids = storage
            .query_by_type(MemoryType::Episodic)
            .context("Failed to query episodes")?;

        let episode_count = episode_ids.len();

        // Detect patterns
        let dominant_patterns = self
            .detector
            .detect_playstyle_patterns(storage)
            .context("Failed to detect patterns")?;

        // Analyze category preferences
        let preferred_categories = self.analyze_category_preferences(storage)?;

        // Analyze optimal companion responses
        let optimal_responses = self.analyze_optimal_responses(storage)?;

        // Calculate learning confidence
        let learning_confidence = self.calculate_learning_confidence(
            episode_count,
            &dominant_patterns,
            &preferred_categories,
        );

        // Check convergence
        let converged = episode_count >= self.convergence_min_episodes
            && learning_confidence >= self.convergence_threshold;

        Ok(PreferenceProfile {
            dominant_patterns,
            preferred_categories,
            optimal_responses,
            learning_confidence,
            episode_count,
            converged,
        })
    }

    /// Analyze category preferences based on quality and frequency
    fn analyze_category_preferences(
        &self,
        storage: &MemoryStorage,
    ) -> Result<HashMap<EpisodeCategory, f32>> {
        let episode_ids = storage.query_by_type(MemoryType::Episodic)?;

        let mut category_stats: HashMap<EpisodeCategory, (usize, f32)> = HashMap::new();

        for id in &episode_ids {
            if let Some(memory) = storage.get_memory(id)? {
                if let Ok(episode) =
                    serde_json::from_value::<GameEpisode>(memory.content.data.clone())
                {
                    if let Some(outcome) = &episode.outcome {
                        let quality = outcome.quality_score();
                        let entry = category_stats.entry(episode.category).or_insert((0, 0.0));
                        entry.0 += 1;
                        entry.1 += quality;
                    }
                }
            }
        }

        // Calculate preference score (frequency × quality)
        let total_episodes = episode_ids.len() as f32;
        let preferences = category_stats
            .into_iter()
            .map(|(category, (count, total_quality))| {
                let frequency = count as f32 / total_episodes;
                let avg_quality = total_quality / count as f32;
                let preference = (frequency * 0.6 + avg_quality * 0.4).clamp(0.0, 1.0);
                (category, preference)
            })
            .collect();

        Ok(preferences)
    }

    /// Analyze optimal companion responses
    fn analyze_optimal_responses(
        &self,
        storage: &MemoryStorage,
    ) -> Result<HashMap<String, CompanionActionPreference>> {
        let episode_ids = storage.query_by_type(MemoryType::Episodic)?;

        let mut action_stats: HashMap<String, (usize, f32, usize)> = HashMap::new();

        for id in &episode_ids {
            if let Some(memory) = storage.get_memory(id)? {
                if let Ok(episode) =
                    serde_json::from_value::<GameEpisode>(memory.content.data.clone())
                {
                    // Extract companion actions and player responses
                    for obs in &episode.observations {
                        if let Some(companion_response) = &obs.companion_response {
                            let action_type = &companion_response.action_type;
                            let effectiveness = companion_response.effectiveness;

                            // Determine if player responded positively (continued engagement)
                            let positive = effectiveness > 0.6;

                            let entry = action_stats
                                .entry(action_type.clone())
                                .or_insert((0, 0.0, 0));
                            entry.0 += 1; // Total occurrences
                            entry.1 += effectiveness; // Total effectiveness
                            if positive {
                                entry.2 += 1; // Positive responses
                            }
                        }
                    }
                }
            }
        }

        // Convert to preferences
        let preferences = action_stats
            .into_iter()
            .filter(|(_, (count, _, _))| *count >= 3) // Minimum 3 occurrences
            .map(
                |(action_type, (total, effectiveness_sum, positive_count))| {
                    let positive_response_rate = positive_count as f32 / total as f32;
                    let avg_effectiveness = effectiveness_sum / total as f32;

                    (
                        action_type.clone(),
                        CompanionActionPreference {
                            action_type,
                            positive_response_rate,
                            avg_effectiveness,
                            sample_count: total,
                        },
                    )
                },
            )
            .collect();

        Ok(preferences)
    }

    /// Calculate overall learning confidence
    fn calculate_learning_confidence(
        &self,
        episode_count: usize,
        patterns: &[PatternStrength],
        categories: &HashMap<EpisodeCategory, f32>,
    ) -> f32 {
        if episode_count == 0 {
            return 0.0;
        }

        // Episode count factor (smoother growth curve)
        // Reaches 0.5 at 15 episodes, 0.75 at 30 episodes, approaches 1.0
        let count_factor = (1.0 - (-0.1 * episode_count as f32).exp()).min(1.0);

        // Pattern strength factor
        let pattern_factor = if !patterns.is_empty() {
            patterns.iter().map(|p| p.confidence).sum::<f32>() / patterns.len() as f32
        } else {
            // Without detected patterns, use episode count as proxy
            (episode_count as f32 / 20.0).min(0.5)
        };

        // Category diversity factor (prefer diverse experience)
        let category_factor = if !categories.is_empty() {
            let diversity = categories.len() as f32 / 6.0; // 6 total categories
            diversity.min(1.0)
        } else {
            0.0
        };

        // Weighted average
        (count_factor * 0.4 + pattern_factor * 0.4 + category_factor * 0.2).clamp(0.0, 1.0)
    }

    /// Get recommended companion action for context
    pub fn recommend_action(&self, profile: &PreferenceProfile, context: &str) -> Option<String> {
        profile
            .optimal_responses
            .get(context)
            .map(|pref| pref.action_type.clone())
    }

    /// Predict player satisfaction for proposed action
    pub fn predict_satisfaction(&self, profile: &PreferenceProfile, action_type: &str) -> f32 {
        profile
            .optimal_responses
            .get(action_type)
            .map(|pref| {
                // Combine response rate and effectiveness
                (pref.positive_response_rate * 0.6 + pref.avg_effectiveness * 0.4).clamp(0.0, 1.0)
            })
            .unwrap_or(0.5) // Default neutral prediction
    }

    /// Get profile convergence status
    pub fn is_converged(&self, profile: &PreferenceProfile) -> bool {
        profile.converged
    }

    /// Get strongest playstyle pattern
    pub fn get_primary_pattern(profile: &PreferenceProfile) -> Option<PlaystylePattern> {
        profile
            .dominant_patterns
            .first()
            .map(|strength| strength.pattern)
    }

    /// Get most preferred category
    pub fn get_preferred_category(profile: &PreferenceProfile) -> Option<EpisodeCategory> {
        profile
            .preferred_categories
            .iter()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(cat, _)| *cat)
    }
}

impl Default for ProfileBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::{ActionResult, CompanionResponse, EpisodeOutcome, Observation};

    fn create_test_episode(
        id: &str,
        category: EpisodeCategory,
        quality: f32,
        companion_action: &str,
        effectiveness: f32,
    ) -> Episode {
        let mut episode = Episode::new(id.to_string(), category);
        episode.outcome = Some(EpisodeOutcome {
            success_rating: quality,
            player_satisfaction: quality,
            companion_effectiveness: effectiveness,
            duration_ms: 10000,
            damage_dealt: 300.0,
            damage_taken: 50.0,
            resources_used: 100.0,
            failure_count: 0,
        });

        // Add observation with companion response
        episode.add_observation(Observation::new(
            0,
            None,
            Some(CompanionResponse {
                action_type: companion_action.to_string(),
                result: ActionResult::Success,
                effectiveness,
            }),
            serde_json::json!({}),
        ));

        episode
    }

    #[test]
    fn test_builder_creation() {
        let builder = ProfileBuilder::new();
        assert_eq!(builder.convergence_threshold, 0.55);
        assert_eq!(builder.convergence_min_episodes, 15);

        let custom = ProfileBuilder::with_thresholds(0.7, 10);
        assert_eq!(custom.convergence_threshold, 0.7);
        assert_eq!(custom.convergence_min_episodes, 10);
    }

    #[test]
    fn test_learning_confidence_calculation() {
        let builder = ProfileBuilder::new();

        // Test with few episodes
        let confidence_low = builder.calculate_learning_confidence(3, &[], &HashMap::new());
        assert!(confidence_low < 0.3);

        // Test with good patterns
        let patterns = vec![PatternStrength {
            pattern: PlaystylePattern::Aggressive,
            confidence: 0.8,
            episode_count: 5,
            avg_quality: 0.75,
        }];

        let categories = [(EpisodeCategory::Combat, 0.9)].iter().cloned().collect();

        let confidence_high = builder.calculate_learning_confidence(15, &patterns, &categories);
        assert!(confidence_high > 0.5);
    }

    #[test]
    fn test_profile_with_storage() {
        let mut storage = MemoryStorage::in_memory().unwrap();

        // Store diverse episodes
        for i in 0..10 {
            let episode = create_test_episode(
                &format!("ep_{}", i),
                EpisodeCategory::Combat,
                0.8,
                "support_spell",
                0.85,
            );
            storage.store_memory(&episode.to_memory().unwrap()).unwrap();
        }

        let builder = ProfileBuilder::new();
        let profile = builder.build_profile(&storage).unwrap();

        assert_eq!(profile.episode_count, 10);
        assert!(profile.learning_confidence > 0.0);
        assert!(!profile.converged); // Only 10 episodes, need 15 for convergence
    }

    #[test]
    fn test_action_recommendation() {
        let builder = ProfileBuilder::new();

        let mut profile = PreferenceProfile {
            dominant_patterns: vec![],
            preferred_categories: HashMap::new(),
            optimal_responses: HashMap::new(),
            learning_confidence: 0.7,
            episode_count: 10,
            converged: false,
        };

        // Add action preference
        profile.optimal_responses.insert(
            "combat_support".to_string(),
            CompanionActionPreference {
                action_type: "healing_spell".to_string(),
                positive_response_rate: 0.85,
                avg_effectiveness: 0.9,
                sample_count: 8,
            },
        );

        let recommended = builder.recommend_action(&profile, "combat_support");
        assert_eq!(recommended, Some("healing_spell".to_string()));
    }

    #[test]
    fn test_satisfaction_prediction() {
        let builder = ProfileBuilder::new();

        let mut profile = PreferenceProfile {
            dominant_patterns: vec![],
            preferred_categories: HashMap::new(),
            optimal_responses: HashMap::new(),
            learning_confidence: 0.7,
            episode_count: 10,
            converged: false,
        };

        profile.optimal_responses.insert(
            "buff_spell".to_string(),
            CompanionActionPreference {
                action_type: "buff_spell".to_string(),
                positive_response_rate: 0.9,
                avg_effectiveness: 0.85,
                sample_count: 10,
            },
        );

        let satisfaction = builder.predict_satisfaction(&profile, "buff_spell");
        assert!(satisfaction > 0.7); // High predicted satisfaction

        let unknown_satisfaction = builder.predict_satisfaction(&profile, "unknown_action");
        assert!((unknown_satisfaction - 0.5).abs() < 0.01); // Default neutral
    }

    #[test]
    fn test_primary_pattern_extraction() {
        let profile = PreferenceProfile {
            dominant_patterns: vec![
                PatternStrength {
                    pattern: PlaystylePattern::Aggressive,
                    confidence: 0.85,
                    episode_count: 10,
                    avg_quality: 0.8,
                },
                PatternStrength {
                    pattern: PlaystylePattern::Efficient,
                    confidence: 0.65,
                    episode_count: 7,
                    avg_quality: 0.75,
                },
            ],
            preferred_categories: HashMap::new(),
            optimal_responses: HashMap::new(),
            learning_confidence: 0.75,
            episode_count: 15,
            converged: false,
        };

        let primary = ProfileBuilder::get_primary_pattern(&profile);
        assert_eq!(primary, Some(PlaystylePattern::Aggressive));
    }

    #[test]
    fn test_preferred_category_extraction() {
        let mut preferred_categories = HashMap::new();
        preferred_categories.insert(EpisodeCategory::Combat, 0.85);
        preferred_categories.insert(EpisodeCategory::Dialogue, 0.65);
        preferred_categories.insert(EpisodeCategory::Exploration, 0.72);

        let profile = PreferenceProfile {
            dominant_patterns: vec![],
            preferred_categories,
            optimal_responses: HashMap::new(),
            learning_confidence: 0.75,
            episode_count: 15,
            converged: false,
        };

        let preferred = ProfileBuilder::get_preferred_category(&profile);
        assert_eq!(preferred, Some(EpisodeCategory::Combat));
    }
}
