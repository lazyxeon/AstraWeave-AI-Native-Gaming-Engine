//! Asynchronous task wrapper for non-blocking LLM execution.
//!
//! This module provides `AsyncTask<T>`, a wrapper around `tokio::task::JoinHandle<T>`
//! that enables non-blocking polling of async operations. This is critical for the
//! AI Arbiter pattern where GOAP maintains control while Hermes plans asynchronously.
//!
//! # Example
//! ```no_run
//! use astraweave_ai::AsyncTask;
//! use tokio::runtime::Runtime;
//!
//! let rt = Runtime::new().unwrap();
//! let handle = rt.spawn(async {
//!     tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
//!     42
//! });
//!
//! let mut task = AsyncTask::new(handle);
//!
//! // Non-blocking poll
//! if let Some(result) = task.try_recv() {
//!     match result {
//!         Ok(value) => println!("Task completed: {}", value),
//!         Err(e) => eprintln!("Task failed: {}", e),
//!     }
//! } else {
//!     println!("Task still running...");
//! }
//! ```

use anyhow::{Context, Result};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use std::time::{Duration, Instant};
use tokio::task::JoinHandle;

/// A wrapper around `tokio::task::JoinHandle<T>` that provides non-blocking polling.
///
/// This type is essential for the AI Arbiter pattern, allowing the game loop to:
/// 1. Dispatch async LLM planning tasks
/// 2. Continue with GOAP-based tactical control
/// 3. Poll for LLM completion without blocking
/// 4. Transition to LLM execution when ready
///
/// # Generic Parameters
/// - `T`: The type of value produced by the async task
///
/// # Lifetime
/// The `AsyncTask` owns the `JoinHandle`, so the underlying task will be aborted
/// if the `AsyncTask` is dropped before completion.
pub struct AsyncTask<T> {
    /// The underlying tokio join handle (Option to allow taking ownership in await_result)
    handle: Option<JoinHandle<T>>,

    /// Timestamp when the task was created (for timeout detection)
    started_at: Instant,

    /// Optional timeout duration (None = no timeout)
    timeout: Option<Duration>,
}

impl<T> AsyncTask<T> {
    /// Create a new `AsyncTask` from a tokio `JoinHandle`.
    ///
    /// # Arguments
    /// - `handle`: The tokio join handle for the async task
    ///
    /// # Returns
    /// A new `AsyncTask` with no timeout
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::AsyncTask;
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new().unwrap();
    /// let handle = rt.spawn(async { 42 });
    /// let task = AsyncTask::new(handle);
    /// ```
    pub fn new(handle: JoinHandle<T>) -> Self {
        Self {
            handle: Some(handle),
            started_at: Instant::now(),
            timeout: None,
        }
    }

    /// Create a new `AsyncTask` with a timeout.
    ///
    /// If the task does not complete within the specified duration,
    /// `try_recv()` will return an error.
    ///
    /// # Arguments
    /// - `handle`: The tokio join handle for the async task
    /// - `timeout`: Maximum duration to wait for task completion
    ///
    /// # Returns
    /// A new `AsyncTask` with the specified timeout
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::AsyncTask;
    /// use tokio::runtime::Runtime;
    /// use std::time::Duration;
    ///
    /// let rt = Runtime::new().unwrap();
    /// let handle = rt.spawn(async { 42 });
    /// let task = AsyncTask::with_timeout(handle, Duration::from_secs(5));
    /// ```
    pub fn with_timeout(handle: JoinHandle<T>, timeout: Duration) -> Self {
        Self {
            handle: Some(handle),
            started_at: Instant::now(),
            timeout: Some(timeout),
        }
    }

