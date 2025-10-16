//! LLM executor for asynchronous plan generation.
//!
//! This module provides `LlmExecutor`, which wraps an `OrchestratorAsync` and enables
//! non-blocking LLM plan generation. This is critical for the AI Arbiter pattern where
//! GOAP maintains control while Hermes generates strategic plans in the background.
//!
//! # Architecture
//!
//! ```text
//! Game Loop (GOAP control)
//!       ↓
//! LlmExecutor::generate_plan_async()  ← Returns immediately
//!       ↓
//! AsyncTask<PlanIntent>  ← Poll via try_recv()
//!       ↓
//! tokio::spawn_blocking  ← CPU-bound LLM work
//!       ↓
//! OrchestratorAsync::plan()  ← 13-21s Hermes inference
//!       ↓
//! PlanIntent  ← Strategic plan ready
//! ```
//!
//! # Example
//! ```no_run
//! use astraweave_ai::LlmExecutor;
//! use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
//! use astraweave_core::{WorldSnapshot, default_tool_registry};
//! use std::sync::Arc;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = Hermes2ProOllama::new(
//!     "http://127.0.0.1:11434".to_string(),
//!     "adrienbrault/nous-hermes2pro:Q4_K_M".to_string(),
//! );
//!
//! let registry = default_tool_registry();
//! let orchestrator = Arc::new(astraweave_llm::FallbackOrchestrator::new(client, registry));
//! let runtime = tokio::runtime::Handle::current();
//!
//! let executor = LlmExecutor::new(orchestrator, runtime);
//!
//! // Non-blocking async plan generation
//! let snapshot = /* build snapshot */;
//! # WorldSnapshot { t: 0.0, me: astraweave_core::CompanionState { ammo: 10, cooldowns: Default::default(), morale: 1.0, pos: astraweave_core::IVec2 { x: 0, y: 0 } }, player: astraweave_core::PlayerState { hp: 100, pos: astraweave_core::IVec2 { x: 0, y: 0 }, stance: "stand".into(), orders: vec![] }, enemies: vec![], pois: vec![], obstacles: vec![], objective: None };
//! let mut task = executor.generate_plan_async(snapshot);
//!
//! // GOAP continues providing actions while LLM plans...
//!
//! // Poll for completion (non-blocking)
//! if let Some(result) = task.try_recv() {
//!     match result {
//!         Ok(plan) => println!("LLM plan ready: {} steps", plan.steps.len()),
//!         Err(e) => eprintln!("LLM planning failed: {}", e),
//!     }
//! }
//! # Ok(())
//! # }
//! ```

use crate::async_task::AsyncTask;
use crate::orchestrator::OrchestratorAsync;
use anyhow::{Context, Result};
use astraweave_core::{PlanIntent, WorldSnapshot};
use std::sync::Arc;
use tokio::runtime::Handle;

/// LLM executor for non-blocking plan generation.
///
/// Wraps an `OrchestratorAsync` (typically a fallback system with Hermes 2 Pro)
/// and provides async plan generation that doesn't block the game loop.
///
/// # Thread Safety
/// Uses `Arc<dyn OrchestratorAsync>` to allow sharing across async task boundaries.
/// The orchestrator must be `Send + Sync`.
///
/// # Performance
/// - `generate_plan_async()`: Returns immediately (<1 ms)
/// - Background task: 13-21s (Hermes 2 Pro inference)
/// - `generate_plan_sync()`: Blocks for full duration (testing only)
pub struct LlmExecutor {
    /// The wrapped orchestrator (Arc for thread-safety)
    orchestrator: Arc<dyn OrchestratorAsync + Send + Sync>,
    
    /// Tokio runtime handle for spawning async tasks
    runtime: Handle,
}

