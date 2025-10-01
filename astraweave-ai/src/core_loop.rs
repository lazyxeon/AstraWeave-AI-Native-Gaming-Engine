//! Core loop wiring for AI planning dispatch.
//!
//! This module provides:
//! - `PlannerMode`: Rule, BehaviorTree, GOAP enum
//! - `CAiController`: Component to attach to entities for AI planning
//! - `dispatch_planner()`: Dispatcher that routes WorldSnapshot to appropriate planner

use anyhow::Result;
use astraweave_core::{PlanIntent, WorldSnapshot};

#[cfg(feature = "ai-goap")]
use astraweave_behavior::goap::{Action, GoapPlanner, WorldState};

#[cfg(feature = "ai-goap")]
use astraweave_core::ActionStep;

use crate::orchestrator::{Orchestrator, RuleOrchestrator};

/// Planner mode selection for AI entities.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlannerMode {
    /// Use rule-based orchestrator (smoke + advance logic).
    Rule,
    /// Use behavior tree planner (requires `ai-bt` feature).
    BehaviorTree,
    /// Use GOAP planner (requires `ai-goap` feature).
    GOAP,
}

/// Component for AI entities to control which planner to use.
///
/// # Example
/// ```
/// use astraweave_ai::core_loop::{CAiController, PlannerMode};
///
/// let controller = CAiController {
///     mode: PlannerMode::GOAP,
///     policy: Some("gather_craft_policy".to_string()),
/// };
/// ```
#[derive(Clone, Debug)]
pub struct CAiController {
    /// Which planner to use for this entity.
    pub mode: PlannerMode,
    /// Optional policy identifier for BT/GOAP configuration.
    /// Can be used to load specific behavior trees or GOAP action sets.
    pub policy: Option<String>,
}

impl Default for CAiController {
    fn default() -> Self {
        Self {
            mode: PlannerMode::Rule,
            policy: None,
        }
    }
}

/// Dispatch planning to the appropriate planner based on controller mode.
///
/// # Arguments
/// - `controller`: The AI controller specifying which planner to use
/// - `snapshot`: World state snapshot for the entity
///
/// # Returns
/// - `Ok(PlanIntent)` with the planned action sequence
/// - `Err` if planning fails
///
/// # Example
/// ```
/// use astraweave_ai::core_loop::{CAiController, PlannerMode, dispatch_planner};
/// use astraweave_core::{WorldSnapshot, PlayerState, CompanionState, IVec2};
/// use std::collections::BTreeMap;
///
/// let controller = CAiController {
///     mode: PlannerMode::Rule,
///     policy: None,
/// };
///
/// let snapshot = WorldSnapshot {
///     t: 0.0,
///     me: CompanionState {
///         ammo: 10,
///         cooldowns: BTreeMap::new(),
///         morale: 1.0,
///         pos: IVec2 { x: 5, y: 5 },
///     },
///     player: PlayerState {
///         hp: 100,
///         pos: IVec2 { x: 5, y: 5 },
///         stance: "stand".into(),
///         orders: vec![],
///     },
///     enemies: vec![],
///     pois: vec![],
///     obstacles: vec![],
///     objective: None,
/// };
///
/// let plan = dispatch_planner(&controller, &snapshot).unwrap();
/// assert!(!plan.steps.is_empty());
/// ```
pub fn dispatch_planner(
    controller: &CAiController,
    snapshot: &WorldSnapshot,
) -> Result<PlanIntent> {
    match controller.mode {
        PlannerMode::Rule => {
            let orch = RuleOrchestrator;
            Ok(orch.propose_plan(snapshot))
        }
        PlannerMode::BehaviorTree => {
            #[cfg(feature = "ai-bt")]
            {
                dispatch_bt(controller, snapshot)
            }
            #[cfg(not(feature = "ai-bt"))]
            {
                anyhow::bail!("BehaviorTree mode requires 'ai-bt' feature")
            }
        }
        PlannerMode::GOAP => {
            #[cfg(feature = "ai-goap")]
            {
                dispatch_goap(controller, snapshot)
            }
            #[cfg(not(feature = "ai-goap"))]
            {
                anyhow::bail!("GOAP mode requires 'ai-goap' feature")
            }
        }
    }
}

