//! P2: Boundary Condition Tests for astraweave-llm
//!
//! Focuses on edge cases for retry/circuit breaker/rate limiter configuration and behavior.

#![cfg(test)]

use astraweave_llm::{
    circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager},
    rate_limiter::{RateLimitContext, RateLimiter, RateLimiterConfig, RequestPriority},
    retry::{RetryConfig, RetryExecutor, RetryableError},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[test]
fn test_retry_backoff_huge_attempt_is_capped() {
    let cfg = RetryConfig {
        max_attempts: 10,
        initial_backoff_ms: 1,
        backoff_multiplier: 10.0,
        max_backoff_ms: 500,
        jitter: false,
    };

    let backoff = cfg.backoff_for_attempt(1000);
    assert_eq!(backoff.as_millis(), 500);
}

#[test]
fn test_retry_backoff_zero_max_backoff_is_zero() {
    let cfg = RetryConfig {
        max_attempts: 3,
        initial_backoff_ms: 50,
        backoff_multiplier: 2.0,
        max_backoff_ms: 0,
        jitter: false,
    };

    let backoff = cfg.backoff_for_attempt(0);
    assert_eq!(backoff.as_millis(), 0);
}

#[tokio::test]
async fn test_retry_executor_max_attempts_zero_calls_once() {
    let cfg = RetryConfig {
        max_attempts: 0,
        initial_backoff_ms: 1,
        backoff_multiplier: 1.0,
        max_backoff_ms: 1,
        jitter: false,
    };
    let exec = RetryExecutor::new(cfg);

    let calls = Arc::new(AtomicUsize::new(0));
    let calls2 = calls.clone();

    let _ = exec
        .execute(|| {
            calls2.fetch_add(1, Ordering::SeqCst);
            async { Err::<(), _>(RetryableError::Timeout) }
        })
        .await;

    assert_eq!(calls.load(Ordering::SeqCst), 1, "should only attempt once");
}

#[tokio::test]
async fn test_rate_limiter_global_rpm_zero_denies() {
    let cfg = RateLimiterConfig {
        global_rpm: 0,
        ..RateLimiterConfig::default()
    };
    let limiter = RateLimiter::new(cfg);

    let ctx = RateLimitContext {
        user_id: Some("u".to_string()),
        model: "m".to_string(),
        estimated_tokens: 1,
        priority: RequestPriority::Normal,
    };

    let res = limiter.check_rate_limit(&ctx).await;
    assert!(!res.allowed);
    assert!(res.reason.unwrap_or_default().contains("Global"));
}

#[tokio::test]
async fn test_circuit_breaker_disabled_never_short_circuits() {
    let cfg = CircuitBreakerConfig {
        enabled: false,
        failure_threshold: 1,
        minimum_requests: 1,
        recovery_timeout: 1,
        failure_window: 60,
        success_threshold: 1,
    };
    let mgr = CircuitBreakerManager::new(cfg);

    let calls = Arc::new(AtomicUsize::new(0));

    for _ in 0..20 {
        let calls2 = calls.clone();
        let result = mgr
            .execute("model", move || async move {
                calls2.fetch_add(1, Ordering::SeqCst);
                Err::<(), _>(anyhow::anyhow!("opfail"))
            })
            .await;

        // Should always be the operation's error, never the circuit breaker's open error.
        let msg = format!("{}", result.result.unwrap_err());
        assert!(msg.contains("opfail"), "unexpected error: {msg}");
    }

    assert_eq!(calls.load(Ordering::SeqCst), 20);
}
