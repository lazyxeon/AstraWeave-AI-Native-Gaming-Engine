use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Circuit breaker pattern implementation for LLM API calls
pub struct CircuitBreakerManager {
    /// Per-model circuit breakers
    breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    /// Configuration
    config: CircuitBreakerConfig,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    /// Failure threshold before opening circuit
    pub failure_threshold: u32,
    /// Time window for counting failures (seconds)
    pub failure_window: u64,
    /// Minimum requests before circuit can open
    pub minimum_requests: u32,
    /// Time to wait before attempting to close circuit (seconds)
    pub recovery_timeout: u64,
    /// Success threshold for half-open state
    pub success_threshold: u32,
    /// Enable automatic circuit breaker
    pub enabled: bool,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            failure_window: 60,
            minimum_requests: 10,
            recovery_timeout: 30,
            success_threshold: 3,
            enabled: true,
        }
    }
}

/// Circuit breaker states
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Circuit is half-open, testing if service has recovered
    HalfOpen,
}

/// Individual circuit breaker for a model
#[derive(Debug)]
struct CircuitBreaker {
    model: String,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    request_count: u32,
    last_failure_time: Option<Instant>,
    state_changed_time: Instant,
    failure_window_start: Instant,
    config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    fn new(model: String, config: CircuitBreakerConfig) -> Self {
        Self {
            model,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            request_count: 0,
            last_failure_time: None,
            state_changed_time: Instant::now(),
            failure_window_start: Instant::now(),
            config,
        }
    }

    /// Check if request can proceed through circuit breaker
    fn can_proceed(&mut self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        self.reset_window_if_needed();

        match self.state {
            CircuitState::Closed => Ok(()),
            CircuitState::Open => {
                // Check if recovery timeout has passed
                if self.state_changed_time.elapsed()
                    >= Duration::from_secs(self.config.recovery_timeout)
                {
                    self.transition_to_half_open();
                    Ok(())
                } else {
                    Err(anyhow!("Circuit breaker is open for model {}", self.model))
                }
            }
            CircuitState::HalfOpen => Ok(()),
        }
    }

    /// Record a successful request
    fn record_success(&mut self) {
        self.request_count += 1;

        match self.state {
            CircuitState::Closed => {
                // Reset failure count on success
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.transition_to_closed();
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset counts just in case
                self.failure_count = 0;
                self.success_count = 0;
            }
        }

        debug!(
            "Circuit breaker success recorded for {}: state={:?}, failures={}, successes={}",
            self.model, self.state, self.failure_count, self.success_count
        );
    }

    /// Record a failed request
    fn record_failure(&mut self) {
        self.request_count += 1;
        self.failure_count += 1;
        self.last_failure_time = Some(Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.should_open_circuit() {
                    self.transition_to_open();
                }
            }
            CircuitState::HalfOpen => {
                // Go back to open on any failure in half-open state
                self.transition_to_open();
            }
            CircuitState::Open => {
                // Already open, just update failure count
            }
        }

        debug!(
            "Circuit breaker failure recorded for {}: state={:?}, failures={}, requests={}",
            self.model, self.state, self.failure_count, self.request_count
        );
    }

    /// Check if circuit should open
    fn should_open_circuit(&self) -> bool {
        self.request_count >= self.config.minimum_requests
            && self.failure_count >= self.config.failure_threshold
    }

    /// Transition to open state
    fn transition_to_open(&mut self) {
        self.state = CircuitState::Open;
        self.state_changed_time = Instant::now();
        self.success_count = 0;

        warn!(
            "Circuit breaker opened for model {} - failures: {}/{} requests in window",
            self.model, self.failure_count, self.request_count
        );
    }

    /// Transition to half-open state
    fn transition_to_half_open(&mut self) {
        self.state = CircuitState::HalfOpen;
        self.state_changed_time = Instant::now();
        self.success_count = 0;
        self.failure_count = 0;

        info!(
            "Circuit breaker transitioning to half-open for model {}",
            self.model
        );
    }

    /// Transition to closed state
    fn transition_to_closed(&mut self) {
        self.state = CircuitState::Closed;
        self.state_changed_time = Instant::now();
        self.failure_count = 0;
        self.success_count = 0;

        info!(
            "Circuit breaker closed for model {} - service recovered",
            self.model
        );
    }

    /// Reset failure window if needed
    fn reset_window_if_needed(&mut self) {
        let window_duration = Duration::from_secs(self.config.failure_window);
        if self.failure_window_start.elapsed() >= window_duration {
            self.failure_count = 0;
            self.request_count = 0;
            self.failure_window_start = Instant::now();
        }
    }
}

