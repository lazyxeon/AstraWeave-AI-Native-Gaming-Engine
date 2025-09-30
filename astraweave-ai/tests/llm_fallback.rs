// Integration test for LLM orchestrator fallback logic (feature-gated)
#![cfg(feature = "llm_orchestrator")]
use astraweave_ai::orchestrator::{LlmOrchestrator, OrchestratorAsync};
use astraweave_core::{default_tool_registry, WorldSnapshot};

#[tokio::test(flavor = "current_thread")] 
async fn test_llm_orchestrator_fallback_to_empty_on_error() {
    // Use a mock client from astraweave-llm to force an error then fallback
    let client = astraweave_llm::AlwaysErrMock;
    let orch = LlmOrchestrator::new(client, Some(default_tool_registry()));
    let snap = WorldSnapshot::default();
    let plan = orch.plan(snap, 1).await.expect("llm plan call failed");
    assert_eq!(plan.plan_id, "llm-fallback");
    assert!(plan.steps.is_empty());
}
