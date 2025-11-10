use super::{Action, Goal, WorldState, ActionHistory};
use std::fmt::Write;

/// Visualization format for plans and goals
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisualizationFormat {
    /// ASCII tree with Unicode box drawing characters
    AsciiTree,
    /// ASCII timeline showing execution sequence
    AsciiTimeline,
    /// DOT format for GraphViz
    Dot,
    /// Simple text list
    Text,
    /// JSON for programmatic consumption
    Json,
}

/// Plan visualizer for rendering plans and goal hierarchies
pub struct PlanVisualizer {
    format: VisualizationFormat,
    show_costs: bool,
    show_risks: bool,
    show_state_changes: bool,
}

impl PlanVisualizer {
    /// Create a new visualizer with specified format
    pub fn new(format: VisualizationFormat) -> Self {
        Self {
            format,
            show_costs: true,
            show_risks: true,
            show_state_changes: false,
        }
    }

    /// Enable/disable cost display
    pub fn with_costs(mut self, show: bool) -> Self {
        self.show_costs = show;
        self
    }

    /// Enable/disable risk display
    pub fn with_risks(mut self, show: bool) -> Self {
        self.show_risks = show;
        self
    }

    /// Enable/disable state change display
    pub fn with_state_changes(mut self, show: bool) -> Self {
        self.show_state_changes = show;
        self
    }

    /// Visualize an action plan
    pub fn visualize_plan(
        &self,
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
        start_state: &WorldState,
    ) -> String {
        match self.format {
            VisualizationFormat::AsciiTree => self.render_plan_tree(plan, actions, history),
            VisualizationFormat::AsciiTimeline => self.render_plan_timeline(plan, actions, history, start_state),
            VisualizationFormat::Dot => self.render_plan_dot(plan, actions, history),
            VisualizationFormat::Text => self.render_plan_text(plan, actions, history),
            VisualizationFormat::Json => self.render_plan_json(plan, actions, history),
        }
    }

    /// Visualize a goal hierarchy
    pub fn visualize_goal_hierarchy(&self, goal: &Goal) -> String {
        match self.format {
            VisualizationFormat::AsciiTree => self.render_goal_tree(goal, 0, true),
            VisualizationFormat::Dot => self.render_goal_dot(goal),
            VisualizationFormat::Text => self.render_goal_text(goal, 0),
            _ => self.render_goal_tree(goal, 0, true),
        }
    }

    /// Render plan as ASCII tree
    fn render_plan_tree(
        &self,
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
    ) -> String {
        let mut output = String::new();

        // Calculate total metrics
        let (total_cost, total_risk) = self.calculate_plan_metrics(plan, actions, history);
        
        writeln!(
            &mut output,
            "Plan ({} actions, cost: {:.1}, risk: {:.2})",
            plan.len(),
            total_cost,
            total_risk
        ).unwrap();

        for (i, action_name) in plan.iter().enumerate() {
            let is_last = i == plan.len() - 1;
            let prefix = if is_last { "└─" } else { "├─" };

            if let Some(action) = actions.iter().find(|a| a.name() == action_name) {
                let cost = action.calculate_cost(&WorldState::new(), history);
                let risk = 1.0 - action.success_probability(&WorldState::new(), history);

                if self.show_costs && self.show_risks {
                    writeln!(
                        &mut output,
                        "{} {} (cost: {:.1}, risk: {:.2})",
                        prefix, action_name, cost, risk
                    ).unwrap();
                } else if self.show_costs {
                    writeln!(
                        &mut output,
                        "{} {} (cost: {:.1})",
                        prefix, action_name, cost
                    ).unwrap();
                } else if self.show_risks {
                    writeln!(
                        &mut output,
                        "{} {} (risk: {:.2})",
                        prefix, action_name, risk
                    ).unwrap();
                } else {
                    writeln!(&mut output, "{} {}", prefix, action_name).unwrap();
                }
            } else {
                writeln!(&mut output, "{} {} (unknown)", prefix, action_name).unwrap();
            }
        }

        output
    }

