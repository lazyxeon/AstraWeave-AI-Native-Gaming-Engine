use std::collections::{BinaryHeap, HashSet};
use std::cmp::Ordering;
use super::{Action, Goal, WorldState, ActionHistory};

/// A* search node for planning
#[derive(Clone)]
struct PlanNode {
    state: WorldState,
    path: Vec<String>,    // Action names taken to reach this state
    g_cost: f32,          // Actual cost from start
    h_cost: f32,          // Heuristic cost to goal
    risk: f32,            // Accumulated risk
}

impl PlanNode {
    /// Total f-cost for A* (g + h + risk_weight * risk)
    fn f_cost(&self, risk_weight: f32) -> f32 {
        self.g_cost + self.h_cost + (self.risk * risk_weight)
    }
}

impl PartialEq for PlanNode {
    fn eq(&self, other: &Self) -> bool {
        // For priority queue, compare by default risk weight
        self.f_cost(5.0) == other.f_cost(5.0)
    }
}

impl Eq for PlanNode {}

impl PartialOrd for PlanNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PlanNode {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other.f_cost(5.0).partial_cmp(&self.f_cost(5.0)).unwrap_or(Ordering::Equal)
    }
}

/// Advanced GOAP planner with learning, risk-awareness, and multi-goal support
pub struct AdvancedGOAP {
    actions: Vec<Box<dyn Action>>,
    history: ActionHistory,
    max_plan_iterations: usize,
    risk_weight: f32, // How much to penalize risky actions (default 5.0)
}

impl AdvancedGOAP {
    pub fn new() -> Self {
        Self {
            actions: Vec::new(),
            history: ActionHistory::new(),
            max_plan_iterations: 10000,
            risk_weight: 5.0,
        }
    }

    /// Add an action to the planner's action library
    pub fn add_action(&mut self, action: Box<dyn Action>) {
        self.actions.push(action);
    }

    /// Get immutable reference to action history
    pub fn get_history(&self) -> &ActionHistory {
        &self.history
    }

    /// Get mutable reference to action history
    pub fn get_history_mut(&mut self) -> &mut ActionHistory {
        &mut self.history
    }

    /// Get reference to all registered actions
    pub fn get_actions(&self) -> &[Box<dyn Action>] {
        &self.actions
    }

    /// Set maximum planning iterations
    pub fn set_max_iterations(&mut self, max_iterations: usize) {
        self.max_plan_iterations = max_iterations;
    }

    /// Set risk weight (higher = more risk-averse planning)
    pub fn set_risk_weight(&mut self, weight: f32) {
        self.risk_weight = weight;
    }

    /// Hierarchical planning with goal decomposition support
    /// Entry point for planning that handles both simple and hierarchical goals
    pub fn plan(&self, start: &WorldState, goal: &Goal) -> Option<Vec<String>> {
        self.plan_hierarchical(start, goal, 0)
    }

    /// Internal hierarchical planning with depth tracking
    fn plan_hierarchical(&self, start: &WorldState, goal: &Goal, depth: usize) -> Option<Vec<String>> {
        // Check if goal is already satisfied
        if goal.is_satisfied(start) {
            tracing::debug!("Goal '{}' already satisfied at depth {}", goal.name, depth);
            return Some(Vec::new());
        }

        // Check if we should decompose this goal
        if goal.should_decompose(depth) {
            tracing::debug!("Decomposing goal '{}' at depth {}", goal.name, depth);
            
            // Try hierarchical decomposition first
            if let Some(plan) = self.plan_decomposed(start, goal, depth) {
                return Some(plan);
            }
            
            // If decomposition failed, fall through to direct planning
            tracing::debug!("Decomposition failed for '{}', trying direct planning", goal.name);
        }

        // Direct planning (standard A* search)
        self.plan_direct(start, goal)
    }

    /// Plan using goal decomposition (HTN-style)
    fn plan_decomposed(&self, start: &WorldState, goal: &Goal, depth: usize) -> Option<Vec<String>> {
        let sub_goals = goal.decompose()?;
        
        match goal.decomposition_strategy {
            super::DecompositionStrategy::Sequential => {
                self.plan_sequential(start, &sub_goals, depth + 1)
            }
            super::DecompositionStrategy::Parallel | super::DecompositionStrategy::AllOf => {
                self.plan_parallel(start, &sub_goals, depth + 1)
            }
            super::DecompositionStrategy::AnyOf => {
                self.plan_any_of(start, &sub_goals, depth + 1)
            }
        }
    }

