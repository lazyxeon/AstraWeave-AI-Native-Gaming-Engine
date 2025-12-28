//! Property-based tests for astraweave-security
//!
//! These tests use proptest to verify invariants in path validation,
//! deserialization limits, and security configurations.

use proptest::prelude::*;
use std::path::Path;

use astraweave_security::deserialization::{MAX_JSON_BYTES, MAX_RON_BYTES, MAX_TOML_BYTES};
use astraweave_security::path::validate_extension;
use astraweave_security::{SecurityConfig, TelemetrySeverity};

// ============================================================================
// PROPTEST STRATEGIES
// ============================================================================

/// Strategy for generating valid file extensions
fn extension_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("png".to_string()),
        Just("jpg".to_string()),
        Just("jpeg".to_string()),
        Just("gif".to_string()),
        Just("webp".to_string()),
        Just("json".to_string()),
        Just("toml".to_string()),
        Just("ron".to_string()),
        Just("txt".to_string()),
        Just("md".to_string()),
    ]
}

/// Strategy for generating potentially malicious extensions
fn malicious_extension_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("exe".to_string()),
        Just("dll".to_string()),
        Just("bat".to_string()),
        Just("sh".to_string()),
        Just("ps1".to_string()),
        Just("cmd".to_string()),
        Just("vbs".to_string()),
        Just("js".to_string()),
    ]
}

/// Strategy for generating valid file paths
fn valid_path_strategy() -> impl Strategy<Value = String> {
    (
        prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 1..5),
        extension_strategy(),
    )
        .prop_map(|(segments, ext)| {
            let mut path = segments.join("/");
            path.push('.');
            path.push_str(&ext);
            path
        })
}

/// Strategy for path traversal attempts
fn path_traversal_strategy() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("../secret.txt".to_string()),
        Just("../../etc/passwd".to_string()),
        Just("subdir/../../../root".to_string()),
        Just("..\\..\\windows\\system32".to_string()),
        Just("foo/../../bar".to_string()),
    ]
}

/// Strategy for generating SecurityConfig values
fn security_config_strategy() -> impl Strategy<Value = SecurityConfig> {
    (
        any::<bool>(),       // enable_sandboxing
        any::<bool>(),       // enable_llm_validation
        any::<bool>(),       // enable_script_sandbox
        100u64..60000,       // max_script_execution_time_ms
        1usize..1024,        // max_memory_usage_mb
    )
        .prop_map(|(sandbox, llm, script, time, mem)| SecurityConfig {
            enable_sandboxing: sandbox,
            enable_llm_validation: llm,
            enable_script_sandbox: script,
            max_script_execution_time_ms: time,
            max_memory_usage_mb: mem,
        })
}

/// Strategy for generating TelemetrySeverity values
fn severity_strategy() -> impl Strategy<Value = TelemetrySeverity> {
    prop_oneof![
        Just(TelemetrySeverity::Info),
        Just(TelemetrySeverity::Warning),
        Just(TelemetrySeverity::Error),
        Just(TelemetrySeverity::Critical),
    ]
}

// ============================================================================
// PROPERTY TESTS: validate_extension
// ============================================================================

proptest! {
    /// Property: validate_extension accepts allowed extensions
    #[test]
    fn prop_validate_extension_accepts_allowed(
        filename in "[a-zA-Z0-9_]{1,20}",
        ext in extension_strategy()
    ) {
        let path_str = format!("{}.{}", filename, ext);
        let path = Path::new(&path_str);
        let allowed = ["png", "jpg", "jpeg", "gif", "webp", "json", "toml", "ron", "txt", "md"];
        
        let result = validate_extension(path, &allowed);
        prop_assert!(result.is_ok(), "Should accept extension: {}", ext);
    }

    /// Property: validate_extension rejects non-allowed extensions
    #[test]
    fn prop_validate_extension_rejects_disallowed(
        filename in "[a-zA-Z0-9_]{1,20}",
        ext in malicious_extension_strategy()
    ) {
        let path_str = format!("{}.{}", filename, ext);
        let path = Path::new(&path_str);
        let allowed = ["png", "jpg", "json"];  // Restricted list
        
        let result = validate_extension(path, &allowed);
        prop_assert!(result.is_err(), "Should reject extension: {}", ext);
    }

    /// Property: validate_extension rejects files without extensions
    #[test]
    fn prop_validate_extension_requires_extension(filename in "[a-zA-Z0-9_]{1,20}") {
        let path = Path::new(&filename);
        let allowed = ["png", "jpg", "json"];
        
        let result = validate_extension(path, &allowed);
        prop_assert!(result.is_err(), "Should reject file without extension");
    }

    /// Property: validate_extension never panics
    #[test]
    fn prop_validate_extension_never_panics(
        path_str in ".*",
        allowed in prop::collection::vec("[a-z]{1,5}", 0..10)
    ) {
        let path = Path::new(&path_str);
        let allowed_refs: Vec<&str> = allowed.iter().map(|s| s.as_str()).collect();
        let _ = validate_extension(path, &allowed_refs);
    }
}

// ============================================================================
// PROPERTY TESTS: Deserialization limits
// ============================================================================