/// Dispatch to behavior tree planner (feature-gated).
#[cfg(feature = "ai-bt")]
fn dispatch_bt(_controller: &CAiController, _snapshot: &WorldSnapshot) -> Result<PlanIntent> {
    // TODO: Implement BT integration
    // 1. Set up Blackboard from WorldSnapshot
    // 2. Tick behavior tree
    // 3. Convert BT outputs → ActionStep sequence
    anyhow::bail!("BehaviorTree integration not yet implemented")
}

/// Dispatch to GOAP planner (feature-gated).
#[cfg(feature = "ai-goap")]
fn dispatch_goap(controller: &CAiController, snapshot: &WorldSnapshot) -> Result<PlanIntent> {
    // Convert WorldSnapshot to GOAP WorldState
    let world_state = snapshot_to_goap_state(snapshot)?;

    // Define goal based on policy or default
    let goal = if let Some(policy_name) = &controller.policy {
        // TODO: Load goal from policy configuration
        // For now, use a simple goal based on policy name
        if policy_name.contains("gather") || policy_name.contains("craft") {
            WorldState {
                has_wood: 5,
                has_food: 1,
                ..WorldState::default()
            }
        } else {
            WorldState {
                has_food: 1,
                ..WorldState::default()
            }
        }
    } else {
        // Default goal: get some food
        WorldState {
            has_food: 1,
            ..WorldState::default()
        }
    };

    // Define available actions
    let actions = create_goap_actions();

    // Run GOAP planner
    let planner = GoapPlanner::new(actions, 100);
    let plan = planner
        .plan(&world_state, &goal)
        .ok_or_else(|| anyhow::anyhow!("GOAP planning failed - no plan found"))?;

    // Convert GOAP action sequence to ActionSteps
    let steps = goap_actions_to_steps(&plan)?;

    Ok(PlanIntent {
        plan_id: format!("goap_{}", plan.len()),
        steps,
    })
}

/// Convert WorldSnapshot to GOAP WorldState.
#[cfg(feature = "ai-goap")]
fn snapshot_to_goap_state(snapshot: &WorldSnapshot) -> Result<WorldState> {
    // Build initial world state from snapshot
    // For now, use simple mapping based on companion state
    let state = WorldState {
        has_wood: 0,                     // Would come from inventory in full system
        has_food: 0,                     // Would come from inventory in full system
        at_tree: false,                  // Would come from nearby resource check
        at_campfire: false,              // Would come from nearby structure check
        hungry: snapshot.player.hp < 50, // Use player HP as hunger proxy
    };

    Ok(state)
}

/// Create standard GOAP action set for gathering and crafting.
#[cfg(feature = "ai-goap")]
fn create_goap_actions() -> Vec<Action> {
    vec![
        Action {
            name: "GoToTree".into(),
            cost: 5,
            preconditions: WorldState::default(),
            effects: WorldState {
                at_tree: true,
                ..WorldState::default()
            },
        },
        Action {
            name: "ChopWood".into(),
            cost: 10,
            preconditions: WorldState {
                at_tree: true,
                ..WorldState::default()
            },
            effects: WorldState {
                has_wood: 3,
                at_tree: false,
                ..WorldState::default()
            },
        },
        Action {
            name: "GoToCampfire".into(),
            cost: 5,
            preconditions: WorldState {
                has_wood: 1,
                ..WorldState::default()
            },
            effects: WorldState {
                at_campfire: true,
                ..WorldState::default()
            },
        },
        Action {
            name: "CookFood".into(),
            cost: 8,
            preconditions: WorldState {
                has_wood: 1,
                at_campfire: true,
                ..WorldState::default()
            },
            effects: WorldState {
                has_food: 1,
                has_wood: 0, // Consume wood
                at_campfire: false,
                ..WorldState::default()
            },
        },
    ]
}

