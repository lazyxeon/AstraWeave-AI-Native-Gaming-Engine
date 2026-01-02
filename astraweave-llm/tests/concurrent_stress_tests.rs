//! P2: Concurrent Stress Tests for astraweave-llm
//!
//! Focus: shared-state managers (rate limiter + circuit breaker) should be safe
//! under moderate concurrent async load.

#![cfg(test)]

use astraweave_llm::{
    circuit_breaker::{CircuitBreakerConfig, CircuitBreakerManager},
    rate_limiter::{RateLimitContext, RateLimiter, RateLimiterConfig, RequestPriority},
};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_rate_limiter_acquire_no_panic() {
    let cfg = RateLimiterConfig {
        default_rpm: 10_000,
        default_tpm: 1_000_000,
        user_rpm: 10_000,
        global_rpm: 10_000,
        adaptive_limiting: false,
        ..RateLimiterConfig::default()
    };

    let limiter = Arc::new(RateLimiter::new(cfg));

    let mut join_set = tokio::task::JoinSet::new();
    for i in 0..200u32 {
        let limiter = limiter.clone();
        join_set.spawn(async move {
            let ctx = RateLimitContext {
                user_id: Some(format!("u{}", i % 4)),
                model: "m".to_string(),
                estimated_tokens: 1,
                priority: RequestPriority::Normal,
            };

            // Acquire + drop immediately to avoid exhausting permits.
            let _permit = limiter.acquire(&ctx).await.expect("acquire should succeed");
        });
    }

    while let Some(res) = join_set.join_next().await {
        res.expect("task panicked");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn test_concurrent_circuit_breaker_execute_no_panic() {
    let cfg = CircuitBreakerConfig {
        enabled: false,
        failure_threshold: 10,
        minimum_requests: 10,
        recovery_timeout: 1,
        failure_window: 60,
        success_threshold: 1,
    };

    let mgr = Arc::new(CircuitBreakerManager::new(cfg));
    let calls = Arc::new(AtomicUsize::new(0));

    let mut join_set = tokio::task::JoinSet::new();
    for _ in 0..200 {
        let mgr = mgr.clone();
        let calls = calls.clone();
        join_set.spawn(async move {
            let r = mgr
                .execute("model", move || async move {
                    calls.fetch_add(1, Ordering::SeqCst);
                    Ok::<(), _>(())
                })
                .await;
            r.result.expect("operation should succeed");
        });
    }

    while let Some(res) = join_set.join_next().await {
        res.expect("task panicked");
    }

    assert_eq!(calls.load(Ordering::SeqCst), 200);
}