    /// Plan sub-goals sequentially
    fn plan_sequential(&self, start: &WorldState, sub_goals: &[Goal], depth: usize) -> Option<Vec<String>> {
        let mut combined_plan = Vec::new();
        let mut current_state = start.clone();

        for sub_goal in sub_goals {
            // Recursively plan for this sub-goal
            let sub_plan = self.plan_hierarchical(&current_state, sub_goal, depth)?;
            
            // Simulate state changes from sub-plan
            for action_name in &sub_plan {
                if let Some(action) = self.actions.iter().find(|a| a.name() == action_name) {
                    current_state.apply_effects(action.effects());
                }
            }
            
            combined_plan.extend(sub_plan);
        }

        Some(combined_plan)
    }

    /// Plan sub-goals in parallel/all-of (try to optimize order)
    fn plan_parallel(&self, start: &WorldState, sub_goals: &[Goal], depth: usize) -> Option<Vec<String>> {
        // For now, treat parallel as sequential with priority ordering
        // TODO: In future, could interleave actions for true parallel execution
        let mut sorted_sub_goals = sub_goals.to_vec();
        sorted_sub_goals.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        
        self.plan_sequential(start, &sorted_sub_goals, depth)
    }

    /// Plan for any-of sub-goals (first successful plan wins)
    fn plan_any_of(&self, start: &WorldState, sub_goals: &[Goal], depth: usize) -> Option<Vec<String>> {
        // Try each sub-goal in priority order until one succeeds
        let mut sorted_sub_goals = sub_goals.to_vec();
        sorted_sub_goals.sort_by(|a, b| b.priority.partial_cmp(&a.priority).unwrap_or(std::cmp::Ordering::Equal));
        
        for sub_goal in sorted_sub_goals {
            if let Some(plan) = self.plan_hierarchical(start, &sub_goal, depth) {
                return Some(plan);
            }
        }
        
        None // No sub-goal could be achieved
    }

    /// Direct A* planning (non-hierarchical)
    fn plan_direct(&self, start: &WorldState, goal: &Goal) -> Option<Vec<String>> {
        let mut open_set = BinaryHeap::new();
        let mut closed_set = HashSet::new();

        let start_node = PlanNode {
            state: start.clone(),
            path: Vec::new(),
            g_cost: 0.0,
            h_cost: start.distance_to(&goal.desired_state),
            risk: 0.0,
        };

        open_set.push(start_node);

        let mut iterations = 0;

        while let Some(current) = open_set.pop() {
            iterations += 1;
            if iterations > self.max_plan_iterations {
                tracing::warn!(
                    "Max iterations ({}) reached in planning for goal '{}'",
                    self.max_plan_iterations,
                    goal.name
                );
                return None;
            }

            // Goal check
            if goal.is_satisfied(&current.state) {
                tracing::debug!(
                    "Plan found for '{}' with {} steps, cost {:.2}, risk {:.2}",
                    goal.name,
                    current.path.len(),
                    current.g_cost,
                    current.risk
                );
                return Some(current.path);
            }

            // State signature for closed set (using hash)
            // Since WorldState implements Hash properly, we can use it directly
            if closed_set.contains(&current.state) {
                continue;
            }
            closed_set.insert(current.state.clone());

            // Expand neighbors (successor states)
            for action in &self.actions {
                if !action.can_execute(&current.state) {
                    continue;
                }

                let mut new_state = current.state.clone();
                new_state.apply_effects(action.effects());

                let action_cost = action.calculate_cost(&current.state, &self.history);
                let action_risk = 1.0 - action.success_probability(&current.state, &self.history);

                let mut new_path = current.path.clone();
                new_path.push(action.name().to_string());

                let new_node = PlanNode {
                    state: new_state.clone(),
                    path: new_path,
                    g_cost: current.g_cost + action_cost,
                    h_cost: new_state.distance_to(&goal.desired_state),
                    risk: current.risk + action_risk,
                };

                open_set.push(new_node);
            }
        }

        tracing::debug!("No plan found for goal '{}' after {} iterations", goal.name, iterations);
        None // No plan found
    }

