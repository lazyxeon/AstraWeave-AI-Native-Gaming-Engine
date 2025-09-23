// Integration test for LLM orchestrator fallback logic
use astraweave_ai::orchestrator::{LlmOrchestrator, RuleOrchestrator, Orchestrator};
use astraweave_core::{WorldSnapshot, PlanIntent};

#[test]
fn test_llm_orchestrator_fallback_to_rule() {
    let llm = LlmOrchestrator;
    let rule = RuleOrchestrator;
    let snap = WorldSnapshot::default();
    // Simulate LLM failure by always returning default
    let plan_llm = llm.plan(0, &snap);
    let plan_rule = rule.plan(0, &snap);
    // Fallback: if LLM returns default, use rule-based
    let plan = if plan_llm.steps.is_empty() { plan_rule } else { plan_llm };
    assert_eq!(plan.steps, plan_rule.steps);
}
