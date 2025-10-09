use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::ab_testing::{ABTestConfig, ABTestFramework};
use crate::backpressure::{BackpressureConfig, BackpressureManager, Priority, RequestMetadata};
use crate::circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager};
use crate::rate_limiter::{RateLimitContext, RateLimiter, RateLimiterConfig, RequestPriority};
use astraweave_observability::llm_telemetry::{LlmTelemetry, TelemetryConfig};

/// Production hardening layer that integrates all reliability systems
pub struct ProductionHardeningLayer {
    /// Rate limiting
    rate_limiter: Arc<RateLimiter>,
    /// Circuit breaker management
    circuit_breaker: Arc<CircuitBreakerManager>,
    /// Backpressure management
    backpressure_manager: Arc<RwLock<BackpressureManager>>,
    /// A/B testing framework
    ab_testing: Arc<ABTestFramework>,
    /// Telemetry system
    telemetry: Arc<LlmTelemetry>,
    /// Configuration
    config: HardeningConfig,
    /// Health checker
    health_checker: Arc<RwLock<HealthChecker>>,
}

#[derive(Debug, Clone)]
pub struct HardeningConfig {
    /// Rate limiter configuration
    pub rate_limiter: RateLimiterConfig,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Backpressure configuration
    pub backpressure: BackpressureConfig,
    /// A/B testing configuration
    pub ab_testing: ABTestConfig,
    /// Telemetry configuration
    pub telemetry: TelemetryConfig,
    /// Health check configuration
    pub health_check: HealthCheckConfig,
    /// Enable graceful shutdown
    pub graceful_shutdown_timeout: Duration,
}

impl Default for HardeningConfig {
    fn default() -> Self {
        Self {
            rate_limiter: RateLimiterConfig::default(),
            circuit_breaker: CircuitBreakerConfig::default(),
            backpressure: BackpressureConfig::default(),
            ab_testing: ABTestConfig::default(),
            telemetry: TelemetryConfig::default(),
            health_check: HealthCheckConfig::default(),
            graceful_shutdown_timeout: Duration::from_secs(30),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Health check interval
    pub check_interval: Duration,
    /// Timeout for individual health checks
    pub check_timeout: Duration,
    /// Unhealthy threshold (consecutive failures)
    pub unhealthy_threshold: u32,
    /// Healthy threshold (consecutive successes after being unhealthy)
    pub healthy_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            check_interval: Duration::from_secs(30),
            check_timeout: Duration::from_secs(5),
            unhealthy_threshold: 3,
            healthy_threshold: 2,
        }
    }
}

/// Request context for production hardening
#[derive(Debug, Clone)]
pub struct HardenedRequest {
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub model: String,
    pub prompt: String,
    pub estimated_tokens: u32,
    pub priority: Priority,
    pub timeout: Option<Duration>,
    pub metadata: HashMap<String, String>,
}

/// Result of production hardening processing
#[derive(Debug)]
pub enum HardeningResult<T> {
    /// Request processed successfully
    Success(T),
    /// Request was rate limited
    RateLimited {
        retry_after: Duration,
        reason: String,
    },
    /// Request failed due to circuit breaker
    CircuitOpen {
        model: String,
        retry_after: Duration,
    },
    /// Request was queued due to backpressure
    Queued {
        position: usize,
        estimated_wait: Duration,
    },
    /// Request was rejected due to system overload
    Rejected {
        reason: String,
        retry_after: Option<Duration>,
    },
    /// Request failed with error
    Error(anyhow::Error),
}

/// System health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    pub overall_status: HealthStatus,
    pub components: HashMap<String, ComponentHealth>,
    pub last_check: String,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    pub last_check: String,
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
    pub response_time_ms: Option<u64>,
}

/// Health checker for system components
#[derive(Debug)]
struct HealthChecker {
    components: HashMap<String, ComponentHealth>,
    config: HealthCheckConfig,
    start_time: Instant,
}

impl HealthChecker {
    fn new(config: HealthCheckConfig) -> Self {
        let mut components = HashMap::new();

        // Initialize component health status
        for component in [
            "rate_limiter",
            "circuit_breaker",
            "backpressure",
            "telemetry",
            "ab_testing",
        ] {
            components.insert(
                component.to_string(),
                ComponentHealth {
                    status: HealthStatus::Healthy,
                    last_check: chrono::Utc::now().to_rfc3339(),
                    consecutive_failures: 0,
                    last_error: None,
                    response_time_ms: None,
                },
            );
        }

        Self {
            components,
            config,
            start_time: Instant::now(),
        }
    }

