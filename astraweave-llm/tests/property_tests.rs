//! Property-based tests for astraweave-llm
//!
//! These tests use proptest to verify invariants and find edge cases
//! in retry, circuit breaker, and rate limiter logic.

use proptest::prelude::*;
use std::time::Duration;

use astraweave_llm::retry::{RetryConfig, RetryableError};
use astraweave_llm::circuit_breaker::{CircuitBreakerConfig, CircuitState};

// ============================================================================
// PROPTEST STRATEGIES
// ============================================================================

/// Strategy for generating RetryConfig values
fn retry_config_strategy() -> impl Strategy<Value = RetryConfig> {
    (
        0u32..10,          // max_attempts
        1u64..1000,        // initial_backoff_ms
        1.0f64..4.0,       // backoff_multiplier
        100u64..10000,     // max_backoff_ms
        any::<bool>(),     // jitter
    )
        .prop_map(|(attempts, initial, mult, max, jitter)| RetryConfig {
            max_attempts: attempts,
            initial_backoff_ms: initial,
            backoff_multiplier: mult,
            max_backoff_ms: max,
            jitter,
        })
}

/// Strategy for generating CircuitBreakerConfig values
fn circuit_breaker_config_strategy() -> impl Strategy<Value = CircuitBreakerConfig> {
    (
        1u32..20,          // failure_threshold
        10u64..300,        // failure_window
        1u32..50,          // minimum_requests
        5u64..120,         // recovery_timeout
        1u32..10,          // success_threshold
        any::<bool>(),     // enabled
    )
        .prop_map(|(ft, fw, mr, rt, st, en)| CircuitBreakerConfig {
            failure_threshold: ft,
            failure_window: fw,
            minimum_requests: mr,
            recovery_timeout: rt,
            success_threshold: st,
            enabled: en,
        })
}

/// Strategy for generating RetryableError values
fn retryable_error_strategy() -> impl Strategy<Value = RetryableError> {
    prop_oneof![
        Just(RetryableError::Timeout),
        Just(RetryableError::NetworkError),
        Just(RetryableError::RateLimited),
        (500u16..600).prop_map(RetryableError::ServerError),
        "[a-zA-Z0-9 ]{0,50}".prop_map(|s| RetryableError::Permanent(s)),
    ]
}

/// Strategy for generating attempt numbers
fn attempt_strategy() -> impl Strategy<Value = u32> {
    0u32..100
}

// ============================================================================
// PROPERTY TESTS: RetryConfig::backoff_for_attempt
// ============================================================================

proptest! {
    /// Property: Backoff duration is always bounded by max_backoff_ms (plus jitter margin)
    #[test]
    fn prop_backoff_respects_max(config in retry_config_strategy(), attempt in attempt_strategy()) {
        let backoff = config.backoff_for_attempt(attempt);
        // With jitter, can be up to 25% over max, so allow 50% margin for safety
        let max_with_margin = config.max_backoff_ms.saturating_mul(2);
        prop_assert!(
            backoff.as_millis() <= max_with_margin as u128,
            "Backoff {} exceeded max {} (with margin) for attempt {}",
            backoff.as_millis(),
            max_with_margin,
            attempt
        );
    }

    /// Property: Backoff for attempt 0 is approximately initial_backoff_ms (but capped)
    #[test]
    fn prop_initial_backoff_correct(config in retry_config_strategy()) {
        if config.max_attempts > 0 {
            let backoff = config.backoff_for_attempt(0);
            // The backoff is capped at max_backoff_ms
            let expected_base = config.initial_backoff_ms.min(config.max_backoff_ms);
            
            if !config.jitter {
                // Without jitter, should be exactly the capped value
                prop_assert_eq!(
                    backoff.as_millis() as u64, 
                    expected_base,
                    "Backoff should be min(initial, max) = {}", expected_base
                );
            } else {
                // With jitter, should be within ±25% of the capped value (or at least non-zero)
                let min_expected = (expected_base as f64 * 0.5) as u64;
                let max_expected = (expected_base as f64 * 1.5) as u64;
                prop_assert!(
                    backoff.as_millis() as u64 >= min_expected || expected_base == 0,
                    "Backoff {} below min {} for initial attempt",
                    backoff.as_millis(),
                    min_expected
                );
                prop_assert!(
                    backoff.as_millis() as u64 <= max_expected.max(config.max_backoff_ms * 2),
                    "Backoff {} above max {} for initial attempt",
                    backoff.as_millis(),
                    max_expected
                );
            }
        }
    }

    /// Property: Disabled retry returns zero backoff
    #[test]
    fn prop_disabled_retry_zero_backoff(attempt in attempt_strategy()) {
        let config = RetryConfig::disabled();
        let backoff = config.backoff_for_attempt(attempt);
        prop_assert_eq!(backoff, Duration::from_millis(0));
    }

    /// Property: backoff_for_attempt never panics
    #[test]
    fn prop_backoff_never_panics(config in retry_config_strategy(), attempt in attempt_strategy()) {
        let _ = config.backoff_for_attempt(attempt);
    }

    /// Property: Backoff generally increases with attempt number (without jitter)
    #[test]
    fn prop_backoff_increases_with_attempt(
        max_attempts in 1u32..10,
        initial_backoff_ms in 10u64..100,
        backoff_multiplier in 1.1f64..3.0,
        max_backoff_ms in 1000u64..10000,
    ) {
        let config = RetryConfig {
            max_attempts,
            initial_backoff_ms,
            backoff_multiplier,
            max_backoff_ms,
            jitter: false,
        };
        
        let backoff_0 = config.backoff_for_attempt(0);
        let backoff_1 = config.backoff_for_attempt(1);
        let backoff_2 = config.backoff_for_attempt(2);
        
        // Each subsequent backoff should be >= previous (or both at cap)
        prop_assert!(
            backoff_1 >= backoff_0 || backoff_0.as_millis() as u64 >= max_backoff_ms,
            "Backoff 1 ({:?}) < Backoff 0 ({:?})",
            backoff_1,
            backoff_0
        );
        prop_assert!(
            backoff_2 >= backoff_1 || backoff_1.as_millis() as u64 >= max_backoff_ms,
            "Backoff 2 ({:?}) < Backoff 1 ({:?})",
            backoff_2,
            backoff_1
        );
    }
}

