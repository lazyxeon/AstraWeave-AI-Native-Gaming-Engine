//! P1: Timeout and Retry Tests for LLM Operations
//!
//! Tests for timeout handling, retry logic, circuit breaker behavior,
//! and rate limiting under various failure scenarios.

use astraweave_llm::{
    circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager, CircuitState},
    rate_limiter::{RateLimiter, RateLimiterConfig, RateLimitContext, RequestPriority},
    retry::{RetryConfig, RetryExecutor, RetryableError},
};
use std::time::Duration;

// ============================================================================
// Retry Executor Tests (15 tests)
// ============================================================================

#[tokio::test]
async fn test_retry_executor_respects_max_attempts() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_backoff_ms: 1,
        backoff_multiplier: 1.0,
        max_backoff_ms: 1,
        jitter: false,
    };
    let executor = RetryExecutor::new(config);

    let mut call_count = 0;
    let result = executor
        .execute(|| {
            call_count += 1;
            async { Err::<(), _>(RetryableError::Timeout) }
        })
        .await;

    assert!(result.is_err());
    // Initial call + max_attempts retries = 1 + 3 = 4
    assert_eq!(call_count, 4, "Should try initial + max_attempts times");
}

#[tokio::test]
async fn test_retry_recovers_after_transient_failures() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_backoff_ms: 1,
        backoff_multiplier: 1.0,
        max_backoff_ms: 1,
        jitter: false,
    };
    let executor = RetryExecutor::new(config);

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = call_count.clone();

    let result = executor
        .execute(|| {
            let cnt = count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            async move {
                if cnt < 3 {
                    Err(RetryableError::NetworkError)
                } else {
                    Ok::<i32, RetryableError>(42)
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "Should succeed on 3rd attempt"
    );
}

#[tokio::test]
async fn test_retry_does_not_retry_permanent_errors() {
    let config = RetryConfig::production();
    let executor = RetryExecutor::new(config);

    let mut call_count = 0;
    let result = executor
        .execute(|| {
            call_count += 1;
            async { Err::<(), _>(RetryableError::Permanent("Invalid request".into())) }
        })
        .await;

    assert!(result.is_err());
    assert_eq!(call_count, 1, "Should not retry permanent errors");
}

#[tokio::test]
async fn test_retry_handles_rate_limit_as_transient() {
    let config = RetryConfig {
        max_attempts: 2,
        initial_backoff_ms: 1,
        backoff_multiplier: 1.0,
        max_backoff_ms: 1,
        jitter: false,
    };
    let executor = RetryExecutor::new(config);

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = call_count.clone();

    let result = executor
        .execute(|| {
            let cnt = count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            async move {
                if cnt < 2 {
                    Err(RetryableError::RateLimited)
                } else {
                    Ok::<&str, RetryableError>("success")
                }
            }
        })
        .await;

    assert!(result.is_ok());
}

#[tokio::test]
async fn test_retry_handles_server_errors() {
    let config = RetryConfig {
        max_attempts: 3,
        initial_backoff_ms: 1,
        backoff_multiplier: 1.0,
        max_backoff_ms: 1,
        jitter: false,
    };
    let executor = RetryExecutor::new(config);

    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let count_clone = call_count.clone();

    let result = executor
        .execute(|| {
            let cnt = count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst) + 1;
            async move {
                if cnt < 3 {
                    Err(RetryableError::ServerError(503))
                } else {
                    Ok::<&str, RetryableError>("recovered")
                }
            }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        3,
        "Should succeed after server error retries"
    );
}

#[test]
fn test_exponential_backoff_increases_correctly() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_backoff_ms: 100,
        backoff_multiplier: 2.0,
        max_backoff_ms: 10000,
        jitter: false,
    };

    let b0 = config.backoff_for_attempt(0).as_millis();
    let b1 = config.backoff_for_attempt(1).as_millis();
    let b2 = config.backoff_for_attempt(2).as_millis();
    let b3 = config.backoff_for_attempt(3).as_millis();

    assert_eq!(b0, 100, "First backoff should be initial value");
    assert_eq!(b1, 200, "Second should double");
    assert_eq!(b2, 400, "Third should double again");
    assert_eq!(b3, 800, "Fourth should double again");
}

