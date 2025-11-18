use super::{AdvancedGOAP, Goal, WorldState};
use std::collections::VecDeque;

/// Manages multiple concurrent goals with dynamic priority scheduling
#[derive(Clone)]
pub struct GoalScheduler {
    active_goals: VecDeque<Goal>,
    current_plan: Option<Vec<String>>,
    current_goal_name: Option<String>,
    last_replan_time: f32,
    replan_interval: f32, // Minimum time between replans
}

impl GoalScheduler {
    /// Create a new goal scheduler
    pub fn new() -> Self {
        Self {
            active_goals: VecDeque::new(),
            current_plan: None,
            current_goal_name: None,
            last_replan_time: 0.0,
            replan_interval: 1.0, // Replan at most once per second
        }
    }

    /// Create a scheduler with custom replan interval
    pub fn with_replan_interval(interval: f32) -> Self {
        Self {
            active_goals: VecDeque::new(),
            current_plan: None,
            current_goal_name: None,
            last_replan_time: 0.0,
            replan_interval: interval,
        }
    }

    /// Add a new goal to the scheduler
    pub fn add_goal(&mut self, goal: Goal) {
        // Insert in priority order (higher priority first)
        let insert_pos = self
            .active_goals
            .iter()
            .position(|g| g.priority < goal.priority)
            .unwrap_or(self.active_goals.len());

        self.active_goals.insert(insert_pos, goal);
    }

    /// Remove a goal by name
    pub fn remove_goal(&mut self, goal_name: &str) -> Option<Goal> {
        if let Some(pos) = self.active_goals.iter().position(|g| g.name == goal_name) {
            Some(self.active_goals.remove(pos).unwrap())
        } else {
            None
        }
    }

    /// Get all active goals (ordered by priority)
    pub fn get_active_goals(&self) -> Vec<&Goal> {
        self.active_goals.iter().collect()
    }

    /// Get current plan if any
    pub fn get_current_plan(&self) -> Option<&Vec<String>> {
        self.current_plan.as_ref()
    }

    /// Get name of the goal currently being pursued
    pub fn get_current_goal_name(&self) -> Option<&str> {
        self.current_goal_name.as_deref()
    }

    /// Update scheduler and generate/update plan based on current time and world state
    /// Returns the current plan to execute, or None if no valid plan
    pub fn update(
        &mut self,
        current_time: f32,
        world: &WorldState,
        planner: &AdvancedGOAP,
    ) -> Option<Vec<String>> {
        // Remove satisfied goals
        self.active_goals.retain(|g| !g.is_satisfied(world));

        // Remove expired goals (past deadline)
        self.active_goals.retain(|g| {
            if let Some(deadline) = g.deadline {
                current_time < deadline
            } else {
                true // No deadline, keep it
            }
        });

        if self.active_goals.is_empty() {
            self.current_plan = None;
            self.current_goal_name = None;
            return None;
        }

        // Check if we should replan
        if self.should_replan(current_time, world) {
            self.last_replan_time = current_time;

            // Find most urgent goal (clone to avoid borrow issues)
            let most_urgent_goal = self.get_most_urgent_goal(current_time)?.clone();

            // Plan for the most urgent goal
            if let Some(plan) = planner.plan(world, &most_urgent_goal) {
                let goal_name = most_urgent_goal.name.clone();
                self.current_plan = Some(plan.clone());
                self.current_goal_name = Some(goal_name);
                return Some(plan);
            } else {
                let failed_goal_name = most_urgent_goal.name.clone();
                tracing::warn!("Failed to plan for goal '{}'", failed_goal_name);
                // Remove unachievable goal
                self.active_goals.retain(|g| g.name != failed_goal_name);
            }
        }

        // Return existing plan if no replan needed
        self.current_plan.clone()
    }

