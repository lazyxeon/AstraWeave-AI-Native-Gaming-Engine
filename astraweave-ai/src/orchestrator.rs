//! Orchestrator trait and implementations for AI planning

/// The Orchestrator trait abstracts over different AI planning strategies (rule-based, utility/GOAP, LLM).
pub trait Orchestrator {
    /// Generate a plan for the given agent and world snapshot.
    fn plan(&self, agent_id: u32, snapshot: &crate::WorldSnapshot) -> crate::PlanIntent;
}

/// Rule-based orchestrator stub
pub struct RuleOrchestrator;
impl Orchestrator for RuleOrchestrator {
    fn plan(&self, _agent_id: u32, _snapshot: &crate::WorldSnapshot) -> crate::PlanIntent {
        // TODO: Implement rule-based planning
        crate::PlanIntent::default()
    }
}

/// Utility/GOAP orchestrator stub
pub struct UtilityOrchestrator;
impl Orchestrator for UtilityOrchestrator {
    fn plan(&self, _agent_id: u32, _snapshot: &crate::WorldSnapshot) -> crate::PlanIntent {
        // TODO: Implement utility/GOAP planning
        crate::PlanIntent::default()
    }
}

/// LLM orchestrator stub
pub struct LlmOrchestrator;
impl Orchestrator for LlmOrchestrator {
    fn plan(&self, _agent_id: u32, _snapshot: &crate::WorldSnapshot) -> crate::PlanIntent {
        // TODO: Call LLM for plan
        crate::PlanIntent::default()
    }
}