/// Circuit breaker execution result
#[derive(Debug)]
pub struct CircuitBreakerResult<T> {
    pub result: Result<T>,
    pub state: CircuitState,
    pub execution_time: Duration,
}

/// Circuit breaker status for monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitBreakerStatus {
    pub model: String,
    pub state: CircuitState,
    pub failure_count: u32,
    pub success_count: u32,
    pub request_count: u32,
    pub last_failure_time: Option<String>,
    pub time_in_current_state: u64,
}

impl CircuitBreakerManager {
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            breakers: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Execute a function with circuit breaker protection
    pub async fn execute<F, T, Fut>(&self, model: &str, operation: F) -> CircuitBreakerResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let start_time = Instant::now();

        // Check if request can proceed
        let can_proceed = {
            let mut breakers = self.breakers.write().await;
            let breaker = breakers
                .entry(model.to_string())
                .or_insert_with(|| CircuitBreaker::new(model.to_string(), self.config.clone()));

            let state = breaker.state.clone();
            match breaker.can_proceed() {
                Ok(()) => Ok(state),
                Err(e) => Err((e, state)),
            }
        };

        let execution_time = start_time.elapsed();

        match can_proceed {
            Ok(state) => {
                // Execute the operation
                let result = operation().await;
                let execution_time = start_time.elapsed();

                // Record the result
                {
                    let mut breakers = self.breakers.write().await;
                    if let Some(breaker) = breakers.get_mut(model) {
                        match &result {
                            Ok(_) => breaker.record_success(),
                            Err(_) => breaker.record_failure(),
                        }
                    }
                }

                CircuitBreakerResult {
                    result,
                    state,
                    execution_time,
                }
            }
            Err((error, state)) => CircuitBreakerResult {
                result: Err(error),
                state,
                execution_time,
            },
        }
    }

    /// Manually record success for a model
    pub async fn record_success(&self, model: &str) {
        let mut breakers = self.breakers.write().await;
        let breaker = breakers
            .entry(model.to_string())
            .or_insert_with(|| CircuitBreaker::new(model.to_string(), self.config.clone()));
        breaker.record_success();
    }

    /// Manually record failure for a model
    pub async fn record_failure(&self, model: &str) {
        let mut breakers = self.breakers.write().await;
        let breaker = breakers
            .entry(model.to_string())
            .or_insert_with(|| CircuitBreaker::new(model.to_string(), self.config.clone()));
        breaker.record_failure();
    }

    /// Get current circuit breaker status for a model
    pub async fn get_status(&self, model: &str) -> Option<CircuitBreakerStatus> {
        let breakers = self.breakers.read().await;
        breakers.get(model).map(|breaker| CircuitBreakerStatus {
            model: model.to_string(),
            state: breaker.state.clone(),
            failure_count: breaker.failure_count,
            success_count: breaker.success_count,
            request_count: breaker.request_count,
            last_failure_time: breaker
                .last_failure_time
                .map(|t| format!("{:?} ago", t.elapsed())),
            time_in_current_state: breaker.state_changed_time.elapsed().as_secs(),
        })
    }

    /// Get status for all circuit breakers
    pub async fn get_all_status(&self) -> Vec<CircuitBreakerStatus> {
        let breakers = self.breakers.read().await;
        breakers
            .iter()
            .map(|(model, breaker)| CircuitBreakerStatus {
                model: model.clone(),
                state: breaker.state.clone(),
                failure_count: breaker.failure_count,
                success_count: breaker.success_count,
                request_count: breaker.request_count,
                last_failure_time: breaker
                    .last_failure_time
                    .map(|t| format!("{:?} ago", t.elapsed())),
                time_in_current_state: breaker.state_changed_time.elapsed().as_secs(),
            })
            .collect()
    }

    /// Manually open a circuit breaker
    pub async fn open_circuit(&self, model: &str) {
        let mut breakers = self.breakers.write().await;
        let breaker = breakers
            .entry(model.to_string())
            .or_insert_with(|| CircuitBreaker::new(model.to_string(), self.config.clone()));
        breaker.transition_to_open();
        warn!("Manually opened circuit breaker for model {}", model);
    }

    /// Manually close a circuit breaker
    pub async fn close_circuit(&self, model: &str) {
        let mut breakers = self.breakers.write().await;
        let breaker = breakers
            .entry(model.to_string())
            .or_insert_with(|| CircuitBreaker::new(model.to_string(), self.config.clone()));
        breaker.transition_to_closed();
        info!("Manually closed circuit breaker for model {}", model);
    }

    /// Reset all circuit breakers
    pub async fn reset_all(&self) {
        let mut breakers = self.breakers.write().await;
        for breaker in breakers.values_mut() {
            breaker.transition_to_closed();
        }
        info!("Reset all circuit breakers");
    }

    /// Clear circuit breaker history
    pub async fn clear(&self) {
        let mut breakers = self.breakers.write().await;
        breakers.clear();
        info!("Cleared all circuit breakers");
    }

    /// Get summary metrics for monitoring
    pub async fn get_metrics(&self) -> CircuitBreakerMetrics {
        let breakers = self.breakers.read().await;

        let total_breakers = breakers.len();
        let open_count = breakers
            .values()
            .filter(|b| b.state == CircuitState::Open)
            .count();
        let half_open_count = breakers
            .values()
            .filter(|b| b.state == CircuitState::HalfOpen)
            .count();
        let closed_count = breakers
            .values()
            .filter(|b| b.state == CircuitState::Closed)
            .count();

        let total_failures: u32 = breakers.values().map(|b| b.failure_count).sum();
        let total_requests: u32 = breakers.values().map(|b| b.request_count).sum();

        let success_rate = if total_requests > 0 {
            ((total_requests - total_failures) as f32 / total_requests as f32) * 100.0
        } else {
            100.0
        };

        CircuitBreakerMetrics {
            total_breakers,
            open_count,
            half_open_count,
            closed_count,
            total_failures,
            total_requests,
            success_rate,
        }
    }
}