    /// Multi-goal planning with priority scheduling
    /// Returns plans for each goal in priority order
    pub fn plan_multiple_goals(
        &self,
        start: &WorldState,
        goals: &[Goal],
        current_time: f32,
    ) -> Vec<(String, Vec<String>)> {
        let mut plans = Vec::new();
        let mut sorted_goals = goals.to_vec();

        // Sort by urgency (considers priority + deadline)
        sorted_goals.sort_by(|a, b| {
            b.urgency(current_time)
                .partial_cmp(&a.urgency(current_time))
                .unwrap_or(Ordering::Equal)
        });

        let mut current_state = start.clone();

        for goal in sorted_goals {
            if let Some(plan) = self.plan(&current_state, &goal) {
                // Simulate state changes from this plan
                for action_name in &plan {
                    if let Some(action) = self.actions.iter().find(|a| a.name() == action_name) {
                        current_state.apply_effects(action.effects());
                    }
                }
                plans.push((goal.name.clone(), plan));
            } else {
                tracing::warn!("Failed to find plan for goal '{}'", goal.name);
            }
        }

        plans
    }

    /// Execute a plan and record results in history
    /// This is for simulation/testing; real execution happens via engine
    pub fn simulate_plan_execution(
        &mut self,
        plan: &[String],
        world: &mut WorldState,
    ) -> Result<(), String> {
        for action_name in plan {
            let action = self
                .actions
                .iter()
                .find(|a| a.name() == action_name)
                .ok_or_else(|| format!("Action not found: {}", action_name))?;

            if !action.can_execute(world) {
                self.history.record_failure(action_name);
                return Err(format!("Action '{}' preconditions not met", action_name));
            }

            // Simulate success (in reality, check actual execution)
            let success_prob = action.success_probability(world, &self.history);
            
            // For simulation, assume action succeeds if probability > 0.5
            if success_prob > 0.5 {
                world.apply_effects(action.effects());
                self.history.record_success(action_name, 1.0);
            } else {
                self.history.record_failure(action_name);
                return Err(format!("Action '{}' simulated failure", action_name));
            }
        }

        Ok(())
    }

    /// Get number of registered actions
    pub fn action_count(&self) -> usize {
        self.actions.len()
    }

    /// Get action names
    pub fn action_names(&self) -> Vec<String> {
        self.actions.iter().map(|a| a.name().to_string()).collect()
    }
}

impl Default for AdvancedGOAP {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goap::action::SimpleAction;
    use std::collections::BTreeMap;
    use crate::goap::StateValue;

    #[test]
    fn test_simple_plan() {
        let mut goap = AdvancedGOAP::new();

        // Define action: move (no preconditions) -> at_goal
        let mut move_effects = BTreeMap::new();
        move_effects.insert("at_goal".to_string(), StateValue::Bool(true));
        let move_action = SimpleAction::new("move", BTreeMap::new(), move_effects, 1.0);

        goap.add_action(Box::new(move_action));

        // Start state: not at goal
        let mut start = WorldState::new();
        start.set("at_goal", StateValue::Bool(false));

        // Goal: be at goal
        let mut goal_state = BTreeMap::new();
        goal_state.insert("at_goal".to_string(), StateValue::Bool(true));
        let goal = Goal::new("reach_goal", goal_state);

        let plan = goap.plan(&start, &goal);

        assert!(plan.is_some());
        assert_eq!(plan.unwrap(), vec!["move"]);
    }

    #[test]
    fn test_multi_step_plan() {
        let mut goap = AdvancedGOAP::new();

        // Action 1: find_weapon -> has_weapon
        let mut find_weapon_effects = BTreeMap::new();
        find_weapon_effects.insert("has_weapon".to_string(), StateValue::Bool(true));
        goap.add_action(Box::new(SimpleAction::new(
            "find_weapon",
            BTreeMap::new(),
            find_weapon_effects,
            2.0,
        )));

        // Action 2: equip_weapon (requires has_weapon) -> weapon_equipped
        let mut equip_preconds = BTreeMap::new();
        equip_preconds.insert("has_weapon".to_string(), StateValue::Bool(true));
        let mut equip_effects = BTreeMap::new();
        equip_effects.insert("weapon_equipped".to_string(), StateValue::Bool(true));
        goap.add_action(Box::new(SimpleAction::new(
            "equip_weapon",
            equip_preconds,
            equip_effects,
            1.0,
        )));

        // Action 3: attack (requires weapon_equipped) -> enemy_defeated
        let mut attack_preconds = BTreeMap::new();
        attack_preconds.insert("weapon_equipped".to_string(), StateValue::Bool(true));
        let mut attack_effects = BTreeMap::new();
        attack_effects.insert("enemy_defeated".to_string(), StateValue::Bool(true));
        goap.add_action(Box::new(SimpleAction::new(
            "attack",
            attack_preconds,
            attack_effects,
            5.0,
        )));

        // Start state: nothing
        let start = WorldState::new();

        // Goal: defeat enemy
        let mut goal_state = BTreeMap::new();
        goal_state.insert("enemy_defeated".to_string(), StateValue::Bool(true));
        let goal = Goal::new("defeat_enemy", goal_state);

        let plan = goap.plan(&start, &goal);

        assert!(plan.is_some());
        let plan_steps = plan.unwrap();
        assert_eq!(plan_steps.len(), 3);
        assert_eq!(plan_steps[0], "find_weapon");
        assert_eq!(plan_steps[1], "equip_weapon");
        assert_eq!(plan_steps[2], "attack");
    }

