//! Behavioral pattern detection from episode history.
//!
//! Analyzes stored episodes to detect player playstyle patterns,
//! action preferences, and companion effectiveness in different contexts.

use crate::{GameEpisode, EpisodeCategory, MemoryStorage, MemoryType};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Detected playstyle pattern
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PlaystylePattern {
    /// Aggressive combat approach (high damage, risky actions)
    Aggressive,
    /// Cautious combat approach (defensive, conservative)
    Cautious,
    /// Exploration-focused (discovery, investigation)
    Explorative,
    /// Social interaction focused (dialogue, relationships)
    Social,
    /// Puzzle-solving focused (analytical, methodical)
    Analytical,
    /// Speed-focused (efficiency, quick completion)
    Efficient,
}

impl std::fmt::Display for PlaystylePattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PlaystylePattern::Aggressive => write!(f, "Aggressive"),
            PlaystylePattern::Cautious => write!(f, "Cautious"),
            PlaystylePattern::Explorative => write!(f, "Explorative"),
            PlaystylePattern::Social => write!(f, "Social"),
            PlaystylePattern::Analytical => write!(f, "Analytical"),
            PlaystylePattern::Efficient => write!(f, "Efficient"),
        }
    }
}

/// Pattern confidence and frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternStrength {
    pub pattern: PlaystylePattern,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Number of episodes supporting this pattern
    pub episode_count: usize,
    /// Average quality score of episodes with this pattern
    pub avg_quality: f32,
}

/// Action sequence pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionPattern {
    /// Sequence of action types
    pub sequence: Vec<String>,
    /// How often this sequence appears
    pub frequency: usize,
    /// Average effectiveness of this sequence
    pub avg_effectiveness: f32,
}

/// Pattern detection engine
pub struct PatternDetector {
    /// Minimum episodes required for pattern confidence
    min_episodes: usize,
    /// Confidence threshold for pattern reporting (0.0 - 1.0)
    confidence_threshold: f32,
}

impl PatternDetector {
    /// Create new pattern detector with default thresholds
    pub fn new() -> Self {
        Self {
            min_episodes: 5,
            confidence_threshold: 0.6,
        }
    }

    /// Create with custom thresholds
    pub fn with_thresholds(min_episodes: usize, confidence_threshold: f32) -> Self {
        Self {
            min_episodes,
            confidence_threshold,
        }
    }

    /// Detect playstyle patterns from episode history
    pub fn detect_playstyle_patterns(
        &self,
        storage: &MemoryStorage,
    ) -> Result<Vec<PatternStrength>> {
        // Get all episodic memories
        let episode_ids = storage
            .query_by_type(MemoryType::Episodic)
            .context("Failed to query episodes")?;

        if episode_ids.len() < self.min_episodes {
            return Ok(vec![]);
        }

        // Load episodes and analyze
        let mut episodes = Vec::new();
        for id in &episode_ids {
            if let Some(memory) = storage.get_memory(id)? {
                if let Ok(episode) = serde_json::from_value::<GameEpisode>(memory.content.data.clone())
                {
                    episodes.push(episode);
                }
            }
        }

        // Analyze patterns
        let mut pattern_scores: HashMap<PlaystylePattern, (usize, f32)> = HashMap::new();

        for episode in &episodes {
            if let Some(outcome) = &episode.outcome {
                let quality = outcome.quality_score();

                // Detect patterns based on episode characteristics
                let patterns = self.detect_episode_patterns(episode);

                for pattern in patterns {
                    let entry = pattern_scores.entry(pattern).or_insert((0, 0.0));
                    entry.0 += 1;
                    entry.1 += quality;
                }
            }
        }

        // Convert to PatternStrength
        let total_episodes = episodes.len();
        let mut strengths: Vec<PatternStrength> = pattern_scores
            .into_iter()
            .map(|(pattern, (count, total_quality))| {
                let confidence = (count as f32 / total_episodes as f32).min(1.0);
                let avg_quality = total_quality / count as f32;

                PatternStrength {
                    pattern,
                    confidence,
                    episode_count: count,
                    avg_quality,
                }
            })
            .filter(|s| s.confidence >= self.confidence_threshold)
            .collect();

        // Sort by confidence descending
        strengths.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        Ok(strengths)
    }

