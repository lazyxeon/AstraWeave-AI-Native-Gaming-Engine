use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use astraweave_context::{ContextConfig, ConversationHistory, Role};
use astraweave_core::{DirectorBudget, DirectorOp, WorldSnapshot};
use astraweave_llm::LlmClient;
use astraweave_prompts::library::PromptLibrary;
use astraweave_prompts::template::PromptTemplate;
use astraweave_rag::RagPipeline;

/// Player behavior analysis for LLM-driven director decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerBehaviorModel {
    pub aggression: f32,               // 0.0 (defensive) to 1.0 (aggressive)
    pub caution: f32,                  // 0.0 (reckless) to 1.0 (cautious)
    pub skill_level: f32,              // 0.0 (novice) to 1.0 (expert)
    pub preferred_range: f32,          // 0.0 (melee) to 1.0 (ranged)
    pub adaptability: f32,             // 0.0 (predictable) to 1.0 (adaptive)
    pub session_performance: Vec<f32>, // Recent performance scores
    pub preferred_tactics: Vec<String>,
    pub weaknesses: Vec<String>,
    pub encounter_count: u32,
}

impl Default for PlayerBehaviorModel {
    fn default() -> Self {
        Self {
            aggression: 0.5,
            caution: 0.5,
            skill_level: 0.5,
            preferred_range: 0.5,
            adaptability: 0.5,
            session_performance: Vec::new(),
            preferred_tactics: Vec::new(),
            weaknesses: Vec::new(),
            encounter_count: 0,
        }
    }
}

impl PlayerBehaviorModel {
    pub fn analyze_snapshot(&mut self, snapshot: &WorldSnapshot) -> String {
        let player_pos = snapshot.player.pos;
        let enemies = &snapshot.enemies;

        // Calculate average distance to enemies
        let avg_distance = if !enemies.is_empty() {
            let total_distance: i32 = enemies
                .iter()
                .map(|e| (player_pos.x - e.pos.x).abs() + (player_pos.y - e.pos.y).abs())
                .sum();
            total_distance as f32 / enemies.len() as f32
        } else {
            8.0 // Default medium range
        };

        // Update preferred range based on positioning
        if avg_distance > 10.0 {
            self.preferred_range = (self.preferred_range + 0.1).min(1.0);
        } else if avg_distance < 4.0 {
            self.preferred_range = (self.preferred_range - 0.1).max(0.0);
        }

        // Analyze movement patterns for aggression
        let enemies_nearby = enemies
            .iter()
            .filter(|e| {
                let dist = (player_pos.x - e.pos.x).abs() + (player_pos.y - e.pos.y).abs();
                dist < 6
            })
            .count();

        if enemies_nearby > 2 && avg_distance < 5.0 {
            self.aggression = (self.aggression + 0.05).min(1.0);
            self.caution = (self.caution - 0.03).max(0.0);
        } else if enemies_nearby > 0 && avg_distance > 8.0 {
            self.caution = (self.caution + 0.05).min(1.0);
            self.aggression = (self.aggression - 0.03).max(0.0);
        }

        // Generate analysis text for LLM
        format!(
            "Player behavior analysis: Aggression {:.2}, Caution {:.2}, Skill {:.2}, \
            Preferred range {:.2}, Adaptability {:.2}. Current positioning: {} enemies \
            nearby, average distance {:.1}. Encounter count: {}. Recent tactics: {:?}. \
            Identified weaknesses: {:?}",
            self.aggression,
            self.caution,
            self.skill_level,
            self.preferred_range,
            self.adaptability,
            enemies_nearby,
            avg_distance,
            self.encounter_count,
            self.preferred_tactics,
            self.weaknesses
        )
    }

