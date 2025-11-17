use super::{Action, Goal, WorldState, StateValue};
use std::collections::BTreeMap;

/// State difference between two world states
#[derive(Debug, Clone)]
pub struct StateDiff {
    pub added: BTreeMap<String, StateValue>,
    pub removed: BTreeMap<String, StateValue>,
    pub changed: Vec<StateChange>,
}

/// A single state change
#[derive(Debug, Clone)]
pub struct StateChange {
    pub key: String,
    pub old_value: StateValue,
    pub new_value: StateValue,
}

/// Explanation for why an action was chosen
#[derive(Debug, Clone)]
pub struct Explanation {
    pub action_name: String,
    pub reason: String,
    pub preconditions_met: Vec<String>,
    pub effects_applied: Vec<String>,
    pub cost: f32,
    pub risk: f32,
    pub alternatives_considered: Vec<String>,
}

/// Progress report for a goal
#[derive(Debug, Clone)]
pub struct ProgressReport {
    pub goal_name: String,
    pub progress: f32, // 0.0 - 1.0
    pub satisfied_conditions: Vec<String>,
    pub unsatisfied_conditions: Vec<String>,
    pub estimated_actions_remaining: usize,
}

/// Interactive plan debugger
pub struct PlanDebugger {
    plan: Vec<String>,
    current_step: usize,
    state_history: Vec<WorldState>,
    actions: Vec<Box<dyn Action>>,
}

impl PlanDebugger {
    /// Create a new debugger
    pub fn new(plan: Vec<String>, start_state: WorldState, actions: Vec<Box<dyn Action>>) -> Self {
        Self {
            plan,
            current_step: 0,
            state_history: vec![start_state],
            actions,
        }
    }

    /// Get current step number
    pub fn current_step(&self) -> usize {
        self.current_step
    }

    /// Get total number of steps
    pub fn total_steps(&self) -> usize {
        self.plan.len()
    }

    /// Check if at beginning
    pub fn at_start(&self) -> bool {
        self.current_step == 0
    }

    /// Check if at end
    pub fn at_end(&self) -> bool {
        self.current_step >= self.plan.len()
    }

    /// Get current action name
    pub fn current_action(&self) -> Option<&str> {
        self.plan.get(self.current_step).map(|s| s.as_str())
    }

    /// Get current world state
    pub fn current_state(&self) -> &WorldState {
        self.state_history.last().unwrap()
    }

    /// Step forward one action
    pub fn step_forward(&mut self) -> Result<(), String> {
        if self.at_end() {
            return Err("Already at end of plan".to_string());
        }

        let action_name = &self.plan[self.current_step];
        let action = self.actions.iter()
            .find(|a| a.name() == action_name)
            .ok_or_else(|| format!("Action '{}' not found", action_name))?;

        let current_state = self.current_state();
        
        if !action.can_execute(current_state) {
            return Err(format!("Action '{}' preconditions not met", action_name));
        }

        // Apply effects
        let mut new_state = current_state.clone();
        new_state.apply_effects(action.effects());
        
        self.state_history.push(new_state);
        self.current_step += 1;

        Ok(())
    }

    /// Step backward one action
    pub fn step_backward(&mut self) -> Result<(), String> {
        if self.at_start() {
            return Err("Already at start of plan".to_string());
        }

        self.state_history.pop();
        self.current_step -= 1;

        Ok(())
    }

    /// Reset to beginning
    pub fn reset(&mut self) {
        let start_state = self.state_history[0].clone();
        self.state_history = vec![start_state];
        self.current_step = 0;
    }

    /// Get state difference from previous step
    pub fn get_state_diff(&self) -> Option<StateDiff> {
        if self.current_step == 0 {
            return None;
        }

        let prev_state = &self.state_history[self.current_step - 1];
        let curr_state = &self.state_history[self.current_step];

        Some(Self::diff_states(prev_state, curr_state))
    }

    /// Calculate difference between two states
    fn diff_states(old_state: &WorldState, new_state: &WorldState) -> StateDiff {
        let mut added = BTreeMap::new();
        let mut removed = BTreeMap::new();
        let mut changed = Vec::new();

        // Find added and changed
        for (key, new_value) in new_state.iter() {
            if let Some(old_value) = old_state.get(key) {
                if old_value != new_value {
                    changed.push(StateChange {
                        key: key.clone(),
                        old_value: old_value.clone(),
                        new_value: new_value.clone(),
                    });
                }
            } else {
                added.insert(key.clone(), new_value.clone());
            }
        }

        // Find removed
        for (key, old_value) in old_state.iter() {
            if !new_state.contains_key(key) {
                removed.insert(key.clone(), old_value.clone());
            }
        }

        StateDiff {
            added,
            removed,
            changed,
        }
    }

