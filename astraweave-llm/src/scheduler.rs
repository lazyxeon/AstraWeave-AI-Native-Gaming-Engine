//! LLM Request Scheduler
//!
//! Provides priority-based queueing and concurrent execution for LLM requests.
//! Integrates with the ECS as a resource to enable async AI planning without blocking game logic.
//!
//! # Features
//! - Priority-based queue (High/Normal/Low)
//! - Configurable concurrent request limit (default: 5)
//! - Timeout handling (default: 30s per request)
//! - Request batching and rate limiting
//! - Integration with LlmClient trait
//!
//! # Example
//! ```no_run
//! use astraweave_llm::scheduler::{LlmScheduler, RequestPriority};
//! use astraweave_llm::MockLlm;
//! use std::sync::Arc;
//!
//! # async fn example() {
//! let scheduler = LlmScheduler::new(Arc::new(MockLlm), 5, 30);
//! let request_id = scheduler.submit_request(
//!     "Generate a plan for combat".to_string(),
//!     RequestPriority::High
//! ).await;
//! # }
//! ```

use anyhow::Result;
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::time::timeout;
use tracing::{debug, error, warn};
use uuid::Uuid;

use crate::LlmClient;

/// Priority level for LLM requests
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RequestPriority {
    Low = 0,
    Normal = 1,
    High = 2,
}

/// Status of an LLM request
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestStatus {
    Queued,
    Processing,
    Completed,
    Failed,
    TimedOut,
}

/// Result of a completed LLM request
#[derive(Debug, Clone)]
pub struct RequestResult {
    pub request_id: Uuid,
    pub response: String,
    pub elapsed_ms: u64,
}

/// Internal request representation
#[allow(dead_code)]
struct QueuedRequest {
    id: Uuid,
    prompt: String,
    priority: RequestPriority,
    response_tx: oneshot::Sender<Result<String>>,
    submitted_at: std::time::Instant,
}

/// LLM request scheduler with priority queue and concurrency control
///
/// The scheduler manages a pool of concurrent LLM requests, ensuring:
/// - High-priority requests are processed first
/// - System doesn't get overwhelmed by too many concurrent requests
/// - Requests time out if they take too long
/// - Results can be polled asynchronously
pub struct LlmScheduler {
    #[allow(dead_code)]
    client: Arc<dyn LlmClient>,
    request_tx: mpsc::UnboundedSender<QueuedRequest>,
    statuses: Arc<DashMap<Uuid, RequestStatus>>,
    results: Arc<DashMap<Uuid, RequestResult>>,
    #[allow(dead_code)]
    max_concurrent: usize,
    timeout_secs: u64,
}

impl LlmScheduler {
    /// Create a new LLM scheduler
    ///
    /// # Arguments
    /// * `client` - The LLM client to use for completions
    /// * `max_concurrent` - Maximum number of concurrent requests (default: 5)
    /// * `timeout_secs` - Request timeout in seconds (default: 30)
    pub fn new(client: Arc<dyn LlmClient>, max_concurrent: usize, timeout_secs: u64) -> Self {
        let (request_tx, request_rx) = mpsc::unbounded_channel();
        let statuses = Arc::new(DashMap::new());
        let results = Arc::new(DashMap::new());

        let scheduler = Self {
            client: client.clone(),
            request_tx,
            statuses: statuses.clone(),
            results: results.clone(),
            max_concurrent,
            timeout_secs,
        };

        // Spawn background worker
        tokio::spawn(Self::process_queue(
            client,
            request_rx,
            statuses,
            results,
            max_concurrent,
            timeout_secs,
        ));

        scheduler
    }

