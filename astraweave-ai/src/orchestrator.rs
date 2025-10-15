//! Orchestrator trait implementations and selection utilities for AstraWeave AI planning.

#[cfg(feature = "profiling")]
use astraweave_profiling::span;

use anyhow::Result;
#[cfg(feature = "llm_orchestrator")]
use astraweave_core::{default_tool_registry, ToolRegistry};
use astraweave_core::{ActionStep, IVec2, PlanIntent, WorldSnapshot};

/// Cooldown key constants for type safety and consistency
const COOLDOWN_THROW_SMOKE: &str = "throw:smoke";

#[async_trait::async_trait]
pub trait OrchestratorAsync {
    async fn plan(&self, snap: WorldSnapshot, budget_ms: u32) -> Result<PlanIntent>;
    fn name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }
}

pub trait Orchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent;
}

/// Minimal rule-based orchestrator:
/// If enemy in LOS-ish and "smoke" not on cooldown:
///   throw smoke midway, move up, cover fire.
/// Else: advance towards nearest enemy.
pub struct RuleOrchestrator;

impl Orchestrator for RuleOrchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent {
        #[cfg(feature = "profiling")]
        span!("AI::RuleOrchestrator::propose_plan");
        
        let plan_id = format!("plan-{}", (snap.t * 1000.0) as i64);
        if let Some(first) = snap.enemies.first() {
            let m = &snap.me;
            let _p = &snap.player;

            // midpoint for smoke
            let mid = IVec2 {
                x: (m.pos.x + first.pos.x) / 2,
                y: (m.pos.y + first.pos.y) / 2,
            };

            let smoke_cd = snap
                .me
                .cooldowns
                .get(COOLDOWN_THROW_SMOKE)
                .copied()
                .unwrap_or(0.0);
            if smoke_cd <= 0.0 {
                return PlanIntent {
                    plan_id,
                    steps: vec![
                        ActionStep::Throw {
                            item: "smoke".into(),
                            x: mid.x,
                            y: mid.y,
                        },
                        ActionStep::MoveTo { speed: None,
                            x: m.pos.x + (first.pos.x - m.pos.x).signum() * 2,
                            y: m.pos.y + (first.pos.y - m.pos.y).signum() * 2,
                        },
                        ActionStep::CoverFire {
                            target_id: first.id,
                            duration: 2.5,
                        },
                    ],
                };
            } else {
                // advance cautiously
                return PlanIntent {
                    plan_id,
                    steps: vec![
                        ActionStep::MoveTo { speed: None,
                            x: m.pos.x + (first.pos.x - m.pos.x).signum(),
                            y: m.pos.y + (first.pos.y - m.pos.y).signum(),
                        },
                        ActionStep::CoverFire {
                            target_id: first.id,
                            duration: 1.5,
                        },
                    ],
                };
            }
        }

        // fallback
        PlanIntent {
            plan_id,
            steps: vec![],
        }
    }
}

#[async_trait::async_trait]
impl OrchestratorAsync for RuleOrchestrator {
    async fn plan(&self, snap: WorldSnapshot, _budget_ms: u32) -> Result<PlanIntent> {
        Ok(self.propose_plan(&snap))
    }
}

/// Utility-based orchestrator: scores a few candidate plans deterministically.
/// Heuristics:
/// - Prefer throwing smoke if an enemy exists and cooldown is ready
/// - Otherwise move closer to nearest enemy; if very close (<3), add brief cover fire
pub struct UtilityOrchestrator;

impl Orchestrator for UtilityOrchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent {
        let plan_id = format!("util-{}", (snap.t * 1000.0) as i64);
        let me = &snap.me;
        // Candidate A: Throw smoke at midpoint to first enemy
        let mut cands: Vec<(f32, Vec<ActionStep>)> = vec![];
        if let Some(enemy) = snap.enemies.first() {
            let cd = me
                .cooldowns
                .get(COOLDOWN_THROW_SMOKE)
                .copied()
                .unwrap_or(0.0);
            if cd <= 0.0 {
                let mid = IVec2 {
                    x: (me.pos.x + enemy.pos.x) / 2,
                    y: (me.pos.y + enemy.pos.y) / 2,
                };
                let steps = vec![
                    ActionStep::Throw {
                        item: "smoke".into(),
                        x: mid.x,
                        y: mid.y,
                    },
                    ActionStep::MoveTo { speed: None,
                        x: me.pos.x + (enemy.pos.x - me.pos.x).signum() * 2,
                        y: me.pos.y + (enemy.pos.y - me.pos.y).signum() * 2,
                    },
                ];
                // Score: encourage if ammo present and enemy hp > 0
                let score = 1.0
                    + (snap.player.hp as f32).min(100.0) * 0.0
                    + (enemy.hp.max(0) as f32) * 0.01;
                cands.push((score, steps));
            }
            // Candidate B: advance; if close, cover fire
            let dx = (enemy.pos.x - me.pos.x).abs();
            let dy = (enemy.pos.y - me.pos.y).abs();
            let dist = (dx + dy) as f32;
            let mut steps = vec![ActionStep::MoveTo { speed: None,
                x: me.pos.x + (enemy.pos.x - me.pos.x).signum(),
                y: me.pos.y + (enemy.pos.y - me.pos.y).signum(),
            }];
            if dist <= 3.0 {
                steps.push(ActionStep::CoverFire {
                    target_id: enemy.id,
                    duration: 1.0,
                });
            }
            let score = 0.8 + (3.0 - dist).max(0.0) * 0.05;
            cands.push((score, steps));
        }
        // Use total_cmp to provide a total ordering for f32 (handles NaN deterministically)
        cands.sort_by(|a, b| b.0.total_cmp(&a.0));
        let steps = cands.first().map(|c| c.1.clone()).unwrap_or_default();
        PlanIntent { plan_id, steps }
    }
}