    /// Explain why current action was chosen
    pub fn explain_action(&self, action_name: &str) -> Option<Explanation> {
        let action = self.actions.iter()
            .find(|a| a.name() == action_name)?;

        let state = self.current_state();

        let mut preconditions_met = Vec::new();
        for (key, value) in action.preconditions() {
            if let Some(current_value) = state.get(&key) {
                if current_value.satisfies(&value) {
                    preconditions_met.push(format!("{}={:?}", key, current_value));
                }
            }
        }

        let mut effects_applied = Vec::new();
        for (key, value) in action.effects() {
            effects_applied.push(format!("{}={:?}", key, value));
        }

        // Find alternatives that were not chosen
        let mut alternatives_considered = Vec::new();
        for other_action in &self.actions {
            if other_action.name() != action_name && other_action.can_execute(state) {
                alternatives_considered.push(other_action.name().to_string());
            }
        }

        Some(Explanation {
            action_name: action_name.to_string(),
            reason: format!("Action satisfies {} preconditions and applies {} effects", 
                           preconditions_met.len(), effects_applied.len()),
            preconditions_met,
            effects_applied,
            cost: action.base_cost(),
            risk: 0.0, // Would need history for accurate risk
            alternatives_considered,
        })
    }

    /// Check progress toward a goal
    pub fn check_goal_progress(&self, goal: &Goal) -> ProgressReport {
        let current_state = self.current_state();
        let progress = goal.progress(current_state);

        let mut satisfied = Vec::new();
        let mut unsatisfied = Vec::new();

        for (key, target_value) in &goal.desired_state {
            if let Some(current_value) = current_state.get(key) {
                if current_value.satisfies(target_value) {
                    satisfied.push(format!("{}={:?}", key, current_value));
                } else {
                    unsatisfied.push(format!("{}={:?} (need {:?})", key, current_value, target_value));
                }
            } else {
                unsatisfied.push(format!("{} (missing)", key));
            }
        }

        let remaining_steps = self.plan.len() - self.current_step;

        ProgressReport {
            goal_name: goal.name.clone(),
            progress,
            satisfied_conditions: satisfied,
            unsatisfied_conditions: unsatisfied,
            estimated_actions_remaining: remaining_steps,
        }
    }

    /// Get all actions in plan
    pub fn get_plan(&self) -> &[String] {
        &self.plan
    }

    /// Get action at specific step
    pub fn get_action_at_step(&self, step: usize) -> Option<&str> {
        self.plan.get(step).map(|s| s.as_str())
    }

    /// Jump to specific step
    pub fn jump_to_step(&mut self, step: usize) -> Result<(), String> {
        if step > self.plan.len() {
            return Err(format!("Step {} is beyond plan length {}", step, self.plan.len()));
        }

        // Reset to start
        self.reset();

        // Step forward to target
        while self.current_step < step {
            self.step_forward()?;
        }

        Ok(())
    }

    /// Get human-readable current state
    pub fn format_current_state(&self) -> String {
        let state = self.current_state();
        let mut output = String::new();

        output.push_str(&format!("=== World State at Step {} ===\n", self.current_step));

        let mut keys: Vec<_> = state.iter().map(|(k, _)| k).collect();
        keys.sort();

        for key in keys {
            if let Some(value) = state.get(key) {
                output.push_str(&format!("  {} = {:?}\n", key, value));
            }
        }

        output
    }