    #[test]
    fn test_no_plan_found() {
        let mut goap = AdvancedGOAP::new();

        // Action that requires something we can't get
        let mut preconds = BTreeMap::new();
        preconds.insert("impossible_condition".to_string(), StateValue::Bool(true));
        let mut effects = BTreeMap::new();
        effects.insert("goal".to_string(), StateValue::Bool(true));
        goap.add_action(Box::new(SimpleAction::new("impossible", preconds, effects, 1.0)));

        let start = WorldState::new();

        let mut goal_state = BTreeMap::new();
        goal_state.insert("goal".to_string(), StateValue::Bool(true));
        let goal = Goal::new("impossible_goal", goal_state);

        let plan = goap.plan(&start, &goal);

        assert!(plan.is_none());
    }

    #[test]
    fn test_multi_goal_planning() {
        let mut goap = AdvancedGOAP::new();

        // Action 1: heal -> health = 100
        let mut heal_effects = BTreeMap::new();
        heal_effects.insert("health".to_string(), StateValue::Int(100));
        goap.add_action(Box::new(SimpleAction::new("heal", BTreeMap::new(), heal_effects, 3.0)));

        // Action 2: reload -> ammo = 30
        let mut reload_effects = BTreeMap::new();
        reload_effects.insert("ammo".to_string(), StateValue::Int(30));
        goap.add_action(Box::new(SimpleAction::new(
            "reload",
            BTreeMap::new(),
            reload_effects,
            2.0,
        )));

        let mut start = WorldState::new();
        start.set("health", StateValue::Int(50));
        start.set("ammo", StateValue::Int(5));

        // Goal 1: High priority heal
        let mut heal_goal_state = BTreeMap::new();
        heal_goal_state.insert("health".to_string(), StateValue::Int(100));
        let heal_goal = Goal::new("stay_alive", heal_goal_state).with_priority(10.0);

        // Goal 2: Lower priority reload
        let mut reload_goal_state = BTreeMap::new();
        reload_goal_state.insert("ammo".to_string(), StateValue::Int(30));
        let reload_goal = Goal::new("reload_weapon", reload_goal_state).with_priority(5.0);

        let plans = goap.plan_multiple_goals(&start, &[heal_goal, reload_goal], 0.0);

        assert_eq!(plans.len(), 2);
        // High priority goal should be first
        assert_eq!(plans[0].0, "stay_alive");
        assert_eq!(plans[0].1, vec!["heal"]);
        assert_eq!(plans[1].0, "reload_weapon");
        assert_eq!(plans[1].1, vec!["reload"]);
    }

    #[test]
    fn test_risk_awareness() {
        let mut goap = AdvancedGOAP::new();

        // Risky action
        let risky = SimpleAction::new("risky_attack", BTreeMap::new(), BTreeMap::new(), 2.0);
        goap.add_action(Box::new(risky));

        // Record failures to build history
        goap.get_history_mut().record_failure("risky_attack");
        goap.get_history_mut().record_failure("risky_attack");
        goap.get_history_mut().record_success("risky_attack", 1.0);

        let stats = goap.get_history().get_action_stats("risky_attack").unwrap();
        assert!((stats.failure_rate() - 0.666666).abs() < 0.01);
    }

    #[test]
    fn test_deterministic_planning() {
        let mut goap = AdvancedGOAP::new();

        let mut effects = BTreeMap::new();
        effects.insert("done".to_string(), StateValue::Bool(true));
        goap.add_action(Box::new(SimpleAction::new("action", BTreeMap::new(), effects, 1.0)));

        let start = WorldState::new();

        let mut goal_state = BTreeMap::new();
        goal_state.insert("done".to_string(), StateValue::Bool(true));
        let goal = Goal::new("goal", goal_state);

        // Multiple planning runs should produce same result
        let plan1 = goap.plan(&start, &goal);
        let plan2 = goap.plan(&start, &goal);
        let plan3 = goap.plan(&start, &goal);

        assert_eq!(plan1, plan2);
        assert_eq!(plan2, plan3);
    }
}

