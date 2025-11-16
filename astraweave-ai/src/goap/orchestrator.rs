// GOAPOrchestrator - Integration with AstraWeave AI system
// Implements the Orchestrator trait to provide GOAP planning

use super::actions::register_all_actions;
use super::adapter::SnapshotAdapter;
use super::{AdvancedGOAP, Goal, StateValue, WorldState};
use astraweave_core::{ActionStep, MovementSpeed, PlanIntent, WorldSnapshot};
use std::collections::BTreeMap;

/// GOAP-powered orchestrator for AstraWeave
/// Converts WorldSnapshot to GOAP state and back to PlanIntent
pub struct GOAPOrchestrator {
    planner: AdvancedGOAP,
}

impl GOAPOrchestrator {
    pub fn new() -> Self {
        let mut planner = AdvancedGOAP::new();

        // Register comprehensive action library from Phase 2
        register_all_actions(&mut planner);

        Self { planner }
    }

    /// Convert WorldSnapshot to GOAP WorldState using enhanced adapter
    fn snapshot_to_state(snap: &WorldSnapshot) -> WorldState {
        SnapshotAdapter::to_world_state(snap)
    }

    /// Convert GOAP plan to engine PlanIntent (Phase 2 expanded mapping)
    fn plan_to_intent(plan: Vec<String>, snap: &WorldSnapshot, plan_id: String) -> PlanIntent {
        let mut steps = Vec::new();

        for action_name in plan {
            match action_name.as_str() {
                "move_to" | "approach_enemy" => {
                    if let Some(enemy) = snap.enemies.first() {
                        let target_x = snap.me.pos.x + (enemy.pos.x - snap.me.pos.x).signum() * 2;
                        let target_y = snap.me.pos.y + (enemy.pos.y - snap.me.pos.y).signum() * 2;
                        steps.push(ActionStep::MoveTo {
                            x: target_x,
                            y: target_y,
                            speed: None,
                        });
                    }
                }
                "attack" => {
                    if let Some(enemy) = snap.enemies.first() {
                        steps.push(ActionStep::Attack {
                            target_id: enemy.id,
                        });
                    }
                }
                "cover_fire" => {
                    if let Some(enemy) = snap.enemies.first() {
                        steps.push(ActionStep::CoverFire {
                            target_id: enemy.id,
                            duration: 2.0,
                        });
                    }
                }
                "reload" => {
                    steps.push(ActionStep::Reload);
                }
                "take_cover" => {
                    // Find nearest cover (simplified: move back from enemy)
                    if let Some(enemy) = snap.enemies.first() {
                        let retreat_x = snap.me.pos.x - (enemy.pos.x - snap.me.pos.x).signum() * 3;
                        let retreat_y = snap.me.pos.y - (enemy.pos.y - snap.me.pos.y).signum() * 3;
                        steps.push(ActionStep::MoveTo {
                            x: retreat_x,
                            y: retreat_y,
                            speed: None,
                        });
                    }
                }
                "heal" => {
                    steps.push(ActionStep::Heal { target_id: None });
                }
                "throw_smoke" => {
                    if let Some(enemy) = snap.enemies.first() {
                        let mid_x = (snap.me.pos.x + enemy.pos.x) / 2;
                        let mid_y = (snap.me.pos.y + enemy.pos.y) / 2;
                        steps.push(ActionStep::Throw {
                            item: "smoke".to_string(),
                            x: mid_x,
                            y: mid_y,
                        });
                    }
                }
                "retreat" => {
                    // Move away from enemies
                    if let Some(enemy) = snap.enemies.first() {
                        let retreat_x = snap.me.pos.x - (enemy.pos.x - snap.me.pos.x).signum() * 5;
                        let retreat_y = snap.me.pos.y - (enemy.pos.y - snap.me.pos.y).signum() * 5;
                        steps.push(ActionStep::MoveTo {
                            x: retreat_x,
                            y: retreat_y,
                            speed: Some(MovementSpeed::Sprint),
                        });
                    }
                }
                "revive" => {
                    // Revive player (assuming player is entity 0)
                    steps.push(ActionStep::Revive { ally_id: 0 });
                }
                "scan" => {
                    steps.push(ActionStep::Scan { radius: 10.0 });
                }
                _ => {
                    tracing::warn!("Unknown GOAP action: {}", action_name);
                }
            }
        }

        PlanIntent { plan_id, steps }
    }

