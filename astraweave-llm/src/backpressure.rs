use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, RwLock, Semaphore};
use tracing::{debug, info};
use uuid::Uuid;

/// Backpressure management system for LLM requests
pub struct BackpressureManager {
    /// Request queues by priority
    queues: Arc<RwLock<PriorityQueues>>,
    /// Active request tracking
    active_requests: Arc<RwLock<HashMap<String, ActiveRequest>>>,
    /// Concurrency control
    semaphore: Arc<Semaphore>,
    /// Configuration
    config: BackpressureConfig,
    /// Metrics
    metrics: Arc<RwLock<BackpressureMetrics>>,
    /// Queue processor handle
    processor_handle: Option<tokio::task::JoinHandle<()>>,
}

#[derive(Debug, Clone)]
pub struct BackpressureConfig {
    /// Maximum concurrent requests
    pub max_concurrent_requests: usize,
    /// Maximum queue size per priority
    pub max_queue_size: usize,
    /// Request timeout
    pub request_timeout: Duration,
    /// Queue processing interval
    pub processing_interval: Duration,
    /// Enable adaptive concurrency
    pub adaptive_concurrency: bool,
    /// Target latency for adaptive adjustment (ms)
    pub target_latency_ms: u64,
    /// Load shedding threshold (0.0 to 1.0)
    pub load_shedding_threshold: f32,
    /// Enable graceful degradation
    pub enable_graceful_degradation: bool,
}

impl Default for BackpressureConfig {
    fn default() -> Self {
        Self {
            max_concurrent_requests: 100,
            max_queue_size: 1000,
            request_timeout: Duration::from_secs(30),
            processing_interval: Duration::from_millis(10),
            adaptive_concurrency: true,
            target_latency_ms: 1000,
            load_shedding_threshold: 0.9,
            enable_graceful_degradation: true,
        }
    }
}

/// Priority levels for request queuing
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Priority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

impl Priority {
    pub fn all() -> Vec<Priority> {
        vec![
            Priority::Critical,
            Priority::High,
            Priority::Normal,
            Priority::Low,
            Priority::Background,
        ]
    }
}

/// Queued request
#[derive(Debug)]
struct QueuedRequest {
    id: String,
    priority: Priority,
    queued_at: Instant,
    timeout: Duration,
    sender: oneshot::Sender<Result<()>>,
    metadata: RequestMetadata,
}

/// Request metadata for processing decisions
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    pub user_id: Option<String>,
    pub model: String,
    pub estimated_tokens: u32,
    pub estimated_cost: f64,
    pub tags: HashMap<String, String>,
}

/// Active request tracking
#[derive(Debug)]
#[allow(dead_code)]
struct ActiveRequest {
    id: String,
    priority: Priority,
    started_at: Instant,
    metadata: RequestMetadata,
}

/// Priority-based queues
#[derive(Debug, Default)]
struct PriorityQueues {
    queues: HashMap<Priority, VecDeque<QueuedRequest>>,
}

impl PriorityQueues {
    fn new() -> Self {
        let mut queues = HashMap::new();
        for priority in Priority::all() {
            queues.insert(priority, VecDeque::new());
        }
        Self { queues }
    }

    fn enqueue(&mut self, request: QueuedRequest, max_size: usize) -> Result<()> {
        let queue = self
            .queues
            .get_mut(&request.priority)
            .ok_or_else(|| anyhow!("Invalid priority: {:?}", request.priority))?;

        if queue.len() >= max_size {
            return Err(anyhow!("Queue full for priority {:?}", request.priority));
        }

        queue.push_back(request);
        Ok(())
    }

    fn dequeue_highest_priority(&mut self) -> Option<QueuedRequest> {
        for priority in Priority::all() {
            if let Some(queue) = self.queues.get_mut(&priority) {
                if let Some(request) = queue.pop_front() {
                    return Some(request);
                }
            }
        }
        None
    }

    fn total_queued(&self) -> usize {
        self.queues.values().map(|q| q.len()).sum()
    }

