use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Statistics for a specific action's execution history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionStats {
    pub executions: u32,
    pub successes: u32,
    pub failures: u32,
    pub avg_duration: f32,
}

impl ActionStats {
    pub fn new() -> Self {
        Self {
            executions: 0,
            successes: 0,
            failures: 0,
            avg_duration: 0.0,
        }
    }

    /// Success rate (0.0 to 1.0)
    pub fn success_rate(&self) -> f32 {
        if self.executions == 0 {
            0.5 // Unknown performance, assume neutral
        } else {
            self.successes as f32 / self.executions as f32
        }
    }

    /// Failure rate (0.0 to 1.0)
    pub fn failure_rate(&self) -> f32 {
        1.0 - self.success_rate()
    }

    /// Reliability score (higher is more reliable)
    /// Factors in both success rate and sample size
    pub fn reliability_score(&self) -> f32 {
        let confidence_factor = (self.executions as f32 / 20.0).min(1.0);
        self.success_rate() * confidence_factor
    }
}

impl Default for ActionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Action execution history for learning and adaptation
/// Tracks success/failure rates and execution times
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionHistory {
    stats: HashMap<String, ActionStats>,
}

impl ActionHistory {
    pub fn new() -> Self {
        Self {
            stats: HashMap::new(),
        }
    }

    /// Record a successful action execution
    pub fn record_success(&mut self, action_name: &str, duration: f32) {
        let stats = self
            .stats
            .entry(action_name.to_string())
            .or_insert_with(ActionStats::new);

        stats.executions += 1;
        stats.successes += 1;

        // Update rolling average duration
        if stats.executions == 1 {
            stats.avg_duration = duration;
        } else {
            let prev_total = stats.avg_duration * (stats.executions - 1) as f32;
            stats.avg_duration = (prev_total + duration) / stats.executions as f32;
        }
    }

    /// Record a failed action execution
    pub fn record_failure(&mut self, action_name: &str) {
        let stats = self
            .stats
            .entry(action_name.to_string())
            .or_insert_with(ActionStats::new);

        stats.executions += 1;
        stats.failures += 1;
    }

    /// Get statistics for a specific action
    pub fn get_action_stats(&self, action_name: &str) -> Option<&ActionStats> {
        self.stats.get(action_name)
    }

    /// Get mutable statistics for a specific action
    pub fn get_action_stats_mut(&mut self, action_name: &str) -> Option<&mut ActionStats> {
        self.stats.get_mut(action_name)
    }

    /// Get all action names with recorded history
    pub fn action_names(&self) -> Vec<String> {
        self.stats.keys().cloned().collect()
    }

    /// Clear all history (for testing or resets)
    pub fn clear(&mut self) {
        self.stats.clear();
    }

    /// Total number of actions tracked
    pub fn len(&self) -> usize {
        self.stats.len()
    }

    pub fn is_empty(&self) -> bool {
        self.stats.is_empty()
    }

    /// Reset stats for an action (useful for testing or manual intervention)
    pub fn reset_action(&mut self, action_name: &str) {
        self.stats.remove(action_name);
    }

    /// Get total number of executions across all actions
    pub fn total_executions(&self) -> u32 {
        self.stats.values().map(|s| s.executions).sum()
    }

    /// Prune actions with very low execution counts (noise reduction)
    pub fn prune_noise(&mut self, min_executions: u32) {
        self.stats
            .retain(|_, stats| stats.executions >= min_executions);
    }

    /// Merge another history into this one
    /// Useful for sharing learning across entities
    pub fn merge(&mut self, other: &ActionHistory) {
        for (action_name, other_stats) in &other.stats {
            let stats = self
                .stats
                .entry(action_name.clone())
                .or_insert_with(ActionStats::new);

            // Merge statistics
            let total_exec = stats.executions + other_stats.executions;
            if total_exec > 0 {
                stats.avg_duration = (stats.avg_duration * stats.executions as f32
                    + other_stats.avg_duration * other_stats.executions as f32)
                    / total_exec as f32;
            }

            stats.executions += other_stats.executions;
            stats.successes += other_stats.successes;
            stats.failures += other_stats.failures;
        }
    }