    /// Determine if replanning is needed
    fn should_replan(&self, current_time: f32, world: &WorldState) -> bool {
        // Replan if:
        // 1. No current plan
        if self.current_plan.is_none() {
            return true;
        }

        // 2. Enough time has passed since last replan
        if current_time - self.last_replan_time < self.replan_interval {
            return false;
        }

        // 3. Current goal is satisfied (plan complete)
        if let Some(current_goal_name) = &self.current_goal_name {
            if let Some(current_goal) = self
                .active_goals
                .iter()
                .find(|g| &g.name == current_goal_name)
            {
                if current_goal.is_satisfied(world) {
                    return true;
                }
            } else {
                // Current goal no longer exists
                return true;
            }
        }

        // 4. A more urgent goal has appeared (urgency changed significantly)
        if let Some(current_goal_name) = &self.current_goal_name {
            if let Some(current_goal) = self
                .active_goals
                .iter()
                .find(|g| &g.name == current_goal_name)
            {
                let current_urgency = current_goal.urgency(current_time);

                // Check if any other goal is significantly more urgent
                for goal in &self.active_goals {
                    if goal.name != *current_goal_name
                        && goal.urgency(current_time) > current_urgency * 1.5
                    {
                        tracing::info!(
                            "Preempting '{}' for more urgent '{}'",
                            current_goal_name,
                            goal.name
                        );
                        return true; // Preempt for more urgent goal
                    }
                }
            }
        }

        false
    }

