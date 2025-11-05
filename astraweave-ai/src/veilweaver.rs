#![allow(clippy::too_many_arguments)]

use std::time::Instant;

use astraweave_behavior::goap::{GoapAction, GoapGoal, GoapPlanner, WorldState};
use astraweave_core::{ActionStep, MovementSpeed, PlanIntent, WorldSnapshot};
use astraweave_observability::{log_companion_action, CompanionActionEvent};
use tracing::info;

use crate::{Orchestrator, OrchestratorAsync};

const ACTION_STABILITY_PULSE: &str = "stability_pulse";
const ACTION_HEAL_PLAYER: &str = "heal_player";
const ACTION_EXECUTE_COMBO: &str = "execute_combo";
const ACTION_MARK_TARGET: &str = "mark_target";
const ACTION_REPOSITION: &str = "reposition";

/// Hybrid GOAP planner tailored to Veilweaver companion behaviors.
pub struct VeilweaverCompanionOrchestrator {
    planner: GoapPlanner,
    actions: Vec<GoapAction>,
    goals: Vec<GoapGoal>,
}

impl VeilweaverCompanionOrchestrator {
    pub fn new() -> Self {
        Self {
            planner: GoapPlanner::new().with_max_iterations(64),
            actions: build_actions(),
            goals: build_goals(),
        }
    }

    fn derive_world_state<'a>(&self, snap: &'a WorldSnapshot) -> (WorldState, HeuristicState<'a>) {
        let heuristics = HeuristicState::from_snapshot(snap);
        let mut state = WorldState::new();
        state.set("player_low", heuristics.player_low);
        state.set("anchor_unstable", heuristics.anchor_unstable);
        state.set("has_echo", heuristics.has_echo);
        state.set("enemy_vulnerable", heuristics.enemy_vulnerable);
        state.set("enemy_tagged", heuristics.enemy_tagged);
        state.set("position_good", heuristics.position_good);
        (state, heuristics)
    }

    fn plan_internal(&self, snap: &WorldSnapshot) -> PlanIntent {
        let start = Instant::now();
        let (world_state, heuristics) = self.derive_world_state(snap);

        let mut selected_goal = None;
        let mut planned_actions = Vec::new();

        let mut goals = self.goals.clone();
        goals.sort_by(|a, b| {
            b.priority
                .partial_cmp(&a.priority)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        for goal in goals {
            if goal.is_satisfied(&world_state) {
                continue;
            }
            if let Some(actions) = self.planner.plan(&world_state, &goal, &self.actions) {
                selected_goal = Some(goal.name.clone());
                planned_actions = actions;
                break;
            }
        }

        let elapsed_ms = start.elapsed().as_secs_f32() * 1000.0;
        let plan_id = format!("vw-companion-{}", (snap.t * 1000.0) as i64);

        if planned_actions.is_empty() {
            info!(
                target: "veilweaver.companion",
                event = "CompanionPlan",
                %plan_id,
                goal = "idle",
                actions = 0,
                elapsed_ms
            );
            return PlanIntent {
                plan_id,
                steps: vec![ActionStep::Wait { duration: 0.5 }],
            };
        }

        let mut steps = Vec::new();
        for action in planned_actions.iter() {
            let action_steps = map_action_to_steps(action.name.as_str(), &heuristics);
            info!(
                target: "veilweaver.companion",
                event = "CompanionAction",
                %plan_id,
                goal = selected_goal.as_deref().unwrap_or("unknown"),
                action = %action.name,
                step_count = action_steps.len(),
                elapsed_ms
            );
            log_companion_action(&CompanionActionEvent {
                action_id: action.name.clone(),
                success: true,
                latency_ms: elapsed_ms,
            });
            steps.extend(action_steps);
        }

        PlanIntent { plan_id, steps }
    }
}

impl Default for VeilweaverCompanionOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl OrchestratorAsync for VeilweaverCompanionOrchestrator {
    async fn plan(&self, snap: WorldSnapshot, _budget_ms: u32) -> anyhow::Result<PlanIntent> {
        Ok(self.plan_internal(&snap))
    }
}

impl Orchestrator for VeilweaverCompanionOrchestrator {
    fn propose_plan(&self, snap: &WorldSnapshot) -> PlanIntent {
        self.plan_internal(snap)
    }
}

fn build_actions() -> Vec<GoapAction> {
    vec![
        GoapAction::new(ACTION_STABILITY_PULSE)
            .with_precondition("anchor_unstable", true)
            .with_precondition("has_echo", true)
            .with_effect("anchor_unstable", false)
            .with_effect("has_echo", false)
            .with_cost(1.0),
        GoapAction::new(ACTION_HEAL_PLAYER)
            .with_precondition("player_low", true)
            .with_effect("player_low", false)
            .with_cost(1.2),
        GoapAction::new(ACTION_EXECUTE_COMBO)
            .with_precondition("enemy_vulnerable", true)
            .with_effect("enemy_vulnerable", false)
            .with_cost(1.1),
        GoapAction::new(ACTION_MARK_TARGET)
            .with_precondition("enemy_tagged", false)
            .with_effect("enemy_tagged", true)
            .with_cost(0.8),
        GoapAction::new(ACTION_REPOSITION)
            .with_precondition("position_good", false)
            .with_effect("position_good", true)
            .with_cost(0.6),
    ]
}

fn build_goals() -> Vec<GoapGoal> {
    vec![
        GoapGoal::new(
            "protect_player",
            WorldState::from_facts(&[("player_low", false)]),
        )
        .with_priority(3.0),
        GoapGoal::new(
            "stabilize_threads",
            WorldState::from_facts(&[("anchor_unstable", false)]),
        )
        .with_priority(2.8),
        GoapGoal::new(
            "exploit_stagger",
            WorldState::from_facts(&[("enemy_vulnerable", false)]),
        )
        .with_priority(2.2),
        GoapGoal::new(
            "maintain_positioning",
            WorldState::from_facts(&[("position_good", true)]),
        )
        .with_priority(1.5),
    ]
}

fn map_action_to_steps(action: &str, heuristics: &HeuristicState<'_>) -> Vec<ActionStep> {
    match action {
        ACTION_STABILITY_PULSE => vec![ActionStep::UseDefensiveAbility {
            ability_name: "stability_pulse".into(),
        }],
        ACTION_HEAL_PLAYER => vec![ActionStep::Heal { target_id: None }],
        ACTION_EXECUTE_COMBO => heuristics
            .closest_enemy
            .map(|enemy| {
                vec![
                    ActionStep::CoordinateAttack {
                        target_id: enemy.id,
                    },
                    ActionStep::QuickAttack {
                        target_id: enemy.id,
                    },
                ]
            })
            .unwrap_or_else(|| vec![ActionStep::Wait { duration: 0.3 }]),
        ACTION_MARK_TARGET => heuristics
            .closest_enemy
            .map(|enemy| {
                vec![ActionStep::MarkTarget {
                    target_id: enemy.id,
                }]
            })
            .unwrap_or_default(),
        ACTION_REPOSITION => {
            let target = heuristics.player_pos_average();
            vec![ActionStep::MoveTo {
                x: target.x,
                y: target.y,
                speed: Some(MovementSpeed::Run),
            }]
        }
        _ => vec![ActionStep::Wait { duration: 0.2 }],
    }
}

#[derive(Clone)]
struct HeuristicState<'a> {
    player_low: bool,
    anchor_unstable: bool,
    enemy_vulnerable: bool,
    position_good: bool,
    has_echo: bool,
    enemy_tagged: bool,
    closest_enemy: Option<&'a astraweave_core::EnemyState>,
    player_pos: astraweave_core::IVec2,
    companion_pos: astraweave_core::IVec2,
}

