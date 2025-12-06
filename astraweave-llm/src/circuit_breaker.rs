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
}