    /// Detect patterns in a single episode
    fn detect_episode_patterns(&self, episode: &GameEpisode) -> Vec<PlaystylePattern> {
        let mut patterns = Vec::new();

        // Category-based pattern detection
        match episode.category {
            EpisodeCategory::Combat => {
                if let Some(outcome) = &episode.outcome {
                    // Aggressive: high damage dealt, accepts damage taken
                    if outcome.damage_dealt > 300.0 && outcome.damage_taken > 50.0 {
                        patterns.push(PlaystylePattern::Aggressive);
                    }

                    // Cautious: low damage taken, conservative resource use
                    if outcome.damage_taken < 30.0 && outcome.resources_used < 100.0 {
                        patterns.push(PlaystylePattern::Cautious);
                    }

                    // Efficient: high success with low duration
                    if outcome.success_rating > 0.8 && outcome.duration_ms < 10000 {
                        patterns.push(PlaystylePattern::Efficient);
                    }
                }
            }
            EpisodeCategory::Dialogue | EpisodeCategory::Social => {
                patterns.push(PlaystylePattern::Social);

                // Analytical if many observations (careful dialogue choices)
                if episode.observations.len() > 5 {
                    patterns.push(PlaystylePattern::Analytical);
                }
            }
            EpisodeCategory::Exploration => {
                patterns.push(PlaystylePattern::Explorative);
            }
            EpisodeCategory::Puzzle => {
                patterns.push(PlaystylePattern::Analytical);

                // Efficient if solved quickly
                if let Some(outcome) = &episode.outcome {
                    if outcome.duration_ms < 30000 && outcome.success_rating > 0.8 {
                        patterns.push(PlaystylePattern::Efficient);
                    }
                }
            }
            EpisodeCategory::Quest => {
                // Quest patterns depend on outcome
                if let Some(outcome) = &episode.outcome {
                    if outcome.duration_ms < 60000 {
                        patterns.push(PlaystylePattern::Efficient);
                    }
                }
            }
        }

        patterns
    }

    /// Detect common action sequences from episodes
    pub fn detect_action_sequences(
        &self,
        storage: &MemoryStorage,
        min_frequency: usize,
    ) -> Result<Vec<ActionPattern>> {
        let episode_ids = storage.query_by_type(MemoryType::Episodic)?;

        let mut sequence_stats: HashMap<Vec<String>, (usize, f32)> = HashMap::new();

        for id in &episode_ids {
            if let Some(memory) = storage.get_memory(id)? {
                if let Ok(episode) = serde_json::from_value::<GameEpisode>(memory.content.data.clone())
                {
                    // Extract action sequences
                    let sequences = self.extract_sequences(&episode, 2, 4);

                    for (seq, effectiveness) in sequences {
                        let entry = sequence_stats.entry(seq).or_insert((0, 0.0));
                        entry.0 += 1;
                        entry.1 += effectiveness;
                    }
                }
            }
        }

        // Convert to ActionPattern
        let mut patterns: Vec<ActionPattern> = sequence_stats
            .into_iter()
            .filter(|(_, (freq, _))| *freq >= min_frequency)
            .map(|(sequence, (frequency, total_effectiveness))| ActionPattern {
                sequence,
                frequency,
                avg_effectiveness: total_effectiveness / frequency as f32,
            })
            .collect();

        // Sort by frequency descending
        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        Ok(patterns)
    }

    /// Extract action sequences from episode
    fn extract_sequences(
        &self,
        episode: &GameEpisode,
        min_length: usize,
        max_length: usize,
    ) -> Vec<(Vec<String>, f32)> {
        let mut sequences = Vec::new();

        // Extract player actions
        let actions: Vec<String> = episode
            .observations
            .iter()
            .filter_map(|obs| obs.player_action.as_ref().map(|a| a.action_type.clone()))
            .collect();

        if actions.len() < min_length {
            return sequences;
        }

        // Calculate episode effectiveness
        let effectiveness = episode
            .outcome
            .as_ref()
            .map(|o| o.companion_effectiveness)
            .unwrap_or(0.5);

        // Generate sequences of different lengths
        for length in min_length..=max_length.min(actions.len()) {
            for i in 0..=(actions.len() - length) {
                let seq = actions[i..i + length].to_vec();
                sequences.push((seq, effectiveness));
            }
        }

        sequences
    }