    async fn check_health(
        &mut self,
        component: &str,
        check_fn: impl std::future::Future<Output = Result<Duration>>,
    ) {
        let start = Instant::now();
        let result = tokio::time::timeout(self.config.check_timeout, check_fn).await;
        let elapsed = start.elapsed();

        let component_health = self
            .components
            .entry(component.to_string())
            .or_insert_with(|| ComponentHealth {
                status: HealthStatus::Healthy,
                last_check: chrono::Utc::now().to_rfc3339(),
                consecutive_failures: 0,
                last_error: None,
                response_time_ms: None,
            });

        match result {
            Ok(Ok(response_time)) => {
                component_health.consecutive_failures = 0;
                component_health.last_error = None;
                component_health.response_time_ms = Some(response_time.as_millis() as u64);

                // Update status based on consecutive successes
                if component_health.status != HealthStatus::Healthy {
                    if component_health.consecutive_failures == 0 {
                        // First success after being unhealthy
                        component_health.status = HealthStatus::Degraded;
                    }
                } else {
                    component_health.status = HealthStatus::Healthy;
                }
            }
            Ok(Err(e)) => {
                component_health.consecutive_failures += 1;
                component_health.last_error = Some(format!("{:?}", e));
                component_health.response_time_ms = Some(elapsed.as_millis() as u64);
            }
            Err(timeout_err) => {
                component_health.consecutive_failures += 1;
                component_health.last_error = Some(format!("Timeout: {:?}", timeout_err));
                component_health.response_time_ms = Some(elapsed.as_millis() as u64);

                // Update status based on consecutive failures
                if component_health.consecutive_failures >= self.config.unhealthy_threshold {
                    component_health.status = HealthStatus::Unhealthy;
                } else {
                    component_health.status = HealthStatus::Degraded;
                }
            }
        }

        component_health.last_check = chrono::Utc::now().to_rfc3339();
    }

    fn get_overall_health(&self) -> SystemHealth {
        let mut overall_status = HealthStatus::Healthy;
        let mut unhealthy_count = 0;
        let mut degraded_count = 0;

        for health in self.components.values() {
            match health.status {
                HealthStatus::Unhealthy => unhealthy_count += 1,
                HealthStatus::Degraded => degraded_count += 1,
                HealthStatus::Healthy => {}
            }
        }

        // Determine overall status
        if unhealthy_count > 0 {
            overall_status = HealthStatus::Unhealthy;
        } else if degraded_count > 0 {
            overall_status = HealthStatus::Degraded;
        }

        SystemHealth {
            overall_status,
            components: self.components.clone(),
            last_check: chrono::Utc::now().to_rfc3339(),
            uptime_seconds: self.start_time.elapsed().as_secs(),
        }
    }
}

impl ProductionHardeningLayer {
    pub fn new(config: HardeningConfig) -> Self {
        let rate_limiter = Arc::new(RateLimiter::new(config.rate_limiter.clone()));
        let circuit_breaker = Arc::new(CircuitBreakerManager::new(config.circuit_breaker.clone()));
        let backpressure_manager = Arc::new(RwLock::new(BackpressureManager::new(
            config.backpressure.clone(),
        )));
        let ab_testing = Arc::new(ABTestFramework::new(config.ab_testing.clone()));
        let telemetry = Arc::new(LlmTelemetry::new(config.telemetry.clone()));
        let health_checker = Arc::new(RwLock::new(HealthChecker::new(config.health_check.clone())));

        Self {
            rate_limiter,
            circuit_breaker,
            backpressure_manager,
            ab_testing,
            telemetry,
            config,
            health_checker,
        }
    }

    /// Start all background services
    pub async fn start(&self) -> Result<()> {
        // Start backpressure manager
        {
            let mut manager = self.backpressure_manager.write().await;
            manager.start().await?;
        }

        // Start health checker
        self.start_health_checker().await;

        info!("Production hardening layer started");
        Ok(())
    }