    /// Render plan as timeline
    fn render_plan_timeline(
        &self,
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
        start_state: &WorldState,
    ) -> String {
        let mut output = String::new();
        let mut current_time = 0.0;
        let mut current_state = start_state.clone();

        // Header
        writeln!(&mut output, "{:<6} | {:<20} | {:<30} | {}", "Time", "Action", "State Changes", "Success").unwrap();
        writeln!(&mut output, "{}", "-".repeat(80)).unwrap();

        for action_name in plan {
            if let Some(action) = actions.iter().find(|a| a.name() == action_name) {
                let success_prob = action.success_probability(&current_state, history);
                let duration = history.get_action_stats(action_name)
                    .map(|s| s.avg_duration)
                    .unwrap_or(1.0);

                let state_changes = if self.show_state_changes {
                    self.format_state_changes(action.effects())
                } else {
                    String::from("...")
                };

                let success_icon = if success_prob > 0.8 {
                    "✓"
                } else if success_prob > 0.5 {
                    "~"
                } else {
                    "✗"
                };

                writeln!(
                    &mut output,
                    "{:<6.1} | {:<20} | {:<30} | {}",
                    current_time, action_name, state_changes, success_icon
                ).unwrap();

                current_state.apply_effects(action.effects());
                current_time += duration;
            }
        }

        output
    }

    /// Render plan as DOT (GraphViz)
    fn render_plan_dot(
        &self,
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
    ) -> String {
        let mut output = String::new();

        writeln!(&mut output, "digraph Plan {{").unwrap();
        writeln!(&mut output, "  rankdir=LR;").unwrap();
        writeln!(&mut output, "  node [shape=box];").unwrap();

        writeln!(&mut output, "  start [label=\"Start\", shape=circle];").unwrap();

        for (i, action_name) in plan.iter().enumerate() {
            let node_id = format!("action_{}", i);
            
            if let Some(action) = actions.iter().find(|a| a.name() == action_name) {
                let cost = action.calculate_cost(&WorldState::new(), history);
                let risk = 1.0 - action.success_probability(&WorldState::new(), history);
                
                let label = if self.show_costs && self.show_risks {
                    format!("{}\\ncost: {:.1}\\nrisk: {:.2}", action_name, cost, risk)
                } else {
                    action_name.clone()
                };

                writeln!(
                    &mut output,
                    "  {} [label=\"{}\"];",
                    node_id, label
                ).unwrap();
            } else {
                writeln!(
                    &mut output,
                    "  {} [label=\"{}\"];",
                    node_id, action_name
                ).unwrap();
            }

            if i == 0 {
                writeln!(&mut output, "  start -> {};", node_id).unwrap();
            } else {
                writeln!(&mut output, "  action_{} -> {};", i - 1, node_id).unwrap();
            }
        }

        if !plan.is_empty() {
            writeln!(&mut output, "  action_{} -> end;", plan.len() - 1).unwrap();
        }

        writeln!(&mut output, "  end [label=\"End\", shape=doublecircle];").unwrap();
        writeln!(&mut output, "}}").unwrap();

        output
    }

    /// Render plan as simple text
    fn render_plan_text(
        &self,
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
    ) -> String {
        let mut output = String::new();

        for (i, action_name) in plan.iter().enumerate() {
            write!(&mut output, "{}. {}", i + 1, action_name).unwrap();

            if let Some(action) = actions.iter().find(|a| a.name() == action_name) {
                if self.show_costs || self.show_risks {
                    let cost = action.calculate_cost(&WorldState::new(), history);
                    let risk = 1.0 - action.success_probability(&WorldState::new(), history);
                    
                    write!(&mut output, " (").unwrap();
                    if self.show_costs {
                        write!(&mut output, "cost: {:.1}", cost).unwrap();
                    }
                    if self.show_costs && self.show_risks {
                        write!(&mut output, ", ").unwrap();
                    }
                    if self.show_risks {
                        write!(&mut output, "risk: {:.2}", risk).unwrap();
                    }
                    write!(&mut output, ")").unwrap();
                }
            }

            writeln!(&mut output).unwrap();
        }

        output
    }

    /// Render plan as JSON
    fn render_plan_json(
        &self,
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
    ) -> String {
        let mut output = String::new();

        writeln!(&mut output, "{{").unwrap();
        writeln!(&mut output, "  \"actions\": [").unwrap();

        for (i, action_name) in plan.iter().enumerate() {
            let comma = if i < plan.len() - 1 { "," } else { "" };
            
            if let Some(action) = actions.iter().find(|a| a.name() == action_name) {
                let cost = action.calculate_cost(&WorldState::new(), history);
                let risk = 1.0 - action.success_probability(&WorldState::new(), history);
                
                writeln!(&mut output, "    {{").unwrap();
                writeln!(&mut output, "      \"name\": \"{}\",", action_name).unwrap();
                writeln!(&mut output, "      \"cost\": {:.2},", cost).unwrap();
                writeln!(&mut output, "      \"risk\": {:.2}", risk).unwrap();
                writeln!(&mut output, "    }}{}", comma).unwrap();
            } else {
                writeln!(&mut output, "    {{").unwrap();
                writeln!(&mut output, "      \"name\": \"{}\"", action_name).unwrap();
                writeln!(&mut output, "    }}{}", comma).unwrap();
            }
        }

        writeln!(&mut output, "  ]").unwrap();
        writeln!(&mut output, "}}").unwrap();

        output
    }