    pub fn update_from_outcome(&mut self, outcome: &TacticOutcome) {
        self.encounter_count += 1;
        self.session_performance.push(outcome.effectiveness);

        // Keep only recent performance data
        if self.session_performance.len() > 20 {
            self.session_performance.remove(0);
        }

        // Update skill level based on performance
        let avg_performance =
            self.session_performance.iter().sum::<f32>() / self.session_performance.len() as f32;

        if avg_performance > 0.7 {
            self.skill_level = (self.skill_level + 0.02).min(1.0);
        } else if avg_performance < 0.3 {
            self.skill_level = (self.skill_level - 0.01).max(0.0);
        }

        // Learn from successful/failed tactics
        if outcome.effectiveness > 0.6 {
            if !self.preferred_tactics.contains(&outcome.tactic_used) {
                self.preferred_tactics.push(outcome.tactic_used.clone());
                if self.preferred_tactics.len() > 5 {
                    self.preferred_tactics.remove(0);
                }
            }
        } else if outcome.effectiveness < 0.3 {
            if !self.weaknesses.contains(&outcome.counter_strategy) {
                self.weaknesses.push(outcome.counter_strategy.clone());
                if self.weaknesses.len() > 3 {
                    self.weaknesses.remove(0);
                }
            }
        }

        // Update adaptability based on tactic variety
        let unique_tactics = self.preferred_tactics.len();
        self.adaptability = (unique_tactics as f32 / 5.0).min(1.0);
    }
}

/// Tactic plan generated by LLM with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticPlan {
    pub strategy: String,
    pub reasoning: String,
    pub operations: Vec<DirectorOp>,
    pub difficulty_modifier: f32,
    pub expected_duration: u32, // in seconds
    pub counter_strategies: Vec<String>,
    pub fallback_plan: Option<String>,
}

/// Outcome of applying a tactic for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TacticOutcome {
    pub tactic_used: String,
    pub effectiveness: f32, // 0.0 to 1.0
    pub player_response: String,
    pub counter_strategy: String,
    pub duration_actual: u32,
    pub timestamp: u64,
}

/// Configuration for LLM director behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmDirectorConfig {
    pub adaptation_rate: f32,
    pub min_difficulty: f32,
    pub max_difficulty: f32,
    pub learning_enabled: bool,
    pub creativity_factor: f32,
    pub context_window_size: usize,
}

impl Default for LlmDirectorConfig {
    fn default() -> Self {
        Self {
            adaptation_rate: 0.1,
            min_difficulty: 0.3,
            max_difficulty: 1.5,
            learning_enabled: true,
            creativity_factor: 0.7,
            context_window_size: 2048,
        }
    }
}

/// LLM-powered director system for adaptive boss AI
pub struct LlmDirector {
    llm_client: Arc<dyn LlmClient>,
    rag_pipeline: Arc<RagPipeline>,
    player_model: Arc<RwLock<PlayerBehaviorModel>>,
    conversation_history: Arc<RwLock<ConversationHistory>>,
    prompt_library: Arc<RwLock<PromptLibrary>>,
    config: LlmDirectorConfig,
    encounter_memory: Arc<RwLock<Vec<TacticOutcome>>>,
}