    fn queued_by_priority(&self, priority: Priority) -> usize {
        self.queues.get(&priority).map(|q| q.len()).unwrap_or(0)
    }

    fn remove_expired(&mut self, now: Instant) -> Vec<QueuedRequest> {
        let mut expired = Vec::new();

        for queue in self.queues.values_mut() {
            let mut i = 0;
            while i < queue.len() {
                if let Some(request) = queue.get(i) {
                    if now.duration_since(request.queued_at) >= request.timeout {
                        if let Some(expired_request) = queue.remove(i) {
                            expired.push(expired_request);
                        }
                        continue;
                    }
                }
                i += 1;
            }
        }

        expired
    }
}

/// Backpressure management result
#[derive(Debug)]
pub enum BackpressureResult {
    /// Request was accepted and can proceed immediately
    Accepted,
    /// Request was queued and will be processed later
    Queued {
        position: usize,
        estimated_wait: Duration,
    },
    /// Request was rejected due to system overload
    Rejected {
        reason: String,
        retry_after: Option<Duration>,
    },
    /// Request was degraded (lower quality response)
    Degraded { reason: String },
}

/// Backpressure metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct BackpressureMetrics {
    pub total_requests: u64,
    pub accepted_requests: u64,
    pub queued_requests: u64,
    pub rejected_requests: u64,
    pub degraded_requests: u64,
    pub expired_requests: u64,
    pub current_queue_size: usize,
    pub current_active_requests: usize,
    pub average_queue_time_ms: f32,
    pub average_processing_time_ms: f32,
    pub queue_sizes_by_priority: HashMap<Priority, usize>,
    pub load_factor: f32,
    pub adaptive_concurrency_limit: usize,
    pub last_updated: String,
}

impl BackpressureManager {
    pub fn new(config: BackpressureConfig) -> Self {
        Self {
            queues: Arc::new(RwLock::new(PriorityQueues::new())),
            active_requests: Arc::new(RwLock::new(HashMap::new())),
            semaphore: Arc::new(Semaphore::new(config.max_concurrent_requests)),
            config,
            metrics: Arc::new(RwLock::new(BackpressureMetrics::default())),
            processor_handle: None,
        }
    }

    /// Start the background queue processor
    pub async fn start(&mut self) -> Result<()> {
        if self.processor_handle.is_some() {
            return Err(anyhow!("Backpressure manager is already running"));
        }

        let queues = self.queues.clone();
        let active_requests = self.active_requests.clone();
        let semaphore = self.semaphore.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();

        let handle = tokio::spawn(async move {
            Self::queue_processor(queues, active_requests, semaphore, metrics, config).await;
        });

        self.processor_handle = Some(handle);
        info!("Started backpressure manager");
        Ok(())
    }

    /// Stop the background queue processor
    pub async fn stop(&mut self) {
        if let Some(handle) = self.processor_handle.take() {
            handle.abort();
            info!("Stopped backpressure manager");
        }
    }

    /// Submit a request for processing
    pub async fn submit_request(
        &self,
        priority: Priority,
        timeout: Option<Duration>,
        metadata: RequestMetadata,
    ) -> Result<BackpressureResult> {
        let request_id = Uuid::new_v4().to_string();
        let timeout = timeout.unwrap_or(self.config.request_timeout);

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_requests += 1;
        }

        // Check if we should reject due to overload
        if self.should_reject_request(priority).await {
            let mut metrics = self.metrics.write().await;
            metrics.rejected_requests += 1;
            return Ok(BackpressureResult::Rejected {
                reason: "System overloaded".to_string(),
                retry_after: Some(Duration::from_secs(1)),
            });
        }

        // Check if we should degrade the request
        if self.should_degrade_request(priority).await {
            let mut metrics = self.metrics.write().await;
            metrics.degraded_requests += 1;
            return Ok(BackpressureResult::Degraded {
                reason: "High load - using faster but lower quality model".to_string(),
            });
        }