#[async_trait::async_trait]
impl OrchestratorAsync for UtilityOrchestrator {
    async fn plan(&self, snap: WorldSnapshot, _budget_ms: u32) -> Result<PlanIntent> {
        Ok(self.propose_plan(&snap))
    }
}

/// Minimal GOAP-style orchestrator for MoveTo -> CoverFire chain towards first enemy.
/// Preconditions: enemy exists. Goal: be within 2 cells and apply CoverFire for 1.5s.
pub struct GoapOrchestrator;

impl GoapOrchestrator {
    /// Fast-path action selection without full plan generation.
    /// 
    /// This is optimized for instant action returns (<100 µs target) by:
    /// - Returning single ActionStep directly (not wrapped in PlanIntent)
    /// - Minimal distance calculation (Manhattan distance)
    /// - No allocations (returns stack-allocated ActionStep)
    /// - Simple heuristic: move toward closest enemy or cover fire if in range
    /// 
    /// # Arguments
    /// - `snap`: Current world snapshot
    /// 
    /// # Returns
    /// An `ActionStep` to execute this frame, or `Wait { duration: 1.0 }` if no enemies
    /// 
    /// # Performance
    /// - **Target**: <100 µs per call
    /// - **Typical**: 5-30 µs (distance calc + conditional)
    /// 
    /// # Example
    /// ```no_run
    /// use astraweave_ai::GoapOrchestrator;
    /// 
    /// let goap = GoapOrchestrator;
    /// let action = goap.next_action(&snapshot); // <100 µs
    /// apply_action(action);
    /// ```
    pub fn next_action(&self, snap: &WorldSnapshot) -> ActionStep {
        #[cfg(feature = "profiling")]
        span!("AI::GoapOrchestrator::next_action");
        
        // Fast path: if enemy exists, move toward or engage
        if let Some(enemy) = snap.enemies.first() {
            let me = &snap.me;
            let dx = enemy.pos.x - me.pos.x;
            let dy = enemy.pos.y - me.pos.y;
            let dist = dx.abs() + dy.abs();
            
            if dist <= 2 {
                // In range: cover fire
                ActionStep::CoverFire {
                    target_id: enemy.id,
                    duration: 1.5,
                }
            } else {
                // Out of range: move one step closer
                ActionStep::MoveTo {
                    speed: None,
                    x: me.pos.x + dx.signum(),
                    y: me.pos.y + dy.signum(),
                }
            }
        } else {
            // No enemies: wait
            ActionStep::Wait { duration: 1.0 }
        }
    }
}

impl Orchestrator for GoapOrchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent {
        let plan_id = format!("goap-{}", (snap.t * 1000.0) as i64);
        if let Some(enemy) = snap.enemies.first() {
            let me = &snap.me;
            let dx = (enemy.pos.x - me.pos.x).abs();
            let dy = (enemy.pos.y - me.pos.y).abs();
            let dist = dx + dy;
            if dist <= 2 {
                return PlanIntent {
                    plan_id,
                    steps: vec![ActionStep::CoverFire {
                        target_id: enemy.id,
                        duration: 1.5,
                    }],
                };
            } else {
                // Move one step closer; replan next tick
                return PlanIntent {
                    plan_id,
                    steps: vec![ActionStep::MoveTo { speed: None,
                        x: me.pos.x + (enemy.pos.x - me.pos.x).signum(),
                        y: me.pos.y + (enemy.pos.y - me.pos.y).signum(),
                    }],
                };
            }
        }
        PlanIntent {
            plan_id,
            steps: vec![],
        }
    }
}

