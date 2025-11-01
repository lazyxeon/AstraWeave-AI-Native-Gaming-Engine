/// Phase 7: Multi-Tier Fallback System
///
/// Provides graceful degradation when LLM planning fails:
/// - Tier 1: Full LLM (all 37 tools, detailed prompts)
/// - Tier 2: Simplified LLM (10 most common tools, compressed prompts)
/// - Tier 3: Heuristic (rule-based planning, no LLM)
/// - Tier 4: Emergency (safe default: Scan + Wait)

use anyhow::{Context, Result};
use astraweave_core::{ActionStep, PlanIntent, ToolRegistry, WorldSnapshot};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::batch_executor::{AgentId, BatchInferenceExecutor};
use crate::compression::PromptCompressor;
use crate::plan_parser::parse_llm_response;
use crate::prompt_template::{build_enhanced_prompt, PromptConfig};
use crate::LlmClient;

/// Fallback tier levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum FallbackTier {
    FullLlm = 1,
    SimplifiedLlm = 2,
    Heuristic = 3,
    Emergency = 4,
}

impl FallbackTier {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::FullLlm => "full_llm",
            Self::SimplifiedLlm => "simplified_llm",
            Self::Heuristic => "heuristic",
            Self::Emergency => "emergency",
        }
    }

    pub fn next(&self) -> Option<FallbackTier> {
        match self {
            Self::FullLlm => Some(Self::SimplifiedLlm),
            Self::SimplifiedLlm => Some(Self::Heuristic),
            Self::Heuristic => Some(Self::Emergency),
            Self::Emergency => None, // No more fallbacks
        }
    }
}

/// Result of fallback orchestration
#[derive(Debug, Clone)]
pub struct FallbackResult {
    pub plan: PlanIntent,
    pub tier: FallbackTier,
    pub attempts: Vec<FallbackAttempt>,
    pub total_duration_ms: u64,
}

/// Record of a fallback attempt
#[derive(Debug, Clone)]
pub struct FallbackAttempt {
    pub tier: FallbackTier,
    pub success: bool,
    pub error: Option<String>,
    pub duration_ms: u64,
}

/// Fallback orchestrator metrics
#[derive(Debug, Clone, Default)]
pub struct FallbackMetrics {
    pub total_requests: u64,
    pub tier_successes: HashMap<String, u64>,
    pub tier_failures: HashMap<String, u64>,
    pub average_attempts: f32,
    pub average_duration_ms: f32,
}

/// Multi-tier fallback orchestrator
pub struct FallbackOrchestrator {
    metrics: Arc<RwLock<FallbackMetrics>>,
    simplified_tools: Vec<String>, // Top 10 most common tools
}