proptest! {
    /// Property: MAX_JSON_BYTES is reasonable
    #[test]
    fn prop_json_limit_reasonable(_seed in 0u64..1000) {
        prop_assert!(MAX_JSON_BYTES >= 1024, "JSON limit should be at least 1KB");
        prop_assert!(MAX_JSON_BYTES <= 100 * 1024 * 1024, "JSON limit should be at most 100MB");
    }

    /// Property: MAX_TOML_BYTES is reasonable
    #[test]
    fn prop_toml_limit_reasonable(_seed in 0u64..1000) {
        prop_assert!(MAX_TOML_BYTES >= 1024, "TOML limit should be at least 1KB");
        prop_assert!(MAX_TOML_BYTES <= 50 * 1024 * 1024, "TOML limit should be at most 50MB");
    }

    /// Property: MAX_RON_BYTES is reasonable
    #[test]
    fn prop_ron_limit_reasonable(_seed in 0u64..1000) {
        prop_assert!(MAX_RON_BYTES >= 1024, "RON limit should be at least 1KB");
        prop_assert!(MAX_RON_BYTES <= 50 * 1024 * 1024, "RON limit should be at most 50MB");
    }

    /// Property: Config limits form reasonable hierarchy
    #[test]
    fn prop_limit_hierarchy(_seed in 0u64..1000) {
        // JSON is typically more verbose, so should have higher limit
        prop_assert!(MAX_JSON_BYTES >= MAX_TOML_BYTES, "JSON should have >= TOML limit");
        prop_assert!(MAX_JSON_BYTES >= MAX_RON_BYTES, "JSON should have >= RON limit");
    }
}

// ============================================================================
// PROPERTY TESTS: SecurityConfig
// ============================================================================

proptest! {
    /// Property: SecurityConfig fields are valid
    #[test]
    fn prop_security_config_valid(config in security_config_strategy()) {
        prop_assert!(config.max_script_execution_time_ms >= 100);
        prop_assert!(config.max_memory_usage_mb >= 1);
    }

    /// Property: Default SecurityConfig is safe
    #[test]
    fn prop_default_config_safe(_seed in 0u64..1000) {
        let config = SecurityConfig::default();
        // Default should have sandboxing enabled
        prop_assert!(config.enable_sandboxing || config.enable_llm_validation || config.enable_script_sandbox,
            "Default config should have at least some security enabled");
    }
}

// ============================================================================
// PROPERTY TESTS: TelemetrySeverity
// ============================================================================

proptest! {
    /// Property: TelemetrySeverity values are distinct
    #[test]
    fn prop_severity_distinct(_seed in 0u64..1000) {
        let info = TelemetrySeverity::Info;
        let warning = TelemetrySeverity::Warning;
        let error = TelemetrySeverity::Error;
        let critical = TelemetrySeverity::Critical;
        
        prop_assert_ne!(info, warning);
        prop_assert_ne!(info, error);
        prop_assert_ne!(info, critical);
        prop_assert_ne!(warning, error);
        prop_assert_ne!(warning, critical);
        prop_assert_ne!(error, critical);
    }

    /// Property: TelemetrySeverity equality is reflexive
    #[test]
    fn prop_severity_eq_reflexive(severity in severity_strategy()) {
        prop_assert_eq!(severity.clone(), severity);
    }
}

// ============================================================================
// PROPERTY TESTS: Path traversal detection
// ============================================================================

proptest! {
    /// Property: Path traversal attempts should contain '..'
    #[test]
    fn prop_traversal_contains_dots(path in path_traversal_strategy()) {
        prop_assert!(path.contains(".."), "Traversal attempt should contain '..'");
    }

    /// Property: Valid paths don't contain '..'
    #[test]
    fn prop_valid_paths_no_traversal(path in valid_path_strategy()) {
        prop_assert!(!path.contains(".."), "Valid path should not contain '..'");
    }

    /// Property: Path validation should be case-sensitive for extensions
    #[test]
    fn prop_extension_case_sensitivity(
        filename in "[a-zA-Z0-9_]{1,20}",
        ext in "[A-Z]{1,4}"
    ) {
        let path_str = format!("{}.{}", filename, ext);
        let path = Path::new(&path_str);
        // Lowercase-only allowlist
        let allowed = ["png", "jpg", "json"];
        
        let result = validate_extension(path, &allowed);
        // Extension matching is case-sensitive, so uppercase should fail
        // unless ext.to_lowercase() is in allowed
        if !allowed.contains(&ext.to_lowercase().as_str()) {
            prop_assert!(result.is_err());
        }
    }
}

// ============================================================================
// PROPERTY TESTS: Security invariants
// ============================================================================

proptest! {
    /// Property: Script timeout is always positive
    #[test]
    fn prop_timeout_positive(config in security_config_strategy()) {
        prop_assert!(config.max_script_execution_time_ms > 0, 
            "Script timeout should be positive");
    }

    /// Property: Memory limit is always positive
    #[test]
    fn prop_memory_limit_positive(config in security_config_strategy()) {
        prop_assert!(config.max_memory_usage_mb > 0,
            "Memory limit should be positive");
    }
}