#[async_trait::async_trait]
impl OrchestratorAsync for GoapOrchestrator {
    async fn plan(&self, snap: WorldSnapshot, _budget_ms: u32) -> Result<PlanIntent> {
        Ok(self.propose_plan(&snap))
    }
}

#[cfg(feature = "llm_orchestrator")]
pub struct LlmOrchestrator<C> {
    pub client: C,
    pub registry: ToolRegistry,
}

#[cfg(feature = "llm_orchestrator")]
impl<C> LlmOrchestrator<C> {
    pub fn new(client: C, registry: Option<ToolRegistry>) -> Self {
        Self {
            client,
            registry: registry.unwrap_or_else(default_tool_registry),
        }
    }
}

#[cfg(feature = "llm_orchestrator")]
#[async_trait::async_trait]
impl<C> OrchestratorAsync for LlmOrchestrator<C>
where
    C: astraweave_llm::LlmClient + Send + Sync,
{
    async fn plan(&self, snap: WorldSnapshot, budget_ms: u32) -> Result<PlanIntent> {
        // Read timeout from environment or use budget_ms
        let timeout_ms = std::env::var("LLM_TIMEOUT_MS")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(budget_ms.max(50)); // Use budget or minimum 50ms
        
        let timeout_duration = std::time::Duration::from_millis(timeout_ms as u64);
        
        // Enforce hard timeout using tokio::time::timeout
        match tokio::time::timeout(
            timeout_duration,
            astraweave_llm::plan_from_llm(&self.client, &snap, &self.registry)
        ).await {
            Ok(plan_source) => {
                // LLM completed within timeout
                match plan_source {
                    astraweave_llm::PlanSource::Llm(plan) => Ok(plan),
                    astraweave_llm::PlanSource::Fallback { plan, reason } => {
                        tracing::warn!("plan_from_llm fell back: {}", reason);
                        Ok(PlanIntent {
                            plan_id: "llm-fallback".into(),
                            steps: plan.steps,
                        })
                    }
                }
            }
            Err(_elapsed) => {
                // Timeout exceeded - return fallback
                tracing::warn!("LLM planning timed out after {}ms, using fallback", timeout_ms);
                Ok(PlanIntent {
                    plan_id: "timeout-fallback".into(),
                    steps: astraweave_llm::fallback_heuristic_plan(&snap, &self.registry).steps,
                })
            }
        }
    }
    fn name(&self) -> &'static str {
        "LlmOrchestrator"
    }
}

/// System-wide wiring utilities for choosing an orchestrator at runtime.
/// Set ASTRAWEAVE_USE_LLM=1 to select the local LLM (phi3:medium by default) if compiled with the llm_orchestrator feature.
#[derive(Clone, Debug)]
pub struct SystemOrchestratorConfig {
    pub use_llm: bool,
    pub ollama_url: String,
    pub ollama_model: String,
}

impl Default for SystemOrchestratorConfig {
    fn default() -> Self {
        let use_llm = std::env::var("ASTRAWEAVE_USE_LLM")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let ollama_url =
            std::env::var("OLLAMA_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
        // Default model to phi3:medium as requested
        let ollama_model =
            std::env::var("OLLAMA_MODEL").unwrap_or_else(|_| "phi3:medium".to_string());
        Self {
            use_llm,
            ollama_url,
            ollama_model,
        }
    }
}

/// Create a boxed orchestrator chosen by config/env.
/// - If use_llm is true and llm_orchestrator is compiled, constructs LlmOrchestrator with Ollama chat client.
/// - Otherwise falls back to UtilityOrchestrator.
pub fn make_system_orchestrator(
    cfg: Option<SystemOrchestratorConfig>,
) -> Box<dyn OrchestratorAsync + Send + Sync> {
    let _cfg = cfg.unwrap_or_default();
    #[cfg(all(feature = "llm_orchestrator"))]
    {
        if _cfg.use_llm {
            // Build an Ollama-backed orchestrator (ollama feature is enabled via Cargo.toml feature wiring)
            let client = astraweave_llm::OllamaChatClient::new(
                _cfg.ollama_url.clone(),
                _cfg.ollama_model.clone(),
            );
            // Optionally warm up model in the background to minimize first-token latency
            let do_warm = std::env::var("OLLAMA_WARMUP")
                .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
                .unwrap_or(true);
            if do_warm {
                let client_clone = client.clone();
                let warm_secs: u64 = std::env::var("OLLAMA_WARMUP_TIMEOUT_SECS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(30);
                // Prefer spawning on Tokio if available; otherwise use a thread
                #[cfg(feature = "llm_orchestrator")]
                {
                    // We cannot assume tokio is always compiled in this crate; call into a helper function only if tokio exists.
                }
                // Try to use tokio if linked, else fallback
                #[cfg(any())]
                {
                    let _ = (client_clone, warm_secs); // placeholder for conditional compilation
                }
                // Fallback: spawn a thread and perform a blocking warmup with a mini runtime if tokio is available in dependencies
                std::thread::spawn(move || {
                    // If tokio is linked, use a small runtime; else, do nothing (can't warmup without async runtime)
                    #[cfg(feature = "llm_orchestrator")]
                    {
                        // Try building a current-thread runtime; ignore errors silently
                        if let Ok(rt) = tokio::runtime::Builder::new_current_thread()
                            .enable_all()
                            .build()
                        {
                            let _ = rt.block_on(client_clone.warmup(warm_secs));
                        }
                    }
                });
            }
            let orch = crate::LlmOrchestrator::new(client, Some(default_tool_registry()));
            return Box::new(orch);
        }
    }
    // Default safe fallback
    Box::new(UtilityOrchestrator)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "llm_orchestrator")]
    use astraweave_core::default_tool_registry;
    use astraweave_core::{CompanionState, EnemyState, PlayerState, WorldSnapshot};
    #[cfg(feature = "llm_orchestrator")]
    use astraweave_llm::MockLlm;
    use futures::executor::block_on;
    use std::collections::BTreeMap;