    /// Render goal hierarchy as tree
    fn render_goal_tree(&self, goal: &Goal, depth: usize, is_last: bool) -> String {
        let mut output = String::new();

        let indent = if depth == 0 {
            String::new()
        } else {
            "  ".repeat(depth - 1) + if is_last { "  └─ " } else { "  ├─ " }
        };

        let strategy_str = match goal.decomposition_strategy {
            super::DecompositionStrategy::Sequential => "[SEQ]",
            super::DecompositionStrategy::Parallel => "[PAR]",
            super::DecompositionStrategy::AnyOf => "[ANY]",
            super::DecompositionStrategy::AllOf => "[ALL]",
        };

        let priority_str = format!("priority: {:.1}", goal.priority);
        let deadline_str = if let Some(deadline) = goal.deadline {
            format!(", deadline: {:.0}s", deadline)
        } else {
            String::new()
        };

        writeln!(
            &mut output,
            "{}{} {} ({}{})",
            indent, strategy_str, goal.name, priority_str, deadline_str
        ).unwrap();

        for (i, sub_goal) in goal.sub_goals.iter().enumerate() {
            let is_last_sub = i == goal.sub_goals.len() - 1;
            output.push_str(&self.render_goal_tree(sub_goal, depth + 1, is_last_sub));
        }

        output
    }

    /// Render goal hierarchy as DOT
    fn render_goal_dot(&self, goal: &Goal) -> String {
        let mut output = String::new();
        let mut node_counter = 0;

        writeln!(&mut output, "digraph GoalHierarchy {{").unwrap();
        writeln!(&mut output, "  rankdir=TB;").unwrap();
        writeln!(&mut output, "  node [shape=box];").unwrap();

        self.render_goal_dot_recursive(goal, &mut output, &mut node_counter, None);

        writeln!(&mut output, "}}").unwrap();

        output
    }

    fn render_goal_dot_recursive(
        &self,
        goal: &Goal,
        output: &mut String,
        node_counter: &mut usize,
        parent_id: Option<usize>,
    ) {
        let current_id = *node_counter;
        *node_counter += 1;

        let strategy_str = match goal.decomposition_strategy {
            super::DecompositionStrategy::Sequential => "SEQ",
            super::DecompositionStrategy::Parallel => "PAR",
            super::DecompositionStrategy::AnyOf => "ANY",
            super::DecompositionStrategy::AllOf => "ALL",
        };

        writeln!(
            output,
            "  node_{} [label=\"[{}] {}\\npriority: {:.1}\"];",
            current_id, strategy_str, goal.name, goal.priority
        ).unwrap();

        if let Some(parent) = parent_id {
            writeln!(output, "  node_{} -> node_{};", parent, current_id).unwrap();
        }

        for sub_goal in &goal.sub_goals {
            self.render_goal_dot_recursive(sub_goal, output, node_counter, Some(current_id));
        }
    }

    /// Render goal as text
    fn render_goal_text(&self, goal: &Goal, depth: usize) -> String {
        let mut output = String::new();

        let indent = "  ".repeat(depth);
        writeln!(&mut output, "{}{} (priority: {:.1})", indent, goal.name, goal.priority).unwrap();

        for sub_goal in &goal.sub_goals {
            output.push_str(&self.render_goal_text(sub_goal, depth + 1));
        }

        output
    }

    /// Calculate total plan metrics
    fn calculate_plan_metrics(
        &self,
        plan: &[String],
        actions: &[Box<dyn Action>],
        history: &ActionHistory,
    ) -> (f32, f32) {
        let mut total_cost = 0.0;
        let mut total_risk = 0.0;

        for action_name in plan {
            if let Some(action) = actions.iter().find(|a| a.name() == action_name) {
                total_cost += action.calculate_cost(&WorldState::new(), history);
                total_risk += 1.0 - action.success_probability(&WorldState::new(), history);
            }
        }

        (total_cost, total_risk)
    }

    /// Format state changes for display
    fn format_state_changes(&self, effects: &std::collections::BTreeMap<String, super::StateValue>) -> String {
        if effects.is_empty() {
            return String::from("(no changes)");
        }

        let changes: Vec<String> = effects
            .iter()
            .take(2) // Limit to 2 for space
            .map(|(k, v)| format!("{}={:?}", k, v))
            .collect();

        let mut result = changes.join(", ");
        if effects.len() > 2 {
            result.push_str(&format!(", +{} more", effects.len() - 2));
        }
        result
    }
}

