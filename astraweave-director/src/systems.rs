use anyhow::Result;
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::{
    CDirectorMetrics, CDirectorState, CTacticExecution, LlmDirector, TacticOutcome, TacticPlan,
};
use astraweave_core::{DirectorBudget, WorldSnapshot};

/// System for updating director AI with LLM integration
pub struct DirectorLlmSystem {
    llm_director: Arc<LlmDirector>,
    adaptation_interval_ms: u64,
}

impl DirectorLlmSystem {
    pub fn new(llm_director: Arc<LlmDirector>, adaptation_interval_ms: u64) -> Self {
        Self {
            llm_director,
            adaptation_interval_ms,
        }
    }

    /// Main system update for LLM director integration
    pub async fn update(
        &self,
        director_state: &mut CDirectorState,
        tactic_execution: &mut Option<CTacticExecution>,
        metrics: &mut CDirectorMetrics,
        snapshot: &WorldSnapshot,
        budget: &DirectorBudget,
        current_time_ms: u64,
    ) -> Result<()> {
        // Check if we should adapt tactics
        if director_state.should_adapt(current_time_ms, self.adaptation_interval_ms) {
            self.adapt_tactics(
                director_state,
                tactic_execution,
                metrics,
                snapshot,
                budget,
                current_time_ms,
            )
            .await?;
        }

        // Update tactic execution if active
        if let Some(execution) = tactic_execution {
            self.update_execution(execution, director_state, metrics, current_time_ms)
                .await?;
        }

        // Periodic difficulty adjustment
        if director_state.recent_outcomes.len() >= 3 {
            self.adjust_difficulty(director_state, metrics).await?;
        }

        Ok(())
    }