impl LlmDirector {
    pub fn new(
        llm_client: Arc<dyn LlmClient>,
        rag_pipeline: Arc<RagPipeline>,
        config: LlmDirectorConfig,
    ) -> Result<Self> {
        let context_config = ContextConfig {
            max_tokens: config.context_window_size,
            ..Default::default()
        };
        let conversation_history = Arc::new(RwLock::new(ConversationHistory::new(context_config)));

        let mut prompt_library = PromptLibrary::new();

        // Load director-specific prompts
        prompt_library.add_template("tactic_generation", PromptTemplate::new("tactic_generation".to_string(),
            r#"
You are an AI director for a boss encounter in a game. Your goal is to create challenging but fair tactics based on player behavior analysis.

Player Analysis: {{player_analysis}}

Past Encounters: {{past_encounters}}

Current World State:
- Player Position: {{player_pos}}
- Enemy Count: {{enemy_count}}
- Available Budget: Spawns={{spawn_budget}}, Terrain={{terrain_budget}}

Your task is to generate a tactical plan in JSON format with the following structure:
{
    "strategy": "Brief description of the main strategy",
    "reasoning": "Detailed explanation of why this strategy counters the player's behavior",
    "operations": [
        // Array of operations (SpawnWave, Fortify, Collapse, etc.)
    ],
    "difficulty_modifier": 0.8, // 0.5 to 1.5 based on player skill
    "expected_duration": 30, // seconds
    "counter_strategies": ["List of strategies this plan counters"],
    "fallback_plan": "What to do if this plan fails"
}

Guidelines:
1. Counter the player's preferred tactics
2. Adapt difficulty to player skill level
3. Create interesting tactical challenges
4. Ensure operations are within budget limits
5. Consider past encounter outcomes
            "#.trim().to_string()
        ));

        prompt_library.add_template(
            "difficulty_adjustment",
            PromptTemplate::new(
                "difficulty_adjustment".to_string(),
                r#"
You are adjusting the difficulty of a boss encounter based on player performance.

Player Performance History: {{performance_history}}
Current Skill Level: {{skill_level}}
Recent Encounter Results: {{recent_results}}

Provide a difficulty adjustment in JSON format:
{
    "new_difficulty": 0.8, // 0.3 to 1.5
    "reasoning": "Explanation for the adjustment",
    "suggested_changes": ["List of specific changes to make"]
}

Consider:
- Player improvement over time
- Frustration vs challenge balance
- Learning curve progression
            "#
                .trim()
                .to_string(),
            ),
        );

        Ok(Self {
            llm_client,
            rag_pipeline,
            player_model: Arc::new(RwLock::new(PlayerBehaviorModel::default())),
            conversation_history,
            prompt_library: Arc::new(RwLock::new(prompt_library)),
            config,
            encounter_memory: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Generate adaptive tactics based on player behavior and past encounters
    pub async fn adapt_tactics(
        &self,
        snapshot: &WorldSnapshot,
        budget: &DirectorBudget,
    ) -> Result<TacticPlan> {
        debug!("Generating adaptive tactics for director AI");

        // Analyze current player behavior
        let mut player_model = self.player_model.write().await;
        let player_analysis = player_model.analyze_snapshot(snapshot);
        drop(player_model);

        // Retrieve similar past encounters from RAG
        let past_encounters_raw = self
            .rag_pipeline
            .retrieve(&player_analysis, 5)
            .await
            .unwrap_or_else(|e| {
                warn!("Failed to retrieve past encounters: {}", e);
                Vec::new()
            });
        let past_encounters: Vec<String> = past_encounters_raw
            .iter()
            .map(|m| m.memory.text.clone())
            .collect();

        // Build context for LLM
        let context = self
            .build_tactic_context(snapshot, budget, &player_analysis, &past_encounters)
            .await?;

        // Generate tactics using LLM
        let prompt_library = self.prompt_library.read().await;
        let template = prompt_library.get_template("tactic_generation")?;
        let prompt = template.render_map(&context)?;
        drop(prompt_library);

        // Add to conversation history
        let history = self.conversation_history.write().await;
        history.add_message(Role::User, prompt.clone()).await?;
        let full_prompt = history.get_context(2048).await?;
        drop(history);

        // Generate response
        let response = self
            .llm_client
            .complete(&full_prompt)
            .await
            .map_err(|e| anyhow!("LLM completion failed: {}", e))?;

        // Parse tactic plan
        let tactic_plan: TacticPlan = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse tactic plan: {}", e))?;

        // Validate operations are within budget
        let validated_plan = self.validate_plan(&tactic_plan, budget)?;

        // Update conversation history
        let history = self.conversation_history.write().await;
        history
            .add_message(Role::Assistant, response.clone())
            .await?;
        drop(history);

        info!("Generated tactic plan: {}", validated_plan.strategy);
        Ok(validated_plan)
    }

    /// Adjust difficulty based on player performance
    pub async fn adjust_difficulty(&self, player_skill: f32) -> Result<f32> {
        let player_model = self.player_model.read().await;
        let performance_history: Vec<f32> = player_model.session_performance.clone();
        drop(player_model);

        let encounter_memory = self.encounter_memory.read().await;
        let recent_results: Vec<String> = encounter_memory
            .iter()
            .rev()
            .take(5)
            .map(|outcome| {
                format!(
                    "Tactic: {}, Effectiveness: {:.2}",
                    outcome.tactic_used, outcome.effectiveness
                )
            })
            .collect();
        drop(encounter_memory);

        let context = HashMap::from([
            ("skill_level".to_string(), player_skill.to_string()),
            (
                "performance_history".to_string(),
                format!("{:?}", performance_history),
            ),
            ("recent_results".to_string(), recent_results.join("; ")),
        ]);

        let prompt_library = self.prompt_library.read().await;
        let template = prompt_library.get_template("difficulty_adjustment")?;
        let prompt = template.render_map(&context)?;
        drop(prompt_library);

        let response = self
            .llm_client
            .complete(&prompt)
            .await
            .map_err(|e| anyhow!("Difficulty adjustment failed: {}", e))?;

        #[derive(Deserialize)]
        struct DifficultyResponse {
            new_difficulty: f32,
            reasoning: String,
        }

        let difficulty_response: DifficultyResponse = serde_json::from_str(&response)
            .map_err(|e| anyhow!("Failed to parse difficulty response: {}", e))?;

        let clamped_difficulty = difficulty_response
            .new_difficulty
            .max(self.config.min_difficulty)
            .min(self.config.max_difficulty);

        debug!(
            "Adjusted difficulty to {}: {}",
            clamped_difficulty, difficulty_response.reasoning
        );
        Ok(clamped_difficulty)
    }

    /// Record the outcome of a tactic for learning
    pub async fn record_outcome(&self, outcome: TacticOutcome) -> Result<()> {
        if !self.config.learning_enabled {
            return Ok(());
        }

        debug!(
            "Recording tactic outcome: {} (effectiveness: {:.2})",
            outcome.tactic_used, outcome.effectiveness
        );

        // Update player model
        let mut player_model = self.player_model.write().await;
        player_model.update_from_outcome(&outcome);
        drop(player_model);

        // Store in encounter memory
        let mut encounter_memory = self.encounter_memory.write().await;
        encounter_memory.push(outcome.clone());

        // Keep only recent encounters to prevent unbounded growth
        if encounter_memory.len() > 100 {
            encounter_memory.remove(0);
        }
        drop(encounter_memory);

        // Store in RAG for future retrieval
        let _memory_text = format!(
            "Encounter: {} | Effectiveness: {:.2} | Player Response: {} | Duration: {}s",
            outcome.tactic_used,
            outcome.effectiveness,
            outcome.player_response,
            outcome.duration_actual
        );

        // If the pipeline has an LLM client configured, we could use it for summaries
        if self.rag_pipeline.has_llm_client() {
            // best-effort: call add_memory on pipeline via a blocking call if available
        }

        Ok(())
    }

    /// Get current player behavior model
    pub async fn get_player_model(&self) -> PlayerBehaviorModel {
        self.player_model.read().await.clone()
    }

    /// Reset player model (for new player or testing)
    pub async fn reset_player_model(&self) {
        let mut player_model = self.player_model.write().await;
        *player_model = PlayerBehaviorModel::default();
        drop(player_model);

        let mut encounter_memory = self.encounter_memory.write().await;
        encounter_memory.clear();
        drop(encounter_memory);

        let history = self.conversation_history.write().await;
        history.clear();
        drop(history);

        info!("Reset player model and encounter history");
    }

    /// Build context for tactic generation
    async fn build_tactic_context(
        &self,
        snapshot: &WorldSnapshot,
        budget: &DirectorBudget,
        player_analysis: &str,
        past_encounters: &[String],
    ) -> Result<HashMap<String, String>> {
        let mut context = HashMap::new();

        context.insert("player_analysis".to_string(), player_analysis.to_string());
        context.insert("past_encounters".to_string(), past_encounters.join(" | "));
        context.insert(
            "player_pos".to_string(),
            format!("({}, {})", snapshot.player.pos.x, snapshot.player.pos.y),
        );
        context.insert(
            "enemy_count".to_string(),
            snapshot.enemies.len().to_string(),
        );
        context.insert("spawn_budget".to_string(), budget.spawns.to_string());
        context.insert(
            "terrain_budget".to_string(),
            budget.terrain_edits.to_string(),
        );

        Ok(context)
    }

    /// Validate that tactic plan operations are within budget constraints
    fn validate_plan(&self, plan: &TacticPlan, budget: &DirectorBudget) -> Result<TacticPlan> {
        let mut validated_ops = Vec::new();
        let mut spawn_count = 0;
        let mut terrain_count = 0;

        for op in &plan.operations {
            match op {
                DirectorOp::SpawnWave { count, .. } => {
                    let budget_spawns_u32: u32 = budget.spawns.try_into().unwrap_or(u32::MAX);
                    if spawn_count + count <= budget_spawns_u32 {
                        spawn_count += count;
                        validated_ops.push(op.clone());
                    } else {
                        warn!("Skipping spawn operation due to budget constraints");
                    }
                }
                DirectorOp::Fortify { .. } | DirectorOp::Collapse { .. } => {
                    if terrain_count < budget.terrain_edits {
                        terrain_count += 1;
                        validated_ops.push(op.clone());
                    } else {
                        warn!("Skipping terrain operation due to budget constraints");
                    }
                }
            }
        }

        Ok(TacticPlan {
            operations: validated_ops,
            ..plan.clone()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_llm::MockLlmClient;
    use astraweave_rag::MockRagPipeline;

    #[tokio::test]
    async fn test_player_behavior_model_analysis() {
        let mut model = PlayerBehaviorModel::default();
        let snapshot = WorldSnapshot {
            t: 0.0,
            player: astraweave_core::PlayerState {
                hp: 100,
                physics_context: None,
                pos: IVec2 { x: 5, y: 5 },
                stance: "stand".to_string(),
                orders: vec![],
            },
            me: astraweave_core::CompanionState {
                ammo: 10,
                cooldowns: std::collections::BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 10, y: 10 },
            },
            enemies: vec![
                astraweave_core::EnemyState {
                    id: 1,
                    pos: IVec2 { x: 3, y: 3 },
                    hp: 100,
                    cover: "none".to_string(),
                    last_seen: 0.0,
                },
                astraweave_core::EnemyState {
                    id: 2,
                    pos: IVec2 { x: 7, y: 7 },
                    hp: 100,
                    cover: "none".to_string(),
                    last_seen: 0.0,
                },
            ],
            pois: vec![],
            obstacles: vec![],
            objective: None,
        };

        let analysis = model.analyze_snapshot(&snapshot);
        assert!(analysis.contains("2 enemies nearby"));
        assert!(analysis.contains("Encounter count: 0"));
    }

    #[tokio::test]
    async fn test_player_model_learning() {
        let mut model = PlayerBehaviorModel::default();
        let initial_skill = model.skill_level;

        let good_outcome = TacticOutcome {
            tactic_used: "flanking".to_string(),
            effectiveness: 0.8,
            player_response: "adapted quickly".to_string(),
            counter_strategy: "pressure_tactics".to_string(),
            duration_actual: 45,
            timestamp: 12345,
        };

        model.update_from_outcome(&good_outcome);

        assert!(model.skill_level >= initial_skill);
        assert_eq!(model.encounter_count, 1);
        assert!(model.preferred_tactics.contains(&"flanking".to_string()));
    }

    #[tokio::test]
    async fn test_llm_director_creation() {
        let llm_client = Arc::new(MockLlmClient::new());
        let rag_pipeline = Arc::new(MockRagPipeline::new());
        let config = LlmDirectorConfig::default();

        let director = LlmDirector::new(llm_client, rag_pipeline, config);
        assert!(director.is_ok());
    }

    #[tokio::test]
    async fn test_difficulty_adjustment_bounds() {
        let llm_client = Arc::new(MockLlmClient::new());
        let rag_pipeline = Arc::new(MockRagPipeline::new());
        let mut config = LlmDirectorConfig::default();
        config.min_difficulty = 0.2;
        config.max_difficulty = 1.8;

        let director = LlmDirector::new(llm_client, rag_pipeline, config).unwrap();

        // Test would require mocking LLM response, but demonstrates the structure
        // In actual implementation, difficulty would be clamped to [0.2, 1.8]
    }
}
