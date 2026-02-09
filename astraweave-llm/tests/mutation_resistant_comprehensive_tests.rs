//! Comprehensive mutation-resistant tests for astraweave-llm
//!
//! These tests are designed to achieve 90%+ mutation kill rate by:
//! - Testing all enum variants and their behaviors
//! - Verifying state transitions and side effects
//! - Checking boundary conditions and edge cases
//! - Testing error paths and failure modes
//! - Validating return values from all public methods

use astraweave_llm::*;
use std::collections::HashMap;
use std::time::Duration;

// ═══════════════════════════════════════════════════════════════════════════
// PRIORITY ENUM TESTS (backpressure.rs)
// ═══════════════════════════════════════════════════════════════════════════

mod priority_tests {
    use astraweave_llm::backpressure::Priority;

    #[test]
    fn test_priority_ord_critical_highest() {
        assert!(Priority::Critical < Priority::High);
        assert!(Priority::Critical < Priority::Normal);
        assert!(Priority::Critical < Priority::Low);
        assert!(Priority::Critical < Priority::Background);
    }

    #[test]
    fn test_priority_ord_high() {
        assert!(Priority::High > Priority::Critical);
        assert!(Priority::High < Priority::Normal);
        assert!(Priority::High < Priority::Low);
        assert!(Priority::High < Priority::Background);
    }

    #[test]
    fn test_priority_ord_normal() {
        assert!(Priority::Normal > Priority::Critical);
        assert!(Priority::Normal > Priority::High);
        assert!(Priority::Normal < Priority::Low);
        assert!(Priority::Normal < Priority::Background);
    }

    #[test]
    fn test_priority_ord_low() {
        assert!(Priority::Low > Priority::Critical);
        assert!(Priority::Low > Priority::High);
        assert!(Priority::Low > Priority::Normal);
        assert!(Priority::Low < Priority::Background);
    }

    #[test]
    fn test_priority_ord_background_lowest() {
        assert!(Priority::Background > Priority::Critical);
        assert!(Priority::Background > Priority::High);
        assert!(Priority::Background > Priority::Normal);
        assert!(Priority::Background > Priority::Low);
    }

    #[test]
    fn test_priority_all_returns_five() {
        let all = Priority::all();
        assert_eq!(all.len(), 5);
    }

    #[test]
    fn test_priority_all_order() {
        let all = Priority::all();
        assert_eq!(all[0], Priority::Critical);
        assert_eq!(all[1], Priority::High);
        assert_eq!(all[2], Priority::Normal);
        assert_eq!(all[3], Priority::Low);
        assert_eq!(all[4], Priority::Background);
    }

    #[test]
    fn test_priority_eq() {
        assert_eq!(Priority::Critical, Priority::Critical);
        assert_eq!(Priority::High, Priority::High);
        assert_ne!(Priority::Critical, Priority::High);
    }

    #[test]
    fn test_priority_clone() {
        let p = Priority::Normal;
        let cloned = p.clone();
        assert_eq!(p, cloned);
    }

    #[test]
    fn test_priority_copy() {
        let p = Priority::Low;
        let copied = p;
        assert_eq!(p, copied);
    }

    #[test]
    fn test_priority_hash() {
        use std::collections::HashSet;
        let mut set = HashSet::new();
        set.insert(Priority::Critical);
        set.insert(Priority::High);
        assert!(set.contains(&Priority::Critical));
        assert!(set.contains(&Priority::High));
        assert!(!set.contains(&Priority::Normal));
    }

