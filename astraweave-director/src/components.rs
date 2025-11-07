use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::{LlmDirectorConfig, PlayerBehaviorModel, TacticOutcome, TacticPlan};

/// ECS component for storing director AI state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CDirectorState {
    /// Current player behavior model
    pub player_model: PlayerBehaviorModel,
    /// Current tactic plan being executed
    pub current_plan: Option<TacticPlan>,
    /// Configuration for LLM director
    pub config: LlmDirectorConfig,
    /// Recent encounter outcomes for learning
    pub recent_outcomes: Vec<TacticOutcome>,
    /// Current difficulty multiplier
    pub difficulty_modifier: f32,
    /// Timestamp of last adaptation
    pub last_adaptation_time: u64,
}

impl Default for CDirectorState {
    fn default() -> Self {
        Self {
            player_model: PlayerBehaviorModel::default(),
            current_plan: None,
            config: LlmDirectorConfig::default(),
            recent_outcomes: Vec::new(),
            difficulty_modifier: 1.0,
            last_adaptation_time: 0,
        }
    }
}

impl CDirectorState {
    pub fn new(config: LlmDirectorConfig) -> Self {
        Self {
            config,
            ..Default::default()
        }
    }

    /// Check if enough time has passed for adaptation
    pub fn should_adapt(&self, current_time: u64, adaptation_interval: u64) -> bool {
        current_time - self.last_adaptation_time >= adaptation_interval
    }

    /// Update with new tactic plan
    pub fn update_plan(&mut self, plan: TacticPlan, current_time: u64) {
        self.current_plan = Some(plan);
        self.last_adaptation_time = current_time;
    }

    /// Record outcome and update learning
    pub fn record_outcome(&mut self, outcome: TacticOutcome) {
        self.player_model.update_from_outcome(&outcome);
        self.recent_outcomes.push(outcome);

        // Keep only recent outcomes
        if self.recent_outcomes.len() > 10 {
            self.recent_outcomes.remove(0);
        }
    }

    /// Get effectiveness of recent tactics
    pub fn get_recent_effectiveness(&self) -> f32 {
        if self.recent_outcomes.is_empty() {
            return 0.5; // neutral
        }

        let total: f32 = self
            .recent_outcomes
            .iter()
            .map(|outcome| outcome.effectiveness)
            .sum();

        total / self.recent_outcomes.len() as f32
    }

    /// Clear learning data (for new encounters or testing)
    pub fn reset_learning(&mut self) {
        self.player_model = PlayerBehaviorModel::default();
        self.recent_outcomes.clear();
        self.current_plan = None;
        self.difficulty_modifier = 1.0;
        self.last_adaptation_time = 0;
    }
}

/// ECS component for LLM-based tactic execution tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CTacticExecution {
    /// The tactic plan being executed
    pub plan: TacticPlan,
    /// Start time of execution
    pub start_time: u64,
    /// Current operation index
    pub current_operation: usize,
    /// Whether execution is paused
    pub is_paused: bool,
    /// Execution metadata
    pub metadata: HashMap<String, String>,
}

impl CTacticExecution {
    pub fn new(plan: TacticPlan, start_time: u64) -> Self {
        Self {
            plan,
            start_time,
            current_operation: 0,
            is_paused: false,
            metadata: HashMap::new(),
        }
    }

    /// Check if execution is complete
    pub fn is_complete(&self) -> bool {
        self.current_operation >= self.plan.operations.len()
    }

    /// Get current operation if not complete
    pub fn get_current_operation(&self) -> Option<&astraweave_core::DirectorOp> {
        if !self.is_complete() && !self.is_paused {
            self.plan.operations.get(self.current_operation)
        } else {
            None
        }
    }

    /// Advance to next operation
    pub fn advance_operation(&mut self) -> bool {
        if !self.is_complete() {
            self.current_operation += 1;
            true
        } else {
            false
        }
    }

    /// Pause execution
    pub fn pause(&mut self) {
        self.is_paused = true;
    }

    /// Resume execution
    pub fn resume(&mut self) {
        self.is_paused = false;
    }

    /// Get execution duration so far
    pub fn get_duration(&self, current_time: u64) -> u64 {
        current_time - self.start_time
    }

    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: String) {
        self.metadata.insert(key, value);
    }
}