        // Try to acquire semaphore immediately
        if let Ok(permit) = self.semaphore.clone().try_acquire_owned() {
            // Can proceed immediately
            let active_request = ActiveRequest {
                id: request_id.clone(),
                priority,
                started_at: Instant::now(),
                metadata,
            };

            {
                let mut active_requests = self.active_requests.write().await;
                active_requests.insert(request_id, active_request);
            }

            {
                let mut metrics = self.metrics.write().await;
                metrics.accepted_requests += 1;
                metrics.current_active_requests = self.active_requests.read().await.len();
            }

            // Release permit when done (in practice, this would be done by the caller)
            tokio::spawn(async move {
                // Simulate some processing time
                tokio::time::sleep(Duration::from_millis(100)).await;
                drop(permit);
            });

            return Ok(BackpressureResult::Accepted);
        }

        // Need to queue the request
        let (sender, _receiver) = oneshot::channel();
        let queued_request = QueuedRequest {
            id: request_id,
            priority,
            queued_at: Instant::now(),
            timeout,
            sender,
            metadata,
        };

        let position = {
            let mut queues = self.queues.write().await;

            queues.enqueue(queued_request, self.config.max_queue_size)?;

            // Calculate position in queue (higher priority requests go first)
            let mut position = 0;
            for p in Priority::all() {
                if p < priority {
                    position += queues.queued_by_priority(p);
                } else if p == priority {
                    position += queues.queued_by_priority(p) - 1; // -1 because we just added this request
                    break;
                }
            }
            position + 1 // 1-indexed position
        };

        {
            let mut metrics = self.metrics.write().await;
            metrics.queued_requests += 1;
            metrics.current_queue_size = self.queues.read().await.total_queued();
        }

        let estimated_wait = self.estimate_wait_time(position).await;