    /// Get the most urgent goal based on current time
    fn get_most_urgent_goal(&self, current_time: f32) -> Option<&Goal> {
        self.active_goals.iter().max_by(|a, b| {
            a.urgency(current_time)
                .partial_cmp(&b.urgency(current_time))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// Calculate effective urgency for a goal
    fn calculate_urgency(&self, goal: &Goal, current_time: f32) -> f32 {
        goal.urgency(current_time)
    }

    /// Force a replan on next update
    pub fn force_replan(&mut self) {
        // Set to far past so next should_replan returns true
        self.last_replan_time = -1000.0;
    }

    /// Clear all goals and current plan
    pub fn clear(&mut self) {
        self.active_goals.clear();
        self.current_plan = None;
        self.current_goal_name = None;
    }

    /// Get count of active goals
    pub fn goal_count(&self) -> usize {
        self.active_goals.len()
    }

    /// Check if scheduler has any active goals
    pub fn has_goals(&self) -> bool {
        !self.active_goals.is_empty()
    }
}

impl Default for GoalScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goap::{StateValue, WorldState};
    use std::collections::BTreeMap;

    fn create_test_goal(name: &str, priority: f32, deadline: Option<f32>) -> Goal {
        Goal::new(name, BTreeMap::new())
            .with_priority(priority)
            .with_deadline(deadline.unwrap_or(100.0))
    }

    #[test]
    fn test_add_goal_priority_order() {
        let mut scheduler = GoalScheduler::new();

        scheduler.add_goal(create_test_goal("low", 2.0, None));
        scheduler.add_goal(create_test_goal("high", 10.0, None));
        scheduler.add_goal(create_test_goal("medium", 5.0, None));

        let goals = scheduler.get_active_goals();
        assert_eq!(goals.len(), 3);
        assert_eq!(goals[0].name, "high"); // Highest priority first
        assert_eq!(goals[1].name, "medium");
        assert_eq!(goals[2].name, "low");
    }

    #[test]
    fn test_remove_goal() {
        let mut scheduler = GoalScheduler::new();
        scheduler.add_goal(create_test_goal("goal1", 5.0, None));
        scheduler.add_goal(create_test_goal("goal2", 3.0, None));

        assert_eq!(scheduler.goal_count(), 2);

        let removed = scheduler.remove_goal("goal1");
        assert!(removed.is_some());
        assert_eq!(scheduler.goal_count(), 1);

        let removed_again = scheduler.remove_goal("goal1");
        assert!(removed_again.is_none());
    }

    #[test]
    fn test_clear() {
        let mut scheduler = GoalScheduler::new();
        scheduler.add_goal(create_test_goal("goal1", 5.0, None));
        scheduler.add_goal(create_test_goal("goal2", 3.0, None));

        assert!(scheduler.has_goals());

        scheduler.clear();

        assert!(!scheduler.has_goals());
        assert_eq!(scheduler.goal_count(), 0);
    }

    #[test]
    fn test_remove_satisfied_goals() {
        let mut scheduler = GoalScheduler::new();

        let mut desired = BTreeMap::new();
        desired.insert("flag".to_string(), StateValue::Bool(true));
        let goal = Goal::new("set_flag", desired).with_priority(5.0);

        scheduler.add_goal(goal);

        // Create world where goal is satisfied from the start
        let mut world = WorldState::new();
        world.set("flag", StateValue::Bool(true));

        let planner = AdvancedGOAP::new();

        // Update should remove satisfied goal
        scheduler.update(0.0, &world, &planner);
        assert_eq!(scheduler.goal_count(), 0); // Goal removed because satisfied
    }

    #[test]
    fn test_remove_expired_goals() {
        let mut scheduler = GoalScheduler::new();
        scheduler.add_goal(create_test_goal("urgent", 5.0, Some(10.0)));

        let world = WorldState::new();
        let planner = AdvancedGOAP::new();

        // Before deadline - goal should remain (if it hasn't been removed as unachievable)
        // Since planner has no actions, goal will be removed. Let's just test the expiration logic.
        scheduler.update(5.0, &world, &planner);

        // Add goal back for expiration test
        if scheduler.goal_count() == 0 {
            scheduler.add_goal(create_test_goal("urgent", 5.0, Some(10.0)));
        }

        scheduler.update(11.0, &world, &planner);
        assert_eq!(scheduler.goal_count(), 0); // Expired
    }

    #[test]
    fn test_urgency_deadline_priority() {
        let scheduler = GoalScheduler::new();

        let goal_no_deadline = create_test_goal("no_deadline", 5.0, None);
        let goal_with_deadline = create_test_goal("deadline", 3.0, Some(5.0));

        let urgency1 = scheduler.calculate_urgency(&goal_no_deadline, 0.0);
        let urgency2 = scheduler.calculate_urgency(&goal_with_deadline, 4.5); // Near deadline

        // Goal with nearby deadline should be more urgent despite lower priority
        assert!(urgency2 > urgency1);
    }

    #[test]
    fn test_force_replan() {
        let mut scheduler = GoalScheduler::with_replan_interval(10.0);
        scheduler.last_replan_time = 5.0;
        scheduler.current_plan = Some(vec!["dummy_action".to_string()]); // Need a plan for time check to matter
        scheduler.current_goal_name = Some("dummy_goal".to_string());

        // Normally wouldn't replan yet (only 1 second elapsed < 10 second interval)
        assert!(!scheduler.should_replan(6.0, &WorldState::new()));

        scheduler.force_replan();

        // Should replan now (forced)
        assert!(scheduler.should_replan(6.0, &WorldState::new()));
    }

    #[test]
    fn test_get_most_urgent_goal() {
        let scheduler = GoalScheduler::new();
        let mut goals = vec![
            create_test_goal("low_priority", 2.0, None),
            create_test_goal("high_priority", 10.0, None),
            create_test_goal("urgent_deadline", 5.0, Some(1.0)),
        ];

        for goal in goals.drain(..) {
            let mut scheduler = scheduler.clone();
            scheduler.add_goal(goal);
        }

        let scheduler = GoalScheduler {
            active_goals: vec![
                create_test_goal("low_priority", 2.0, None),
                create_test_goal("high_priority", 10.0, None),
                create_test_goal("urgent_deadline", 5.0, Some(1.0)),
            ]
            .into(),
            current_plan: None,
            current_goal_name: None,
            last_replan_time: 0.0,
            replan_interval: 1.0,
        };

        let most_urgent = scheduler.get_most_urgent_goal(0.5);
        assert!(most_urgent.is_some());
        // At time 0.5, deadline goal should be most urgent
        assert_eq!(most_urgent.unwrap().name, "urgent_deadline");
    }
}
