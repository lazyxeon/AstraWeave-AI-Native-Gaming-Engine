//! Active episode recording with ECS integration.
//!
//! The `EpisodeRecorder` manages ongoing episodes for each companion,
//! providing auto-flush functionality and integration with the ECS
//! system for seamless recording during gameplay.

use super::episode::{Episode, EpisodeCategory, EpisodeOutcome, Observation};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Manages active episode recording for companions
///
/// This structure is designed to be used as an ECS resource,
/// running in `SystemStage::POST_SIMULATION` to record observations
/// and flush completed episodes to persistent storage.
pub struct EpisodeRecorder {
    /// Active episodes by companion ID
    active_episodes: HashMap<String, Episode>,
    /// Next auto-flush time
    next_flush: SystemTime,
    /// Auto-flush interval in seconds
    flush_interval_secs: u64,
}

impl Default for EpisodeRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl EpisodeRecorder {
    /// Create new episode recorder with default settings
    pub fn new() -> Self {
        Self {
            active_episodes: HashMap::new(),
            next_flush: SystemTime::now() + Duration::from_secs(60),
            flush_interval_secs: 60,
        }
    }

    /// Create episode recorder with custom flush interval
    pub fn with_flush_interval(flush_interval_secs: u64) -> Self {
        Self {
            active_episodes: HashMap::new(),
            next_flush: SystemTime::now() + Duration::from_secs(flush_interval_secs),
            flush_interval_secs,
        }
    }

    /// Start new episode for companion
    ///
    /// Returns the episode ID for reference. If companion already has
    /// an active episode, it will be replaced (previous episode is lost).
    pub fn start_episode(&mut self, companion_id: String, category: EpisodeCategory) -> String {
        let episode_id = Uuid::new_v4().to_string();
        let episode = Episode::new(episode_id.clone(), category);

        self.active_episodes.insert(companion_id, episode);
        episode_id
    }

    /// Record observation for companion's active episode
    ///
    /// If no active episode exists for companion, observation is ignored.
    pub fn record_observation(&mut self, companion_id: &str, observation: Observation) {
        if let Some(episode) = self.active_episodes.get_mut(companion_id) {
            episode.add_observation(observation);
        }
    }

    /// End episode for companion with outcome
    ///
    /// Returns the completed episode for storage, or None if no
    /// active episode exists for this companion.
    pub fn end_episode(&mut self, companion_id: &str, outcome: EpisodeOutcome) -> Option<Episode> {
        if let Some(mut episode) = self.active_episodes.remove(companion_id) {
            episode.complete(outcome);
            Some(episode)
        } else {
            None
        }
    }

    /// Add tag to companion's active episode
    ///
    /// Useful for marking episodes with contextual information
    /// (e.g., "boss_fight", "underwater", "tutorial").
    pub fn tag_active_episode(&mut self, companion_id: &str, tag: String) {
        if let Some(episode) = self.active_episodes.get_mut(companion_id) {
            episode.add_tag(tag);
        }
    }

    /// Get reference to companion's active episode
    pub fn get_active_episode(&self, companion_id: &str) -> Option<&Episode> {
        self.active_episodes.get(companion_id)
    }

    /// Get mutable reference to companion's active episode
    pub fn get_active_episode_mut(&mut self, companion_id: &str) -> Option<&mut Episode> {
        self.active_episodes.get_mut(companion_id)
    }

    /// Check if companion has active episode
    pub fn has_active_episode(&self, companion_id: &str) -> bool {
        self.active_episodes.contains_key(companion_id)
    }

    /// Get count of active episodes
    pub fn active_count(&self) -> usize {
        self.active_episodes.len()
    }

    /// Check if auto-flush should occur
    pub fn should_flush(&self) -> bool {
        SystemTime::now() >= self.next_flush
    }

    /// Update flush timer (call after flushing)
    pub fn update_flush_timer(&mut self) {
        self.next_flush = SystemTime::now() + Duration::from_secs(self.flush_interval_secs);
    }

    /// Abort active episode for companion without outcome
    ///
    /// Used when episode should be discarded (e.g., player quit mid-encounter).
    /// Returns the incomplete episode if it existed.
    pub fn abort_episode(&mut self, companion_id: &str) -> Option<Episode> {
        self.active_episodes.remove(companion_id)
    }

    /// Abort all active episodes
    ///
    /// Returns all incomplete episodes. Useful for cleanup on shutdown.
    pub fn abort_all_episodes(&mut self) -> Vec<Episode> {
        let episodes: Vec<Episode> = self.active_episodes.drain().map(|(_, ep)| ep).collect();
        episodes
    }

    /// Get all active companion IDs
    pub fn active_companion_ids(&self) -> Vec<String> {
        self.active_episodes.keys().cloned().collect()
    }

    /// Force-complete all active episodes with default outcomes
    ///
    /// Used for graceful shutdown or save-and-quit scenarios.
    /// Returns all completed episodes.
    pub fn complete_all_episodes(&mut self) -> Vec<Episode> {
        let mut completed = Vec::new();

        for (_, mut episode) in self.active_episodes.drain() {
            // Create default outcome from episode data
            let duration_ms = episode
                .duration()
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0);

            let outcome = EpisodeOutcome {
                success_rating: 0.5, // Neutral outcome
                player_satisfaction: 0.5,
                companion_effectiveness: 0.5,
                duration_ms,
                damage_dealt: 0.0,
                damage_taken: 0.0,
                resources_used: 0.0,
                failure_count: 0,
            };

            episode.complete(outcome);
            completed.push(episode);
        }

        completed
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::episode::{ActionResult, CompanionResponse, PlayerAction};