/// Convert GOAP action sequence to ActionSteps.
#[cfg(feature = "ai-goap")]
fn goap_actions_to_steps(actions: &[Action]) -> Result<Vec<ActionStep>> {
    let mut steps = Vec::new();

    for action in actions {
        let step = match action.name.as_str() {
            "GoToTree" => ActionStep::MoveTo { x: 10, y: 10 }, // TODO: Use actual tree positions
            "ChopWood" => ActionStep::MoveTo { x: 10, y: 10 }, // Stay at tree for now
            "GoToCampfire" => ActionStep::MoveTo { x: 5, y: 5 }, // TODO: Use actual campfire position
            "CookFood" => ActionStep::MoveTo { x: 5, y: 5 },     // Stay at campfire for now
            _ => {
                anyhow::bail!("Unknown GOAP action: {}", action.name)
            }
        };
        steps.push(step);
    }

    Ok(steps)
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{CompanionState, EnemyState, IVec2, PlayerState};
    use std::collections::BTreeMap;

    fn make_test_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            t: 0.0,
            me: CompanionState {
                ammo: 10,
                cooldowns: BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 5, y: 5 },
            },
            player: PlayerState {
                hp: 100,
                pos: IVec2 { x: 5, y: 5 },
                stance: "stand".into(),
                orders: vec![],
            },
            enemies: vec![EnemyState {
                id: 1,
                pos: IVec2 { x: 10, y: 10 },
                hp: 100,
                cover: "none".into(),
                last_seen: 0.0,
            }],
            pois: vec![],
            obstacles: vec![],
            objective: None,
        }
    }

    #[test]
    fn test_controller_default() {
        let controller = CAiController::default();
        assert_eq!(controller.mode, PlannerMode::Rule);
        assert_eq!(controller.policy, None);
    }

    #[test]
    fn test_dispatch_rule_mode() {
        let controller = CAiController {
            mode: PlannerMode::Rule,
            policy: None,
        };

        let snapshot = make_test_snapshot();
        let result = dispatch_planner(&controller, &snapshot);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(!plan.steps.is_empty());
    }

    #[test]
    fn test_dispatch_bt_mode_without_feature() {
        #[cfg(not(feature = "ai-bt"))]
        {
            let controller = CAiController {
                mode: PlannerMode::BehaviorTree,
                policy: None,
            };

            let snapshot = make_test_snapshot();
            let result = dispatch_planner(&controller, &snapshot);
            assert!(result.is_err());
            assert!(result.unwrap_err().to_string().contains("ai-bt' feature"));
        }
    }

    #[test]
    #[cfg(feature = "ai-goap")]
    fn test_dispatch_goap_mode() {
        let controller = CAiController {
            mode: PlannerMode::GOAP,
            policy: Some("gather_craft_policy".to_string()),
        };

        let mut snapshot = make_test_snapshot();
        // Make companion "hungry" by using low ammo as proxy since we can't set HP
        snapshot.me.ammo = 0;

        let result = dispatch_planner(&controller, &snapshot);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert!(!plan.steps.is_empty());
        assert!(plan.plan_id.contains("goap"));
    }

    #[test]
    #[cfg(feature = "ai-goap")]
    fn test_snapshot_to_goap_state() {
        let snapshot = make_test_snapshot();
        let state = snapshot_to_goap_state(&snapshot).unwrap();
        assert_eq!(state.has_wood, 0);
        assert_eq!(state.has_food, 0);
        assert_eq!(state.hungry, false); // Player HP = 100
    }

    #[test]
    #[cfg(feature = "ai-goap")]
    fn test_create_goap_actions() {
        let actions = create_goap_actions();
        assert_eq!(actions.len(), 4);
        assert_eq!(actions[0].name, "GoToTree");
        assert_eq!(actions[1].name, "ChopWood");
        assert_eq!(actions[2].name, "GoToCampfire");
        assert_eq!(actions[3].name, "CookFood");
    }
}