    /// Analyze companion effectiveness by context
    pub fn analyze_companion_effectiveness(
        &self,
        storage: &MemoryStorage,
    ) -> Result<HashMap<EpisodeCategory, f32>> {
        let episode_ids = storage.query_by_type(MemoryType::Episodic)?;

        let mut category_stats: HashMap<EpisodeCategory, (usize, f32)> = HashMap::new();

        for id in &episode_ids {
            if let Some(memory) = storage.get_memory(id)? {
                if let Ok(episode) = serde_json::from_value::<GameEpisode>(memory.content.data.clone())
                {
                    if let Some(outcome) = &episode.outcome {
                        let entry = category_stats
                            .entry(episode.category)
                            .or_insert((0, 0.0));
                        entry.0 += 1;
                        entry.1 += outcome.companion_effectiveness;
                    }
                }
            }
        }

        // Calculate averages
        let effectiveness_map = category_stats
            .into_iter()
            .map(|(category, (count, total))| (category, total / count as f32))
            .collect();

        Ok(effectiveness_map)
    }

    /// Get episode category distribution
    pub fn get_category_distribution(
        &self,
        storage: &MemoryStorage,
    ) -> Result<HashMap<EpisodeCategory, usize>> {
        let episode_ids = storage.query_by_type(MemoryType::Episodic)?;

        let mut distribution: HashMap<EpisodeCategory, usize> = HashMap::new();

        for id in &episode_ids {
            if let Some(memory) = storage.get_memory(id)? {
                if let Ok(episode) = serde_json::from_value::<GameEpisode>(memory.content.data.clone())
                {
                    *distribution.entry(episode.category).or_insert(0) += 1;
                }
            }
        }

        Ok(distribution)
    }

    /// Calculate pattern confidence based on consistency
    pub fn calculate_pattern_confidence(
        &self,
        pattern: PlaystylePattern,
        episodes: &[GameEpisode],
    ) -> f32 {
        if episodes.is_empty() {
            return 0.0;
        }

        let mut matches = 0;
        let mut total_quality = 0.0;

        for episode in episodes {
            let patterns = self.detect_episode_patterns(episode);

            if patterns.contains(&pattern) {
                matches += 1;

                if let Some(outcome) = &episode.outcome {
                    total_quality += outcome.quality_score();
                }
            }
        }

        if matches == 0 {
            return 0.0;
        }

        // Confidence based on match frequency and quality
        let frequency_score = matches as f32 / episodes.len() as f32;
        let quality_score = total_quality / matches as f32;

        // Weighted average (60% frequency, 40% quality)
        (frequency_score * 0.6 + quality_score * 0.4).clamp(0.0, 1.0)
    }
}