impl<'a> HeuristicState<'a> {
    fn from_snapshot(snap: &'a WorldSnapshot) -> Self {
        let player_low = snap.player.hp <= 40;
        let anchor_unstable = snap
            .objective
            .as_ref()
            .map(|o| o.contains("anchor_unstable") || o.contains("stabilize_anchor"))
            .unwrap_or(false);

        let has_echo = snap.me.ammo > 0;

        let closest_enemy = snap.enemies.iter().min_by_key(|enemy| {
            let dx = enemy.pos.x - snap.me.pos.x;
            let dy = enemy.pos.y - snap.me.pos.y;
            dx.abs() + dy.abs()
        });

        let enemy_vulnerable = closest_enemy.map(|enemy| enemy.hp <= 30).unwrap_or(false);

        let dx = snap.player.pos.x - snap.me.pos.x;
        let dy = snap.player.pos.y - snap.me.pos.y;
        let position_good = (dx.abs() + dy.abs()) <= 6;

        Self {
            player_low,
            anchor_unstable,
            enemy_vulnerable,
            position_good,
            has_echo,
            enemy_tagged: false,
            closest_enemy,
            player_pos: snap.player.pos,
            companion_pos: snap.me.pos,
        }
    }

    fn player_pos_average(&self) -> astraweave_core::IVec2 {
        astraweave_core::IVec2 {
            x: (self.player_pos.x + self.companion_pos.x) / 2,
            y: (self.player_pos.y + self.companion_pos.y) / 2,
        }
    }
}