    /// Process a request through all hardening layers
    pub async fn process_request<F, T, Fut>(
        &self,
        request: HardenedRequest,
        operation: F,
    ) -> HardeningResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = Result<T>>,
    {
        let request_id = uuid::Uuid::new_v4().to_string();

        // Start telemetry tracking
        let tracker = self.telemetry.start_request(
            request_id.clone(),
            request.model.clone(),
            "production_hardening".to_string(),
            request.estimated_tokens as usize,
        );

        // 1. Rate limiting check
        let rate_limit_context = RateLimitContext {
            user_id: request.user_id.clone(),
            model: request.model.clone(),
            estimated_tokens: request.estimated_tokens,
            priority: match request.priority {
                Priority::Critical => RequestPriority::Critical,
                Priority::High => RequestPriority::High,
                Priority::Normal => RequestPriority::Normal,
                Priority::Low => RequestPriority::Low,
                Priority::Background => RequestPriority::Low,
            },
        };

        let rate_limit_check = self
            .rate_limiter
            .check_rate_limit(&rate_limit_context)
            .await;
        if !rate_limit_check.allowed {
            let error_msg = rate_limit_check
                .reason
                .unwrap_or("Rate limited".to_string());
            self.record_failure(&request.model, &error_msg).await;

            tracker
                .complete(
                    request.model.clone(),
                    false,
                    0,
                    0.0,
                    Some(error_msg.clone()),
                    Some("rate_limit".to_string()),
                    "production_hardening".to_string(),
                    None,
                    None,
                    HashMap::new(),
                )
                .await
                .ok();

            return HardeningResult::RateLimited {
                retry_after: rate_limit_check
                    .retry_after
                    .unwrap_or(Duration::from_secs(1)),
                reason: error_msg,
            };
        }

        // 2. Circuit breaker check
        let circuit_result = self
            .circuit_breaker
            .execute(&request.model, || async {
                // 3. Backpressure management
                let backpressure_metadata = RequestMetadata {
                    user_id: request.user_id.clone(),
                    model: request.model.clone(),
                    estimated_tokens: request.estimated_tokens,
                    estimated_cost: request.estimated_tokens as f64 * 0.0001, // Rough estimate
                    tags: request.metadata.clone(),
                };

                let backpressure_result = self
                    .backpressure_manager
                    .read()
                    .await
                    .submit_request(request.priority, request.timeout, backpressure_metadata)
                    .await?;

                match backpressure_result {
                    crate::backpressure::BackpressureResult::Accepted => {
                        // 4. A/B testing (if applicable)
                        let _assignment: Option<String> = if let Some(_user_id) = &request.user_id {
                            // This would be used to determine which model variant to use
                            // For now, just use the requested model
                            None
                        } else {
                            None
                        };

                        // 5. Execute the actual operation
                        operation().await
                    }
                    crate::backpressure::BackpressureResult::Queued {
                        position,
                        estimated_wait,
                    } => {
                        return Err(anyhow!(
                            "Request queued: position {}, wait time: {:?}",
                            position,
                            estimated_wait
                        ));
                    }
                    crate::backpressure::BackpressureResult::Rejected { reason, .. } => {
                        return Err(anyhow!("Request rejected: {}", reason));
                    }
                    crate::backpressure::BackpressureResult::Degraded { reason } => {
                        warn!("Request degraded: {}", reason);
                        // Continue with degraded processing
                        operation().await
                    }
                }
            })
            .await;

        // Process circuit breaker result
        match circuit_result.result {
            Ok(result) => {
                // Record success
                self.record_success(&request.model).await;

                tracker
                    .complete(
                        request.model.clone(),
                        true,
                        0,     // Response tokens would be counted here
                        0.001, // Cost would be calculated here
                        None,
                        None,
                        "production_hardening".to_string(),
                        Some(request.prompt),
                        None, // Response would be logged here if enabled
                        request.metadata,
                    )
                    .await
                    .ok();

                HardeningResult::Success(result)
            }
            Err(e) => {
                // Record failure
                self.record_failure(&request.model, &e.to_string()).await;

                tracker
                    .complete(
                        request.model.clone(),
                        false,
                        0,
                        0.0,
                        Some(e.to_string()),
                        Some("execution_error".to_string()),
                        "production_hardening".to_string(),
                        Some(request.prompt),
                        None,
                        request.metadata,
                    )
                    .await
                    .ok();

                match circuit_result.state {
                    crate::circuit_breaker::CircuitState::Open => HardeningResult::CircuitOpen {
                        model: request.model,
                        retry_after: Duration::from_secs(30),
                    },
                    _ => HardeningResult::Error(e),
                }
            }
        }
    }