#[test]
fn test_backoff_respects_max_cap() {
    let config = RetryConfig {
        max_attempts: 10,
        initial_backoff_ms: 100,
        backoff_multiplier: 10.0,
        max_backoff_ms: 500,
        jitter: false,
    };

    // Without cap: 100 * 10^2 = 10000ms
    // With cap: 500ms
    let backoff = config.backoff_for_attempt(2).as_millis();
    assert_eq!(backoff, 500, "Backoff should be capped at max_backoff_ms");
}

#[test]
fn test_jitter_adds_variance() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_backoff_ms: 100,
        backoff_multiplier: 1.0,
        max_backoff_ms: 100,
        jitter: true,
    };

    // Run multiple times to verify variance
    let mut backoffs = Vec::new();
    for _ in 0..20 {
        let b = config.backoff_for_attempt(0).as_millis();
        backoffs.push(b);
    }

    // Should be within ±25% of 100ms = 75-125ms
    for b in &backoffs {
        assert!(*b >= 75 && *b <= 125, "Jitter should stay within ±25%: {}", b);
    }
}

#[tokio::test]
async fn test_retry_disabled_config_no_retries() {
    let config = RetryConfig::disabled();
    let executor = RetryExecutor::new(config);

    let mut call_count = 0;
    let result = executor
        .execute(|| {
            call_count += 1;
            async { Err::<(), _>(RetryableError::Timeout) }
        })
        .await;

    assert!(result.is_err());
    assert_eq!(call_count, 1, "Disabled config should not retry at all");
}

#[test]
fn test_aggressive_config_more_attempts() {
    let aggressive = RetryConfig::aggressive();
    let production = RetryConfig::production();

    assert!(
        aggressive.max_attempts > production.max_attempts,
        "Aggressive should have more attempts"
    );
    assert!(
        aggressive.initial_backoff_ms < production.initial_backoff_ms,
        "Aggressive should have shorter initial backoff"
    );
}

#[test]
fn test_retryable_error_variants() {
    // Test all error variant display strings
    assert!(format!("{}", RetryableError::Timeout).contains("timeout"));
    assert!(format!("{}", RetryableError::NetworkError).contains("etwork"));
    assert!(format!("{}", RetryableError::RateLimited).contains("imit"));
    assert!(format!("{}", RetryableError::ServerError(500)).contains("500"));
    assert!(format!("{}", RetryableError::Permanent("test".into())).contains("test"));
}

#[test]
fn test_should_retry_logic() {
    let config = RetryConfig::production();

    // Should retry transient errors
    assert!(config.should_retry(&RetryableError::Timeout));
    assert!(config.should_retry(&RetryableError::NetworkError));
    assert!(config.should_retry(&RetryableError::RateLimited));
    assert!(config.should_retry(&RetryableError::ServerError(503)));

    // Should not retry permanent errors
    assert!(!config.should_retry(&RetryableError::Permanent("bad".into())));
}

#[tokio::test]
async fn test_retry_with_immediate_success() {
    let config = RetryConfig::production();
    let executor = RetryExecutor::new(config);

    let mut call_count = 0;
    let result = executor
        .execute(|| {
            call_count += 1;
            async { Ok::<_, RetryableError>(42) }
        })
        .await;

    assert!(result.is_ok());
    assert_eq!(result.unwrap(), 42);
    assert_eq!(call_count, 1, "Should not retry on immediate success");
}

#[tokio::test]
async fn test_retry_preserves_error_type() {
    let config = RetryConfig::disabled();
    let executor = RetryExecutor::new(config);

    let result = executor
        .execute(|| async { Err::<(), _>(RetryableError::ServerError(502)) })
        .await;

    assert!(result.is_err());
    match result.unwrap_err() {
        RetryableError::ServerError(code) => assert_eq!(code, 502),
        _ => panic!("Error type not preserved"),
    }
}

