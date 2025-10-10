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
