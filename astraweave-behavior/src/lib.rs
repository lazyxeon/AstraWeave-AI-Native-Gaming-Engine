//! Behavior Graphs (BT/HTN) and GOAP for AstraWeave AI

pub mod ecs;
pub mod goap;
pub mod goap_cache; // Week 3 Action 9: GOAP plan caching with LRU eviction

use std::collections::HashMap;

/// Node types for behavior trees and HTN
#[derive(Debug, Clone)]
pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Action(String),
    Condition(String),
    Decorator(DecoratorType, Box<BehaviorNode>),
    Parallel(Vec<BehaviorNode>, usize), // children, success threshold
}

#[derive(Debug, Clone)]
pub enum DecoratorType {
    Inverter,
    Succeeder,
    Failer,
    Repeat(u32), // max repeats
    Retry(u32),  // max retries
}

/// Behavior graph structure
#[derive(Clone)]
pub struct BehaviorGraph {
    pub root: BehaviorNode,
}

impl BehaviorGraph {
    pub fn new(root: BehaviorNode) -> Self {
        Self { root }
    }

    pub fn tick(&self, context: &BehaviorContext) -> BehaviorStatus {
        self.root.tick(context)
    }

    pub fn current_node_name(&self) -> Option<String> {
        // For now, return a placeholder. In a more complex implementation,
        // this would track the currently executing node.
        Some("root".to_string())
    }
}