    /// Get comprehensive system status
    pub async fn get_system_status(&self) -> ProductionStatus {
        let rate_limiter_status = self.rate_limiter.get_status().await;
        let circuit_breaker_status = self.circuit_breaker.get_all_status().await;
        let backpressure_metrics = self.backpressure_manager.read().await.get_metrics().await;
        let telemetry_metrics = self.telemetry.get_metrics().await;
        let health_status = self.health_checker.read().await.get_overall_health();

        ProductionStatus {
            health: health_status,
            rate_limiter: rate_limiter_status,
            circuit_breakers: circuit_breaker_status,
            backpressure: backpressure_metrics,
            telemetry: telemetry_metrics,
            last_updated: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Graceful shutdown of all services
    pub async fn shutdown(&self) -> Result<()> {
        info!("Starting graceful shutdown of production hardening layer");

        // Stop backpressure manager
        {
            let mut manager = self.backpressure_manager.write().await;
            manager.stop().await;
        }

        // Clear telemetry data if needed
        // self.telemetry.clear_data().await?;

        info!("Production hardening layer shutdown complete");
        Ok(())
    }

    /// Start background health checker
    async fn start_health_checker(&self) {
        let health_checker = self.health_checker.clone();
        let rate_limiter = self.rate_limiter.clone();
        let circuit_breaker = self.circuit_breaker.clone();
        let backpressure_manager = self.backpressure_manager.clone();
        let telemetry = self.telemetry.clone();
        let check_interval = self.config.health_check.check_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(check_interval);

            loop {
                interval.tick().await;

                let mut checker = health_checker.write().await;

                // Check rate limiter health
                checker
                    .check_health("rate_limiter", async {
                        let _status = rate_limiter.get_status().await;
                        Ok(Duration::from_millis(1))
                    })
                    .await;

                // Check circuit breaker health
                checker
                    .check_health("circuit_breaker", async {
                        let _status = circuit_breaker.get_all_status().await;
                        Ok(Duration::from_millis(1))
                    })
                    .await;

                // Check backpressure health
                checker
                    .check_health("backpressure", async {
                        let _metrics = backpressure_manager.read().await.get_metrics().await;
                        Ok(Duration::from_millis(1))
                    })
                    .await;

                // Check telemetry health
                checker
                    .check_health("telemetry", async {
                        let _metrics = telemetry.get_metrics().await;
                        Ok(Duration::from_millis(1))
                    })
                    .await;
            }
        });
    }

    /// Record successful operation
    async fn record_success(&self, model: &str) {
        self.circuit_breaker.record_success(model).await;

        let context = RateLimitContext {
            user_id: None,
            model: model.to_string(),
            estimated_tokens: 0,
            priority: RequestPriority::Normal,
        };
        self.rate_limiter.report_result(&context, true).await;
    }

    /// Record failed operation
    async fn record_failure(&self, model: &str, error: &str) {
        self.circuit_breaker.record_failure(model).await;

        let context = RateLimitContext {
            user_id: None,
            model: model.to_string(),
            estimated_tokens: 0,
            priority: RequestPriority::Normal,
        };
        self.rate_limiter.report_result(&context, false).await;

        warn!("Recorded failure for model {}: {}", model, error);
    }
}

/// Comprehensive production status
#[derive(Debug, Serialize, Deserialize)]
pub struct ProductionStatus {
    pub health: SystemHealth,
    pub rate_limiter: crate::rate_limiter::RateLimitStatus,
    pub circuit_breakers: Vec<crate::circuit_breaker::CircuitBreakerStatus>,
    pub backpressure: crate::backpressure::BackpressureMetrics,
    pub telemetry: astraweave_observability::llm_telemetry::LlmMetrics,
    pub last_updated: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_production_hardening_layer_creation() {
        let config = HardeningConfig::default();
        let layer = ProductionHardeningLayer::new(config);

        // Test that layer can be created without panicking
        assert!(true);
    }

    #[tokio::test]
    async fn test_system_status() {
        let config = HardeningConfig::default();
        let layer = ProductionHardeningLayer::new(config);

        let status = layer.get_system_status().await;
        assert!(!status.last_updated.is_empty());
    }

    #[tokio::test]
    async fn test_successful_request_processing() {
        let config = HardeningConfig::default();
        let layer = ProductionHardeningLayer::new(config);
        layer.start().await.unwrap();

        let request = HardenedRequest {
            user_id: Some("test_user".to_string()),
            session_id: None,
            model: "gpt-3.5-turbo".to_string(),
            prompt: "Hello world".to_string(),
            estimated_tokens: 10,
            priority: Priority::Normal,
            timeout: None,
            metadata: HashMap::new(),
        };

        let result = layer
            .process_request(request, || async {
                Ok::<String, anyhow::Error>("Hello response".to_string())
            })
            .await;

        match result {
            HardeningResult::Success(response) => {
                assert_eq!(response, "Hello response");
            }
            _ => panic!("Expected successful result"),
        }

        layer.shutdown().await.unwrap();
    }
}