impl Default for PlanVisualizer {
    fn default() -> Self {
        Self::new(VisualizationFormat::AsciiTree)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::goap::SimpleAction;
    use std::collections::BTreeMap;

    fn create_test_action(name: &str, cost: f32) -> Box<dyn Action> {
        Box::new(SimpleAction::new(
            name,
            BTreeMap::new(),
            BTreeMap::new(),
            cost,
        ))
    }

    #[test]
    fn test_visualize_simple_plan() {
        let visualizer = PlanVisualizer::new(VisualizationFormat::AsciiTree);
        let actions: Vec<Box<dyn Action>> = vec![
            create_test_action("move", 1.0),
            create_test_action("attack", 2.0),
        ];
        let plan = vec!["move".to_string(), "attack".to_string()];
        let history = ActionHistory::new();
        let start = WorldState::new();

        let output = visualizer.visualize_plan(&plan, &actions, &history, &start);
        
        assert!(output.contains("Plan"));
        assert!(output.contains("move"));
        assert!(output.contains("attack"));
        assert!(output.contains("cost"));
    }

    #[test]
    fn test_visualize_goal_hierarchy() {
        let visualizer = PlanVisualizer::new(VisualizationFormat::AsciiTree);
        
        let sub_goal = Goal::new("sub", BTreeMap::new()).with_priority(5.0);
        let main_goal = Goal::new("main", BTreeMap::new())
            .with_priority(10.0)
            .with_sub_goals(vec![sub_goal]);

        let output = visualizer.visualize_goal_hierarchy(&main_goal);
        
        assert!(output.contains("main"));
        assert!(output.contains("sub"));
        assert!(output.contains("priority"));
    }

    #[test]
    fn test_text_format() {
        let visualizer = PlanVisualizer::new(VisualizationFormat::Text);
        let actions: Vec<Box<dyn Action>> = vec![
            create_test_action("action1", 1.0),
            create_test_action("action2", 2.0),
        ];
        let plan = vec!["action1".to_string(), "action2".to_string()];
        let history = ActionHistory::new();
        let start = WorldState::new();

        let output = visualizer.visualize_plan(&plan, &actions, &history, &start);
        
        assert!(output.contains("1. action1"));
        assert!(output.contains("2. action2"));
    }

    #[test]
    fn test_dot_format() {
        let visualizer = PlanVisualizer::new(VisualizationFormat::Dot);
        let actions: Vec<Box<dyn Action>> = vec![
            create_test_action("action1", 1.0),
        ];
        let plan = vec!["action1".to_string()];
        let history = ActionHistory::new();
        let start = WorldState::new();

        let output = visualizer.visualize_plan(&plan, &actions, &history, &start);
        
        assert!(output.contains("digraph Plan"));
        assert!(output.contains("action_0"));
        assert!(output.contains("start"));
        assert!(output.contains("end"));
    }

    #[test]
    fn test_timeline_format() {
        let visualizer = PlanVisualizer::new(VisualizationFormat::AsciiTimeline);
        let actions: Vec<Box<dyn Action>> = vec![
            create_test_action("action1", 1.0),
        ];
        let plan = vec!["action1".to_string()];
        let history = ActionHistory::new();
        let start = WorldState::new();

        let output = visualizer.visualize_plan(&plan, &actions, &history, &start);
        
        assert!(output.contains("Time"));
        assert!(output.contains("Action"));
        assert!(output.contains("Success"));
    }

    #[test]
    fn test_json_format() {
        let visualizer = PlanVisualizer::new(VisualizationFormat::Json);
        let actions: Vec<Box<dyn Action>> = vec![
            create_test_action("action1", 1.0),
        ];
        let plan = vec!["action1".to_string()];
        let history = ActionHistory::new();
        let start = WorldState::new();

        let output = visualizer.visualize_plan(&plan, &actions, &history, &start);
        
        assert!(output.contains("\"actions\""));
        assert!(output.contains("\"name\""));
        assert!(output.contains("\"cost\""));
    }

    #[test]
    fn test_with_options() {
        let visualizer = PlanVisualizer::new(VisualizationFormat::Text)
            .with_costs(false)
            .with_risks(false);
        
        let actions: Vec<Box<dyn Action>> = vec![
            create_test_action("action1", 1.0),
        ];
        let plan = vec!["action1".to_string()];
        let history = ActionHistory::new();
        let start = WorldState::new();

        let output = visualizer.visualize_plan(&plan, &actions, &history, &start);
        
        assert!(!output.contains("cost"));
        assert!(!output.contains("risk"));
    }
}