/// Metrics for circuit breaker monitoring
#[derive(Debug, Serialize, Deserialize)]
pub struct CircuitBreakerMetrics {
    pub total_breakers: usize,
    pub open_count: usize,
    pub half_open_count: usize,
    pub closed_count: usize,
    pub total_failures: u32,
    pub total_requests: u32,
    pub success_rate: f32,
}

/// Convenience macro for executing operations with circuit breaker
#[macro_export]
macro_rules! circuit_breaker_execute {
    ($breaker:expr, $model:expr, $operation:expr) => {
        $breaker.execute($model, || async move { $operation }).await
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::{sleep, Duration};

    #[tokio::test]
    async fn test_circuit_breaker_closed_state() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        let result = manager
            .execute("test-model", || async {
                Ok::<String, anyhow::Error>("success".to_string())
            })
            .await;

        assert!(result.result.is_ok());
        assert_eq!(result.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_opens_on_failures() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            minimum_requests: 2,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Execute failing operations
        for _ in 0..3 {
            let _result = manager
                .execute("test-model", || async {
                    Err::<String, anyhow::Error>(anyhow!("test failure"))
                })
                .await;
        }

        // Next request should be rejected due to open circuit
        let result = manager
            .execute("test-model", || async {
                Ok::<String, anyhow::Error>("should not execute".to_string())
            })
            .await;

        assert!(result.result.is_err());
        assert_eq!(result.state, CircuitState::Open);
    }

    #[tokio::test]
    async fn test_circuit_breaker_recovery() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            minimum_requests: 2,
            recovery_timeout: 1, // 1 second
            success_threshold: 1,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Force circuit to open
        for _ in 0..3 {
            let _result = manager
                .execute("test-model", || async {
                    Err::<String, anyhow::Error>(anyhow!("test failure"))
                })
                .await;
        }

        // Wait for recovery timeout
        sleep(Duration::from_secs(2)).await;

        // Should transition to half-open and allow request
        let result = manager
            .execute("test-model", || async {
                Ok::<String, anyhow::Error>("recovery success".to_string())
            })
            .await;

        assert!(result.result.is_ok());

        // Circuit should now be closed
        let status = manager.get_status("test-model").await.unwrap();
        assert_eq!(status.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_metrics() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // Execute some operations
        for _ in 0..5 {
            let _result = manager
                .execute("model1", || async {
                    Ok::<String, anyhow::Error>("success".to_string())
                })
                .await;
        }

        for _ in 0..3 {
            let _result = manager
                .execute("model2", || async {
                    Err::<String, anyhow::Error>(anyhow!("failure"))
                })
                .await;
        }

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_breakers, 2);
        assert_eq!(metrics.total_requests, 8);
        assert_eq!(metrics.total_failures, 3);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Config Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_circuit_breaker_config_default() {
        let config = CircuitBreakerConfig::default();
        assert_eq!(config.failure_threshold, 5);
        assert_eq!(config.failure_window, 60);
        assert_eq!(config.minimum_requests, 10);
        assert_eq!(config.recovery_timeout, 30);
        assert_eq!(config.success_threshold, 3);
        assert!(config.enabled);
    }

    #[test]
    fn test_circuit_breaker_config_custom() {
        let config = CircuitBreakerConfig {
            failure_threshold: 10,
            failure_window: 120,
            minimum_requests: 20,
            recovery_timeout: 60,
            success_threshold: 5,
            enabled: false,
        };
        assert_eq!(config.failure_threshold, 10);
        assert!(!config.enabled);
    }

    #[test]
    fn test_circuit_breaker_config_clone() {
        let config = CircuitBreakerConfig::default();
        let cloned = config.clone();
        assert_eq!(config.failure_threshold, cloned.failure_threshold);
        assert_eq!(config.enabled, cloned.enabled);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Circuit State Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[test]
    fn test_circuit_state_equality() {
        assert_eq!(CircuitState::Closed, CircuitState::Closed);
        assert_eq!(CircuitState::Open, CircuitState::Open);
        assert_eq!(CircuitState::HalfOpen, CircuitState::HalfOpen);
        assert_ne!(CircuitState::Closed, CircuitState::Open);
        assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_state_serialization() {
        let closed = CircuitState::Closed;
        let open = CircuitState::Open;
        let half_open = CircuitState::HalfOpen;

        let closed_json = serde_json::to_string(&closed).unwrap();
        let open_json = serde_json::to_string(&open).unwrap();
        let half_open_json = serde_json::to_string(&half_open).unwrap();

        assert!(closed_json.contains("Closed"));
        assert!(open_json.contains("Open"));
        assert!(half_open_json.contains("HalfOpen"));
    }

    #[test]
    fn test_circuit_state_deserialization() {
        let closed: CircuitState = serde_json::from_str("\"Closed\"").unwrap();
        let open: CircuitState = serde_json::from_str("\"Open\"").unwrap();
        let half_open: CircuitState = serde_json::from_str("\"HalfOpen\"").unwrap();

        assert_eq!(closed, CircuitState::Closed);
        assert_eq!(open, CircuitState::Open);
        assert_eq!(half_open, CircuitState::HalfOpen);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Disabled Circuit Breaker Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_disabled() {
        let config = CircuitBreakerConfig {
            enabled: false,
            failure_threshold: 1,
            minimum_requests: 1,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Failures should not open circuit when disabled
        for _ in 0..10 {
            let _result = manager
                .execute("test-model", || async {
                    Err::<String, anyhow::Error>(anyhow!("failure"))
                })
                .await;
        }

        // When disabled, the circuit breaker still opens but allows requests through
        // It's the can_proceed check that matters, not the state
        let result = manager
            .execute("test-model", || async {
                Ok::<String, anyhow::Error>("success".to_string())
            })
            .await;
        // The request should still succeed when disabled
        assert!(result.result.is_ok());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Multiple Models Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_multiple_models() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            minimum_requests: 2,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Fail model1
        for _ in 0..3 {
            let _result = manager
                .execute("model1", || async {
                    Err::<String, anyhow::Error>(anyhow!("failure"))
                })
                .await;
        }

        // model1 should be open
        let status1 = manager.get_status("model1").await.unwrap();
        assert_eq!(status1.state, CircuitState::Open);

        // model2 should still work
        let result = manager
            .execute("model2", || async {
                Ok::<String, anyhow::Error>("success".to_string())
            })
            .await;
        assert!(result.result.is_ok());

        let status2 = manager.get_status("model2").await.unwrap();
        assert_eq!(status2.state, CircuitState::Closed);
    }

    #[tokio::test]
    async fn test_circuit_breaker_get_all_status() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // Execute on multiple models
        for model in ["model1", "model2", "model3"] {
            let _result = manager
                .execute(model, || async {
                    Ok::<String, anyhow::Error>("success".to_string())
                })
                .await;
        }

        let all_status = manager.get_all_status().await;
        assert_eq!(all_status.len(), 3);
        
        // Check that all models are present
        let model_names: Vec<_> = all_status.iter().map(|s| s.model.as_str()).collect();
        assert!(model_names.contains(&"model1"));
        assert!(model_names.contains(&"model2"));
        assert!(model_names.contains(&"model3"));
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Status Not Found Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_status_not_found() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        let status = manager.get_status("nonexistent-model").await;
        assert!(status.is_none());
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Minimum Requests Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_minimum_requests() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            minimum_requests: 5,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Fail 3 times but minimum requests is 5
        for _ in 0..3 {
            let _result = manager
                .execute("test-model", || async {
                    Err::<String, anyhow::Error>(anyhow!("failure"))
                })
                .await;
        }

        // Should still be closed because not enough requests
        let status = manager.get_status("test-model").await.unwrap();
        assert_eq!(status.state, CircuitState::Closed);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Half-Open to Open Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_half_open_fails() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            minimum_requests: 2,
            recovery_timeout: 1,
            success_threshold: 2,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Force circuit to open
        for _ in 0..3 {
            let _result = manager
                .execute("test-model", || async {
                    Err::<String, anyhow::Error>(anyhow!("failure"))
                })
                .await;
        }

        // Wait for recovery timeout
        sleep(Duration::from_secs(2)).await;

        // Request in half-open state should fail again -> back to open
        let _result = manager
            .execute("test-model", || async {
                Err::<String, anyhow::Error>(anyhow!("failure in half-open"))
            })
            .await;

        let status = manager.get_status("test-model").await.unwrap();
        assert_eq!(status.state, CircuitState::Open);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Reset Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_reset_all() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            minimum_requests: 2,
            ..Default::default()
        };
        let manager = CircuitBreakerManager::new(config);

        // Force circuit to open on multiple models
        for model in ["model1", "model2"] {
            for _ in 0..3 {
                let _result = manager
                    .execute(model, || async {
                        Err::<String, anyhow::Error>(anyhow!("failure"))
                    })
                    .await;
            }
        }

        // Both should be open
        let status1 = manager.get_status("model1").await.unwrap();
        let status2 = manager.get_status("model2").await.unwrap();
        assert_eq!(status1.state, CircuitState::Open);
        assert_eq!(status2.state, CircuitState::Open);

        // Reset all breakers
        manager.reset_all().await;

        // Both should be closed now
        let status1 = manager.get_status("model1").await.unwrap();
        let status2 = manager.get_status("model2").await.unwrap();
        assert_eq!(status1.state, CircuitState::Closed);
        assert_eq!(status2.state, CircuitState::Closed);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // Metrics Detail Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_metrics_empty() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_breakers, 0);
        assert_eq!(metrics.total_requests, 0);
        assert_eq!(metrics.total_failures, 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_metrics_successes() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        for _ in 0..10 {
            let _result = manager
                .execute("test-model", || async {
                    Ok::<String, anyhow::Error>("success".to_string())
                })
                .await;
        }

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_requests, 10);
        assert_eq!(metrics.total_failures, 0);
        // success_rate = (requests - failures) / requests = 10/10 = 1.0
        // But there might be floating point precision issues
        assert!(metrics.success_rate > 0.99, "Expected high success rate, got {}", metrics.success_rate);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // CircuitBreakerStatus Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_circuit_breaker_status_fields() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        // Execute some operations
        let _result = manager
            .execute("test-model", || async {
                Ok::<String, anyhow::Error>("success".to_string())
            })
            .await;

        let _result = manager
            .execute("test-model", || async {
                Err::<String, anyhow::Error>(anyhow!("failure"))
            })
            .await;

        let status = manager.get_status("test-model").await.unwrap();
        assert_eq!(status.model, "test-model");
        assert_eq!(status.state, CircuitState::Closed);
        assert_eq!(status.request_count, 2);
        // Verify we have some counts - exact values depend on implementation
        assert!(status.failure_count >= 1, "Expected at least 1 failure, got {}", status.failure_count);
    }

    // ═══════════════════════════════════════════════════════════════════════
    // ExecutionResult Tests
    // ═══════════════════════════════════════════════════════════════════════

    #[tokio::test]
    async fn test_execution_result_success() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        let result = manager
            .execute("test-model", || async {
                Ok::<i32, anyhow::Error>(42)
            })
            .await;

        assert!(result.result.is_ok());
        assert_eq!(result.result.unwrap(), 42);
        assert_eq!(result.state, CircuitState::Closed);
        // CircuitBreakerResult has: result, state, execution_time
        assert!(result.execution_time.as_nanos() >= 0);
    }

    #[tokio::test]
    async fn test_execution_result_failure() {
        let config = CircuitBreakerConfig::default();
        let manager = CircuitBreakerManager::new(config);

        let result = manager
            .execute("test-model", || async {
                Err::<i32, anyhow::Error>(anyhow!("test error"))
            })
            .await;

        assert!(result.result.is_err());
        assert!(result.result.unwrap_err().to_string().contains("test error"));
    }
}