    /// Adapt tactics based on current game state
    async fn adapt_tactics(
        &self,
        director_state: &mut CDirectorState,
        tactic_execution: &mut Option<CTacticExecution>,
        metrics: &mut CDirectorMetrics,
        snapshot: &WorldSnapshot,
        budget: &DirectorBudget,
        current_time_ms: u64,
    ) -> Result<()> {
        debug!("Adapting director tactics based on player behavior");

        let start_time = std::time::Instant::now();

        // Generate new tactics using LLM
        match self.llm_director.adapt_tactics(snapshot, budget).await {
            Ok(plan) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                metrics.record_llm_call(response_time, true);

                info!("Generated new tactic plan: {}", plan.strategy);

                // Stop current execution if any
                if tactic_execution.is_some() {
                    debug!("Interrupting current tactic execution for new plan");
                }

                // Update director state
                director_state.update_plan(plan.clone(), current_time_ms);

                // Start new execution
                *tactic_execution = Some(CTacticExecution::new(plan, current_time_ms));

                Ok(())
            }
            Err(e) => {
                let response_time = start_time.elapsed().as_millis() as u64;
                metrics.record_llm_call(response_time, false);

                error!("Failed to adapt tactics: {}", e);

                // Fall back to previous plan or default behavior
                if director_state.current_plan.is_none() {
                    warn!("No fallback plan available, using default tactics");
                    self.use_fallback_tactics(
                        director_state,
                        tactic_execution,
                        snapshot,
                        budget,
                        current_time_ms,
                    );
                }

                Err(e)
            }
        }
    }

    /// Update ongoing tactic execution
    async fn update_execution(
        &self,
        execution: &mut CTacticExecution,
        director_state: &mut CDirectorState,
        metrics: &mut CDirectorMetrics,
        current_time_ms: u64,
    ) -> Result<()> {
        if execution.is_paused || execution.is_complete() {
            return Ok(());
        }

        let duration = execution.get_duration(current_time_ms);

        // Check if execution has taken too long
        if duration > (execution.plan.expected_duration as u64 * 1000 * 2) {
            warn!("Tactic execution taking longer than expected, evaluating effectiveness");

            let outcome = TacticOutcome {
                tactic_used: execution.plan.strategy.clone(),
                effectiveness: 0.3, // Low effectiveness for timeout
                player_response: "timeout".to_string(),
                counter_strategy: "time_pressure".to_string(),
                duration_actual: (duration / 1000) as u32,
                timestamp: current_time_ms,
            };

            self.complete_execution(execution, director_state, metrics, outcome)
                .await?;
        }

        Ok(())
    }

    /// Adjust difficulty based on player performance
    async fn adjust_difficulty(
        &self,
        director_state: &mut CDirectorState,
        metrics: &mut CDirectorMetrics,
    ) -> Result<()> {
        let current_skill = director_state.player_model.skill_level;
        let recent_effectiveness = director_state.get_recent_effectiveness();

        debug!(
            "Current player skill: {:.2}, Recent effectiveness: {:.2}",
            current_skill, recent_effectiveness
        );

        let start_time = std::time::Instant::now();

        match self.llm_director.adjust_difficulty(current_skill).await {
            Ok(new_difficulty) => {
                let adjustment_time = start_time.elapsed().as_millis() as u64;
                metrics.record_difficulty_adjustment(adjustment_time);

                if (new_difficulty - director_state.difficulty_modifier).abs() > 0.1 {
                    info!(
                        "Adjusted difficulty from {:.2} to {:.2}",
                        director_state.difficulty_modifier, new_difficulty
                    );

                    director_state.difficulty_modifier = new_difficulty;
                }

                Ok(())
            }
            Err(e) => {
                error!("Failed to adjust difficulty: {}", e);
                Err(e)
            }
        }
    }

    /// Complete tactic execution and record outcome
    async fn complete_execution(
        &self,
        execution: &mut CTacticExecution,
        director_state: &mut CDirectorState,
        metrics: &mut CDirectorMetrics,
        outcome: TacticOutcome,
    ) -> Result<()> {
        debug!(
            "Completing tactic execution: {} (effectiveness: {:.2})",
            outcome.tactic_used, outcome.effectiveness
        );

        // Record outcome
        director_state.record_outcome(outcome.clone());
        metrics.record_tactic(&outcome, execution.get_duration(outcome.timestamp));

        // Store in LLM director for learning
        self.llm_director.record_outcome(outcome).await?;

        // Mark execution as complete
        execution.current_operation = execution.plan.operations.len();

        Ok(())
    }

    /// Use fallback tactics when LLM fails
    fn use_fallback_tactics(
        &self,
        director_state: &mut CDirectorState,
        tactic_execution: &mut Option<CTacticExecution>,
        snapshot: &WorldSnapshot,
        budget: &DirectorBudget,
        current_time_ms: u64,
    ) {
        use astraweave_core::{DirectorOp, IVec2};

        warn!("Using fallback tactics due to LLM failure");

        // Simple fallback: spawn enemies near player
        let plan = TacticPlan {
            strategy: "fallback_spawn".to_string(),
            reasoning: "LLM failure fallback - spawn enemies near player".to_string(),
            operations: vec![DirectorOp::SpawnWave {
                archetype: "minion".into(),
                // budget.spawns is i32; ensure we produce a u32 count safely
                count: {
                    let max_spawn = 3i32;
                    let chosen = std::cmp::min(budget.spawns, max_spawn).max(0) as u32;
                    chosen
                },
                origin: IVec2 {
                    x: snapshot.player.pos.x - 3,
                    y: snapshot.player.pos.y - 3,
                },
            }],
            difficulty_modifier: director_state.difficulty_modifier,
            expected_duration: 20,
            counter_strategies: vec!["player_movement".to_string()],
            fallback_plan: None,
        };

        director_state.update_plan(plan.clone(), current_time_ms);
        *tactic_execution = Some(CTacticExecution::new(plan, current_time_ms));
    }

    /// Evaluate current tactic effectiveness (called by game systems)
    pub async fn evaluate_tactic_effectiveness(
        &self,
        execution: &CTacticExecution,
        director_state: &mut CDirectorState,
        metrics: &mut CDirectorMetrics,
        player_response: String,
        actual_effectiveness: f32,
        current_time_ms: u64,
    ) -> Result<()> {
        let outcome = TacticOutcome {
            tactic_used: execution.plan.strategy.clone(),
            effectiveness: actual_effectiveness,
            player_response,
            counter_strategy: self.determine_counter_strategy(actual_effectiveness),
            duration_actual: (execution.get_duration(current_time_ms) / 1000) as u32,
            timestamp: current_time_ms,
        };

        director_state.record_outcome(outcome.clone());
        metrics.record_tactic(&outcome, execution.get_duration(current_time_ms));

        // Store learning data
        self.llm_director.record_outcome(outcome).await?;

        Ok(())
    }

    /// Determine counter-strategy based on effectiveness
    fn determine_counter_strategy(&self, effectiveness: f32) -> String {
        match effectiveness {
            e if e > 0.8 => "overwhelm_tactics".to_string(),
            e if e > 0.6 => "pressure_tactics".to_string(),
            e if e > 0.4 => "adaptive_strategy".to_string(),
            e if e > 0.2 => "defensive_counter".to_string(),
            _ => "retreat_regroup".to_string(),
        }
    }

    /// Force immediate adaptation (for testing or special events)
    pub async fn force_adaptation(
        &self,
        director_state: &mut CDirectorState,
        tactic_execution: &mut Option<CTacticExecution>,
        metrics: &mut CDirectorMetrics,
        snapshot: &WorldSnapshot,
        budget: &DirectorBudget,
        current_time_ms: u64,
    ) -> Result<()> {
        info!("Forcing immediate director adaptation");

        director_state.last_adaptation_time = 0; // Force adaptation
        self.adapt_tactics(
            director_state,
            tactic_execution,
            metrics,
            snapshot,
            budget,
            current_time_ms,
        )
        .await
    }

    /// Reset director learning (for new encounters)
    pub async fn reset_learning(
        &self,
        director_state: &mut CDirectorState,
        tactic_execution: &mut Option<CTacticExecution>,
        metrics: &mut CDirectorMetrics,
    ) -> Result<()> {
        info!("Resetting director learning data");

        director_state.reset_learning();
        *tactic_execution = None;
        metrics.reset();

        self.llm_director.reset_player_model().await;

        Ok(())
    }
}