impl Default for PatternDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::Episode;
    use crate::{EpisodeOutcome, Observation, PlayerAction};

    fn create_combat_episode(
        id: &str,
        damage_dealt: f32,
        damage_taken: f32,
        duration_ms: u64,
    ) -> Episode {
        let mut episode = Episode::new(id.to_string(), EpisodeCategory::Combat);
        episode.outcome = Some(EpisodeOutcome {
            success_rating: 0.8,
            player_satisfaction: 0.75,
            companion_effectiveness: 0.7,
            duration_ms,
            damage_dealt,
            damage_taken,
            resources_used: 100.0,
            failure_count: 0,
        });
        episode
    }

    #[test]
    fn test_detector_creation() {
        let detector = PatternDetector::new();
        assert_eq!(detector.min_episodes, 5);
        assert_eq!(detector.confidence_threshold, 0.6);

        let custom = PatternDetector::with_thresholds(10, 0.8);
        assert_eq!(custom.min_episodes, 10);
        assert_eq!(custom.confidence_threshold, 0.8);
    }

    #[test]
    fn test_aggressive_pattern_detection() {
        let detector = PatternDetector::new();
        let episode = create_combat_episode("aggressive_001", 500.0, 75.0, 8000);

        let patterns = detector.detect_episode_patterns(&episode);
        assert!(patterns.contains(&PlaystylePattern::Aggressive));
    }

    #[test]
    fn test_cautious_pattern_detection() {
        let detector = PatternDetector::new();
        let episode = create_combat_episode("cautious_001", 200.0, 20.0, 15000);

        let patterns = detector.detect_episode_patterns(&episode);
        assert!(patterns.contains(&PlaystylePattern::Cautious));
    }

    #[test]
    fn test_efficient_pattern_detection() {
        let detector = PatternDetector::new();
        let mut episode = Episode::new("efficient_001".to_string(), EpisodeCategory::Puzzle);
        episode.outcome = Some(EpisodeOutcome {
            success_rating: 0.9,
            player_satisfaction: 0.85,
            companion_effectiveness: 0.8,
            duration_ms: 20000,
            damage_dealt: 0.0,
            damage_taken: 0.0,
            resources_used: 50.0,
            failure_count: 0,
        });

        let patterns = detector.detect_episode_patterns(&episode);
        assert!(patterns.contains(&PlaystylePattern::Analytical));
        assert!(patterns.contains(&PlaystylePattern::Efficient));
    }

    #[test]
    fn test_social_pattern_detection() {
        let detector = PatternDetector::new();
        let episode = Episode::new("social_001".to_string(), EpisodeCategory::Dialogue);

        let patterns = detector.detect_episode_patterns(&episode);
        assert!(patterns.contains(&PlaystylePattern::Social));
    }

    #[test]
    fn test_explorative_pattern_detection() {
        let detector = PatternDetector::new();
        let episode = Episode::new("explore_001".to_string(), EpisodeCategory::Exploration);

        let patterns = detector.detect_episode_patterns(&episode);
        assert!(patterns.contains(&PlaystylePattern::Explorative));
    }

    #[test]
    fn test_pattern_confidence_calculation() {
        let detector = PatternDetector::new();

        let episodes = vec![
            create_combat_episode("ep1", 500.0, 75.0, 8000),
            create_combat_episode("ep2", 450.0, 60.0, 9000),
            create_combat_episode("ep3", 200.0, 20.0, 15000), // Cautious, not aggressive
        ];

        let confidence = detector.calculate_pattern_confidence(PlaystylePattern::Aggressive, &episodes);

        // 2 out of 3 episodes are aggressive
        assert!(confidence > 0.4); // At least 40% confidence
        assert!(confidence < 0.8); // Less than 80% due to mixed patterns
    }

    #[test]
    fn test_action_sequence_extraction() {
        let detector = PatternDetector::new();

        let mut episode = Episode::new("seq_001".to_string(), EpisodeCategory::Combat);
        episode.outcome = Some(EpisodeOutcome {
            success_rating: 0.8,
            player_satisfaction: 0.75,
            companion_effectiveness: 0.85,
            duration_ms: 10000,
            damage_dealt: 400.0,
            damage_taken: 50.0,
            resources_used: 100.0,
            failure_count: 0,
        });

        // Add observations with actions
        episode.add_observation(Observation::new(
            0,
            Some(PlayerAction {
                action_type: "melee_attack".to_string(),
                target: Some("enemy".to_string()),
                parameters: serde_json::json!({}),
            }),
            None,
            serde_json::json!({}),
        ));

        episode.add_observation(Observation::new(
            1000,
            Some(PlayerAction {
                action_type: "dodge".to_string(),
                target: None,
                parameters: serde_json::json!({}),
            }),
            None,
            serde_json::json!({}),
        ));

        episode.add_observation(Observation::new(
            2000,
            Some(PlayerAction {
                action_type: "melee_attack".to_string(),
                target: Some("enemy".to_string()),
                parameters: serde_json::json!({}),
            }),
            None,
            serde_json::json!({}),
        ));

        let sequences = detector.extract_sequences(&episode, 2, 3);

        // Should have sequences like ["melee_attack", "dodge"], ["dodge", "melee_attack"]
        assert!(!sequences.is_empty());
        assert!(sequences
            .iter()
            .any(|(seq, _)| seq.len() >= 2 && seq.contains(&"melee_attack".to_string())));
    }
}