    /// Prune old or unreliable data
    /// Keeps only the N most executed actions
    pub fn prune(&mut self, keep_top_n: usize) {
        if self.stats.len() <= keep_top_n {
            return;
        }

        let mut actions: Vec<_> = self.stats.iter().collect();
        actions.sort_by(|a, b| b.1.executions.cmp(&a.1.executions));

        let keep_names: Vec<_> = actions
            .iter()
            .take(keep_top_n)
            .map(|(k, _)| k.to_string())
            .collect();
        self.stats.retain(|k, _| keep_names.contains(k));
    }
}

impl Default for ActionHistory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_stats_success_rate() {
        let mut stats = ActionStats::new();
        assert_eq!(stats.success_rate(), 0.5); // Default for unknown

        stats.executions = 10;
        stats.successes = 7;
        stats.failures = 3;

        assert_eq!(stats.success_rate(), 0.7);
        assert_eq!(stats.failure_rate(), 0.3);
    }

    #[test]
    fn test_record_success() {
        let mut history = ActionHistory::new();

        history.record_success("attack", 1.5);
        history.record_success("attack", 2.5);

        let stats = history.get_action_stats("attack").unwrap();
        assert_eq!(stats.executions, 2);
        assert_eq!(stats.successes, 2);
        assert_eq!(stats.failures, 0);
        assert_eq!(stats.avg_duration, 2.0);
    }

    #[test]
    fn test_record_failure() {
        let mut history = ActionHistory::new();

        history.record_success("move", 1.0);
        history.record_failure("move");
        history.record_failure("move");

        let stats = history.get_action_stats("move").unwrap();
        assert_eq!(stats.executions, 3);
        assert_eq!(stats.successes, 1);
        assert_eq!(stats.failures, 2);
        assert!((stats.success_rate() - 0.333333).abs() < 0.01);
    }

    #[test]
    fn test_rolling_average_duration() {
        let mut history = ActionHistory::new();

        history.record_success("heal", 1.0);
        history.record_success("heal", 2.0);
        history.record_success("heal", 3.0);

        let stats = history.get_action_stats("heal").unwrap();
        assert_eq!(stats.avg_duration, 2.0);
    }

    #[test]
    fn test_merge_histories() {
        let mut history1 = ActionHistory::new();
        history1.record_success("attack", 1.0);
        history1.record_success("attack", 2.0);

        let mut history2 = ActionHistory::new();
        history2.record_success("attack", 3.0);
        history2.record_failure("attack");

        history1.merge(&history2);

        let stats = history1.get_action_stats("attack").unwrap();
        assert_eq!(stats.executions, 4);
        assert_eq!(stats.successes, 3);
        assert_eq!(stats.failures, 1);
        assert_eq!(stats.success_rate(), 0.75);
    }

    #[test]
    fn test_prune() {
        let mut history = ActionHistory::new();

        history.record_success("action1", 1.0);
        history.record_success("action1", 1.0);
        history.record_success("action1", 1.0);

        history.record_success("action2", 1.0);
        history.record_success("action2", 1.0);

        history.record_success("action3", 1.0);

        assert_eq!(history.len(), 3);

        history.prune(2);

        assert_eq!(history.len(), 2);
        assert!(history.get_action_stats("action1").is_some());
        assert!(history.get_action_stats("action2").is_some());
        assert!(history.get_action_stats("action3").is_none());
    }

    #[test]
    fn test_reliability_score() {
        let mut stats = ActionStats::new();

        // Low confidence (few executions)
        stats.executions = 2;
        stats.successes = 2;
        assert!(stats.reliability_score() < 0.2);

        // High confidence (many executions)
        stats.executions = 20;
        stats.successes = 18;
        assert!(stats.reliability_score() > 0.85);
    }
}