/// ECS component for director performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CDirectorMetrics {
    /// Total number of tactics executed
    pub tactics_executed: u64,
    /// Total number of successful tactics
    pub successful_tactics: u64,
    /// Average tactic effectiveness
    pub average_effectiveness: f32,
    /// Total adaptation time spent (milliseconds)
    pub total_adaptation_time: u64,
    /// Number of difficulty adjustments made
    pub difficulty_adjustments: u64,
    /// Player skill progression over time
    pub skill_progression: Vec<(u64, f32)>, // (timestamp, skill_level)
    /// LLM call statistics
    pub llm_calls: u64,
    pub llm_failures: u64,
    pub average_response_time: f32, // milliseconds
}

impl CDirectorMetrics {
    /// Record a completed tactic
    pub fn record_tactic(&mut self, outcome: &TacticOutcome, _response_time_ms: u64) {
        self.tactics_executed += 1;

        if outcome.effectiveness > 0.6 {
            self.successful_tactics += 1;
        }

        // Update average effectiveness (rolling average)
        let total_effectiveness =
            self.average_effectiveness * (self.tactics_executed - 1) as f32 + outcome.effectiveness;
        self.average_effectiveness = total_effectiveness / self.tactics_executed as f32;
    }

    /// Record an LLM call
    pub fn record_llm_call(&mut self, response_time_ms: u64, success: bool) {
        self.llm_calls += 1;

        if !success {
            self.llm_failures += 1;
        }

        // Update average response time
        let total_time =
            self.average_response_time * (self.llm_calls - 1) as f32 + response_time_ms as f32;
        self.average_response_time = total_time / self.llm_calls as f32;
    }

    /// Record difficulty adjustment
    pub fn record_difficulty_adjustment(&mut self, adaptation_time_ms: u64) {
        self.difficulty_adjustments += 1;
        self.total_adaptation_time += adaptation_time_ms;
    }

    /// Record player skill progression
    pub fn record_skill_progression(&mut self, timestamp: u64, skill_level: f32) {
        self.skill_progression.push((timestamp, skill_level));

        // Keep only recent progression data
        if self.skill_progression.len() > 100 {
            self.skill_progression.remove(0);
        }
    }

    /// Get success rate
    pub fn get_success_rate(&self) -> f32 {
        if self.tactics_executed == 0 {
            0.0
        } else {
            self.successful_tactics as f32 / self.tactics_executed as f32
        }
    }

    /// Get LLM failure rate
    pub fn get_llm_failure_rate(&self) -> f32 {
        if self.llm_calls == 0 {
            0.0
        } else {
            self.llm_failures as f32 / self.llm_calls as f32
        }
    }

    /// Get average adaptation time
    pub fn get_average_adaptation_time(&self) -> f32 {
        if self.difficulty_adjustments == 0 {
            0.0
        } else {
            self.total_adaptation_time as f32 / self.difficulty_adjustments as f32
        }
    }

    /// Clear all metrics (for testing or reset)
    pub fn reset(&mut self) {
        *self = Default::default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_director_state_adaptation_timing() {
        let state = CDirectorState::default();

        // Should adapt immediately on first check
        assert!(state.should_adapt(1000, 500));

        let mut state = CDirectorState::default();
        state.last_adaptation_time = 1000;

        // Should not adapt before interval
        assert!(!state.should_adapt(1200, 500));

        // Should adapt after interval
        assert!(state.should_adapt(1600, 500));
    }

    #[test]
    fn test_tactic_execution_flow() {
        use astraweave_core::DirectorOp;

        let plan = TacticPlan {
            strategy: "test".to_string(),
            reasoning: "test".to_string(),
            operations: vec![DirectorOp::SpawnWave {
                archetype: "minion".to_string(),
                count: 3,
                origin: astraweave_core::IVec2 { x: 0, y: 0 },
            }],
            difficulty_modifier: 1.0,
            expected_duration: 30,
            counter_strategies: vec![],
            fallback_plan: None,
        };

        let mut execution = CTacticExecution::new(plan, 1000);

        assert!(!execution.is_complete());
        assert!(execution.get_current_operation().is_some());

        assert!(execution.advance_operation());
        assert!(execution.is_complete());
        assert!(execution.get_current_operation().is_none());
    }

    #[test]
    fn test_metrics_tracking() {
        let mut metrics = CDirectorMetrics::default();

        let outcome = TacticOutcome {
            tactic_used: "test".to_string(),
            effectiveness: 0.8,
            player_response: "good".to_string(),
            counter_strategy: "none".to_string(),
            duration_actual: 30,
            timestamp: 1000,
        };

        metrics.record_tactic(&outcome, 100);

        assert_eq!(metrics.tactics_executed, 1);
        assert_eq!(metrics.successful_tactics, 1);
        assert_eq!(metrics.get_success_rate(), 1.0);
        assert_eq!(metrics.average_effectiveness, 0.8);
    }
}