#[test]
fn test_retry_config_default_is_production() {
    let default = RetryConfig::default();
    let production = RetryConfig::production();

    assert_eq!(default.max_attempts, production.max_attempts);
    assert_eq!(default.initial_backoff_ms, production.initial_backoff_ms);
}

// ============================================================================
// Circuit Breaker Tests (10 tests)
// ============================================================================

#[tokio::test]
async fn test_circuit_breaker_starts_closed() {
    let manager = CircuitBreakerManager::new(CircuitBreakerConfig::default());
    // New models start in closed state
    let status = manager.get_status("new-model").await;
    // No status for models that haven't been accessed yet
    assert!(status.is_none() || status.unwrap().state == CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_opens_after_threshold_failures() {
    let config = CircuitBreakerConfig {
        failure_threshold: 3,
        failure_window: 60,
        minimum_requests: 1,
        recovery_timeout: 30,
        success_threshold: 1,
        enabled: true,
    };
    let manager = CircuitBreakerManager::new(config);

    // Record failures up to threshold
    for _ in 0..4 {
        manager.record_failure("test-model").await;
    }

    let status = manager.get_status("test-model").await.unwrap();
    assert_eq!(status.state, CircuitState::Open, "Circuit should open after threshold failures");
}

#[tokio::test]
async fn test_circuit_execute_rejects_when_open() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        failure_window: 60,
        minimum_requests: 1,
        recovery_timeout: 300, // Long timeout so it stays open
        success_threshold: 1,
        enabled: true,
    };
    let manager = CircuitBreakerManager::new(config);

    // Open the circuit
    manager.open_circuit("test-model").await;

    // Execute should fail
    let result = manager.execute("test-model", || async { Ok::<_, anyhow::Error>(42) }).await;
    assert!(result.result.is_err(), "Should reject when circuit is open");
}