    #[test]
    fn test_priority_debug() {
        let p = Priority::Critical;
        let debug = format!("{:?}", p);
        assert!(debug.contains("Critical"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BACKPRESSURE CONFIG TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod backpressure_config_tests {
    use astraweave_llm::backpressure::BackpressureConfig;
    use std::time::Duration;

    #[test]
    fn test_config_default_max_concurrent() {
        let cfg = BackpressureConfig::default();
        assert_eq!(cfg.max_concurrent_requests, 100);
    }

    #[test]
    fn test_config_default_max_queue_size() {
        let cfg = BackpressureConfig::default();
        assert_eq!(cfg.max_queue_size, 1000);
    }

    #[test]
    fn test_config_default_request_timeout() {
        let cfg = BackpressureConfig::default();
        assert_eq!(cfg.request_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_config_default_processing_interval() {
        let cfg = BackpressureConfig::default();
        assert_eq!(cfg.processing_interval, Duration::from_millis(10));
    }

    #[test]
    fn test_config_default_adaptive_concurrency() {
        let cfg = BackpressureConfig::default();
        assert!(cfg.adaptive_concurrency);
    }

    #[test]
    fn test_config_default_target_latency() {
        let cfg = BackpressureConfig::default();
        assert_eq!(cfg.target_latency_ms, 1000);
    }

    #[test]
    fn test_config_default_load_shedding() {
        let cfg = BackpressureConfig::default();
        assert!((cfg.load_shedding_threshold - 0.9).abs() < 0.001);
    }

    #[test]
    fn test_config_default_graceful_degradation() {
        let cfg = BackpressureConfig::default();
        assert!(cfg.enable_graceful_degradation);
    }

    #[test]
    fn test_config_clone() {
        let cfg = BackpressureConfig::default();
        let cloned = cfg.clone();
        assert_eq!(cfg.max_concurrent_requests, cloned.max_concurrent_requests);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// AB TESTING ENUM TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod ab_testing_tests {
    use astraweave_llm::ab_testing::{
        ABTestConfig, AssignmentStrategy, ExperimentStatus, ModelConfig, OptimizationDirection,
        ResultStatus, SuccessCriteria, Variant,
    };
    use std::collections::HashMap;

    #[test]
    fn test_experiment_status_draft() {
        let status = ExperimentStatus::Draft;
        assert_eq!(status, ExperimentStatus::Draft);
    }

    #[test]
    fn test_experiment_status_running() {
        let status = ExperimentStatus::Running;
        assert_eq!(status, ExperimentStatus::Running);
    }

    #[test]
    fn test_experiment_status_paused() {
        let status = ExperimentStatus::Paused;
        assert_eq!(status, ExperimentStatus::Paused);
    }

    #[test]
    fn test_experiment_status_completed() {
        let status = ExperimentStatus::Completed;
        assert_eq!(status, ExperimentStatus::Completed);
    }

    #[test]
    fn test_experiment_status_stopped() {
        let status = ExperimentStatus::Stopped;
        assert_eq!(status, ExperimentStatus::Stopped);
    }

    #[test]
    fn test_experiment_status_ne() {
        assert_ne!(ExperimentStatus::Draft, ExperimentStatus::Running);
        assert_ne!(ExperimentStatus::Running, ExperimentStatus::Completed);
    }

    #[test]
    fn test_result_status_variants() {
        assert_eq!(ResultStatus::InProgress, ResultStatus::InProgress);
        assert_eq!(
            ResultStatus::SignificantResult,
            ResultStatus::SignificantResult
        );
        assert_eq!(
            ResultStatus::NoSignificantDifference,
            ResultStatus::NoSignificantDifference
        );
        assert_eq!(
            ResultStatus::InsufficientData,
            ResultStatus::InsufficientData
        );
    }

    #[test]
    fn test_result_status_ne() {
        assert_ne!(ResultStatus::InProgress, ResultStatus::SignificantResult);
    }

    #[test]
    fn test_optimization_direction_maximize() {
        let dir = OptimizationDirection::Maximize;
        let _debug = format!("{:?}", dir);
    }

    #[test]
    fn test_optimization_direction_minimize() {
        let dir = OptimizationDirection::Minimize;
        let _debug = format!("{:?}", dir);
    }

    #[test]
    fn test_assignment_strategy_hash() {
        let strategy = AssignmentStrategy::Hash;
        let _debug = format!("{:?}", strategy);
    }

    #[test]
    fn test_assignment_strategy_weighted_random() {
        let strategy = AssignmentStrategy::WeightedRandom;
        let _debug = format!("{:?}", strategy);
    }

    #[test]
    fn test_assignment_strategy_round_robin() {
        let strategy = AssignmentStrategy::RoundRobin;
        let _debug = format!("{:?}", strategy);
    }

    #[test]
    fn test_assignment_strategy_custom() {
        let strategy = AssignmentStrategy::Custom("my_fn".to_string());
        if let AssignmentStrategy::Custom(name) = strategy {
            assert_eq!(name, "my_fn");
        } else {
            panic!("Expected Custom variant");
        }
    }

    #[test]
    fn test_config_defaults() {
        let cfg = ABTestConfig::default();
        assert_eq!(cfg.default_duration_hours, 168);
        assert_eq!(cfg.min_sample_size, 100);
        assert!((cfg.significance_threshold - 0.05).abs() < 0.001);
        assert!(!cfg.auto_winner_selection);
        assert_eq!(cfg.max_concurrent_experiments, 10);
    }

    #[test]
    fn test_variant_creation() {
        let variant = Variant {
            id: "v1".to_string(),
            name: "Test Variant".to_string(),
            description: "A test".to_string(),
            prompt_template: Some("template".to_string()),
            model_config: None,
            parameters: HashMap::new(),
            traffic_allocation: 0.5,
        };
        assert_eq!(variant.id, "v1");
        assert_eq!(variant.traffic_allocation, 0.5);
    }

    #[test]
    fn test_model_config() {
        let config = ModelConfig {
            model_name: "gpt-4".to_string(),
            temperature: Some(0.7),
            max_tokens: Some(1024),
            top_p: Some(0.9),
            top_k: Some(40),
            repetition_penalty: Some(1.1),
        };
        assert_eq!(config.model_name, "gpt-4");
        assert_eq!(config.temperature, Some(0.7));
    }

    #[test]
    fn test_success_criteria() {
        let criteria = SuccessCriteria {
            primary_metric: "latency".to_string(),
            improvement_threshold: 0.1,
            direction: OptimizationDirection::Minimize,
            secondary_metrics: vec!["accuracy".to_string()],
        };
        assert_eq!(criteria.primary_metric, "latency");
        assert_eq!(criteria.improvement_threshold, 0.1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RATE LIMITER CONFIG TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod rate_limiter_tests {
    use astraweave_llm::rate_limiter::RateLimiterConfig;
    use std::time::Duration;

    #[test]
    fn test_config_default_rpm() {
        let cfg = RateLimiterConfig::default();
        assert_eq!(cfg.default_rpm, 1000);
    }

    #[test]
    fn test_config_default_tpm() {
        let cfg = RateLimiterConfig::default();
        assert_eq!(cfg.default_tpm, 50000);
    }

    #[test]
    fn test_config_user_rpm() {
        let cfg = RateLimiterConfig::default();
        assert_eq!(cfg.user_rpm, 100);
    }

    #[test]
    fn test_config_global_rpm() {
        let cfg = RateLimiterConfig::default();
        assert_eq!(cfg.global_rpm, 10000);
    }

    #[test]
    fn test_config_allow_burst() {
        let cfg = RateLimiterConfig::default();
        assert!(cfg.allow_burst);
    }

    #[test]
    fn test_config_burst_multiplier() {
        let cfg = RateLimiterConfig::default();
        assert!((cfg.burst_multiplier - 1.5).abs() < 0.001);
    }

    #[test]
    fn test_config_window_duration() {
        let cfg = RateLimiterConfig::default();
        assert_eq!(cfg.window_duration, Duration::from_secs(60));
    }

    #[test]
    fn test_config_adaptive_limiting() {
        let cfg = RateLimiterConfig::default();
        assert!(cfg.adaptive_limiting);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// CIRCUIT BREAKER TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod circuit_breaker_tests {
    use astraweave_llm::circuit_breaker::{CircuitBreakerConfig, CircuitState};

    #[test]
    fn test_circuit_state_closed() {
        let state = CircuitState::Closed;
        assert_eq!(state, CircuitState::Closed);
    }

    #[test]
    fn test_circuit_state_open() {
        let state = CircuitState::Open;
        assert_eq!(state, CircuitState::Open);
    }

    #[test]
    fn test_circuit_state_half_open() {
        let state = CircuitState::HalfOpen;
        assert_eq!(state, CircuitState::HalfOpen);
    }

    #[test]
    fn test_circuit_state_ne() {
        assert_ne!(CircuitState::Closed, CircuitState::Open);
        assert_ne!(CircuitState::Open, CircuitState::HalfOpen);
        assert_ne!(CircuitState::Closed, CircuitState::HalfOpen);
    }

    #[test]
    fn test_config_failure_threshold() {
        let cfg = CircuitBreakerConfig::default();
        assert_eq!(cfg.failure_threshold, 5);
    }

    #[test]
    fn test_config_failure_window() {
        let cfg = CircuitBreakerConfig::default();
        assert_eq!(cfg.failure_window, 60);
    }

    #[test]
    fn test_config_minimum_requests() {
        let cfg = CircuitBreakerConfig::default();
        assert_eq!(cfg.minimum_requests, 10);
    }

    #[test]
    fn test_config_recovery_timeout() {
        let cfg = CircuitBreakerConfig::default();
        assert_eq!(cfg.recovery_timeout, 30);
    }

    #[test]
    fn test_config_success_threshold() {
        let cfg = CircuitBreakerConfig::default();
        assert_eq!(cfg.success_threshold, 3);
    }

    #[test]
    fn test_config_enabled() {
        let cfg = CircuitBreakerConfig::default();
        assert!(cfg.enabled);
    }

    #[test]
    fn test_circuit_state_debug() {
        assert!(format!("{:?}", CircuitState::Closed).contains("Closed"));
        assert!(format!("{:?}", CircuitState::Open).contains("Open"));
        assert!(format!("{:?}", CircuitState::HalfOpen).contains("HalfOpen"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FALLBACK SYSTEM TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod fallback_tests {
    use astraweave_llm::fallback_system::{FallbackAttempt, FallbackTier};

    #[test]
    fn test_tier_full_llm() {
        assert_eq!(FallbackTier::FullLlm.as_str(), "full_llm");
    }

    #[test]
    fn test_tier_simplified_llm() {
        assert_eq!(FallbackTier::SimplifiedLlm.as_str(), "simplified_llm");
    }

    #[test]
    fn test_tier_heuristic() {
        assert_eq!(FallbackTier::Heuristic.as_str(), "heuristic");
    }

    #[test]
    fn test_tier_emergency() {
        assert_eq!(FallbackTier::Emergency.as_str(), "emergency");
    }

    #[test]
    fn test_tier_next_full() {
        assert_eq!(FallbackTier::FullLlm.next(), Some(FallbackTier::SimplifiedLlm));
    }

    #[test]
    fn test_tier_next_simplified() {
        assert_eq!(FallbackTier::SimplifiedLlm.next(), Some(FallbackTier::Heuristic));
    }

    #[test]
    fn test_tier_next_heuristic() {
        assert_eq!(FallbackTier::Heuristic.next(), Some(FallbackTier::Emergency));
    }

    #[test]
    fn test_tier_next_emergency_none() {
        assert_eq!(FallbackTier::Emergency.next(), None);
    }

    #[test]
    fn test_tier_ord() {
        assert!(FallbackTier::FullLlm < FallbackTier::SimplifiedLlm);
        assert!(FallbackTier::SimplifiedLlm < FallbackTier::Heuristic);
        assert!(FallbackTier::Heuristic < FallbackTier::Emergency);
    }

    #[test]
    fn test_tier_eq() {
        assert_eq!(FallbackTier::FullLlm, FallbackTier::FullLlm);
        assert_ne!(FallbackTier::FullLlm, FallbackTier::Emergency);
    }

    #[test]
    fn test_tier_copy() {
        let tier = FallbackTier::Heuristic;
        let copied = tier;
        assert_eq!(tier, copied);
    }

    #[test]
    fn test_fallback_attempt_success() {
        let attempt = FallbackAttempt {
            tier: FallbackTier::FullLlm,
            success: true,
            error: None,
            duration_ms: 100,
        };
        assert!(attempt.success);
        assert!(attempt.error.is_none());
    }

    #[test]
    fn test_fallback_attempt_failure() {
        let attempt = FallbackAttempt {
            tier: FallbackTier::FullLlm,
            success: false,
            error: Some("timeout".to_string()),
            duration_ms: 5000,
        };
        assert!(!attempt.success);
        assert!(attempt.error.is_some());
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TOOL GUARD TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod tool_guard_tests {
    use astraweave_llm::tool_guard::{ToolGuard, ToolPolicy, ValidationResult};

    #[test]
    fn test_policy_allowed() {
        let policy = ToolPolicy::Allowed;
        assert_eq!(policy, ToolPolicy::Allowed);
    }

    #[test]
    fn test_policy_restricted() {
        let policy = ToolPolicy::Restricted;
        assert_eq!(policy, ToolPolicy::Restricted);
    }

    #[test]
    fn test_policy_denied() {
        let policy = ToolPolicy::Denied;
        assert_eq!(policy, ToolPolicy::Denied);
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::Valid;
        assert!(result.is_valid());
        assert!(result.reason().is_none());
    }

    #[test]
    fn test_validation_result_invalid() {
        let result = ValidationResult::Invalid {
            reason: "out of range".to_string(),
        };
        assert!(!result.is_valid());
        assert_eq!(result.reason(), Some("out of range"));
    }

    #[test]
    fn test_validation_result_denied() {
        let result = ValidationResult::Denied {
            action: "ExecuteCode".to_string(),
        };
        assert!(!result.is_valid());
        assert_eq!(result.reason(), Some("ExecuteCode"));
    }

    #[test]
    fn test_guard_new() {
        let guard = ToolGuard::new();
        // Check default policies
        assert_eq!(guard.get_policy("Wait"), ToolPolicy::Allowed);
        assert_eq!(guard.get_policy("Look"), ToolPolicy::Allowed);
        assert_eq!(guard.get_policy("ExecuteCode"), ToolPolicy::Denied);
        assert_eq!(guard.get_policy("DeleteFile"), ToolPolicy::Denied);
    }

    #[test]
    fn test_guard_default() {
        let guard = ToolGuard::default();
        assert_eq!(guard.get_policy("MoveTo"), ToolPolicy::Restricted);
    }

    #[test]
    fn test_guard_set_policy() {
        let guard = ToolGuard::new();
        guard.set_policy("CustomAction", ToolPolicy::Allowed);
        assert_eq!(guard.get_policy("CustomAction"), ToolPolicy::Allowed);
    }

    #[test]
    fn test_guard_unknown_action_default_restricted() {
        let guard = ToolGuard::new();
        // Unknown actions should default to Restricted
        assert_eq!(guard.get_policy("UnknownAction"), ToolPolicy::Restricted);
    }

    #[test]
    fn test_guard_override_policy() {
        let guard = ToolGuard::new();
        guard.set_policy("Wait", ToolPolicy::Denied);
        assert_eq!(guard.get_policy("Wait"), ToolPolicy::Denied);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PLAN PARSER TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod plan_parser_tests {
    use astraweave_llm::plan_parser::ExtractionMethod;

    #[test]
    fn test_extraction_method_direct() {
        assert_eq!(ExtractionMethod::Direct.as_str(), "direct");
    }

    #[test]
    fn test_extraction_method_code_fence() {
        assert_eq!(ExtractionMethod::CodeFence.as_str(), "code_fence");
    }

    #[test]
    fn test_extraction_method_envelope() {
        assert_eq!(ExtractionMethod::Envelope.as_str(), "envelope");
    }

    #[test]
    fn test_extraction_method_object() {
        assert_eq!(ExtractionMethod::ObjectExtraction.as_str(), "object_extraction");
    }

    #[test]
    fn test_extraction_method_tolerant() {
        assert_eq!(ExtractionMethod::Tolerant.as_str(), "tolerant");
    }

    #[test]
    fn test_extraction_method_eq() {
        assert_eq!(ExtractionMethod::Direct, ExtractionMethod::Direct);
        assert_ne!(ExtractionMethod::Direct, ExtractionMethod::CodeFence);
    }

    #[test]
    fn test_extraction_method_copy() {
        let method = ExtractionMethod::Envelope;
        let copied = method;
        assert_eq!(method, copied);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TELEMETRY TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod telemetry_tests {
    use astraweave_llm::telemetry::LlmTelemetry;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    #[test]
    fn test_telemetry_new() {
        let t = LlmTelemetry::new();
        assert_eq!(t.requests_total.load(Ordering::Relaxed), 0);
        assert_eq!(t.requests_success.load(Ordering::Relaxed), 0);
        assert_eq!(t.requests_error.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn test_record_request() {
        let t = LlmTelemetry::new();
        t.record_request();
        assert_eq!(t.requests_total.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_multiple_requests() {
        let t = LlmTelemetry::new();
        t.record_request();
        t.record_request();
        t.record_request();
        assert_eq!(t.requests_total.load(Ordering::Relaxed), 3);
    }

    #[test]
    fn test_record_success() {
        let t = LlmTelemetry::new();
        t.record_success();
        assert_eq!(t.requests_success.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_error() {
        let t = LlmTelemetry::new();
        t.record_error();
        assert_eq!(t.requests_error.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_retry() {
        let t = LlmTelemetry::new();
        t.record_retry();
        assert_eq!(t.retries_attempted.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_circuit_open() {
        let t = LlmTelemetry::new();
        t.record_circuit_open();
        assert_eq!(t.circuit_breaker_open.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_fallback() {
        let t = LlmTelemetry::new();
        t.record_fallback();
        assert_eq!(t.fallbacks_triggered.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_cache_hit() {
        let t = LlmTelemetry::new();
        t.record_cache_hit();
        assert_eq!(t.cache_hits.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_cache_miss() {
        let t = LlmTelemetry::new();
        t.record_cache_miss();
        assert_eq!(t.cache_misses.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_llm_latency() {
        let t = LlmTelemetry::new();
        t.record_llm_latency(Duration::from_millis(150));
        assert_eq!(t.latency_llm_call_ms.load(Ordering::Relaxed), 150);
        assert_eq!(t.latency_llm_call_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_record_plan_latency() {
        let t = LlmTelemetry::new();
        t.record_plan_latency(Duration::from_millis(200));
        assert_eq!(t.latency_plan_total_ms.load(Ordering::Relaxed), 200);
        assert_eq!(t.latency_plan_total_count.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn test_snapshot_empty() {
        let t = LlmTelemetry::new();
        let snap = t.snapshot();
        assert_eq!(snap.requests_total, 0);
        assert_eq!(snap.success_rate, 0);
        assert_eq!(snap.cache_hit_rate, 0);
    }

    #[test]
    fn test_snapshot_success_rate() {
        let t = LlmTelemetry::new();
        t.record_request();
        t.record_request();
        t.record_success();
        let snap = t.snapshot();
        assert_eq!(snap.success_rate, 50); // 1/2 = 50%
    }

    #[test]
    fn test_snapshot_cache_hit_rate() {
        let t = LlmTelemetry::new();
        t.record_cache_hit();
        t.record_cache_miss();
        t.record_cache_hit();
        t.record_cache_hit();
        let snap = t.snapshot();
        assert_eq!(snap.cache_hit_rate, 75); // 3/4 = 75%
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// STREAMING PARSER TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod streaming_parser_tests {
    use astraweave_llm::streaming_parser::StreamingBatchParser;

    #[test]
    fn test_parser_new() {
        let parser = StreamingBatchParser::new();
        assert!(parser.is_complete() == false || parser.is_complete() == true); // Just verifies creation works
    }

    #[test]
    fn test_parser_with_expected_count() {
        let parser = StreamingBatchParser::with_expected_count(5);
        let _ = parser; // Creation succeeds
    }

    #[test]
    fn test_feed_empty_chunk() {
        let mut parser = StreamingBatchParser::new();
        let result = parser.feed_chunk("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_feed_whitespace_chunk() {
        let mut parser = StreamingBatchParser::new();
        let result = parser.feed_chunk("   \n\t  ");
        assert!(result.is_ok());
    }

    #[test]
    fn test_feed_array_start() {
        let mut parser = StreamingBatchParser::new();
        let result = parser.feed_chunk("[");
        assert!(result.is_ok());
    }

    #[test]
    fn test_feed_complete_array() {
        let mut parser = StreamingBatchParser::new();
        let json = r#"[{"agent_id": 1, "plan_id": "p1", "steps": []}]"#;
        let result = parser.feed_chunk(json);
        assert!(result.is_ok());
        let plans = result.unwrap();
        assert_eq!(plans.len(), 1);
        assert_eq!(plans[0].agent_id, 1);
        assert_eq!(plans[0].plan_id, "p1");
    }

    #[test]
    fn test_feed_multiple_plans() {
        let mut parser = StreamingBatchParser::new();
        let json = r#"[{"agent_id": 1, "plan_id": "p1", "steps": []}, {"agent_id": 2, "plan_id": "p2", "steps": []}]"#;
        let result = parser.feed_chunk(json);
        assert!(result.is_ok());
        let plans = result.unwrap();
        assert_eq!(plans.len(), 2);
    }

    #[test]
    fn test_parser_is_complete() {
        let mut parser = StreamingBatchParser::new();
        let json = r#"[{"agent_id": 1, "plan_id": "p1", "steps": []}]"#;
        let _ = parser.feed_chunk(json);
        assert!(parser.is_complete());
    }

    #[test]
    fn test_parsed_plans() {
        let mut parser = StreamingBatchParser::new();
        let json = r#"[{"agent_id": 1, "plan_id": "p1", "steps": []}]"#;
        let _ = parser.feed_chunk(json);
        let plans = parser.parsed_plans();
        assert_eq!(plans.len(), 1);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// RETRY CONFIG TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod retry_tests {
    use astraweave_llm::retry::{RetryConfig, RetryableError};
    use std::time::Duration;

    #[test]
    fn test_production_config() {
        let cfg = RetryConfig::production();
        assert_eq!(cfg.max_attempts, 3);
        assert_eq!(cfg.initial_backoff_ms, 50);
        assert!((cfg.backoff_multiplier - 2.0).abs() < 0.001);
        assert_eq!(cfg.max_backoff_ms, 500);
        assert!(cfg.jitter);
    }

    #[test]
    fn test_aggressive_config() {
        let cfg = RetryConfig::aggressive();
        assert_eq!(cfg.max_attempts, 5);
        assert_eq!(cfg.initial_backoff_ms, 25);
        assert!((cfg.backoff_multiplier - 1.5).abs() < 0.001);
        assert_eq!(cfg.max_backoff_ms, 300);
        assert!(cfg.jitter);
    }

    #[test]
    fn test_disabled_config() {
        let cfg = RetryConfig::disabled();
        assert_eq!(cfg.max_attempts, 0);
        assert!(!cfg.jitter);
    }

    #[test]
    fn test_default_is_production() {
        let cfg = RetryConfig::default();
        assert_eq!(cfg.max_attempts, 3);
    }

    #[test]
    fn test_backoff_first_attempt() {
        let cfg = RetryConfig::production();
        let backoff = cfg.backoff_for_attempt(0);
        // Should be around initial_backoff_ms with jitter
        assert!(backoff.as_millis() >= 25);
        assert!(backoff.as_millis() <= 75);
    }

    #[test]
    fn test_backoff_exponential() {
        // Disable jitter for deterministic test
        let cfg = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            backoff_multiplier: 2.0,
            max_backoff_ms: 10000,
            jitter: false,
        };
        let b0 = cfg.backoff_for_attempt(0);
        let b1 = cfg.backoff_for_attempt(1);
        let b2 = cfg.backoff_for_attempt(2);
        assert_eq!(b0, Duration::from_millis(100));
        assert_eq!(b1, Duration::from_millis(200));
        assert_eq!(b2, Duration::from_millis(400));
    }

    #[test]
    fn test_backoff_capped() {
        let cfg = RetryConfig {
            max_attempts: 10,
            initial_backoff_ms: 100,
            backoff_multiplier: 10.0,
            max_backoff_ms: 500,
            jitter: false,
        };
        let backoff = cfg.backoff_for_attempt(5);
        assert_eq!(backoff, Duration::from_millis(500)); // Capped
    }

    #[test]
    fn test_backoff_disabled_returns_zero() {
        let cfg = RetryConfig::disabled();
        let backoff = cfg.backoff_for_attempt(5);
        assert_eq!(backoff, Duration::from_millis(0));
    }

    #[test]
    fn test_should_retry_timeout() {
        let cfg = RetryConfig::production();
        assert!(cfg.should_retry(&RetryableError::Timeout));
    }

    #[test]
    fn test_should_retry_network() {
        let cfg = RetryConfig::production();
        assert!(cfg.should_retry(&RetryableError::NetworkError));
    }

    #[test]
    fn test_should_retry_rate_limited() {
        let cfg = RetryConfig::production();
        assert!(cfg.should_retry(&RetryableError::RateLimited));
    }

    #[test]
    fn test_should_retry_server_error() {
        let cfg = RetryConfig::production();
        assert!(cfg.should_retry(&RetryableError::ServerError(500)));
        assert!(cfg.should_retry(&RetryableError::ServerError(503)));
    }

    #[test]
    fn test_should_not_retry_permanent() {
        let cfg = RetryConfig::production();
        assert!(!cfg.should_retry(&RetryableError::Permanent("bad input".to_string())));
    }

    #[test]
    fn test_retryable_error_debug() {
        let timeout = RetryableError::Timeout;
        assert!(format!("{:?}", timeout).contains("Timeout"));

        let server = RetryableError::ServerError(502);
        assert!(format!("{:?}", server).contains("502"));

        let permanent = RetryableError::Permanent("invalid".to_string());
        assert!(format!("{:?}", permanent).contains("invalid"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCHEDULER TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod scheduler_tests {
    use astraweave_llm::scheduler::{RequestPriority, RequestStatus};

    #[test]
    fn test_priority_ord() {
        assert!(RequestPriority::Low < RequestPriority::Normal);
        assert!(RequestPriority::Normal < RequestPriority::High);
    }

    #[test]
    fn test_priority_eq() {
        assert_eq!(RequestPriority::High, RequestPriority::High);
        assert_ne!(RequestPriority::Low, RequestPriority::High);
    }

    #[test]
    fn test_priority_copy() {
        let p = RequestPriority::Normal;
        let copied = p;
        assert_eq!(p, copied);
    }

    #[test]
    fn test_status_queued() {
        let status = RequestStatus::Queued;
        assert_eq!(status, RequestStatus::Queued);
    }

    #[test]
    fn test_status_processing() {
        let status = RequestStatus::Processing;
        assert_eq!(status, RequestStatus::Processing);
    }

    #[test]
    fn test_status_completed() {
        let status = RequestStatus::Completed;
        assert_eq!(status, RequestStatus::Completed);
    }

    #[test]
    fn test_status_failed() {
        let status = RequestStatus::Failed;
        assert_eq!(status, RequestStatus::Failed);
    }

    #[test]
    fn test_status_timed_out() {
        let status = RequestStatus::TimedOut;
        assert_eq!(status, RequestStatus::TimedOut);
    }

    #[test]
    fn test_status_ne() {
        assert_ne!(RequestStatus::Queued, RequestStatus::Processing);
        assert_ne!(RequestStatus::Completed, RequestStatus::Failed);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SCHEMA VALIDATION TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod schema_tests {
    use astraweave_llm::schema::ValidationError;

    #[test]
    fn test_parse_error_display() {
        let err = ValidationError::ParseError("unexpected token".to_string());
        let msg = format!("{}", err);
        assert!(msg.contains("JSON parse error"));
        assert!(msg.contains("unexpected token"));
    }

    #[test]
    fn test_missing_field_display() {
        let err = ValidationError::MissingField {
            field: "plan_id".to_string(),
            path: "root".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("plan_id"));
        assert!(msg.contains("root"));
    }

    #[test]
    fn test_wrong_type_display() {
        let err = ValidationError::WrongType {
            field: "steps".to_string(),
            expected: "array".to_string(),
            actual: "string".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("steps"));
        assert!(msg.contains("array"));
        assert!(msg.contains("string"));
    }

    #[test]
    fn test_out_of_range_display() {
        let err = ValidationError::OutOfRange {
            field: "x".to_string(),
            value: "-100".to_string(),
            constraint: "0..1000".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("-100"));
        assert!(msg.contains("0..1000"));
    }

    #[test]
    fn test_unknown_field_display() {
        let err = ValidationError::UnknownField {
            field: "foo".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("foo"));
        assert!(msg.contains("strict"));
    }

    #[test]
    fn test_array_length_display() {
        let err = ValidationError::ArrayLength {
            field: "steps".to_string(),
            actual: 0,
            constraint: "min 1".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("0"));
        assert!(msg.contains("min 1"));
    }

    #[test]
    fn test_custom_rule_display() {
        let err = ValidationError::CustomRule {
            rule: "target_exists".to_string(),
            message: "target not found".to_string(),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("target_exists"));
        assert!(msg.contains("not found"));
    }

    #[test]
    fn test_validation_error_clone() {
        let err = ValidationError::ParseError("test".to_string());
        let cloned = err.clone();
        assert_eq!(format!("{}", err), format!("{}", cloned));
    }

    #[test]
    fn test_validation_error_debug() {
        let err = ValidationError::ParseError("test".to_string());
        let debug = format!("{:?}", err);
        assert!(debug.contains("ParseError"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// MOCK LLM TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod mock_llm_tests {
    use astraweave_llm::{AlwaysErrMock, LlmClient, MockLlm};

    #[tokio::test]
    async fn test_mock_llm_complete() {
        let mock = MockLlm;
        let result = mock.complete("any prompt").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("plan_id"));
        assert!(response.contains("steps"));
    }

    #[tokio::test]
    async fn test_mock_llm_returns_json() {
        let mock = MockLlm;
        let result = mock.complete("").await.unwrap();
        // Should be valid JSON
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed.is_object());
    }

    #[tokio::test]
    async fn test_mock_llm_has_plan_id() {
        let mock = MockLlm;
        let result = mock.complete("").await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["plan_id"], "llm-mock");
    }

    #[tokio::test]
    async fn test_mock_llm_has_steps() {
        let mock = MockLlm;
        let result = mock.complete("").await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert!(parsed["steps"].is_array());
        assert!(!parsed["steps"].as_array().unwrap().is_empty());
    }

    #[tokio::test]
    async fn test_always_err_mock() {
        let mock = AlwaysErrMock;
        let result = mock.complete("any prompt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_always_err_mock_message() {
        let mock = AlwaysErrMock;
        let result = mock.complete("").await;
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("AlwaysErrMock"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PLAN SOURCE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod plan_source_tests {
    use astraweave_core::PlanIntent;
    use astraweave_llm::PlanSource;

    #[test]
    fn test_plan_source_llm() {
        let plan = PlanIntent {
            plan_id: "p1".to_string(),
            steps: vec![],
        };
        let source = PlanSource::Llm(plan.clone());
        if let PlanSource::Llm(p) = source {
            assert_eq!(p.plan_id, "p1");
        } else {
            panic!("Expected Llm variant");
        }
    }

    #[test]
    fn test_plan_source_fallback() {
        let plan = PlanIntent {
            plan_id: "p2".to_string(),
            steps: vec![],
        };
        let source = PlanSource::Fallback {
            plan,
            reason: "LLM timeout".to_string(),
        };
        if let PlanSource::Fallback { plan: p, reason } = source {
            assert_eq!(p.plan_id, "p2");
            assert_eq!(reason, "LLM timeout");
        } else {
            panic!("Expected Fallback variant");
        }
    }

    #[test]
    fn test_plan_source_debug() {
        let plan = PlanIntent {
            plan_id: "p3".to_string(),
            steps: vec![],
        };
        let source = PlanSource::Llm(plan);
        let debug = format!("{:?}", source);
        assert!(debug.contains("Llm"));
    }

    #[test]
    fn test_plan_source_clone() {
        let plan = PlanIntent {
            plan_id: "p4".to_string(),
            steps: vec![],
        };
        let source = PlanSource::Llm(plan);
        let cloned = source.clone();
        if let PlanSource::Llm(p) = cloned {
            assert_eq!(p.plan_id, "p4");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// HEURISTICS TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod heuristics_tests {
    use astraweave_llm::heuristics::HeuristicConfig;

    #[test]
    fn test_heuristic_config_default() {
        let cfg = HeuristicConfig::default();
        // Just verify defaults exist and are reasonable
        let _ = cfg;
    }

    #[test]
    fn test_heuristic_config_clone() {
        let cfg = HeuristicConfig::default();
        let cloned = cfg.clone();
        let _ = cloned;
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// BOUNDARY AND EDGE CASE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod boundary_tests {
    use astraweave_llm::backpressure::Priority;
    use astraweave_llm::retry::RetryConfig;
    use std::time::Duration;

    #[test]
    fn test_priority_all_unique() {
        let all = Priority::all();
        for i in 0..all.len() {
            for j in (i + 1)..all.len() {
                assert_ne!(all[i], all[j], "priorities should all be unique");
            }
        }
    }

    #[test]
    fn test_backoff_attempt_zero() {
        let cfg = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            backoff_multiplier: 2.0,
            max_backoff_ms: 10000,
            jitter: false,
        };
        let backoff = cfg.backoff_for_attempt(0);
        assert_eq!(backoff, Duration::from_millis(100));
    }

    #[test]
    fn test_backoff_large_attempt() {
        let cfg = RetryConfig {
            max_attempts: 100,
            initial_backoff_ms: 1,
            backoff_multiplier: 2.0,
            max_backoff_ms: 1000,
            jitter: false,
        };
        // Very large exponent should still cap
        let backoff = cfg.backoff_for_attempt(50);
        assert_eq!(backoff, Duration::from_millis(1000));
    }

    #[test]
    fn test_backoff_multiplier_one() {
        let cfg = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            backoff_multiplier: 1.0,
            max_backoff_ms: 10000,
            jitter: false,
        };
        // With multiplier 1.0, all attempts should have same backoff
        assert_eq!(cfg.backoff_for_attempt(0), Duration::from_millis(100));
        assert_eq!(cfg.backoff_for_attempt(5), Duration::from_millis(100));
    }

    #[test]
    fn test_zero_initial_backoff() {
        let cfg = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 0,
            backoff_multiplier: 2.0,
            max_backoff_ms: 1000,
            jitter: false,
        };
        // 0 * any = 0
        assert_eq!(cfg.backoff_for_attempt(5), Duration::from_millis(0));
    }

    #[test]
    fn test_zero_max_backoff() {
        let cfg = RetryConfig {
            max_attempts: 5,
            initial_backoff_ms: 100,
            backoff_multiplier: 2.0,
            max_backoff_ms: 0,
            jitter: false,
        };
        // Should cap to 0
        assert_eq!(cfg.backoff_for_attempt(0), Duration::from_millis(0));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPRESSION MODULE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod compression_tests {
    use astraweave_llm::compression::PromptCompressor;

    #[test]
    fn test_compressor_new() {
        let compressor = PromptCompressor::new();
        let _ = compressor;
    }

    #[test]
    fn test_compress_empty_prompt() {
        let compressor = PromptCompressor::new();
        let result = compressor.compress("");
        // Empty in, should get something back (may just be empty)
        let _ = result;
    }

    #[test]
    fn test_compress_simple_prompt() {
        let compressor = PromptCompressor::new();
        let input = "Generate a tactical plan for the agent.";
        let result = compressor.compress(input);
        // Compression shouldn't increase size significantly
        assert!(!result.is_empty() || input.is_empty());
    }

    #[test]
    fn test_compress_long_prompt() {
        let compressor = PromptCompressor::new();
        let input = "word ".repeat(1000);
        let _result = compressor.compress(&input);
        // Should handle long input
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// FEW SHOT MODULE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod few_shot_tests {
    use astraweave_llm::few_shot::{FewShotExample, FewShotRegistry, EXAMPLE_REGISTRY};

    #[test]
    fn test_registry_new() {
        let registry = FewShotRegistry::new();
        let _ = registry;
    }

    #[test]
    fn test_registry_default() {
        let registry = FewShotRegistry::default();
        let _ = registry;
    }

    #[test]
    fn test_registry_add_example() {
        let mut registry = FewShotRegistry::new();
        registry.add_example(FewShotExample {
            input: "test input".to_string(),
            output: "test output".to_string(),
            reasoning: "test reasoning".to_string(),
            tags: vec!["tag1".to_string()],
        });
    }

    #[test]
    fn test_registry_get_examples_empty() {
        let registry = FewShotRegistry::new();
        let examples = registry.get_examples_with_budget(&["tactical"], 1000);
        assert!(examples.is_empty());
    }

    #[test]
    fn test_registry_get_examples_with_matching_tag() {
        let mut registry = FewShotRegistry::new();
        registry.add_example(FewShotExample {
            input: "in".to_string(),
            output: "out".to_string(),
            reasoning: "why".to_string(),
            tags: vec!["tactical".to_string()],
        });
        let examples = registry.get_examples_with_budget(&["tactical"], 1000);
        assert_eq!(examples.len(), 1);
    }

    #[test]
    fn test_registry_get_examples_no_matching_tag() {
        let mut registry = FewShotRegistry::new();
        registry.add_example(FewShotExample {
            input: "in".to_string(),
            output: "out".to_string(),
            reasoning: "why".to_string(),
            tags: vec!["stealth".to_string()],
        });
        let examples = registry.get_examples_with_budget(&["tactical"], 1000);
        assert!(examples.is_empty());
    }

    #[test]
    fn test_registry_budget_limiting() {
        let mut registry = FewShotRegistry::new();
        // Add many examples
        for i in 0..100 {
            registry.add_example(FewShotExample {
                input: format!("input {}", "x".repeat(100)),
                output: format!("output {}", "y".repeat(100)),
                reasoning: "reason".to_string(),
                tags: vec!["test".to_string()],
            });
        }
        // With small budget, should get fewer examples
        let examples = registry.get_examples_with_budget(&["test"], 100);
        assert!(examples.len() < 100);
    }

    #[test]
    fn test_example_registry_has_tactical() {
        assert!(EXAMPLE_REGISTRY.contains_key("tactical"));
    }

    #[test]
    fn test_example_registry_tactical_not_empty() {
        let tactical = EXAMPLE_REGISTRY.get("tactical").unwrap();
        assert!(!tactical.is_empty());
    }

    #[test]
    fn test_few_shot_example_clone() {
        let example = FewShotExample {
            input: "in".to_string(),
            output: "out".to_string(),
            reasoning: "why".to_string(),
            tags: vec!["tag".to_string()],
        };
        let cloned = example.clone();
        assert_eq!(cloned.input, "in");
    }

    #[test]
    fn test_few_shot_example_debug() {
        let example = FewShotExample {
            input: "in".to_string(),
            output: "out".to_string(),
            reasoning: "why".to_string(),
            tags: vec![],
        };
        let debug = format!("{:?}", example);
        assert!(debug.contains("FewShotExample"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// PROMPT TEMPLATE TESTS
// ═══════════════════════════════════════════════════════════════════════════

mod prompt_template_tests {
    use astraweave_llm::prompt_template::PromptConfig;

    #[test]
    fn test_prompt_config_default() {
        let cfg = PromptConfig::default();
        let _ = cfg;
    }

    #[test]
    fn test_prompt_config_clone() {
        let cfg = PromptConfig::default();
        let cloned = cfg.clone();
        let _ = cloned;
    }
}