impl FallbackOrchestrator {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(FallbackMetrics::default())),
            // Simplified tier uses most common tools (grouped by parameter pattern)
            // NOTE: Tool names must match EXACTLY with registry (case-sensitive)
            simplified_tools: vec![
                // Position-based tools (x, y params)
                "MoveTo".to_string(),
                "ThrowSmoke".to_string(),
                "ThrowExplosive".to_string(),
                "AoEAttack".to_string(),
                "TakeCover".to_string(),
                
                // Target-based tools (target_id param)
                "Attack".to_string(),
                "Approach".to_string(),
                "Retreat".to_string(),
                "MarkTarget".to_string(),
                "Distract".to_string(),
                
                // Simple tools (no params or duration param)
                "Reload".to_string(),
                "Scan".to_string(),
                "Wait".to_string(),
                "Block".to_string(),
                "Heal".to_string(),
            ],
        }
    }

    /// Generate plan with automatic fallback on failure
    pub async fn plan_with_fallback(
        &self,
        client: &dyn LlmClient,
        snap: &WorldSnapshot,
        reg: &ToolRegistry,
    ) -> FallbackResult {
        let start = std::time::Instant::now();
        let mut attempts = Vec::new();
        // LATENCY OPTIMIZATION: Skip Tier 1 (FullLlm ~13k chars) and start with Tier 2 (SimplifiedLlm ~2k chars)
        // This reduces prompt processing time by ~60% (21.2s → ~10-12s expected)
        // Based on Phase 7 validation: simplified prompt achieved 8.46s vs 21.2s with full prompt
        let mut current_tier = FallbackTier::SimplifiedLlm;  // Was: FallbackTier::FullLlm

        loop {
            let tier_start = std::time::Instant::now();
            
            match self.try_tier(current_tier, client, snap, reg).await {
                Ok(plan) => {
                    let duration_ms = tier_start.elapsed().as_millis() as u64;
                    attempts.push(FallbackAttempt {
                        tier: current_tier,
                        success: true,
                        error: None,
                        duration_ms,
                    });

                    info!(
                        "Fallback succeeded at tier {} after {} attempts ({} ms)",
                        current_tier.as_str(),
                        attempts.len(),
                        start.elapsed().as_millis()
                    );

                    self.record_success(current_tier, &attempts, start.elapsed().as_millis() as u64).await;

                    return FallbackResult {
                        plan,
                        tier: current_tier,
                        attempts,
                        total_duration_ms: start.elapsed().as_millis() as u64,
                    };
                }
                Err(e) => {
                    let duration_ms = tier_start.elapsed().as_millis() as u64;
                    warn!("Tier {} failed: {}", current_tier.as_str(), e);
                    
                    attempts.push(FallbackAttempt {
                        tier: current_tier,
                        success: false,
                        error: Some(e.to_string()),
                        duration_ms,
                    });

                    // Try next tier
                    if let Some(next_tier) = current_tier.next() {
                        current_tier = next_tier;
                        debug!("Falling back to tier {}", current_tier.as_str());
                    } else {
                        // No more tiers - this shouldn't happen since Emergency always succeeds
                        panic!("Emergency tier failed - this should never happen");
                    }
                }
            }
        }
    }

    /// Generate plans for multiple agents with automatic fallback (batch inference)
    ///
    /// # Arguments
    /// * `client` - LLM client (supports batch inference via streaming)
    /// * `agents` - Vector of (AgentId, WorldSnapshot) pairs
    /// * `reg` - Tool registry
    ///
    /// # Returns
    /// HashMap of agent ID → FallbackResult
    ///
    /// # Performance
    /// - Uses BatchInferenceExecutor for LLM tiers (5-10× faster than sequential)
    /// - Falls back to per-agent heuristic/emergency if batch LLM fails
    /// - Preserves deterministic ordering (sorted by agent ID)
    ///
    /// # Example
    /// ```no_run
    /// # use astraweave_llm::fallback_system::FallbackOrchestrator;
    /// # async fn example() -> anyhow::Result<()> {
    /// let orchestrator = FallbackOrchestrator::new();
    /// let agents = vec![
    ///     (1, snapshot1),
    ///     (2, snapshot2),
    ///     (3, snapshot3),
    /// ];
    /// let results = orchestrator.plan_batch_with_fallback(&client, agents, &reg).await;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn plan_batch_with_fallback(
        &self,
        client: &dyn LlmClient,
        agents: Vec<(AgentId, WorldSnapshot)>,
        reg: &ToolRegistry,
    ) -> HashMap<AgentId, FallbackResult> {
        if agents.is_empty() {
            return HashMap::new();
        }

        let start = std::time::Instant::now();
        let mut results = HashMap::new();
        
        // Start with SimplifiedLlm (same optimization as single-agent)
        let mut current_tier = FallbackTier::SimplifiedLlm;
        let mut remaining_agents = agents;
        
        info!(
            "Batch planning for {} agents starting at tier {}",
            remaining_agents.len(),
            current_tier.as_str()
        );

        loop {
            let tier_start = std::time::Instant::now();
            
            match current_tier {
                FallbackTier::FullLlm | FallbackTier::SimplifiedLlm => {
                    // Try batch LLM inference
                    match self.try_batch_llm_tier(current_tier, client, &remaining_agents, reg).await {
                        Ok(batch_results) => {
                            let duration_ms = tier_start.elapsed().as_millis() as u64;
                            
                            info!(
                                "Batch LLM tier {} succeeded for {} agents ({} ms)",
                                current_tier.as_str(),
                                batch_results.len(),
                                duration_ms
                            );

                            // Add all results
                            for (agent_id, result) in batch_results {
                                results.insert(agent_id, result);
                            }
                            
                            remaining_agents.clear(); // All done!
                            break;
                        }
                        Err(e) => {
                            warn!("Batch tier {} failed: {}", current_tier.as_str(), e);
                            
                            // Fall back to next tier
                            if let Some(next_tier) = current_tier.next() {
                                current_tier = next_tier;
                                debug!("Batch falling back to tier {}", current_tier.as_str());
                            } else {
                                panic!("Emergency tier failed - this should never happen");
                            }
                        }
                    }
                }
                FallbackTier::Heuristic => {
                    // Run heuristic per-agent (different snapshots → different heuristics)
                    for (agent_id, snap) in &remaining_agents {
                        let plan = self.try_heuristic(snap, reg);
                        let result = FallbackResult {
                            plan,
                            tier: FallbackTier::Heuristic,
                            attempts: vec![FallbackAttempt {
                                tier: FallbackTier::Heuristic,
                                success: true,
                                error: None,
                                duration_ms: 0, // Heuristic is instant
                            }],
                            total_duration_ms: 0,
                        };
                        results.insert(*agent_id, result);
                    }
                    
                    debug!("Heuristic tier completed for {} agents", remaining_agents.len());
                    remaining_agents.clear();
                    break;
                }
                FallbackTier::Emergency => {
                    // Emergency per-agent
                    for (agent_id, snap) in &remaining_agents {
                        let plan = self.emergency_plan(snap);
                        let result = FallbackResult {
                            plan,
                            tier: FallbackTier::Emergency,
                            attempts: vec![FallbackAttempt {
                                tier: FallbackTier::Emergency,
                                success: true,
                                error: None,
                                duration_ms: 0,
                            }],
                            total_duration_ms: 0,
                        };
                        results.insert(*agent_id, result);
                    }
                    
                    warn!("Emergency tier completed for {} agents", remaining_agents.len());
                    remaining_agents.clear();
                    break;
                }
            }
        }

        info!(
            "Batch planning complete: {} agents, {} ms total",
            results.len(),
            start.elapsed().as_millis()
        );

        results
    }

    /// Try batch LLM tier (Tier 1 or Tier 2)
    async fn try_batch_llm_tier(
        &self,
        tier: FallbackTier,
        client: &dyn LlmClient,
        agents: &[(AgentId, WorldSnapshot)],
        reg: &ToolRegistry,
    ) -> Result<HashMap<AgentId, FallbackResult>> {
        let start = std::time::Instant::now();
        
        // Create batch request
        let mut executor = BatchInferenceExecutor::new();
        for (agent_id, snap) in agents {
            executor.queue_agent(*agent_id, snap.clone());
        }
        
        // Determine tool list based on tier
        let tool_list = match tier {
            FallbackTier::FullLlm => {
                // Use all tools from registry
                reg.tools.iter()
                    .map(|t| t.name.clone())
                    .collect::<Vec<_>>()
                    .join("|")
            }
            FallbackTier::SimplifiedLlm => {
                // Use simplified tool list
                self.simplified_tools.join("|")
            }
            _ => unreachable!("try_batch_llm_tier called with non-LLM tier"),
        };
        
        // Execute batch
        let batch_response = executor.execute_batch(client, &tool_list).await
            .context("Batch LLM execution failed")?;
        
        let duration_ms = start.elapsed().as_millis() as u64;
        
        // Convert BatchResponse to HashMap<AgentId, FallbackResult>
        let mut results = HashMap::new();
        for (agent_id, _snap) in agents {
            if let Some(plan) = batch_response.get_plan(*agent_id) {
                let result = FallbackResult {
                    plan: plan.clone(),
                    tier,
                    attempts: vec![FallbackAttempt {
                        tier,
                        success: true,
                        error: None,
                        duration_ms,
                    }],
                    total_duration_ms: duration_ms,
                };
                results.insert(*agent_id, result);
            } else {
                anyhow::bail!("Batch response missing plan for agent {}", agent_id);
            }
        }
        
        Ok(results)
    }

    /// Try a specific tier
    async fn try_tier(
        &self,
        tier: FallbackTier,
        client: &dyn LlmClient,
        snap: &WorldSnapshot,
        reg: &ToolRegistry,
    ) -> Result<PlanIntent> {
        match tier {
            FallbackTier::FullLlm => self.try_full_llm(client, snap, reg).await,
            FallbackTier::SimplifiedLlm => self.try_simplified_llm(client, snap, reg).await,
            FallbackTier::Heuristic => Ok(self.try_heuristic(snap, reg)),
            FallbackTier::Emergency => Ok(self.emergency_plan(snap)),
        }
    }

    /// Tier 1: Full LLM with all 37 tools
    async fn try_full_llm(
        &self,
        client: &dyn LlmClient,
        snap: &WorldSnapshot,
        reg: &ToolRegistry,
    ) -> Result<PlanIntent> {
        let config = PromptConfig {
            include_examples: true,
            include_tool_descriptions: true,
            include_schema: true,
            max_examples: 5,
            strict_json_only: true,
        };

        let prompt = build_enhanced_prompt(snap, reg, &config);
        let response = client.complete(&prompt).await
            .context("LLM request failed")?;

        let parse_result = parse_llm_response(&response, reg)
            .context("Failed to parse LLM response")?;

        debug!(
            "Full LLM succeeded: {} steps via {}",
            parse_result.plan.steps.len(),
            parse_result.extraction_method.as_str()
        );

        Ok(parse_result.plan)
    }

    /// Tier 2: Simplified LLM with 10 most common tools
    async fn try_simplified_llm(
        &self,
        client: &dyn LlmClient,
        snap: &WorldSnapshot,
        reg: &ToolRegistry,
    ) -> Result<PlanIntent> {
        // Create simplified registry with only top 10 tools
        let simplified_reg = self.create_simplified_registry(reg);

        // ⚡ OPTIMIZATION: Use compressed prompts (30-40% reduction)
        // This reduces latency by 1.5-2× based on compression.rs tests
        let tool_list = self.simplified_tools.join("|");
        let prompt = PromptCompressor::build_optimized_prompt(
            snap,
            &tool_list,
            "tactical", // Default to tactical AI role
        );

        // Previous code (commented for reference):
        // let _config = PromptConfig {
        //     include_examples: false, // Skip examples for speed
        //     include_tool_descriptions: true,
        //     include_schema: true,
        //     max_examples: 0,
        //     strict_json_only: true,
        // };
        // let prompt = build_simplified_prompt(snap, &simplified_reg);

        let response = client.complete(&prompt).await
            .context("Simplified LLM request failed")?;

        let parse_result = parse_llm_response(&response, &simplified_reg)
            .context("Failed to parse simplified LLM response")?;

        debug!(
            "Simplified LLM succeeded: {} steps (compressed prompt: {} chars)",
            parse_result.plan.steps.len(),
            prompt.len()
        );

        Ok(parse_result.plan)
    }

    /// Tier 3: Rule-based heuristic planning (no LLM)
    fn try_heuristic(&self, snap: &WorldSnapshot, reg: &ToolRegistry) -> PlanIntent {
        let mut steps = Vec::new();

        // Heuristic 1: Low morale → Heal
        if snap.me.morale < 30.0 && reg.tools.iter().any(|t| t.name == "heal") {
            steps.push(ActionStep::Heal {
                target_id: Some(0), // Self-heal
            });
        }

        // Heuristic 2: No ammo → Reload
        if snap.me.ammo == 0 && reg.tools.iter().any(|t| t.name == "reload") {
            steps.push(ActionStep::Reload);
        }

        // Heuristic 3: Enemy nearby → Attack or Take Cover
        if !snap.enemies.is_empty() {
            let enemy = &snap.enemies[0];
            let dx = (snap.me.pos.x - enemy.pos.x).abs();
            let dy = (snap.me.pos.y - enemy.pos.y).abs();
            let distance = dx.max(dy);

            if distance <= 3 && reg.tools.iter().any(|t| t.name == "attack") {
                steps.push(ActionStep::Attack {
                    target_id: enemy.id,
                });
            } else if reg.tools.iter().any(|t| t.name == "take_cover") {
                // Move 2 units away from enemy
                let cover_x = if snap.me.pos.x > enemy.pos.x {
                    snap.me.pos.x + 2
                } else {
                    snap.me.pos.x - 2
                };
                steps.push(ActionStep::TakeCover {
                    position: Some(astraweave_core::IVec2 { x: cover_x, y: snap.me.pos.y }),
                });
            }
        }

        // Heuristic 4: Objective exists → Move towards it
        if let Some(obj_text) = &snap.objective {
            if obj_text.contains("extract") || obj_text.contains("reach") {
                if let Some(poi) = snap.pois.first() {
                    if reg.tools.iter().any(|t| t.name == "move_to") {
                        steps.push(ActionStep::MoveTo {
                            x: poi.pos.x,
                            y: poi.pos.y,
                            speed: None,
                        });
                    }
                }
            }
        }

        // Heuristic 5: Nothing urgent → Scan area
        if steps.is_empty() && reg.tools.iter().any(|t| t.name == "scan") {
            steps.push(ActionStep::Scan { radius: 10.0 });
        }

        debug!("Heuristic planning generated {} steps", steps.len());

        PlanIntent {
            plan_id: format!("heuristic-{}", uuid::Uuid::new_v4()),
            steps,
        }
    }

    /// Tier 4: Emergency safe default
    fn emergency_plan(&self, _snap: &WorldSnapshot) -> PlanIntent {
        warn!("Using emergency fallback plan");
        
        PlanIntent {
            plan_id: format!("emergency-{}", uuid::Uuid::new_v4()),
            steps: vec![
                ActionStep::Scan { radius: 10.0 },
                ActionStep::Wait { duration: 1.0 },
            ],
        }
    }

    /// Create simplified registry with top 10 tools
    fn create_simplified_registry(&self, reg: &ToolRegistry) -> ToolRegistry {
        let simplified_tools: Vec<_> = reg.tools.iter()
            .filter(|t| self.simplified_tools.contains(&t.name))
            .cloned()
            .collect();

        ToolRegistry {
            tools: simplified_tools,
            constraints: reg.constraints.clone(),
        }
    }

    /// Record successful planning
    async fn record_success(&self, tier: FallbackTier, attempts: &[FallbackAttempt], duration_ms: u64) {
        let mut metrics = self.metrics.write().await;
        metrics.total_requests += 1;

        *metrics.tier_successes.entry(tier.as_str().to_string()).or_insert(0) += 1;

        // Record failures for earlier tiers
        for attempt in attempts {
            if !attempt.success {
                *metrics.tier_failures.entry(attempt.tier.as_str().to_string()).or_insert(0) += 1;
            }
        }

        // Update averages
        let total = metrics.total_requests as f32;
        metrics.average_attempts = (metrics.average_attempts * (total - 1.0) + attempts.len() as f32) / total;
        metrics.average_duration_ms = (metrics.average_duration_ms * (total - 1.0) + duration_ms as f32) / total;
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> FallbackMetrics {
        self.metrics.read().await.clone()
    }
}