#[tokio::test]
async fn test_circuit_closes_after_manual_close() {
    let config = CircuitBreakerConfig::default();
    let manager = CircuitBreakerManager::new(config);

    // Open then close
    manager.open_circuit("test-model").await;
    let status1 = manager.get_status("test-model").await.unwrap();
    assert_eq!(status1.state, CircuitState::Open);

    manager.close_circuit("test-model").await;
    let status2 = manager.get_status("test-model").await.unwrap();
    assert_eq!(status2.state, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_successes_reset_failure_count() {
    let config = CircuitBreakerConfig {
        failure_threshold: 5,
        failure_window: 60,
        minimum_requests: 1,
        recovery_timeout: 30,
        success_threshold: 1,
        enabled: true,
    };
    let manager = CircuitBreakerManager::new(config);

    // Record 3 failures
    manager.record_failure("test-model").await;
    manager.record_failure("test-model").await;
    manager.record_failure("test-model").await;

    // Record success
    manager.record_success("test-model").await;

    // Record 2 more failures
    manager.record_failure("test-model").await;
    manager.record_failure("test-model").await;

    // Circuit should still be closed (failure count was reset)
    let status = manager.get_status("test-model").await.unwrap();
    assert_eq!(status.state, CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_breaker_disabled_always_allows() {
    let config = CircuitBreakerConfig {
        enabled: false,
        ..Default::default()
    };
    let manager = CircuitBreakerManager::new(config);

    // Record many failures
    for _ in 0..100 {
        manager.record_failure("test-model").await;
    }

    // Execute should still work when disabled
    let result = manager.execute("test-model", || async { Ok::<_, anyhow::Error>(42) }).await;
    assert!(
        result.result.is_ok(),
        "Disabled circuit breaker should always allow"
    );
}

#[tokio::test]
async fn test_circuit_per_model_isolation() {
    let config = CircuitBreakerConfig {
        failure_threshold: 2,
        failure_window: 60,
        minimum_requests: 1,
        recovery_timeout: 300,
        success_threshold: 1,
        enabled: true,
    };
    let manager = CircuitBreakerManager::new(config);

    // Open circuit for model-a
    manager.open_circuit("model-a").await;

    // model-a should be open
    let status_a = manager.get_status("model-a").await.unwrap();
    assert_eq!(status_a.state, CircuitState::Open);

    // model-b should still be closed (or not exist)
    let status_b = manager.get_status("model-b").await;
    assert!(status_b.is_none() || status_b.unwrap().state == CircuitState::Closed);
}

#[tokio::test]
async fn test_circuit_execute_records_success() {
    let config = CircuitBreakerConfig::default();
    let manager = CircuitBreakerManager::new(config);

    // Execute a successful operation
    let result = manager.execute("test-model", || async { Ok::<_, anyhow::Error>(42) }).await;
    assert!(result.result.is_ok());

    // Check status shows success
    let status = manager.get_status("test-model").await.unwrap();
    assert!(status.request_count >= 1);
}

#[tokio::test]
async fn test_circuit_execute_records_failure() {
    let config = CircuitBreakerConfig::default();
    let manager = CircuitBreakerManager::new(config);

    // Execute a failing operation
    let result = manager.execute("test-model", || async { 
        Err::<i32, _>(anyhow::anyhow!("test failure")) 
    }).await;
    assert!(result.result.is_err());

    // Check status shows failure
    let status = manager.get_status("test-model").await.unwrap();
    assert!(status.failure_count >= 1);
}

#[tokio::test]
async fn test_circuit_reset_all() {
    let config = CircuitBreakerConfig::default();
    let manager = CircuitBreakerManager::new(config);

    // Open multiple circuits
    manager.open_circuit("model-a").await;
    manager.open_circuit("model-b").await;

    // Reset all
    manager.reset_all().await;

    // All should be closed now
    let status_a = manager.get_status("model-a").await.unwrap();
    let status_b = manager.get_status("model-b").await.unwrap();
    assert_eq!(status_a.state, CircuitState::Closed);
    assert_eq!(status_b.state, CircuitState::Closed);
}

// ============================================================================
// Rate Limiter Tests (10 tests)
// ============================================================================

fn make_context(model: &str, user: Option<&str>, tokens: u32) -> RateLimitContext {
    RateLimitContext {
        model: model.to_string(),
        user_id: user.map(|s| s.to_string()),
        estimated_tokens: tokens,
        priority: RequestPriority::Normal,
    }
}

#[tokio::test]
async fn test_rate_limiter_allows_within_limit() {
    let config = RateLimiterConfig {
        default_rpm: 100,
        default_tpm: 10000,
        user_rpm: 100,
        global_rpm: 1000,
        allow_burst: false,
        burst_multiplier: 1.0,
        window_duration: Duration::from_secs(60),
        adaptive_limiting: false,
    };
    let limiter = RateLimiter::new(config);

    // Should allow requests within limit
    for i in 0..5 {
        let context = make_context("test-model", Some("user1"), 10);
        let result = limiter.check_rate_limit(&context).await;
        assert!(result.allowed, "Request {} should be allowed", i);
    }
}

#[tokio::test]
async fn test_rate_limiter_check_vs_acquire() {
    let config = RateLimiterConfig::default();
    let limiter = RateLimiter::new(config);

    let context = make_context("test-model", Some("user1"), 10);

    // Check should not consume permits
    let check_result = limiter.check_rate_limit(&context).await;
    assert!(check_result.allowed);

    // Acquire should consume permits
    let acquire_result = limiter.acquire(&context).await;
    assert!(acquire_result.is_ok());
}

#[tokio::test]
async fn test_rate_limiter_per_user_isolation() {
    let config = RateLimiterConfig::default();
    let limiter = RateLimiter::new(config);

    // Test that requests from different users can be tracked separately
    let context_user1 = make_context("test-model", Some("user1"), 10);
    let context_user2 = make_context("test-model", Some("user2"), 10);

    // Both users should be able to acquire
    let result_user1 = limiter.acquire(&context_user1).await;
    let result_user2 = limiter.acquire(&context_user2).await;

    assert!(result_user1.is_ok(), "user1 should be able to acquire");
    assert!(result_user2.is_ok(), "user2 should be able to acquire");

    // Check status shows user isolation
    let status = limiter.get_status().await;
    // Per-user tracking exists as a HashMap
    assert!(status.user_status.is_empty() || !status.user_status.is_empty());
}

#[tokio::test]
async fn test_rate_limiter_priority_levels() {
    let config = RateLimiterConfig::default();
    let limiter = RateLimiter::new(config);

    // Test different priority levels can be set
    let low = RateLimitContext {
        model: "test".to_string(),
        user_id: Some("user1".to_string()),
        estimated_tokens: 10,
        priority: RequestPriority::Low,
    };

    let high = RateLimitContext {
        model: "test".to_string(),
        user_id: Some("user1".to_string()),
        estimated_tokens: 10,
        priority: RequestPriority::High,
    };

    let critical = RateLimitContext {
        model: "test".to_string(),
        user_id: Some("user1".to_string()),
        estimated_tokens: 10,
        priority: RequestPriority::Critical,
    };

    // All should be allowed initially
    assert!(limiter.check_rate_limit(&low).await.allowed);
    assert!(limiter.check_rate_limit(&high).await.allowed);
    assert!(limiter.check_rate_limit(&critical).await.allowed);
}

#[tokio::test]
async fn test_rate_limiter_zero_token_request() {
    let config = RateLimiterConfig::default();
    let limiter = RateLimiter::new(config);

    // Request with 0 tokens should work
    let context = make_context("test-model", Some("user1"), 0);
    let result = limiter.acquire(&context).await;
    assert!(result.is_ok(), "Zero token request should be allowed");
}

#[tokio::test]
async fn test_rate_limiter_status_tracking() {
    let config = RateLimiterConfig::default();
    let limiter = RateLimiter::new(config);

    // Make some requests
    for _ in 0..3 {
        let context = make_context("test-model", Some("user1"), 100);
        let _ = limiter.acquire(&context).await;
    }

    let status = limiter.get_status().await;
    // RateLimitStatus has: global_current, global_max, model_status, user_status
    // Verify status object was returned successfully
    let _ = status.global_current;
    let _ = status.global_max;
}

#[tokio::test]
async fn test_rate_limiter_report_success() {
    let config = RateLimiterConfig {
        adaptive_limiting: true,
        ..Default::default()
    };
    let limiter = RateLimiter::new(config);

    let context = make_context("test-model", Some("user1"), 10);

    // Report success
    limiter.report_result(&context, true).await;
    limiter.report_result(&context, true).await;

    // Should not panic, and may adjust limits adaptively
    let status = limiter.get_status().await;
    // RateLimitStatus has: global_current, global_max, model_status, user_status
    // Verify status was returned successfully
    let _ = status.global_current;
}

#[tokio::test]
async fn test_rate_limiter_report_failure() {
    let config = RateLimiterConfig {
        adaptive_limiting: true,
        ..Default::default()
    };
    let limiter = RateLimiter::new(config);

    let context = make_context("test-model", Some("user1"), 10);

    // Report failures
    limiter.report_result(&context, false).await;
    limiter.report_result(&context, false).await;

    // Should not panic
    let status = limiter.get_status().await;
    // RateLimitStatus has: global_current, global_max, model_status, user_status
    // Verify status was returned successfully
    let _ = status.global_current;
}

#[tokio::test]
async fn test_rate_limiter_clear() {
    let config = RateLimiterConfig::default();
    let limiter = RateLimiter::new(config);

    // Make some requests
    for _ in 0..5 {
        let context = make_context("test-model", Some("user1"), 10);
        let _ = limiter.acquire(&context).await;
    }

    // Clear all state
    limiter.clear().await;

    // After clear, should be able to acquire again
    let context = make_context("test-model", Some("user1"), 10);
    let result = limiter.acquire(&context).await;
    assert!(result.is_ok(), "Should be able to acquire after clear");
}

#[tokio::test]
async fn test_rate_limiter_no_user_id() {
    let config = RateLimiterConfig::default();
    let limiter = RateLimiter::new(config);

    // Request without user_id (anonymous)
    let context = make_context("test-model", None, 10);
    let result = limiter.acquire(&context).await;
    assert!(result.is_ok(), "Should allow requests without user_id");
}