    /// Get human-readable diff
    pub fn format_state_diff(&self) -> String {
        let Some(diff) = self.get_state_diff() else {
            return "No previous state to compare".to_string();
        };

        let mut output = String::new();

        output.push_str("=== State Changes ===\n");

        if !diff.added.is_empty() {
            output.push_str("\nAdded:\n");
            for (key, value) in &diff.added {
                output.push_str(&format!("  + {} = {:?}\n", key, value));
            }
        }

        if !diff.removed.is_empty() {
            output.push_str("\nRemoved:\n");
            for (key, value) in &diff.removed {
                output.push_str(&format!("  - {} = {:?}\n", key, value));
            }
        }

        if !diff.changed.is_empty() {
            output.push_str("\nChanged:\n");
            for change in &diff.changed {
                output.push_str(&format!("  ~ {} = {:?} -> {:?}\n", 
                    change.key, change.old_value, change.new_value));
            }
        }

        if diff.added.is_empty() && diff.removed.is_empty() && diff.changed.is_empty() {
            output.push_str("  (no changes)\n");
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goap::SimpleAction;
    use std::collections::BTreeMap;

    fn create_test_action(name: &str, effects: Vec<(&str, StateValue)>) -> Box<dyn Action> {
        let mut effects_map = BTreeMap::new();
        for (key, value) in effects {
            effects_map.insert(key.to_string(), value);
        }
        Box::new(SimpleAction::new(name, BTreeMap::new(), effects_map, 1.0))
    }

    #[test]
    fn test_debugger_creation() {
        let plan = vec!["action1".to_string(), "action2".to_string()];
        let start = WorldState::new();
        let actions = vec![create_test_action("action1", vec![])];

        let debugger = PlanDebugger::new(plan, start, actions);

        assert_eq!(debugger.current_step(), 0);
        assert_eq!(debugger.total_steps(), 2);
        assert!(debugger.at_start());
        assert!(!debugger.at_end());
    }

    #[test]
    fn test_step_forward() {
        let plan = vec!["action1".to_string()];
        let start = WorldState::new();
        let actions = vec![
            create_test_action("action1", vec![("flag", StateValue::Bool(true))])
        ];

        let mut debugger = PlanDebugger::new(plan, start, actions);

        assert!(debugger.step_forward().is_ok());
        assert_eq!(debugger.current_step(), 1);
        assert!(debugger.at_end());

        // Check that state was updated
        let state = debugger.current_state();
        assert_eq!(state.get("flag"), Some(&StateValue::Bool(true)));
    }

    #[test]
    fn test_step_backward() {
        let plan = vec!["action1".to_string()];
        let start = WorldState::new();
        let actions = vec![
            create_test_action("action1", vec![("flag", StateValue::Bool(true))])
        ];

        let mut debugger = PlanDebugger::new(plan, start, actions);

        debugger.step_forward().unwrap();
        assert_eq!(debugger.current_step(), 1);

        debugger.step_backward().unwrap();
        assert_eq!(debugger.current_step(), 0);

        // Should be back to original state
        let state = debugger.current_state();
        assert_eq!(state.get("flag"), None);
    }

    #[test]
    fn test_state_diff() {
        let plan = vec!["action1".to_string()];
        let mut start = WorldState::new();
        start.set("existing", StateValue::Int(10));

        let actions = vec![
            create_test_action("action1", vec![
                ("existing", StateValue::Int(20)),
                ("new", StateValue::Bool(true)),
            ])
        ];

        let mut debugger = PlanDebugger::new(plan, start, actions);
        debugger.step_forward().unwrap();

        let diff = debugger.get_state_diff().unwrap();

        assert_eq!(diff.added.len(), 1);
        assert!(diff.added.contains_key("new"));

        assert_eq!(diff.changed.len(), 1);
        assert_eq!(diff.changed[0].key, "existing");
    }

    #[test]
    fn test_reset() {
        let plan = vec!["action1".to_string()];
        let start = WorldState::new();
        let actions = vec![
            create_test_action("action1", vec![("flag", StateValue::Bool(true))])
        ];

        let mut debugger = PlanDebugger::new(plan, start, actions);

        debugger.step_forward().unwrap();
        assert_eq!(debugger.current_step(), 1);

        debugger.reset();
        assert_eq!(debugger.current_step(), 0);
        assert!(debugger.current_state().get("flag").is_none());
    }

    #[test]
    fn test_jump_to_step() {
        let plan = vec!["action1".to_string(), "action2".to_string(), "action3".to_string()];
        let start = WorldState::new();
        let actions = vec![
            create_test_action("action1", vec![("step", StateValue::Int(1))]),
            create_test_action("action2", vec![("step", StateValue::Int(2))]),
            create_test_action("action3", vec![("step", StateValue::Int(3))]),
        ];

        let mut debugger = PlanDebugger::new(plan, start, actions);

        debugger.jump_to_step(2).unwrap();
        assert_eq!(debugger.current_step(), 2);
        assert_eq!(debugger.current_state().get("step"), Some(&StateValue::Int(2)));
    }

    #[test]
    fn test_goal_progress() {
        let plan = vec!["action1".to_string()];
        let start = WorldState::new();
        let actions = vec![
            create_test_action("action1", vec![("goal_met", StateValue::Bool(true))])
        ];

        let mut debugger = PlanDebugger::new(plan, start, actions);

        let mut desired = BTreeMap::new();
        desired.insert("goal_met".to_string(), StateValue::Bool(true));
        let goal = Goal::new("test_goal", desired);

        // Before action
        let progress1 = debugger.check_goal_progress(&goal);
        assert_eq!(progress1.progress, 0.0);
        assert_eq!(progress1.unsatisfied_conditions.len(), 1);

        // After action
        debugger.step_forward().unwrap();
        let progress2 = debugger.check_goal_progress(&goal);
        assert_eq!(progress2.progress, 1.0);
        assert_eq!(progress2.satisfied_conditions.len(), 1);
    }
}