    /// Attempt to receive the result of the async task without blocking.
    ///
    /// This method uses `JoinHandle::is_finished()` to check if the task has
    /// completed, then uses `try_now()` (via `futures::poll_fn`) to retrieve
    /// the result if available.
    ///
    /// # Returns
    /// - `None`: Task is still running
    /// - `Some(Ok(T))`: Task completed successfully with value `T`
    /// - `Some(Err(e))`: Task failed or timed out
    ///
    /// # Timeout Behavior
    /// If a timeout was set via `with_timeout()` and has elapsed, this will
    /// return `Some(Err(...))` even if the task is still running. The task
    /// will be aborted when the `AsyncTask` is dropped.
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::AsyncTask;
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new().unwrap();
    /// let handle = rt.spawn(async { 42 });
    /// let mut task = AsyncTask::new(handle);
    ///
    /// match task.try_recv() {
    ///     Some(Ok(value)) => println!("Completed: {}", value),
    ///     Some(Err(e)) => eprintln!("Failed: {}", e),
    ///     None => println!("Still running..."),
    /// }
    /// ```
    pub fn try_recv(&mut self) -> Option<Result<T>> {
        // Get mutable reference to handle
        let handle = self.handle.as_mut()?;

        // Check timeout first (if set)
        if let Some(timeout) = self.timeout {
            if self.started_at.elapsed() > timeout {
                // Timeout exceeded - abort task and return error
                handle.abort();
                self.handle = None; // Take ownership to prevent further use
                return Some(Err(anyhow::anyhow!(
                    "AsyncTask timed out after {:?}",
                    timeout
                )));
            }
        }

        // Check if task is finished (non-blocking)
        if !handle.is_finished() {
            return None;
        }

        // Task is finished - poll it to extract the result
        // Create a no-op waker since we know the task is ready
        use std::task::{RawWaker, RawWakerVTable, Waker};

        unsafe fn clone_raw(_: *const ()) -> RawWaker {
            noop_raw_waker()
        }
        unsafe fn wake_raw(_: *const ()) {}
        unsafe fn wake_by_ref_raw(_: *const ()) {}
        unsafe fn drop_raw(_: *const ()) {}

        fn noop_raw_waker() -> RawWaker {
            let vtable = &RawWakerVTable::new(clone_raw, wake_raw, wake_by_ref_raw, drop_raw);
            RawWaker::new(std::ptr::null(), vtable)
        }

        let waker = unsafe { Waker::from_raw(noop_raw_waker()) };
        let mut cx = TaskContext::from_waker(&waker);

        // Poll the JoinHandle
        match Pin::new(handle).poll(&mut cx) {
            Poll::Ready(Ok(value)) => {
                self.handle = None; // Consume the handle
                Some(Ok(value))
            }
            Poll::Ready(Err(e)) => {
                self.handle = None; // Consume the handle
                Some(Err(anyhow::anyhow!("AsyncTask join error: {}", e)))
            }
            Poll::Pending => {
                // This shouldn't happen since we checked is_finished()
                // But handle gracefully
                None
            }
        }
    }

    /// Check if the underlying task has finished (without consuming the result).
    ///
    /// This is a cheaper operation than `try_recv()` if you only want to know
    /// completion status without retrieving the value.
    ///
    /// # Returns
    /// - `true`: Task has completed (success or failure)
    /// - `false`: Task is still running
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::AsyncTask;
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new().unwrap();
    /// let handle = rt.spawn(async { 42 });
    /// let task = AsyncTask::new(handle);
    ///
    /// if task.is_finished() {
    ///     println!("Task completed!");
    /// }
    /// ```
    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map_or(true, |h| h.is_finished())
    }

    /// Get the time elapsed since the task was created.
    ///
    /// This is useful for monitoring task duration and detecting long-running
    /// operations.
    ///
    /// # Returns
    /// Duration since task creation
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::AsyncTask;
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new().unwrap();
    /// let handle = rt.spawn(async { 42 });
    /// let task = AsyncTask::new(handle);
    ///
    /// println!("Task running for: {:?}", task.elapsed());
    /// ```
    pub fn elapsed(&self) -> Duration {
        self.started_at.elapsed()
    }

    /// Consume the `AsyncTask` and block until the underlying task completes.
    ///
    /// ⚠️ **WARNING**: This is a blocking operation and should NOT be used in
    /// the AI Arbiter update loop. It's provided for testing and shutdown scenarios.
    ///
    /// # Returns
    /// The result of the async task
    ///
    /// # Example
    /// ```no_run
    /// use astraweave_ai::AsyncTask;
    /// use tokio::runtime::Runtime;
    ///
    /// let rt = Runtime::new().unwrap();
    /// let handle = rt.spawn(async { 42 });
    /// let task = AsyncTask::new(handle);
    ///
    /// let result = rt.block_on(async { task.await_result().await }).unwrap();
    /// assert_eq!(result, 42);
    /// ```
    pub async fn await_result(mut self) -> Result<T> {
        let handle = self
            .handle
            .take()
            .ok_or_else(|| anyhow::anyhow!("AsyncTask already consumed"))?;

        handle
            .await
            .context("AsyncTask join failed during await_result")
    }
}