        Ok(BackpressureResult::Queued {
            position,
            estimated_wait,
        })
    }

    /// Complete a request
    pub async fn complete_request(&self, request_id: &str, success: bool) -> Result<()> {
        let processing_time = {
            let mut active_requests = self.active_requests.write().await;
            if let Some(request) = active_requests.remove(request_id) {
                Some(request.started_at.elapsed())
            } else {
                None
            }
        };

        if let Some(duration) = processing_time {
            let mut metrics = self.metrics.write().await;
            let current_avg = metrics.average_processing_time_ms;
            let new_sample = duration.as_millis() as f32;
            metrics.average_processing_time_ms = if current_avg == 0.0 {
                new_sample
            } else {
                0.1 * new_sample + 0.9 * current_avg
            };
            metrics.current_active_requests = self.active_requests.read().await.len();

            // Adjust adaptive concurrency if enabled
            if self.config.adaptive_concurrency {
                self.adjust_concurrency(&mut metrics, success, duration)
                    .await;
            }
        }

        debug!("Completed request {} (success: {})", request_id, success);
        Ok(())
    }

    /// Get current metrics
    pub async fn get_metrics(&self) -> BackpressureMetrics {
        let mut metrics = self.metrics.read().await.clone();
        let queues = self.queues.read().await;

        metrics.current_queue_size = queues.total_queued();
        metrics.current_active_requests = self.active_requests.read().await.len();
        metrics.load_factor =
            metrics.current_active_requests as f32 / metrics.adaptive_concurrency_limit as f32;

        for priority in Priority::all() {
            metrics
                .queue_sizes_by_priority
                .insert(priority, queues.queued_by_priority(priority));
        }

        metrics.last_updated = chrono::Utc::now().to_rfc3339();
        metrics
    }

    /// Background queue processor
    async fn queue_processor(
        queues: Arc<RwLock<PriorityQueues>>,
        active_requests: Arc<RwLock<HashMap<String, ActiveRequest>>>,
        semaphore: Arc<Semaphore>,
        metrics: Arc<RwLock<BackpressureMetrics>>,
        config: BackpressureConfig,
    ) {
        let mut processing_interval = tokio::time::interval(config.processing_interval);

        loop {
            processing_interval.tick().await;

            // Remove expired requests
            let expired_requests = {
                let mut queues = queues.write().await;
                queues.remove_expired(Instant::now())
            };

            for expired in expired_requests {
                let _ = expired.sender.send(Err(anyhow!("Request timed out")));
                let mut metrics = metrics.write().await;
                metrics.expired_requests += 1;
            }

            // Process queued requests
            while let Ok(permit) = semaphore.clone().try_acquire_owned() {
                let queued_request = {
                    let mut queues = queues.write().await;
                    queues.dequeue_highest_priority()
                };

                if let Some(request) = queued_request {
                    let queue_time = request.queued_at.elapsed();

                    // Update queue time metrics
                    {
                        let mut metrics = metrics.write().await;
                        let current_avg = metrics.average_queue_time_ms;
                        let new_sample = queue_time.as_millis() as f32;
                        metrics.average_queue_time_ms = if current_avg == 0.0 {
                            new_sample
                        } else {
                            0.1 * new_sample + 0.9 * current_avg
                        };
                        metrics.accepted_requests += 1;
                    }

                    // Track as active request
                    let active_request = ActiveRequest {
                        id: request.id.clone(),
                        priority: request.priority,
                        started_at: Instant::now(),
                        metadata: request.metadata,
                    };

                    {
                        let mut active_requests = active_requests.write().await;
                        active_requests.insert(request.id, active_request);
                    }

                    // Notify that request can proceed
                    let _ = request.sender.send(Ok(()));

                    // Release permit after some time (this would be done by the actual request processor)
                    tokio::spawn(async move {
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        drop(permit);
                    });
                } else {
                    // No more queued requests
                    drop(permit);
                    break;
                }
            }
        }
    }

    /// Check if request should be rejected due to overload
    async fn should_reject_request(&self, priority: Priority) -> bool {
        if !self.config.enable_graceful_degradation {
            return false;
        }

        let metrics = self.metrics.read().await;
        let load_factor =
            metrics.current_active_requests as f32 / metrics.adaptive_concurrency_limit as f32;

        // Only reject lower priority requests when system is heavily loaded
        match priority {
            Priority::Critical => false,
            Priority::High => load_factor > 0.99,
            Priority::Normal => load_factor > 0.95,
            Priority::Low => load_factor > self.config.load_shedding_threshold,
            Priority::Background => load_factor > 0.8,
        }
    }

    /// Check if request should be degraded
    async fn should_degrade_request(&self, priority: Priority) -> bool {
        if !self.config.enable_graceful_degradation {
            return false;
        }

        let metrics = self.metrics.read().await;
        let load_factor =
            metrics.current_active_requests as f32 / metrics.adaptive_concurrency_limit as f32;

        // Degrade lower priority requests when system is moderately loaded
        match priority {
            Priority::Critical | Priority::High => false,
            Priority::Normal => load_factor > 0.8,
            Priority::Low => load_factor > 0.7,
            Priority::Background => load_factor > 0.6,
        }
    }

    /// Estimate wait time for a queued request
    async fn estimate_wait_time(&self, position: usize) -> Duration {
        let metrics = self.metrics.read().await;
        let avg_processing_time = Duration::from_millis(metrics.average_processing_time_ms as u64);
        let concurrent_capacity = metrics.adaptive_concurrency_limit;

        if concurrent_capacity > 0 {
            let estimated_cycles = position.div_ceil(concurrent_capacity);
            avg_processing_time * estimated_cycles as u32
        } else {
            Duration::from_secs(60) // Fallback estimate
        }
    }

    /// Adjust adaptive concurrency based on performance
    async fn adjust_concurrency(
        &self,
        metrics: &mut BackpressureMetrics,
        success: bool,
        latency: Duration,
    ) {
        let current_limit = metrics.adaptive_concurrency_limit.max(1);
        let target_latency = Duration::from_millis(self.config.target_latency_ms);

        let new_limit = if success && latency < target_latency {
            // Performance is good, try increasing concurrency
            (current_limit as f32 * 1.05).min(self.config.max_concurrent_requests as f32) as usize
        } else if !success || latency > target_latency * 2 {
            // Performance is poor, decrease concurrency
            (current_limit as f32 * 0.9).max(1.0) as usize
        } else {
            current_limit
        };

        if new_limit != current_limit {
            metrics.adaptive_concurrency_limit = new_limit;
            debug!(
                "Adjusted adaptive concurrency limit: {} -> {}",
                current_limit, new_limit
            );
        }
    }
}