impl BehaviorNode {
    pub fn tick(&self, context: &BehaviorContext) -> BehaviorStatus {
        match self {
            BehaviorNode::Action(name) => context.evaluate_action(name),
            BehaviorNode::Condition(name) => context.evaluate_condition(name),
            BehaviorNode::Sequence(children) => {
                for child in children {
                    match child.tick(context) {
                        BehaviorStatus::Running => return BehaviorStatus::Running,
                        BehaviorStatus::Failure => return BehaviorStatus::Failure,
                        BehaviorStatus::Success => continue,
                    }
                }
                BehaviorStatus::Success
            }
            BehaviorNode::Selector(children) => {
                for child in children {
                    match child.tick(context) {
                        BehaviorStatus::Running => return BehaviorStatus::Running,
                        BehaviorStatus::Success => return BehaviorStatus::Success,
                        BehaviorStatus::Failure => continue,
                    }
                }
                BehaviorStatus::Failure
            }
            BehaviorNode::Decorator(decorator, child) => match decorator {
                DecoratorType::Inverter => match child.tick(context) {
                    BehaviorStatus::Success => BehaviorStatus::Failure,
                    BehaviorStatus::Failure => BehaviorStatus::Success,
                    r => r,
                },
                DecoratorType::Succeeder => {
                    child.tick(context);
                    BehaviorStatus::Success
                }
                DecoratorType::Failer => {
                    child.tick(context);
                    BehaviorStatus::Failure
                }
                DecoratorType::Repeat(max) => {
                    for _ in 0..*max {
                        match child.tick(context) {
                            BehaviorStatus::Running => return BehaviorStatus::Running,
                            BehaviorStatus::Success => continue,
                            BehaviorStatus::Failure => return BehaviorStatus::Failure,
                        }
                    }
                    BehaviorStatus::Success
                }
                DecoratorType::Retry(max) => {
                    for _ in 0..*max {
                        match child.tick(context) {
                            BehaviorStatus::Running => return BehaviorStatus::Running,
                            BehaviorStatus::Success => return BehaviorStatus::Success,
                            BehaviorStatus::Failure => continue,
                        }
                    }
                    BehaviorStatus::Failure
                }
            },
            BehaviorNode::Parallel(children, threshold) => {
                // Ensure the threshold is within sensible bounds
                if *threshold == 0 {
                    return BehaviorStatus::Success;
                }
                if *threshold > children.len() {
                    return BehaviorStatus::Failure;
                }

                let mut success_count = 0;
                let mut running_count = 0;
                for child in children {
                    match child.tick(context) {
                        BehaviorStatus::Success => success_count += 1,
                        BehaviorStatus::Running => running_count += 1,
                        BehaviorStatus::Failure => {} // do nothing
                    }
                }
                if success_count >= *threshold {
                    BehaviorStatus::Success
                } else if running_count > 0 {
                    BehaviorStatus::Running
                } else {
                    BehaviorStatus::Failure
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ===== BehaviorContext Tests =====

    #[test]
    fn test_behavior_context_default() {
        let ctx = BehaviorContext::default();
        assert_eq!(ctx.actions.len(), 0);
        assert_eq!(ctx.conditions.len(), 0);
    }

    #[test]
    fn test_register_action() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("test_action", || BehaviorStatus::Success);
        assert_eq!(ctx.actions.len(), 1);
        assert!(ctx.actions.contains_key("test_action"));
    }

    #[test]
    fn test_register_condition() {
        let mut ctx = BehaviorContext::new();
        ctx.register_condition("test_condition", || true);
        assert_eq!(ctx.conditions.len(), 1);
        assert!(ctx.conditions.contains_key("test_condition"));
    }

    #[test]
    fn test_evaluate_action_success() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("succeed", || BehaviorStatus::Success);
        let node = BehaviorNode::Action("succeed".to_string());
        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_evaluate_action_failure() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("fail", || BehaviorStatus::Failure);
        let node = BehaviorNode::Action("fail".to_string());
        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_evaluate_action_running() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("running", || BehaviorStatus::Running);
        let node = BehaviorNode::Action("running".to_string());
        assert_eq!(node.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_evaluate_condition_true() {
        let mut ctx = BehaviorContext::new();
        ctx.register_condition("is_true", || true);
        let node = BehaviorNode::Condition("is_true".to_string());
        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_evaluate_condition_false() {
        let mut ctx = BehaviorContext::new();
        ctx.register_condition("is_false", || false);
        let node = BehaviorNode::Condition("is_false".to_string());
        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    // ===== Sequence Node Tests =====

    #[test]
    fn test_sequence_all_success() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Success);
        ctx.register_action("a2", || BehaviorStatus::Success);
        ctx.register_action("a3", || BehaviorStatus::Success);

        let seq = BehaviorNode::Sequence(vec![
            BehaviorNode::Action("a1".to_string()),
            BehaviorNode::Action("a2".to_string()),
            BehaviorNode::Action("a3".to_string()),
        ]);

        assert_eq!(seq.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_sequence_early_failure() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Success);
        ctx.register_action("a2", || BehaviorStatus::Failure);
        ctx.register_action("a3", || BehaviorStatus::Success);

        let seq = BehaviorNode::Sequence(vec![
            BehaviorNode::Action("a1".to_string()),
            BehaviorNode::Action("a2".to_string()),
            BehaviorNode::Action("a3".to_string()),
        ]);

        assert_eq!(seq.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_sequence_running_short_circuit() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Success);
        ctx.register_action("a2", || BehaviorStatus::Running);
        ctx.register_action("a3", || BehaviorStatus::Success);

        let seq = BehaviorNode::Sequence(vec![
            BehaviorNode::Action("a1".to_string()),
            BehaviorNode::Action("a2".to_string()),
            BehaviorNode::Action("a3".to_string()),
        ]);

        assert_eq!(seq.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_sequence_empty() {
        let ctx = BehaviorContext::new();
        let seq = BehaviorNode::Sequence(vec![]);
        assert_eq!(seq.tick(&ctx), BehaviorStatus::Success);
    }

    // ===== Selector Node Tests =====

    #[test]
    fn test_selector_all_failure() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Failure);
        ctx.register_action("a2", || BehaviorStatus::Failure);
        ctx.register_action("a3", || BehaviorStatus::Failure);

        let sel = BehaviorNode::Selector(vec![
            BehaviorNode::Action("a1".to_string()),
            BehaviorNode::Action("a2".to_string()),
            BehaviorNode::Action("a3".to_string()),
        ]);

        assert_eq!(sel.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_selector_early_success() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Failure);
        ctx.register_action("a2", || BehaviorStatus::Success);
        ctx.register_action("a3", || BehaviorStatus::Failure);

        let sel = BehaviorNode::Selector(vec![
            BehaviorNode::Action("a1".to_string()),
            BehaviorNode::Action("a2".to_string()),
            BehaviorNode::Action("a3".to_string()),
        ]);

        assert_eq!(sel.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_selector_running_short_circuit() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Failure);
        ctx.register_action("a2", || BehaviorStatus::Running);
        ctx.register_action("a3", || BehaviorStatus::Success);

        let sel = BehaviorNode::Selector(vec![
            BehaviorNode::Action("a1".to_string()),
            BehaviorNode::Action("a2".to_string()),
            BehaviorNode::Action("a3".to_string()),
        ]);

        assert_eq!(sel.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_selector_empty() {
        let ctx = BehaviorContext::new();
        let sel = BehaviorNode::Selector(vec![]);
        assert_eq!(sel.tick(&ctx), BehaviorStatus::Failure);
    }

    // ===== Decorator Tests =====

    #[test]
    fn test_inverter_success_to_failure() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("succeed", || BehaviorStatus::Success);
        let node = BehaviorNode::Decorator(
            DecoratorType::Inverter,
            Box::new(BehaviorNode::Action("succeed".to_string())),
        );
        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_inverter_failure_to_success() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("fail", || BehaviorStatus::Failure);
        let node = BehaviorNode::Decorator(
            DecoratorType::Inverter,
            Box::new(BehaviorNode::Action("fail".to_string())),
        );
        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_inverter_running_unchanged() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("running", || BehaviorStatus::Running);
        let node = BehaviorNode::Decorator(
            DecoratorType::Inverter,
            Box::new(BehaviorNode::Action("running".to_string())),
        );
        assert_eq!(node.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_succeeder_forces_success() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("fail", || BehaviorStatus::Failure);
        let node = BehaviorNode::Decorator(
            DecoratorType::Succeeder,
            Box::new(BehaviorNode::Action("fail".to_string())),
        );
        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_failer_forces_failure() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("succeed", || BehaviorStatus::Success);
        let node = BehaviorNode::Decorator(
            DecoratorType::Failer,
            Box::new(BehaviorNode::Action("succeed".to_string())),
        );
        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_repeat_success() {
        use std::sync::{Arc, Mutex};
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        let mut ctx = BehaviorContext::new();
        ctx.register_action("increment", move || {
            *count_clone.lock().unwrap() += 1;
            BehaviorStatus::Success
        });

        let node = BehaviorNode::Decorator(
            DecoratorType::Repeat(5),
            Box::new(BehaviorNode::Action("increment".to_string())),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
        assert_eq!(*count.lock().unwrap(), 5);
    }

    #[test]
    fn test_repeat_early_failure() {
        use std::sync::{Arc, Mutex};
        let count = Arc::new(Mutex::new(0));
        let count_clone = count.clone();
        let mut ctx = BehaviorContext::new();
        ctx.register_action("fail_at_3", move || {
            *count_clone.lock().unwrap() += 1;
            if *count_clone.lock().unwrap() < 3 {
                BehaviorStatus::Success
            } else {
                BehaviorStatus::Failure
            }
        });

        let node = BehaviorNode::Decorator(
            DecoratorType::Repeat(5),
            Box::new(BehaviorNode::Action("fail_at_3".to_string())),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
        assert_eq!(*count.lock().unwrap(), 3);
    }

    #[test]
    fn test_retry_eventual_success() {
        use std::sync::{Arc, Mutex};
        let attempt = Arc::new(Mutex::new(0));
        let attempt_clone = attempt.clone();
        let mut ctx = BehaviorContext::new();
        ctx.register_action("succeed_at_3", move || {
            *attempt_clone.lock().unwrap() += 1;
            if *attempt_clone.lock().unwrap() >= 3 {
                BehaviorStatus::Success
            } else {
                BehaviorStatus::Failure
            }
        });

        let node = BehaviorNode::Decorator(
            DecoratorType::Retry(5),
            Box::new(BehaviorNode::Action("succeed_at_3".to_string())),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
        assert_eq!(*attempt.lock().unwrap(), 3);
    }

    #[test]
    fn test_retry_exhausted() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("always_fail", || BehaviorStatus::Failure);

        let node = BehaviorNode::Decorator(
            DecoratorType::Retry(3),
            Box::new(BehaviorNode::Action("always_fail".to_string())),
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    // ===== Parallel Node Tests =====

    #[test]
    fn test_parallel_threshold_met() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Success);
        ctx.register_action("a2", || BehaviorStatus::Success);
        ctx.register_action("a3", || BehaviorStatus::Failure);

        let node = BehaviorNode::Parallel(
            vec![
                BehaviorNode::Action("a1".to_string()),
                BehaviorNode::Action("a2".to_string()),
                BehaviorNode::Action("a3".to_string()),
            ],
            2, // threshold: 2 successes needed
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_parallel_threshold_not_met() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Success);
        ctx.register_action("a2", || BehaviorStatus::Failure);
        ctx.register_action("a3", || BehaviorStatus::Failure);

        let node = BehaviorNode::Parallel(
            vec![
                BehaviorNode::Action("a1".to_string()),
                BehaviorNode::Action("a2".to_string()),
                BehaviorNode::Action("a3".to_string()),
            ],
            2, // threshold: 2 successes needed
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    #[test]
    fn test_parallel_running() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Success);
        ctx.register_action("a2", || BehaviorStatus::Running);
        ctx.register_action("a3", || BehaviorStatus::Failure);

        let node = BehaviorNode::Parallel(
            vec![
                BehaviorNode::Action("a1".to_string()),
                BehaviorNode::Action("a2".to_string()),
                BehaviorNode::Action("a3".to_string()),
            ],
            2, // threshold: 2 successes needed
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Running);
    }

    #[test]
    fn test_parallel_zero_threshold() {
        let ctx = BehaviorContext::new();
        let node = BehaviorNode::Parallel(vec![], 0);
        assert_eq!(node.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_parallel_threshold_exceeds_children() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Success);

        let node = BehaviorNode::Parallel(
            vec![BehaviorNode::Action("a1".to_string())],
            5, // threshold > children count
        );

        assert_eq!(node.tick(&ctx), BehaviorStatus::Failure);
    }

    // ===== BehaviorGraph Tests =====

    #[test]
    fn test_behavior_graph_creation() {
        let root = BehaviorNode::Action("test".to_string());
        let graph = BehaviorGraph::new(root);
        assert!(matches!(graph.root, BehaviorNode::Action(_)));
    }

    #[test]
    fn test_behavior_graph_tick() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("succeed", || BehaviorStatus::Success);

        let root = BehaviorNode::Action("succeed".to_string());
        let graph = BehaviorGraph::new(root);

        assert_eq!(graph.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_behavior_graph_current_node_name() {
        let root = BehaviorNode::Action("test".to_string());
        let graph = BehaviorGraph::new(root);

        assert_eq!(graph.current_node_name(), Some("root".to_string()));
    }

    // ===== Complex Integration Tests =====

    #[test]
    fn test_nested_sequence_selector() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("a1", || BehaviorStatus::Failure);
        ctx.register_action("a2", || BehaviorStatus::Success);
        ctx.register_action("a3", || BehaviorStatus::Success);
        ctx.register_action("a4", || BehaviorStatus::Success);

        // Selector [ Sequence [a1, a2], Sequence [a3, a4] ]
        let root = BehaviorNode::Selector(vec![
            BehaviorNode::Sequence(vec![
                BehaviorNode::Action("a1".to_string()),
                BehaviorNode::Action("a2".to_string()),
            ]),
            BehaviorNode::Sequence(vec![
                BehaviorNode::Action("a3".to_string()),
                BehaviorNode::Action("a4".to_string()),
            ]),
        ]);

        assert_eq!(root.tick(&ctx), BehaviorStatus::Success);
    }

    #[test]
    fn test_nested_decorator_sequence() {
        let mut ctx = BehaviorContext::new();
        ctx.register_action("fail", || BehaviorStatus::Failure);
        ctx.register_action("succeed", || BehaviorStatus::Success);

        // Sequence [ Inverter(fail), succeed ]
        let root = BehaviorNode::Sequence(vec![
            BehaviorNode::Decorator(
                DecoratorType::Inverter,
                Box::new(BehaviorNode::Action("fail".to_string())),
            ),
            BehaviorNode::Action("succeed".to_string()),
        ]);

        assert_eq!(root.tick(&ctx), BehaviorStatus::Success);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    Success,
    Failure,
    Running,
}

/// Context for behavior evaluation
pub struct BehaviorContext {
    pub actions: HashMap<String, Box<dyn Fn() -> BehaviorStatus + Send + Sync>>,
    pub conditions: HashMap<String, Box<dyn Fn() -> bool + Send + Sync>>,
}

impl Default for BehaviorContext {
    fn default() -> Self {
        Self::new()
    }
}

impl BehaviorContext {
    pub fn new() -> Self {
        Self {
            actions: HashMap::new(),
            conditions: HashMap::new(),
        }
    }

    pub fn register_action<F>(&mut self, name: &str, f: F)
    where
        F: Fn() -> BehaviorStatus + Send + Sync + 'static,
    {
        self.actions.insert(name.to_string(), Box::new(f));
    }

    pub fn register_condition<F>(&mut self, name: &str, f: F)
    where
        F: Fn() -> bool + Send + Sync + 'static,
    {
        self.conditions.insert(name.to_string(), Box::new(f));
    }

    fn evaluate_action(&self, name: &str) -> BehaviorStatus {
        if let Some(action) = self.actions.get(name) {
            action()
        } else {
            debug_assert!(false, "Action '{}' not registered in BehaviorContext", name);
            BehaviorStatus::Failure
        }
    }

    fn evaluate_condition(&self, name: &str) -> BehaviorStatus {
        if let Some(condition) = self.conditions.get(name) {
            if condition() {
                BehaviorStatus::Success
            } else {
                BehaviorStatus::Failure
            }
        } else {
            debug_assert!(
                false,
                "Condition '{}' not registered in BehaviorContext",
                name
            );
            BehaviorStatus::Failure
        }
    }
}