impl<T> Drop for AsyncTask<T> {
    /// Abort the underlying task when `AsyncTask` is dropped.
    ///
    /// This ensures that async tasks don't continue running after the
    /// `AsyncTask` handle is discarded.
    fn drop(&mut self) {
        if let Some(ref handle) = self.handle {
            if !handle.is_finished() {
                handle.abort();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_async_task_pending() {
        // Spawn a long-running task
        let handle = tokio::spawn(async {
            sleep(Duration::from_secs(10)).await;
            42
        });

        let mut task = AsyncTask::new(handle);

        // Task should not be finished immediately
        assert!(!task.is_finished());

        // try_recv should return None (still running)
        assert!(task.try_recv().is_none());
    }

    #[tokio::test]
    async fn test_async_task_complete() {
        // Spawn an instant task
        let handle = tokio::spawn(async { 42 });

        // Give tokio a chance to complete the task
        sleep(Duration::from_millis(10)).await;

        let mut task = AsyncTask::new(handle);

        // Task should be finished
        assert!(task.is_finished());

        // try_recv should return Some(Ok(42))
        match task.try_recv() {
            Some(Ok(value)) => assert_eq!(value, 42),
            other => panic!("Expected Some(Ok(42)), got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_async_task_timeout() {
        // Spawn a task that takes longer than timeout
        let handle = tokio::spawn(async {
            sleep(Duration::from_secs(10)).await;
            42
        });

        let mut task = AsyncTask::with_timeout(handle, Duration::from_millis(100));

        // Wait for timeout to elapse
        sleep(Duration::from_millis(150)).await;

        // try_recv should return timeout error
        match task.try_recv() {
            Some(Err(e)) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("timed out"),
                    "Expected timeout error, got: {}",
                    err_msg
                );
            }
            other => panic!("Expected timeout error, got {:?}", other),
        }
    }

    #[tokio::test]
    async fn test_async_task_elapsed() {
        let handle = tokio::spawn(async { 42 });
        let task = AsyncTask::new(handle);

        // Elapsed should be approximately zero initially
        assert!(task.elapsed() < Duration::from_millis(10));

        // Wait a bit
        sleep(Duration::from_millis(50)).await;

        // Elapsed should be around 50ms
        let elapsed = task.elapsed();
        assert!(
            elapsed >= Duration::from_millis(45) && elapsed <= Duration::from_millis(100),
            "Expected elapsed ~50ms, got {:?}",
            elapsed
        );
    }

    #[tokio::test]
    async fn test_async_task_block_on() {
        let handle = tokio::spawn(async {
            sleep(Duration::from_millis(10)).await;
            42
        });

        let task = AsyncTask::new(handle);

        // await_result should wait for completion
        let result = task.await_result().await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[tokio::test]
    async fn test_async_task_abort_on_drop() {
        let handle = tokio::spawn(async {
            sleep(Duration::from_secs(10)).await;
            42
        });

        let task = AsyncTask::new(handle);
        assert!(!task.is_finished());

        // Drop the task (should abort underlying JoinHandle)
        drop(task);

        // Give tokio a chance to process the abort
        sleep(Duration::from_millis(10)).await;

        // If we try to join the original handle, it should be aborted
        // (but we can't access it after moving into AsyncTask)
        // So this test just verifies Drop doesn't panic
    }

    #[tokio::test]
    async fn test_async_task_with_error() {
        // Spawn a task that panics
        let handle = tokio::spawn(async {
            panic!("Test panic");
        });

        // Give tokio a chance to complete (and panic)
        sleep(Duration::from_millis(10)).await;

        let mut task = AsyncTask::new(handle);

        // try_recv should return error (join error from panic)
        match task.try_recv() {
            Some(Err(e)) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("join error"),
                    "Expected join error, got: {}",
                    err_msg
                );
            }
            other => panic!("Expected join error, got {:?}", other),
        }
    }
}