    #[test]
    fn test_recorder_creation() {
        let recorder = EpisodeRecorder::new();
        assert_eq!(recorder.active_count(), 0);
        assert!(recorder.flush_interval_secs > 0);
    }

    #[test]
    fn test_start_episode() {
        let mut recorder = EpisodeRecorder::new();
        let episode_id = recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);

        assert!(!episode_id.is_empty());
        assert_eq!(recorder.active_count(), 1);
        assert!(recorder.has_active_episode("companion_1"));
    }

    #[test]
    fn test_record_observation() {
        let mut recorder = EpisodeRecorder::new();
        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);

        let obs = Observation::new(
            1000,
            Some(PlayerAction {
                action_type: "attack".to_string(),
                target: Some("enemy".to_string()),
                parameters: serde_json::json!({}),
            }),
            Some(CompanionResponse {
                action_type: "defend".to_string(),
                result: ActionResult::Success,
                effectiveness: 0.9,
            }),
            serde_json::json!({}),
        );

        recorder.record_observation("companion_1", obs);

        let episode = recorder
            .get_active_episode("companion_1")
            .expect("Should have episode");
        assert_eq!(episode.observations.len(), 1);
    }

    #[test]
    fn test_end_episode() {
        let mut recorder = EpisodeRecorder::new();
        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);

        let outcome = EpisodeOutcome {
            success_rating: 0.8,
            player_satisfaction: 0.9,
            companion_effectiveness: 0.85,
            duration_ms: 30_000,
            damage_dealt: 200.0,
            damage_taken: 50.0,
            resources_used: 75.0,
            failure_count: 0,
        };

        let episode = recorder
            .end_episode("companion_1", outcome)
            .expect("Should return episode");

        assert!(episode.is_complete());
        assert_eq!(recorder.active_count(), 0);
        assert!(!recorder.has_active_episode("companion_1"));
    }

    #[test]
    fn test_tag_active_episode() {
        let mut recorder = EpisodeRecorder::new();
        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);

        recorder.tag_active_episode("companion_1", "boss_fight".to_string());
        recorder.tag_active_episode("companion_1", "underwater".to_string());

        let episode = recorder
            .get_active_episode("companion_1")
            .expect("Should have episode");
        assert_eq!(episode.tags.len(), 2);
        assert!(episode.tags.contains(&"boss_fight".to_string()));
    }

    #[test]
    fn test_abort_episode() {
        let mut recorder = EpisodeRecorder::new();
        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);

        let aborted = recorder
            .abort_episode("companion_1")
            .expect("Should return episode");

        assert!(!aborted.is_complete());
        assert_eq!(recorder.active_count(), 0);
    }

    #[test]
    fn test_multiple_companions() {
        let mut recorder = EpisodeRecorder::new();

        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);
        recorder.start_episode("companion_2".to_string(), EpisodeCategory::Dialogue);
        recorder.start_episode("companion_3".to_string(), EpisodeCategory::Exploration);

        assert_eq!(recorder.active_count(), 3);

        let ids = recorder.active_companion_ids();
        assert_eq!(ids.len(), 3);
        assert!(ids.contains(&"companion_1".to_string()));
        assert!(ids.contains(&"companion_2".to_string()));
        assert!(ids.contains(&"companion_3".to_string()));
    }

    #[test]
    fn test_flush_timer() {
        let mut recorder = EpisodeRecorder::with_flush_interval(1); // 1 second

        assert!(!recorder.should_flush());

        // Simulate passage of time (would need tokio sleep in real code)
        recorder.next_flush = SystemTime::now() - Duration::from_secs(1);

        assert!(recorder.should_flush());

        recorder.update_flush_timer();
        assert!(!recorder.should_flush());
    }

    #[test]
    fn test_complete_all_episodes() {
        let mut recorder = EpisodeRecorder::new();

        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);
        recorder.start_episode("companion_2".to_string(), EpisodeCategory::Dialogue);

        // Record some observations
        for _ in 0..3 {
            recorder.record_observation(
                "companion_1",
                Observation::new(1000, None, None, serde_json::json!({})),
            );
        }

        let completed = recorder.complete_all_episodes();

        assert_eq!(completed.len(), 2);
        assert!(completed.iter().all(|ep| ep.is_complete()));
        assert_eq!(recorder.active_count(), 0);
    }

    #[test]
    fn test_episode_replacement() {
        let mut recorder = EpisodeRecorder::new();

        let id1 = recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);
        let id2 = recorder.start_episode("companion_1".to_string(), EpisodeCategory::Dialogue);

        assert_ne!(id1, id2);
        assert_eq!(recorder.active_count(), 1);

        let episode = recorder
            .get_active_episode("companion_1")
            .expect("Should have episode");
        assert_eq!(episode.category, EpisodeCategory::Dialogue);
    }

    #[test]
    fn test_get_mutable_episode() {
        let mut recorder = EpisodeRecorder::new();
        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);

        if let Some(episode) = recorder.get_active_episode_mut("companion_1") {
            episode.add_tag("test".to_string());
        }

        let episode = recorder
            .get_active_episode("companion_1")
            .expect("Should have episode");
        assert!(episode.tags.contains(&"test".to_string()));
    }

    #[test]
    fn test_abort_all_episodes() {
        let mut recorder = EpisodeRecorder::new();

        recorder.start_episode("companion_1".to_string(), EpisodeCategory::Combat);
        recorder.start_episode("companion_2".to_string(), EpisodeCategory::Dialogue);
        recorder.start_episode("companion_3".to_string(), EpisodeCategory::Exploration);

        let aborted = recorder.abort_all_episodes();

        assert_eq!(aborted.len(), 3);
        assert!(aborted.iter().all(|ep| !ep.is_complete()));
        assert_eq!(recorder.active_count(), 0);
    }
}