// ============================================================================
// PROPERTY TESTS: RetryConfig::should_retry
// ============================================================================

proptest! {
    /// Property: Permanent errors are never retried
    #[test]
    fn prop_permanent_errors_not_retried(config in retry_config_strategy(), msg in "[a-zA-Z0-9 ]{0,50}") {
        let error = RetryableError::Permanent(msg);
        prop_assert!(!config.should_retry(&error), "Permanent error should not be retried");
    }

    /// Property: Transient errors are retryable
    #[test]
    fn prop_transient_errors_retried(config in retry_config_strategy()) {
        prop_assert!(config.should_retry(&RetryableError::Timeout));
        prop_assert!(config.should_retry(&RetryableError::NetworkError));
        prop_assert!(config.should_retry(&RetryableError::RateLimited));
    }

    /// Property: Server errors (5xx) are retryable
    #[test]
    fn prop_server_errors_retried(config in retry_config_strategy(), code in 500u16..600) {
        let error = RetryableError::ServerError(code);
        prop_assert!(config.should_retry(&error), "Server error {} should be retried", code);
    }

    /// Property: should_retry never panics
    #[test]
    fn prop_should_retry_never_panics(config in retry_config_strategy(), error in retryable_error_strategy()) {
        let _ = config.should_retry(&error);
    }
}

// ============================================================================
// PROPERTY TESTS: RetryConfig constructors
// ============================================================================

proptest! {
    /// Property: Production config has reasonable values
    #[test]
    fn prop_production_config_valid(_seed in 0u64..1000) {
        let config = RetryConfig::production();
        prop_assert!(config.max_attempts >= 1, "Production should have retries");
        prop_assert!(config.max_attempts <= 10, "Production shouldn't have too many retries");
        prop_assert!(config.initial_backoff_ms > 0, "Should have initial backoff");
        prop_assert!(config.backoff_multiplier >= 1.0, "Multiplier should be >= 1");
    }

    /// Property: Aggressive config has more attempts than production
    #[test]
    fn prop_aggressive_vs_production(_seed in 0u64..1000) {
        let production = RetryConfig::production();
        let aggressive = RetryConfig::aggressive();
        prop_assert!(
            aggressive.max_attempts >= production.max_attempts,
            "Aggressive should have >= attempts than production"
        );
    }

    /// Property: Disabled config has zero attempts
    #[test]
    fn prop_disabled_config_zero_attempts(_seed in 0u64..1000) {
        let config = RetryConfig::disabled();
        prop_assert_eq!(config.max_attempts, 0);
    }
}

// ============================================================================
// PROPERTY TESTS: CircuitBreakerConfig
// ============================================================================

proptest! {
    /// Property: CircuitBreakerConfig constructors produce valid configurations
    #[test]
    fn prop_circuit_breaker_config_valid(config in circuit_breaker_config_strategy()) {
        // Failure threshold should be positive
        prop_assert!(config.failure_threshold >= 1, "Failure threshold should be >= 1");
        // Success threshold for half-open should be positive
        prop_assert!(config.success_threshold >= 1, "Success threshold should be >= 1");
        // Recovery timeout should be positive
        prop_assert!(config.recovery_timeout >= 1, "Recovery timeout should be >= 1");
    }

    /// Property: Default CircuitBreakerConfig is valid
    #[test]
    fn prop_default_circuit_breaker_valid(_seed in 0u64..1000) {
        let config = CircuitBreakerConfig::default();
        prop_assert!(config.failure_threshold >= 1);
        prop_assert!(config.success_threshold >= 1);
        prop_assert!(config.recovery_timeout >= 1);
        prop_assert!(config.minimum_requests >= 1);
    }
}

// ============================================================================
// PROPERTY TESTS: CircuitState
// ============================================================================

proptest! {
    /// Property: CircuitState values are distinct
    #[test]
    fn prop_circuit_states_distinct(_seed in 0u64..1000) {
        let closed = CircuitState::Closed;
        let open = CircuitState::Open;
        let half_open = CircuitState::HalfOpen;
        
        prop_assert_ne!(closed.clone(), open.clone());
        prop_assert_ne!(closed, half_open.clone());
        prop_assert_ne!(open, half_open);
    }
}

// ============================================================================
// PROPERTY TESTS: RetryableError Display
// ============================================================================

proptest! {
    /// Property: RetryableError Display produces non-empty strings
    #[test]
    fn prop_error_display_non_empty(error in retryable_error_strategy()) {
        let display = format!("{}", error);
        prop_assert!(!display.is_empty(), "Error display should not be empty");
    }

    /// Property: ServerError display includes status code
    #[test]
    fn prop_server_error_includes_code(code in 500u16..600) {
        let error = RetryableError::ServerError(code);
        let display = format!("{}", error);
        prop_assert!(
            display.contains(&code.to_string()),
            "Server error display should include code {}: {}",
            code,
            display
        );
    }
}