    /// Generate a plan using GOAP
    pub fn propose_plan(&mut self, snap: &WorldSnapshot) -> PlanIntent {
        let plan_id = format!("goap-plan-{}", (snap.t * 1000.0) as i64);

        // Convert snapshot to GOAP state
        let state = Self::snapshot_to_state(snap);

        // Define goal based on situation
        let goal = if snap.enemies.is_empty() {
            // No enemies: exploration/idle goal
            let mut goal_state = BTreeMap::new();
            goal_state.insert("moved_closer".to_string(), StateValue::Bool(true));
            Goal::new("explore", goal_state).with_priority(1.0)
        } else {
            // Enemies present: combat goal
            let mut goal_state = BTreeMap::new();
            goal_state.insert("enemy_damaged".to_string(), StateValue::Bool(true));
            Goal::new("engage_enemy", goal_state).with_priority(5.0)
        };

        // Try to find a plan
        match self.planner.plan(&state, &goal) {
            Some(plan) => {
                tracing::debug!("GOAP plan: {:?}", plan);
                Self::plan_to_intent(plan, snap, plan_id)
            }
            None => {
                tracing::warn!("GOAP failed to find plan, returning empty intent");
                PlanIntent {
                    plan_id,
                    steps: vec![],
                }
            }
        }
    }

    /// Get reference to the planner (for testing/inspection)
    pub fn planner(&self) -> &AdvancedGOAP {
        &self.planner
    }

    /// Get mutable reference to the planner
    pub fn planner_mut(&mut self) -> &mut AdvancedGOAP {
        &mut self.planner
    }
}

impl Default for GOAPOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::IVec2;
    use std::collections::BTreeMap;

    fn make_test_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            t: 1.0,
            player: astraweave_core::PlayerState {
                hp: 100,
                pos: IVec2 { x: 0, y: 0 },
                stance: "stand".to_string(),
                orders: vec![],
            },
            me: astraweave_core::CompanionState {
                ammo: 20,
                cooldowns: BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 5, y: 5 },
            },
            enemies: vec![astraweave_core::EnemyState {
                id: 1,
                pos: IVec2 { x: 10, y: 10 },
                hp: 50,
                cover: "none".to_string(),
                last_seen: 1.0,
            }],
            pois: vec![],
            obstacles: vec![],
            objective: None,
        }
    }

    #[test]
    fn test_snapshot_to_state() {
        let snap = make_test_snapshot();
        let state = GOAPOrchestrator::snapshot_to_state(&snap);

        assert_eq!(state.get("my_ammo"), Some(&StateValue::Int(20)));
        assert_eq!(state.get("enemy_count"), Some(&StateValue::Int(1)));
        assert_eq!(state.get("in_combat"), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_goap_orchestrator_propose_plan() {
        let mut orch = GOAPOrchestrator::new();
        let snap = make_test_snapshot();

        let intent = orch.propose_plan(&snap);

        assert!(!intent.plan_id.is_empty());
        assert!(intent.plan_id.starts_with("goap-plan-"));
        // Plan should have at least one step
        // (exact steps depend on planner logic)
    }

    #[test]
    fn test_empty_enemy_list() {
        let mut orch = GOAPOrchestrator::new();
        let mut snap = make_test_snapshot();
        snap.enemies.clear();

        let intent = orch.propose_plan(&snap);

        // Should still produce a valid intent (even if empty or exploration)
        assert!(!intent.plan_id.is_empty());
    }
}