    fn snap_basic(px: i32, py: i32, ex: i32, ey: i32, smoke_cd: f32) -> WorldSnapshot {
        WorldSnapshot {
            t: 1.234,
            player: PlayerState {
                hp: 80,
                pos: IVec2 { x: px, y: py },
                stance: "stand".into(),
                orders: vec![],
            },
            me: CompanionState {
                ammo: 10,
                cooldowns: BTreeMap::from([(COOLDOWN_THROW_SMOKE.to_string(), smoke_cd)]),
                morale: 0.5,
                pos: IVec2 { x: px, y: py },
            },
            enemies: vec![EnemyState {
                id: 2,
                pos: IVec2 { x: ex, y: ey },
                hp: 50,
                cover: "low".into(),
                last_seen: 0.0,
            }],
            pois: vec![],
            obstacles: vec![],
            objective: None,
        }
    }

    #[test]
    fn utility_prefers_smoke_when_ready() {
        let s = snap_basic(0, 0, 4, 0, 0.0);
        let o = UtilityOrchestrator;
        let plan = o.propose_plan(&s);
        match plan.steps.first() {
            Some(ActionStep::Throw { .. }) => {}
            _ => panic!("expected Throw first"),
        }
    }

    #[test]
    fn goap_moves_then_covers() {
        let s_far = snap_basic(0, 0, 5, 0, 10.0);
        let goap = GoapOrchestrator;
        let plan1 = goap.propose_plan(&s_far);
        assert!(matches!(
            plan1.steps.first(),
            Some(ActionStep::MoveTo { speed: None, .. })
        ));
        let s_close = snap_basic(0, 0, 1, 0, 10.0);
        let plan2 = goap.propose_plan(&s_close);
        assert!(matches!(
            plan2.steps.first(),
            Some(ActionStep::CoverFire { .. })
        ));
    }

    #[test]
    fn async_trait_adapter_returns_same_plan() {
        let s = snap_basic(0, 0, 3, 0, 0.0);
        let rule = RuleOrchestrator;
        let plan_sync = rule.propose_plan(&s);
        let plan_async = block_on(rule.plan(s, 2)).expect("rule.plan failed");
        assert_eq!(plan_sync.steps.len(), plan_async.steps.len());
    }

    #[cfg(feature = "llm_orchestrator")]
    #[test]
    fn llm_orchestrator_with_mock_produces_plan() {
        let s = snap_basic(0, 0, 6, 2, 0.0);
        let client = MockLlm;
        let orch = crate::LlmOrchestrator::new(client, Some(default_tool_registry()));
        let plan = block_on(orch.plan(s, 10)).expect("llm mock plan failed");
        assert_eq!(plan.plan_id, "llm-mock");
        assert!(!plan.steps.is_empty());
    }

    #[cfg(feature = "llm_orchestrator")]
    #[test]
    fn llm_orchestrator_disallowed_tools_fallbacks_empty() {
        let s = snap_basic(0, 0, 6, 2, 0.0);
        let client = MockLlm;
        // Empty registry to force parse failure
        let mut reg = default_tool_registry();
        reg.tools.clear();
        let orch = crate::LlmOrchestrator::new(client, Some(reg));
        let plan = block_on(orch.plan(s, 10)).expect("llm plan call failed");
        assert_eq!(plan.plan_id, "llm-fallback");
        assert!(plan.steps.is_empty());
    }
}