/// Helper functions for integration with game systems
pub mod integration {
    use super::*;

    /// Initialize director system for a new encounter
    pub async fn initialize_director_encounter(
        llm_director: Arc<LlmDirector>,
        adaptation_interval_ms: u64,
    ) -> (DirectorLlmSystem, CDirectorState, CDirectorMetrics) {
        let system = DirectorLlmSystem::new(llm_director, adaptation_interval_ms);
        let state = CDirectorState::default();
        let metrics = CDirectorMetrics::default();

        (system, state, metrics)
    }

    /// Process director operations for ECS integration
    pub fn process_director_operations(
        execution: &CTacticExecution,
        _current_time_ms: u64,
    ) -> Vec<astraweave_core::DirectorOp> {
        if let Some(op) = execution.get_current_operation() {
            vec![op.clone()]
        } else {
            Vec::new()
        }
    }

    /// Check if director needs player performance feedback
    pub fn should_provide_feedback(
        execution: &CTacticExecution,
        last_feedback_time: u64,
        current_time_ms: u64,
        feedback_interval_ms: u64,
    ) -> bool {
        !execution.is_complete() && current_time_ms - last_feedback_time >= feedback_interval_ms
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::LlmDirectorConfig;
    use astraweave_llm::MockLlmClient;
    use astraweave_rag::MockRagPipeline;

    #[tokio::test]
    async fn test_director_system_creation() {
        let llm_client = Arc::new(MockLlmClient::new());
        let rag_pipeline = Arc::new(MockRagPipeline::new());
        let llm_director = Arc::new(
            LlmDirector::new(llm_client, rag_pipeline, LlmDirectorConfig::default()).unwrap(),
        );

        let system = DirectorLlmSystem::new(llm_director, 5000);

        // Test would require more setup for actual functionality
        // This demonstrates the integration structure
    }

    #[test]
    fn test_counter_strategy_determination() {
        let llm_client = Arc::new(MockLlmClient::new());
        let system = DirectorLlmSystem::new(llm_client, 5000);

        assert_eq!(system.determine_counter_strategy(0.9), "overwhelm_tactics");
        assert_eq!(system.determine_counter_strategy(0.7), "pressure_tactics");
        assert_eq!(system.determine_counter_strategy(0.5), "adaptive_strategy");
        assert_eq!(system.determine_counter_strategy(0.3), "defensive_counter");
        assert_eq!(system.determine_counter_strategy(0.1), "retreat_regroup");
    }
}