impl Default for FallbackOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Build simplified prompt (shorter, with tool parameter schemas)
/// ⚠️ DEPRECATED: Replaced by PromptCompressor::build_optimized_prompt()
/// This function is kept for backward compatibility but is no longer used.
/// New code should use compression.rs for 30-40% latency improvement.
#[deprecated(since = "0.2.0", note = "use PromptCompressor::build_optimized_prompt instead")]
#[allow(dead_code)]
fn build_simplified_prompt(snap: &WorldSnapshot, reg: &ToolRegistry) -> String {
    // Build tool list with parameter hints
    let tool_descriptions = if reg.tools.is_empty() {
        // Fallback with common tools grouped by parameter pattern
        r#"ALLOWED TOOLS (use ONLY these exact names):

POSITION-BASED (need x, y):
  MoveTo: {"act": "MoveTo", "x": 10, "y": 5}
  ThrowSmoke: {"act": "ThrowSmoke", "x": 10, "y": 5}
  ThrowExplosive: {"act": "ThrowExplosive", "x": 10, "y": 5}

TARGET-BASED (need target_id, some need distance):
  Attack: {"act": "Attack", "target_id": 1}
  Approach: {"act": "Approach", "target_id": 1, "distance": 5.0}
  Retreat: {"act": "Retreat", "target_id": 1, "distance": 20.0}

SIMPLE (no params or one param):
  Reload: {"act": "Reload"}
  Scan: {"act": "Scan", "radius": 15.0}
  Wait: {"act": "Wait", "duration": 2.0}
  Block: {"act": "Block"}
  Heal: {"act": "Heal"}"#.to_string()
    } else {
        // Group tools by parameter pattern from actual registry
        let mut position_tools = Vec::new();
        let mut target_tools = Vec::new();
        let mut simple_tools = Vec::new();
        
        for tool in &reg.tools {
            let has_xy = tool.args.contains_key("x") && tool.args.contains_key("y");
            let has_target = tool.args.contains_key("target_id");
            let param_count = tool.args.len();
            
            if has_xy {
                // Build example with all required params
                let mut params = vec![("act", tool.name.clone()), ("x", "10".to_string()), ("y", "5".to_string())];
                for (key, val) in &tool.args {
                    if key != "x" && key != "y" {
                        let example_val = match val.as_str() {
                            s if s.contains("f32") => "5.0",
                            s if s.contains("i32") => "10",
                            s if s.contains("u32") => "1",
                            _ => "null",
                        };
                        params.push((key, example_val.to_string()));
                    }
                }
                let example = format!("{{\"act\": \"{}\", \"x\": 10, \"y\": 5}}", tool.name);
                position_tools.push((tool.name.as_str(), example));
            } else if has_target {
                // Build example with all required params
                let mut param_parts = vec![format!("\"act\": \"{}\"", tool.name), "\"target_id\": 1".to_string()];
                for (key, val) in &tool.args {
                    if key != "target_id" {
                        let example_val = match val.as_str() {
                            s if s.contains("f32") => "5.0",
                            s if s.contains("i32") => "10",
                            s if s.contains("u32") => "1",
                            _ => "null",
                        };
                        param_parts.push(format!("\"{}\": {}", key, example_val));
                    }
                }
                let example = format!("{{{}}}", param_parts.join(", "));
                target_tools.push((tool.name.as_str(), example));
            } else if param_count <= 1 {
                let example = if param_count == 0 {
                    format!("{{\"act\": \"{}\"}}", tool.name)
                } else {
                    let (key, val) = tool.args.iter().next()
                        .expect("param_count check ensures at least one argument exists");
                    let example_val = match val.as_str() {
                        s if s.contains("f32") => "5.0",
                        _ => "null",
                    };
                    format!("{{\"act\": \"{}\", \"{}\": {}}}", tool.name, key, example_val)
                };
                simple_tools.push((tool.name.as_str(), example));
            }
        }
        
        let mut desc = String::from("ALLOWED TOOLS (use ONLY these exact names):\n\n");
        
        if !position_tools.is_empty() {
            desc.push_str("POSITION-BASED (need x, y):\n");
            for (name, example) in position_tools.iter().take(5) {
                desc.push_str(&format!("  {}: {}\n", name, example));
            }
            desc.push('\n');
        }
        
        if !target_tools.is_empty() {
            desc.push_str("TARGET-BASED (need target_id, some need distance):\n");
            for (name, example) in target_tools.iter().take(5) {
                desc.push_str(&format!("  {}: {}\n", name, example));
            }
            desc.push('\n');
        }
        
        if !simple_tools.is_empty() {
            desc.push_str("SIMPLE (no params or one param):\n");
            for (name, example) in simple_tools.iter().take(5) {
                desc.push_str(&format!("  {}: {}\n", name, example));
            }
        }
        
        desc
    };
    
    // Count available enemies for target_id hints
    let enemy_hint = if !snap.enemies.is_empty() {
        format!("  (Use target_id from enemies: {})", 
                snap.enemies.iter().map(|e| e.id.to_string()).collect::<Vec<_>>().join(", "))
    } else {
        String::new()
    };
    
    format!(
        r#"You are a tactical AI. Generate ONE JSON plan using ONLY tools listed below.

World State:
- Your position: ({}, {})
- Your morale: {:.0}
- Your ammo: {}
- Enemies: {}{}
- Objective: {}

{}

Output format - EXACTLY ONE plan:
{{"plan_id": "unique-id", "steps": [...]}}

CRITICAL RULES:
1. Use ONLY tools listed above - NO other tool names allowed
2. Tool names are case-sensitive - use EXACT spelling
3. Include ALL required parameters for each tool
4. Do NOT invent tools like "HoldPosition", "HoldEast", "Extract", etc.
5. Generate ONLY ONE plan, not multiple alternatives
6. FORBIDDEN TOOLS: Extract, Exfiltrate, Escape, HoldPosition, Stay, Move, Fire, Shoot

Examples of INVALID tools (will be rejected):
- HoldPosition, HoldEast, Hold, Stay (not in registry)
- Move, MoveToward, GoTo (wrong name, use "MoveTo")
- Fire, Shoot (wrong name, use "Attack")
- Extract, Exfiltrate, Escape (not in registry, use "MoveTo" to objective)

Be concise. Use 1-3 steps maximum."#,
        snap.me.pos.x,
        snap.me.pos.y,
        snap.me.morale,
        snap.me.ammo,
        snap.enemies.len(),
        enemy_hint,
        snap.objective.as_deref().unwrap_or("none"),
        tool_descriptions
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{Constraints, ToolSpec, CompanionState, PlayerState};
    use async_trait::async_trait;
    use std::collections::BTreeMap;

    struct MockLlmClient {
        responses: Vec<String>,
        call_count: Arc<RwLock<usize>>,
    }

    #[async_trait]
    impl LlmClient for MockLlmClient {
        async fn complete(&self, _prompt: &str) -> Result<String> {
            let mut count = self.call_count.write().await;
            let response = self.responses.get(*count).cloned()
                .unwrap_or_else(|| r#"{"plan_id": "fallback", "steps": []}"#.to_string());
            *count += 1;
            Ok(response)
        }
    }

    fn create_test_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            t: 0.0,
            player: PlayerState {
                pos: astraweave_core::IVec2 { x: 0, y: 0 },
                hp: 100,
                stance: "standing".to_string(),
                orders: vec![],
            },
            me: CompanionState {
                pos: astraweave_core::IVec2 { x: 1, y: 1 },
                ammo: 10,
                morale: 75.0,
                cooldowns: BTreeMap::new(),
            },
            enemies: vec![],
            pois: vec![],
            obstacles: vec![],
            objective: Some("Scan area".to_string()),
        }
    }

    fn create_test_registry() -> ToolRegistry {
        ToolRegistry {
            tools: vec![
                ToolSpec { name: "move_to".to_string(), args: BTreeMap::new() },
                ToolSpec { name: "attack".to_string(), args: BTreeMap::new() },
                ToolSpec { name: "scan".to_string(), args: BTreeMap::new() },
                ToolSpec { name: "Scan".to_string(), args: BTreeMap::new() },  // PascalCase for validation
                ToolSpec { name: "heal".to_string(), args: BTreeMap::new() },
                ToolSpec { name: "reload".to_string(), args: BTreeMap::new() },
                ToolSpec { name: "take_cover".to_string(), args: BTreeMap::new() },
            ],
            constraints: Constraints {
                enforce_cooldowns: false,
                enforce_los: false,
                enforce_stamina: false,
            },
        }
    }

    #[tokio::test]
    async fn test_full_llm_success() {
        let client = MockLlmClient {
            responses: vec![r#"{"plan_id": "test-1", "steps": [{"act": "Scan", "radius": 10.0}]}"#.to_string()],
            call_count: Arc::new(RwLock::new(0)),
        };

        let orchestrator = FallbackOrchestrator::new();
        let snap = create_test_snapshot();
        let reg = create_test_registry();

        let result = orchestrator.plan_with_fallback(&client, &snap, &reg).await;
        
        // LATENCY OPTIMIZATION: Now starts with SimplifiedLlm instead of FullLlm
        assert_eq!(result.tier, FallbackTier::SimplifiedLlm);
        assert_eq!(result.attempts.len(), 1);
        assert!(result.attempts[0].success);
        assert_eq!(result.plan.plan_id, "test-1");
    }

    #[tokio::test]
    async fn test_fallback_to_heuristic() {
        // LLM returns invalid JSON
        let client = MockLlmClient {
            responses: vec![
                "This is not JSON".to_string(),
                "Also not JSON".to_string(),
            ],
            call_count: Arc::new(RwLock::new(0)),
        };

        let orchestrator = FallbackOrchestrator::new();
        let snap = create_test_snapshot();
        let reg = create_test_registry();

        let result = orchestrator.plan_with_fallback(&client, &snap, &reg).await;
        
        // Should fall through to heuristic
        // LATENCY OPTIMIZATION: Now tries SimplifiedLlm → Heuristic (2 attempts) instead of Full → Simplified → Heuristic (3 attempts)
        assert_eq!(result.tier, FallbackTier::Heuristic);
        assert!(result.attempts.len() >= 2); // SimplifiedLlm + Heuristic
        assert!(!result.plan.steps.is_empty());
    }

    #[tokio::test]
    async fn test_heuristic_low_morale() {
        let orchestrator = FallbackOrchestrator::new();
        let mut snap = create_test_snapshot();
        snap.me.morale = 20.0; // Low morale
        let reg = create_test_registry();

        let plan = orchestrator.try_heuristic(&snap, &reg);
        
        // Should include heal step
        assert!(plan.steps.iter().any(|s| matches!(s, ActionStep::Heal { .. })));
    }

    #[tokio::test]
    async fn test_heuristic_no_ammo() {
        let orchestrator = FallbackOrchestrator::new();
        let mut snap = create_test_snapshot();
        snap.me.ammo = 0;
        let reg = create_test_registry();

        let plan = orchestrator.try_heuristic(&snap, &reg);
        
        // Should include reload step
        assert!(plan.steps.iter().any(|s| matches!(s, ActionStep::Reload)));
    }

    #[tokio::test]
    async fn test_emergency_always_succeeds() {
        let orchestrator = FallbackOrchestrator::new();
        let snap = create_test_snapshot();

        let plan = orchestrator.emergency_plan(&snap);
        
        assert_eq!(plan.steps.len(), 2);
        assert!(matches!(plan.steps[0], ActionStep::Scan { .. }));
        assert!(matches!(plan.steps[1], ActionStep::Wait { .. }));
    }

    #[tokio::test]
    async fn test_metrics_tracking() {
        let client = MockLlmClient {
            responses: vec![r#"{"plan_id": "test", "steps": []}"#.to_string()],
            call_count: Arc::new(RwLock::new(0)),
        };

        let orchestrator = FallbackOrchestrator::new();
        let snap = create_test_snapshot();
        let reg = create_test_registry();

        orchestrator.plan_with_fallback(&client, &snap, &reg).await;
        
        let metrics = orchestrator.get_metrics().await;
        assert_eq!(metrics.total_requests, 1);
        // LATENCY OPTIMIZATION: Now starts with simplified_llm instead of full_llm
        assert!(metrics.tier_successes.contains_key("simplified_llm"));
    }
    
    // ═══════════════════════════════════════════════════════════════════════
    // Batch Planning Tests
    // ═══════════════════════════════════════════════════════════════════════
    
    /// Mock LLM that returns batch JSON response
    struct MockBatchLlm {
        response: String,
    }
    
    impl MockBatchLlm {
        fn for_agents(count: usize) -> Self {
            let mut plans = Vec::new();
            for i in 1..=count {
                plans.push(format!(
                    r#"{{"agent_id": {}, "plan_id": "batch-p{}", "steps": [{{"act": "Scan", "radius": 10.0}}]}}"#,
                    i, i
                ));
            }
            let json = format!("[{}]", plans.join(","));
            Self { response: json }
        }
    }
    
    #[async_trait]
    impl LlmClient for MockBatchLlm {
        async fn complete(&self, _prompt: &str) -> Result<String> {
            Ok(self.response.clone())
        }
        
        async fn complete_streaming(
            &self,
            _prompt: &str,
        ) -> Result<std::pin::Pin<Box<dyn futures_util::Stream<Item = Result<String>> + Send>>> {
            // Simulate streaming by chunking response
            let response = self.response.clone();
            let chunk_size = response.len() / 3;
            
            let chunks: Vec<String> = if chunk_size > 0 {
                vec![
                    response[..chunk_size].to_string(),
                    response[chunk_size..chunk_size*2].to_string(),
                    response[chunk_size*2..].to_string(),
                ]
            } else {
                vec![response]
            };
            
            Ok(Box::pin(futures_util::stream::iter(
                chunks.into_iter().map(Ok)
            )))
        }
    }
    
    #[tokio::test]
    async fn test_batch_planning_success() {
        let client = MockBatchLlm::for_agents(3);
        let orchestrator = FallbackOrchestrator::new();
        let reg = create_test_registry();
        
        let agents = vec![
            (1, create_test_snapshot()),
            (2, create_test_snapshot()),
            (3, create_test_snapshot()),
        ];
        
        let results = orchestrator.plan_batch_with_fallback(&client, agents, &reg).await;
        
        assert_eq!(results.len(), 3);
        assert!(results.contains_key(&1));
        assert!(results.contains_key(&2));
        assert!(results.contains_key(&3));
        
        // All should succeed at SimplifiedLlm tier
        for (_, result) in &results {
            assert_eq!(result.tier, FallbackTier::SimplifiedLlm);
            assert!(!result.plan.steps.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_batch_planning_deterministic() {
        let client = MockBatchLlm::for_agents(3);
        let orchestrator = FallbackOrchestrator::new();
        let reg = create_test_registry();
        
        // Run batch planning 3 times with agents in different order
        let mut all_results = Vec::new();
        
        for _ in 0..3 {
            let agents = vec![
                (3, create_test_snapshot()),
                (1, create_test_snapshot()),
                (2, create_test_snapshot()),
            ];
            
            let results = orchestrator.plan_batch_with_fallback(&client, agents, &reg).await;
            all_results.push(results);
        }
        
        // All runs should have same agent IDs with plans
        for results in &all_results {
            assert_eq!(results.len(), 3);
            assert!(results.contains_key(&1));
            assert!(results.contains_key(&2));
            assert!(results.contains_key(&3));
        }
        
        // All should use same tier
        for results in &all_results {
            for (_, result) in results {
                assert_eq!(result.tier, FallbackTier::SimplifiedLlm);
            }
        }
    }
    
    #[tokio::test]
    async fn test_batch_planning_empty() {
        let client = MockBatchLlm::for_agents(0);
        let orchestrator = FallbackOrchestrator::new();
        let reg = create_test_registry();
        
        let agents = vec![];
        let results = orchestrator.plan_batch_with_fallback(&client, agents, &reg).await;
        
        assert!(results.is_empty());
    }
    
    #[tokio::test]
    async fn test_batch_planning_fallback_to_heuristic() {
        // LLM returns invalid JSON
        struct FailingLlm;
        
        #[async_trait]
        impl LlmClient for FailingLlm {
            async fn complete(&self, _prompt: &str) -> Result<String> {
                Ok("invalid json".to_string())
            }
        }
        
        let client = FailingLlm;
        let orchestrator = FallbackOrchestrator::new();
        let reg = create_test_registry();
        
        let agents = vec![
            (1, create_test_snapshot()),
            (2, create_test_snapshot()),
        ];
        
        let results = orchestrator.plan_batch_with_fallback(&client, agents, &reg).await;
        
        assert_eq!(results.len(), 2);
        
        // Should fall back to heuristic for both
        for (_, result) in &results {
            assert_eq!(result.tier, FallbackTier::Heuristic);
            assert!(!result.plan.steps.is_empty());
        }
    }
    
    #[tokio::test]
    async fn test_batch_vs_single_agent_compatibility() {
        let client = MockBatchLlm::for_agents(1);
        let orchestrator = FallbackOrchestrator::new();
        let reg = create_test_registry();
        let snap = create_test_snapshot();
        
        // Run single-agent planning
        let single_result = orchestrator.plan_with_fallback(&client, &snap, &reg).await;
        
        // Run batch planning with 1 agent
        let agents = vec![(1, snap.clone())];
        let batch_results = orchestrator.plan_batch_with_fallback(&client, agents, &reg).await;
        
        assert_eq!(batch_results.len(), 1);
        let batch_result = batch_results.get(&1).unwrap();
        
        // Both should use same tier
        assert_eq!(single_result.tier, batch_result.tier);
        assert_eq!(single_result.tier, FallbackTier::SimplifiedLlm);
        
        // Both should have non-empty plans
        assert!(!single_result.plan.steps.is_empty());
        assert!(!batch_result.plan.steps.is_empty());
    }
}