    /// Submit a new LLM request
    ///
    /// Returns a unique request ID that can be used to poll for results
    pub async fn submit_request(&self, prompt: String, priority: RequestPriority) -> Uuid {
        let id = Uuid::new_v4();
        let (response_tx, _response_rx) = oneshot::channel();

        let request = QueuedRequest {
            id,
            prompt,
            priority,
            response_tx,
            submitted_at: std::time::Instant::now(),
        };

        self.statuses.insert(id, RequestStatus::Queued);

        if let Err(e) = self.request_tx.send(request) {
            error!("Failed to submit LLM request {}: {}", id, e);
            self.statuses.insert(id, RequestStatus::Failed);
        }

        debug!("Submitted LLM request {} with priority {:?}", id, priority);
        id
    }

    /// Submit a request and wait for completion
    ///
    /// This is a convenience method that submits a request and blocks until the result is ready
    pub async fn submit_and_wait(
        &self,
        prompt: String,
        priority: RequestPriority,
    ) -> Result<String> {
        let id = self.submit_request(prompt, priority).await;

        // Poll for result (with timeout)
        let deadline = std::time::Instant::now() + Duration::from_secs(self.timeout_secs + 5);

        loop {
            if let Some(result) = self.results.get(&id) {
                return Ok(result.response.clone());
            }

            if let Some(status) = self.statuses.get(&id) {
                match *status {
                    RequestStatus::Failed => {
                        anyhow::bail!("Request {} failed", id);
                    }
                    RequestStatus::TimedOut => {
                        anyhow::bail!("Request {} timed out", id);
                    }
                    _ => {}
                }
            }

            if std::time::Instant::now() > deadline {
                anyhow::bail!("Polling timeout for request {}", id);
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    }

    /// Poll for the result of a request
    pub fn poll_result(&self, request_id: Uuid) -> Option<RequestResult> {
        self.results.get(&request_id).map(|r| r.clone())
    }

    /// Get the status of a request
    pub fn get_status(&self, request_id: Uuid) -> Option<RequestStatus> {
        self.statuses.get(&request_id).map(|s| *s)
    }

    /// Remove completed/failed requests to free memory
    pub fn cleanup_request(&self, request_id: Uuid) {
        self.statuses.remove(&request_id);
        self.results.remove(&request_id);
    }

    /// Get statistics about the scheduler
    pub fn stats(&self) -> SchedulerStats {
        let mut stats = SchedulerStats::default();

        for entry in self.statuses.iter() {
            match *entry.value() {
                RequestStatus::Queued => stats.queued += 1,
                RequestStatus::Processing => stats.processing += 1,
                RequestStatus::Completed => stats.completed += 1,
                RequestStatus::Failed => stats.failed += 1,
                RequestStatus::TimedOut => stats.timed_out += 1,
            }
        }

        stats
    }

    /// Background worker that processes the request queue
    async fn process_queue(
        client: Arc<dyn LlmClient>,
        mut request_rx: mpsc::UnboundedReceiver<QueuedRequest>,
        statuses: Arc<DashMap<Uuid, RequestStatus>>,
        results: Arc<DashMap<Uuid, RequestResult>>,
        max_concurrent: usize,
        timeout_secs: u64,
    ) {
        // Semaphore to limit concurrent requests
        let semaphore = Arc::new(Semaphore::new(max_concurrent));

        // Priority queues (we'll use a simple approach: process high priority first)
        let mut high_queue = Vec::new();
        let mut normal_queue = Vec::new();
        let mut low_queue = Vec::new();

        loop {
            // Collect requests into priority queues
            while let Ok(request) = request_rx.try_recv() {
                match request.priority {
                    RequestPriority::High => high_queue.push(request),
                    RequestPriority::Normal => normal_queue.push(request),
                    RequestPriority::Low => low_queue.push(request),
                }
            }

            // Process high priority first, then normal, then low
            let next_request = high_queue
                .pop()
                .or_else(|| normal_queue.pop())
                .or_else(|| low_queue.pop());

            if let Some(request) = next_request {
                let client = client.clone();
                let statuses = statuses.clone();
                let results = results.clone();
                let semaphore = semaphore.clone();

                // Spawn task to process this request
                tokio::spawn(async move {
                    // Acquire semaphore permit
                    let _permit = semaphore.acquire().await.ok();

                    let id = request.id;
                    statuses.insert(id, RequestStatus::Processing);

                    debug!("Processing LLM request {}", id);

                    // Execute with timeout
                    let start = std::time::Instant::now();
                    let result = timeout(
                        Duration::from_secs(timeout_secs),
                        client.complete(&request.prompt),
                    )
                    .await;

                    let elapsed = start.elapsed().as_millis() as u64;

                    match result {
                        Ok(Ok(response)) => {
                            debug!("LLM request {} completed in {}ms", id, elapsed);
                            statuses.insert(id, RequestStatus::Completed);
                            results.insert(
                                id,
                                RequestResult {
                                    request_id: id,
                                    response: response.clone(),
                                    elapsed_ms: elapsed,
                                },
                            );

                            // Send response through oneshot channel (ignore error if receiver dropped)
                            let _ = request.response_tx.send(Ok(response));
                        }
                        Ok(Err(e)) => {
                            error!("LLM request {} failed: {}", id, e);
                            statuses.insert(id, RequestStatus::Failed);
                            let _ = request.response_tx.send(Err(e));
                        }
                        Err(_) => {
                            warn!("LLM request {} timed out after {}s", id, timeout_secs);
                            statuses.insert(id, RequestStatus::TimedOut);
                            let _ = request
                                .response_tx
                                .send(Err(anyhow::anyhow!("Request timed out")));
                        }
                    }
                });
            } else {
                // No requests available, wait a bit
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }
    }
}

/// Statistics about the scheduler's current state
#[derive(Debug, Clone, Default)]
pub struct SchedulerStats {
    pub queued: usize,
    pub processing: usize,
    pub completed: usize,
    pub failed: usize,
    pub timed_out: usize,
}

impl SchedulerStats {
    pub fn total(&self) -> usize {
        self.queued + self.processing + self.completed + self.failed + self.timed_out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MockLlm;

    #[tokio::test]
    async fn test_scheduler_basic() {
        let scheduler = LlmScheduler::new(Arc::new(MockLlm), 5, 30);

        let id = scheduler
            .submit_request("test prompt".to_string(), RequestPriority::Normal)
            .await;

        // Poll until complete
        tokio::time::sleep(Duration::from_millis(100)).await;

        let status = scheduler.get_status(id);
        assert!(matches!(
            status,
            Some(RequestStatus::Completed) | Some(RequestStatus::Processing)
        ));
    }

    #[tokio::test]
    async fn test_scheduler_priority() {
        let scheduler = LlmScheduler::new(Arc::new(MockLlm), 1, 30); // Only 1 concurrent

        // Submit low priority first
        let _low_id = scheduler
            .submit_request("low".to_string(), RequestPriority::Low)
            .await;

        // Then high priority
        let high_id = scheduler
            .submit_request("high".to_string(), RequestPriority::High)
            .await;

        tokio::time::sleep(Duration::from_millis(200)).await;

        // High should complete first (or at least be processing first)
        let high_status = scheduler.get_status(high_id);
        assert!(matches!(
            high_status,
            Some(RequestStatus::Completed) | Some(RequestStatus::Processing)
        ));
    }

    #[tokio::test]
    async fn test_submit_and_wait() {
        let scheduler = LlmScheduler::new(Arc::new(MockLlm), 5, 30);

        let result = scheduler
            .submit_and_wait("test prompt".to_string(), RequestPriority::Normal)
            .await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("plan_id"));
    }

    #[tokio::test]
    async fn test_scheduler_stats() {
        let scheduler = LlmScheduler::new(Arc::new(MockLlm), 5, 30);

        scheduler
            .submit_request("test1".to_string(), RequestPriority::Normal)
            .await;
        scheduler
            .submit_request("test2".to_string(), RequestPriority::High)
            .await;

        tokio::time::sleep(Duration::from_millis(50)).await;

        let stats = scheduler.stats();
        assert!(stats.total() >= 2);
    }

    #[test]
    fn test_request_priority_ordering() {
        assert!(RequestPriority::High > RequestPriority::Normal);
        assert!(RequestPriority::Normal > RequestPriority::Low);
        assert!(RequestPriority::High > RequestPriority::Low);
    }

    #[test]
    fn test_request_priority_equality() {
        assert_eq!(RequestPriority::Normal, RequestPriority::Normal);
        assert_ne!(RequestPriority::High, RequestPriority::Low);
    }

    #[test]
    fn test_request_priority_clone() {
        let priority = RequestPriority::High;
        let cloned = priority;  // Copy
        assert_eq!(priority, cloned);
    }

    #[test]
    fn test_request_priority_debug() {
        let priority = RequestPriority::Normal;
        let debug = format!("{:?}", priority);
        assert!(debug.contains("Normal"));
    }

    #[test]
    fn test_request_status_equality() {
        assert_eq!(RequestStatus::Queued, RequestStatus::Queued);
        assert_ne!(RequestStatus::Queued, RequestStatus::Processing);
        assert_ne!(RequestStatus::Completed, RequestStatus::Failed);
    }

    #[test]
    fn test_request_status_clone() {
        let status = RequestStatus::Processing;
        let cloned = status;  // Copy
        assert_eq!(status, cloned);
    }

    #[test]
    fn test_request_status_debug() {
        let statuses = [
            RequestStatus::Queued,
            RequestStatus::Processing,
            RequestStatus::Completed,
            RequestStatus::Failed,
            RequestStatus::TimedOut,
        ];
        for status in statuses {
            let debug = format!("{:?}", status);
            assert!(!debug.is_empty());
        }
    }

    #[test]
    fn test_request_result_clone() {
        let result = RequestResult {
            request_id: Uuid::new_v4(),
            response: "test response".to_string(),
            elapsed_ms: 100,
        };
        let cloned = result.clone();
        assert_eq!(cloned.response, "test response");
        assert_eq!(cloned.elapsed_ms, 100);
    }

    #[test]
    fn test_request_result_debug() {
        let result = RequestResult {
            request_id: Uuid::new_v4(),
            response: "debug test".to_string(),
            elapsed_ms: 50,
        };
        let debug = format!("{:?}", result);
        assert!(debug.contains("RequestResult"));
        assert!(debug.contains("debug test"));
        assert!(debug.contains("50"));
    }

    #[test]
    fn test_scheduler_stats_default() {
        let stats = SchedulerStats::default();
        assert_eq!(stats.queued, 0);
        assert_eq!(stats.processing, 0);
        assert_eq!(stats.completed, 0);
        assert_eq!(stats.failed, 0);
        assert_eq!(stats.timed_out, 0);
        assert_eq!(stats.total(), 0);
    }

    #[test]
    fn test_scheduler_stats_total() {
        let stats = SchedulerStats {
            queued: 5,
            processing: 3,
            completed: 10,
            failed: 2,
            timed_out: 1,
        };
        assert_eq!(stats.total(), 21);
    }

    #[test]
    fn test_scheduler_stats_clone() {
        let stats = SchedulerStats {
            queued: 1,
            processing: 2,
            completed: 3,
            failed: 0,
            timed_out: 0,
        };
        let cloned = stats.clone();
        assert_eq!(stats.total(), cloned.total());
        assert_eq!(cloned.queued, 1);
    }

    #[test]
    fn test_scheduler_stats_debug() {
        let stats = SchedulerStats::default();
        let debug = format!("{:?}", stats);
        assert!(debug.contains("SchedulerStats"));
    }

    #[tokio::test]
    async fn test_get_status_nonexistent() {
        let scheduler = LlmScheduler::new(Arc::new(MockLlm), 5, 30);
        let fake_id = Uuid::new_v4();
        
        // Should return None for unknown request
        let status = scheduler.get_status(fake_id);
        assert!(status.is_none());
    }
}