impl Drop for BackpressureManager {
    fn drop(&mut self) {
        if let Some(handle) = self.processor_handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, timeout};

    #[tokio::test]
    async fn test_backpressure_manager_creation() {
        let config = BackpressureConfig::default();
        let manager = BackpressureManager::new(config);

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_requests, 0);
    }

    #[tokio::test]
    async fn test_request_acceptance() {
        let mut manager = BackpressureManager::new(BackpressureConfig::default());
        manager.start().await.unwrap();

        let metadata = RequestMetadata {
            user_id: Some("test_user".to_string()),
            model: "gpt-3.5-turbo".to_string(),
            estimated_tokens: 100,
            estimated_cost: 0.01,
            tags: HashMap::new(),
        };

        let result = manager
            .submit_request(Priority::Normal, None, metadata)
            .await
            .unwrap();

        match result {
            BackpressureResult::Accepted => {
                // Expected result
            }
            _ => panic!("Expected request to be accepted"),
        }

        manager.stop().await;
    }

    #[tokio::test]
    async fn test_request_queuing() {
        let config = BackpressureConfig {
            max_concurrent_requests: 1,         // Force queuing
            enable_graceful_degradation: false, // Disable rejection logic for this test
            ..Default::default()
        };
        let mut manager = BackpressureManager::new(config);
        manager.start().await.unwrap();

        let metadata = RequestMetadata {
            user_id: Some("test_user".to_string()),
            model: "gpt-3.5-turbo".to_string(),
            estimated_tokens: 100,
            estimated_cost: 0.01,
            tags: HashMap::new(),
        };

        // First request should be accepted
        let result1 = manager
            .submit_request(Priority::Normal, None, metadata.clone())
            .await
            .unwrap();

        // Verify first request was accepted
        assert!(
            matches!(result1, BackpressureResult::Accepted),
            "First request should be accepted, got: {:?}",
            result1
        );

        // Second request should be queued immediately (semaphore exhausted)
        // No delay needed - the permit is held for 100ms, so it's definitely still held
        let result2 = manager
            .submit_request(Priority::Normal, None, metadata)
            .await
            .unwrap();

        match &result2 {
            BackpressureResult::Queued { position, .. } => {
                assert_eq!(*position, 1);
            }
            _ => panic!("Expected request to be queued, got: {:?}", result2),
        }

        manager.stop().await;
    }

    #[tokio::test]
    async fn test_priority_ordering() {
        let config = BackpressureConfig {
            max_concurrent_requests: 0, // Force all requests to queue
            ..Default::default()
        };
        let mut manager = BackpressureManager::new(config);
        manager.start().await.unwrap();

        let metadata = RequestMetadata {
            user_id: Some("test_user".to_string()),
            model: "gpt-3.5-turbo".to_string(),
            estimated_tokens: 100,
            estimated_cost: 0.01,
            tags: HashMap::new(),
        };

        // Submit requests in reverse priority order
        let _low = manager
            .submit_request(Priority::Low, None, metadata.clone())
            .await
            .unwrap();
        let _normal = manager
            .submit_request(Priority::Normal, None, metadata.clone())
            .await
            .unwrap();
        let _high = manager
            .submit_request(Priority::High, None, metadata)
            .await
            .unwrap();

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.current_queue_size, 3);

        manager.stop().await;
    }
}