impl LlmExecutor {
    /// Create a new `LlmExecutor`.
    ///
    /// # Arguments
    /// - `orchestrator`: The LLM orchestrator to use for planning (typically `FallbackOrchestrator`)
    /// - `runtime`: Tokio runtime handle for spawning async tasks
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::LlmExecutor;
    /// use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
    /// use astraweave_core::default_tool_registry;
    /// use std::sync::Arc;
    ///
    /// # async fn example() {
    /// let client = Hermes2ProOllama::new(
    ///     "http://127.0.0.1:11434".to_string(),
    ///     "adrienbrault/nous-hermes2pro:Q4_K_M".to_string(),
    /// );
    ///
    /// let registry = default_tool_registry();
    /// let orchestrator = Arc::new(astraweave_llm::FallbackOrchestrator::new(client, registry));
    /// let runtime = tokio::runtime::Handle::current();
    ///
    /// let executor = LlmExecutor::new(orchestrator, runtime);
    /// # }
    /// ```
    pub fn new(
        orchestrator: Arc<dyn OrchestratorAsync + Send + Sync>,
        runtime: Handle,
    ) -> Self {
        Self {
            orchestrator,
            runtime,
        }
    }

    /// Generate a plan asynchronously (non-blocking).
    ///
    /// This method returns immediately with an `AsyncTask` that can be polled
    /// for completion. The actual LLM inference happens in a background task.
    ///
    /// # Arguments
    /// - `snap`: World snapshot to plan from (cloned into async task)
    ///
    /// # Returns
    /// An `AsyncTask<Result<PlanIntent>>` that can be polled via `try_recv()`
    ///
    /// # Performance
    /// - Returns in: <1 ms
    /// - Background task completes in: 13-21s (Hermes 2 Pro)
    /// - Memory: Clones `WorldSnapshot` (~1-2 KB)
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::LlmExecutor;
    /// # use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
    /// # use astraweave_core::{WorldSnapshot, default_tool_registry};
    /// # use std::sync::Arc;
    ///
    /// # async fn example(executor: LlmExecutor, snapshot: WorldSnapshot) {
    /// let mut task = executor.generate_plan_async(snapshot);
    ///
    /// // Continue with GOAP while LLM plans...
    /// loop {
    ///     // Poll for completion (non-blocking)
    ///     if let Some(result) = task.try_recv() {
    ///         match result {
    ///             Ok(Ok(plan)) => {
    ///                 println!("LLM plan ready!");
    ///                 break;
    ///             }
    ///             Ok(Err(e)) => {
    ///                 eprintln!("LLM planning failed: {}", e);
    ///                 break;
    ///             }
    ///             Err(e) => {
    ///                 eprintln!("Task join error: {}", e);
    ///                 break;
    ///             }
    ///         }
    ///     }
    ///     
    ///     // GOAP provides instant action here...
    /// }
    /// # }
    /// ```
    pub fn generate_plan_async(&self, snap: WorldSnapshot) -> AsyncTask<Result<PlanIntent>> {
        let orchestrator = Arc::clone(&self.orchestrator);
        
        // Spawn blocking task (LLM inference is CPU-bound, not I/O-bound)
        let handle = self.runtime.spawn_blocking(move || {
            // Create a nested runtime for the async orchestrator
            // This is safe because we're in spawn_blocking (dedicated thread)
            let rt = tokio::runtime::Runtime::new()
                .expect("Failed to create nested runtime for LLM inference");
            
            rt.block_on(async move {
                // Call the orchestrator with default budget (60s from env or config)
                // Budget is handled internally by orchestrator (Phase 7 timeout logic)
                let budget_ms = std::env::var("LLM_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60_000);  // Default 60s
                
                // Convert Result to PlanIntent directly (propagate error via AsyncTask)
                orchestrator.plan(snap, budget_ms).await
            })
        });

        // Map JoinHandle<Result<PlanIntent>> to AsyncTask<PlanIntent>
        // AsyncTask::try_recv() will return Option<Result<PlanIntent>>
        // where the inner Result comes from the orchestrator
        AsyncTask::new(handle)
    }

    /// Generate a plan synchronously (blocking).
    ///
    /// ⚠️ **WARNING**: This method blocks the calling thread for 13-21 seconds.
    /// **DO NOT USE** in the game loop. Provided for testing and initialization only.
    ///
    /// # Arguments
    /// - `snap`: World snapshot to plan from
    ///
    /// # Returns
    /// A `PlanIntent` or error
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::LlmExecutor;
    /// # use astraweave_llm::hermes2pro_ollama::Hermes2ProOllama;
    /// # use astraweave_core::{WorldSnapshot, default_tool_registry};
    /// # use std::sync::Arc;
    ///
    /// # async fn example(executor: LlmExecutor, snapshot: WorldSnapshot) -> anyhow::Result<()> {
    /// // For testing only - blocks for 13-21s!
    /// let plan = executor.generate_plan_sync(&snapshot)?;
    /// println!("Generated plan: {} steps", plan.steps.len());
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate_plan_sync(&self, snap: &WorldSnapshot) -> Result<PlanIntent> {
        let orchestrator = Arc::clone(&self.orchestrator);
        let snap_clone = snap.clone();
        
        // Block on the runtime
        self.runtime
            .block_on(async move {
                let budget_ms = std::env::var("LLM_TIMEOUT_MS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(60_000);
                
                orchestrator.plan(snap_clone, budget_ms).await
            })
            .context("LlmExecutor::generate_plan_sync failed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_core::{CompanionState, EnemyState, IVec2, PlayerState};
    use std::collections::BTreeMap;
    use std::time::Duration;
    use tokio::time::sleep;

    /// Mock orchestrator for testing (instant response)
    struct MockOrchestrator {
        delay_ms: u64,
        should_fail: bool,
    }

    #[async_trait::async_trait]
    impl OrchestratorAsync for MockOrchestrator {
        async fn plan(&self, snap: WorldSnapshot, _budget_ms: u32) -> Result<PlanIntent> {
            // Simulate LLM latency
            if self.delay_ms > 0 {
                sleep(Duration::from_millis(self.delay_ms)).await;
            }

            if self.should_fail {
                anyhow::bail!("Mock orchestrator failure");
            }

            Ok(PlanIntent {
                plan_id: format!("mock-plan-{}", snap.t as i64),
                steps: vec![
                    astraweave_core::ActionStep::MoveTo {
                        x: 10,
                        y: 10,
                        speed: None,
                    },
                    astraweave_core::ActionStep::TakeCover { position: None },
                ],
            })
        }

        fn name(&self) -> &'static str {
            "MockOrchestrator"
        }
    }

    fn create_test_snapshot() -> WorldSnapshot {
        WorldSnapshot {
            t: 1.234,
            me: CompanionState {
                ammo: 10,
                cooldowns: BTreeMap::new(),
                morale: 1.0,
                pos: IVec2 { x: 5, y: 5 },
            },
            player: PlayerState {
                hp: 100,
                pos: IVec2 { x: 5, y: 5 },
                stance: "stand".into(),
                orders: vec![],
            },
            enemies: vec![EnemyState {
                id: 1,
                pos: IVec2 { x: 10, y: 10 },
                hp: 100,
                cover: "none".into(),
                last_seen: 0.0,
            }],
            pois: vec![],
            obstacles: vec![],
            objective: Some("extract".into()),
        }
    }

    #[tokio::test]
    async fn test_llm_executor_async_returns_immediately() {
        let mock = Arc::new(MockOrchestrator {
            delay_ms: 1000, // 1 second delay
            should_fail: false,
        });

        let executor = LlmExecutor::new(mock, tokio::runtime::Handle::current());
        let snapshot = create_test_snapshot();

        // Measure time for generate_plan_async to return
        let start = std::time::Instant::now();
        let mut task = executor.generate_plan_async(snapshot);
        let elapsed = start.elapsed();

        // Should return in <10ms (not wait for LLM)
        assert!(
            elapsed < Duration::from_millis(10),
            "generate_plan_async took {:?}, expected <10ms",
            elapsed
        );

        // Task should not be finished immediately
        assert!(!task.is_finished(), "Task should not be finished immediately");

        // Wait for task to complete
        sleep(Duration::from_millis(1100)).await;

        // Now task should be finished
        assert!(task.is_finished(), "Task should be finished after delay");

        // Extract result
        let result = task.try_recv();
        assert!(result.is_some(), "Task result should be available");

        match result.unwrap() {
            Ok(Ok(plan)) => {
                assert_eq!(plan.steps.len(), 2, "Expected 2-step plan");
                assert!(plan.plan_id.starts_with("mock-plan-"));
            }
            Ok(Err(e)) => panic!("Orchestrator failed: {}", e),
            Err(e) => panic!("Task join error: {}", e),
        }
    }

    #[tokio::test]
    async fn test_llm_executor_async_completion() {
        let mock = Arc::new(MockOrchestrator {
            delay_ms: 10, // Fast mock
            should_fail: false,
        });

        let executor = LlmExecutor::new(mock, tokio::runtime::Handle::current());
        let snapshot = create_test_snapshot();

        let mut task = executor.generate_plan_async(snapshot);

        // Poll until complete
        let mut attempts = 0;
        let max_attempts = 100;
        let mut plan_result = None;

        while attempts < max_attempts {
            if let Some(result) = task.try_recv() {
                plan_result = Some(result);
                break;
            }
            sleep(Duration::from_millis(10)).await;
            attempts += 1;
        }

        assert!(
            plan_result.is_some(),
            "Task did not complete within {} attempts",
            max_attempts
        );

        match plan_result.unwrap() {
            Ok(Ok(plan)) => {
                assert_eq!(plan.steps.len(), 2);
            }
            Ok(Err(e)) => panic!("Orchestrator failed: {}", e),
            Err(e) => panic!("Task join error: {}", e),
        }
    }

    #[test]  // Changed from #[tokio::test] since we need to test blocking behavior
    fn test_llm_executor_sync_blocks() {
        // Create a new runtime for this test (not inside tokio::test)
        let rt = tokio::runtime::Runtime::new().unwrap();
        
        let mock = Arc::new(MockOrchestrator {
            delay_ms: 100,
            should_fail: false,
        });

        let executor = LlmExecutor::new(mock, rt.handle().clone());
        let snapshot = create_test_snapshot();

        // Measure time for generate_plan_sync
        let start = std::time::Instant::now();
        let result = executor.generate_plan_sync(&snapshot);
        let elapsed = start.elapsed();

        // Should block for at least the delay time
        assert!(
            elapsed >= Duration::from_millis(90),
            "generate_plan_sync took {:?}, expected >=90ms",
            elapsed
        );

        assert!(result.is_ok(), "generate_plan_sync should succeed");
        let plan = result.unwrap();
        assert_eq!(plan.steps.len(), 2);
    }

    #[tokio::test]
    async fn test_llm_executor_failure_handling() {
        let mock = Arc::new(MockOrchestrator {
            delay_ms: 10,
            should_fail: true, // Force failure
        });

        let executor = LlmExecutor::new(mock, tokio::runtime::Handle::current());
        let snapshot = create_test_snapshot();

        let mut task = executor.generate_plan_async(snapshot);

        // Wait for task to complete
        sleep(Duration::from_millis(50)).await;

        // Extract result
        let result = task.try_recv();
        assert!(result.is_some(), "Task should complete (with error)");

        match result.unwrap() {
            Ok(Ok(_)) => panic!("Expected error, got success"),
            Ok(Err(e)) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("Mock orchestrator failure"),
                    "Error message should contain 'Mock orchestrator failure', got: {}",
                    err_msg
                );
            }
            Err(e) => panic!("Task join error (expected orchestrator error): {}", e),
        }
    }

    #[tokio::test]
    async fn test_llm_executor_multiple_concurrent_tasks() {
        let mock = Arc::new(MockOrchestrator {
            delay_ms: 50,
            should_fail: false,
        });

        let executor = LlmExecutor::new(mock, tokio::runtime::Handle::current());
        let snapshot = create_test_snapshot();

        // Spawn 3 concurrent tasks
        let mut task1 = executor.generate_plan_async(snapshot.clone());
        let mut task2 = executor.generate_plan_async(snapshot.clone());
        let mut task3 = executor.generate_plan_async(snapshot);

        // Wait for all to complete
        sleep(Duration::from_millis(100)).await;

        // All should be finished
        assert!(task1.is_finished());
        assert!(task2.is_finished());
        assert!(task3.is_finished());

        // All should succeed
        assert!(task1.try_recv().unwrap().is_ok());
        assert!(task2.try_recv().unwrap().is_ok());
        assert!(task3.try_recv().unwrap().is_ok());
    }
}
