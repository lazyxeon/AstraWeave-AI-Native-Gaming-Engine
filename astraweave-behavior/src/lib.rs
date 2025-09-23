//! Behavior Graphs (BT/HTN) for AstraWeave AI

/// Node types for behavior trees and HTN
#[derive(Debug, Clone)]
pub enum BehaviorNode {
    Sequence(Vec<BehaviorNode>),
    Selector(Vec<BehaviorNode>),
    Action(String),
    Condition(String),
}

/// Behavior graph structure
#[derive(Debug, Clone)]
pub struct BehaviorGraph {
    pub root: BehaviorNode,
}

impl BehaviorGraph {
    pub fn tick(&self, _context: &crate::BehaviorContext) -> BehaviorStatus {
        // TODO: Implement BT/HTN tick logic
        BehaviorStatus::Running
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BehaviorStatus {
    Success,
    Failure,
    Running,
}

/// Context for behavior evaluation (stub)
pub struct BehaviorContext;
