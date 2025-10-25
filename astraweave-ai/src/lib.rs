pub mod core_loop;
pub mod ecs_ai_plugin;
pub mod orchestrator;
pub mod tool_sandbox;

// Phase 7 Arbiter: Async infrastructure for GOAP+Hermes hybrid control
#[cfg(feature = "llm_orchestrator")]
pub mod async_task;

#[cfg(feature = "llm_orchestrator")]
pub mod llm_executor;

#[cfg(feature = "llm_orchestrator")]
pub mod ai_arbiter;

pub use core_loop::*;
pub use ecs_ai_plugin::{build_app_with_ai, AiPlanningPlugin};
pub use orchestrator::*;
pub use tool_sandbox::*;

#[cfg(feature = "llm_orchestrator")]
pub use async_task::AsyncTask;

#[cfg(feature = "llm_orchestrator")]
pub use llm_executor::LlmExecutor;

#[cfg(feature = "llm_orchestrator")]
pub use ai_arbiter::{AIArbiter, AIControlMode};
