//! Mutation-resistant tests for security systems.
//!
//! These tests are designed to catch common mutations in anti-cheat,
//! LLM validation, signature verification, and sandboxing logic.

use crate::{
    generate_keypair, generate_signature, hash_data, sanitize_llm_prompt,
    validate_player_input, verify_signature, CAntiCheat, ExecutionLimits,
    LLMValidator, SecurityConfig, TelemetryEvent, TelemetrySeverity,
};

// ============================================================================
// SecurityConfig Tests
// ============================================================================

mod security_config_tests {
    use super::*;

    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig {
            enable_sandboxing: true,
            enable_llm_validation: true,
            enable_script_sandbox: true,
            max_script_execution_time_ms: 1000,
            max_memory_usage_mb: 50,
        };
        
        assert!(config.enable_sandboxing);
        assert!(config.enable_llm_validation);
        assert!(config.enable_script_sandbox);
    }

    #[test]
    fn test_security_config_max_time() {
        let config = SecurityConfig {
            enable_sandboxing: true,
            enable_llm_validation: true,
            enable_script_sandbox: true,
            max_script_execution_time_ms: 2000,
            max_memory_usage_mb: 100,
        };
        
        assert_eq!(config.max_script_execution_time_ms, 2000);
        assert_eq!(config.max_memory_usage_mb, 100);
    }
}

// ============================================================================
// TelemetrySeverity Tests
// ============================================================================

mod telemetry_severity_tests {
    use super::*;

    #[test]
    fn test_severity_equality() {
        assert_eq!(TelemetrySeverity::Info, TelemetrySeverity::Info);
        assert_eq!(TelemetrySeverity::Warning, TelemetrySeverity::Warning);
        assert_eq!(TelemetrySeverity::Error, TelemetrySeverity::Error);
        assert_eq!(TelemetrySeverity::Critical, TelemetrySeverity::Critical);
    }

    #[test]
    fn test_severity_inequality() {
        assert_ne!(TelemetrySeverity::Info, TelemetrySeverity::Warning);
        assert_ne!(TelemetrySeverity::Warning, TelemetrySeverity::Error);
        assert_ne!(TelemetrySeverity::Error, TelemetrySeverity::Critical);
    }
}

// ============================================================================
// TelemetryEvent Tests
// ============================================================================

mod telemetry_event_tests {
    use super::*;

    #[test]
    fn test_telemetry_event_creation() {
        let event = TelemetryEvent {
            timestamp: 12345,
            event_type: "test_event".to_string(),
            severity: TelemetrySeverity::Info,
            data: serde_json::json!({"key": "value"}),
        };
        
        assert_eq!(event.timestamp, 12345);
        assert_eq!(event.event_type, "test_event");
        assert_eq!(event.severity, TelemetrySeverity::Info);
    }

    #[test]
    fn test_telemetry_event_serialization() {
        let event = TelemetryEvent {
            timestamp: 100,
            event_type: "anomaly".to_string(),
            severity: TelemetrySeverity::Warning,
            data: serde_json::json!({}),
        };
        
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: TelemetryEvent = serde_json::from_str(&json).unwrap();
        
        assert_eq!(deserialized.timestamp, 100);
        assert_eq!(deserialized.event_type, "anomaly");
    }
}

// ============================================================================
// ExecutionLimits Tests
// ============================================================================

mod execution_limits_tests {
    use super::*;

    #[test]
    fn test_execution_limits_creation() {
        let limits = ExecutionLimits {
            max_operations: 10000,
            max_memory_bytes: 1024 * 1024,
            timeout_ms: 5000,
        };
        
        assert_eq!(limits.max_operations, 10000);
        assert_eq!(limits.max_memory_bytes, 1024 * 1024);
        assert_eq!(limits.timeout_ms, 5000);
    }

    #[test]
    fn test_execution_limits_memory_size() {
        let limits = ExecutionLimits {
            max_operations: 10000,
            max_memory_bytes: 1024 * 1024 * 10, // 10 MB
            timeout_ms: 5000,
        };
        
        assert!(limits.max_memory_bytes >= 1024 * 1024, "Should have at least 1MB");
    }
}

// ============================================================================
// CAntiCheat Tests
// ============================================================================

mod anti_cheat_tests {
    use super::*;

    #[test]
    fn test_anti_cheat_creation() {
        let ac = CAntiCheat {
            player_id: "player_123".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![],
        };
        
        assert_eq!(ac.player_id, "player_123");
        assert_eq!(ac.trust_score, 1.0);
        assert!(ac.anomaly_flags.is_empty());
    }

    #[test]
    fn test_anti_cheat_with_anomaly() {
        let ac = CAntiCheat {
            player_id: "player_456".to_string(),
            trust_score: 0.5,
            last_validation: 12345,
            anomaly_flags: vec!["rapid_input".to_string()],
        };
        
        assert!(!ac.anomaly_flags.is_empty());
        assert!(ac.anomaly_flags.contains(&"rapid_input".to_string()));
    }
}

// ============================================================================
// ValidationResult Tests
// ============================================================================

mod validation_result_tests {
    use super::*;

    #[test]
    fn test_validation_clean_player() {
        let ac = CAntiCheat {
            player_id: "clean_player".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![],
        };
        
        let result = validate_player_input(&ac);
        
        assert!(result.is_valid);
        assert_eq!(result.trust_score, 1.0);
        assert!(result.anomalies.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn test_validation_rapid_input() {
        let ac = CAntiCheat {
            player_id: "suspicious_player".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string()],
        };
        
        let result = validate_player_input(&ac);
        
        assert!(result.is_valid, "Rapid input alone shouldn't invalidate");
        assert!(result.trust_score < 1.0, "Trust should decrease");
        assert!(!result.anomalies.is_empty());
    }

    #[test]
    fn test_validation_impossible_movement() {
        let ac = CAntiCheat {
            player_id: "cheater".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["impossible_movement".to_string()],
        };
        
        let result = validate_player_input(&ac);
        
        assert!(result.trust_score < 1.0);
        assert!(result.trust_score < 0.6, "Should significantly reduce trust");
    }

    #[test]
    fn test_validation_memory_tamper() {
        let ac = CAntiCheat {
            player_id: "serious_cheater".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["memory_tamper".to_string()],
        };
        
        let result = validate_player_input(&ac);
        
        assert!(result.trust_score < 0.5, "Memory tampering should heavily reduce trust");
    }

    #[test]
    fn test_validation_multiple_anomalies() {
        let ac = CAntiCheat {
            player_id: "flagged_player".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![
                "rapid_input".to_string(),
                "impossible_movement".to_string(),
                "memory_tamper".to_string(),
            ],
        };
        
        let result = validate_player_input(&ac);
        
        // With all three penalties: 1.0 * 0.8 * 0.5 * 0.3 = 0.12
        assert!(!result.is_valid, "Multiple severe anomalies should invalidate");
        assert!(result.trust_score < 0.2);
    }

    #[test]
    fn test_validation_threshold() {
        // Test the validity threshold (trust_score > 0.2)
        let ac = CAntiCheat {
            player_id: "borderline".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["memory_tamper".to_string()], // 0.3 multiplier
        };
        
        let result = validate_player_input(&ac);
        
        // 1.0 * 0.3 = 0.3, which is > 0.2
        assert!(result.is_valid);
    }
}

// ============================================================================
// LLM Validation Tests
// ============================================================================

mod llm_validation_tests {
    use super::*;

    fn create_validator() -> LLMValidator {
        LLMValidator {
            banned_patterns: vec![
                "system(".to_string(),
                "exec(".to_string(),
                "eval(".to_string(),
            ],
            allowed_domains: vec!["localhost".to_string()],
            max_prompt_length: 1000,
            enable_content_filtering: false,
        }
    }

    #[test]
    fn test_sanitize_valid_prompt() {
        let validator = create_validator();
        let result = sanitize_llm_prompt("Hello, how are you?", &validator);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Hello, how are you?");
    }

    #[test]
    fn test_sanitize_banned_pattern_system() {
        let validator = create_validator();
        let result = sanitize_llm_prompt("run system(rm -rf /)", &validator);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_banned_pattern_exec() {
        let validator = create_validator();
        let result = sanitize_llm_prompt("exec(malicious_code)", &validator);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_banned_pattern_eval() {
        let validator = create_validator();
        let result = sanitize_llm_prompt("eval(user_input)", &validator);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_prompt_too_long() {
        let validator = create_validator();
        let long_prompt = "a".repeat(2000);
        let result = sanitize_llm_prompt(&long_prompt, &validator);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_sanitize_prompt_max_length() {
        let validator = create_validator();
        let max_prompt = "a".repeat(1000);
        let result = sanitize_llm_prompt(&max_prompt, &validator);
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_sanitize_content_filtering() {
        let validator = LLMValidator {
            banned_patterns: vec![],
            allowed_domains: vec![],
            max_prompt_length: 10000,
            enable_content_filtering: true,
        };
        
        let result = sanitize_llm_prompt("how to hack the game", &validator);
        
        // Content filtering prefixes suspicious content
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.starts_with("SAFE:"), "Should prefix suspicious content");
    }
}

// ============================================================================
// Signature Tests
// ============================================================================

mod crypto_signature_tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let (signing_key, verifying_key) = generate_keypair();
        
        // Verify that keys are different (can sign and verify)
        let data = b"test data";
        let signature = generate_signature(data, &signing_key);
        assert!(verify_signature(data, &signature, &verifying_key));
    }

    #[test]
    fn test_signature_verification_success() {
        let (signing_key, verifying_key) = generate_keypair();
        let data = b"important message";
        
        let signature = generate_signature(data, &signing_key);
        
        assert!(verify_signature(data, &signature, &verifying_key));
    }

    #[test]
    fn test_signature_verification_wrong_data() {
        let (signing_key, verifying_key) = generate_keypair();
        let data = b"original message";
        let tampered = b"tampered message";
        
        let signature = generate_signature(data, &signing_key);
        
        assert!(!verify_signature(tampered, &signature, &verifying_key));
    }

    #[test]
    fn test_signature_verification_wrong_key() {
        let (signing_key1, _verifying_key1) = generate_keypair();
        let (_signing_key2, verifying_key2) = generate_keypair();
        
        let data = b"test data";
        let signature = generate_signature(data, &signing_key1);
        
        // Verify with different key should fail
        assert!(!verify_signature(data, &signature, &verifying_key2));
    }

    #[test]
    fn test_signature_empty_data() {
        let (signing_key, verifying_key) = generate_keypair();
        let data = b"";
        
        let signature = generate_signature(data, &signing_key);
        
        assert!(verify_signature(data, &signature, &verifying_key));
    }

    #[test]
    fn test_signature_large_data() {
        let (signing_key, verifying_key) = generate_keypair();
        let data = vec![0u8; 10000]; // 10KB of data
        
        let signature = generate_signature(&data, &signing_key);
        
        assert!(verify_signature(&data, &signature, &verifying_key));
    }
}

// ============================================================================
// Hash Tests
// ============================================================================

mod hash_tests {
    use super::*;

    #[test]
    fn test_hash_consistency() {
        let data = b"test data";
        let hash1 = hash_data(data);
        let hash2 = hash_data(data);
        
        assert_eq!(hash1, hash2, "Same data should produce same hash");
    }

    #[test]
    fn test_hash_different_data() {
        let data1 = b"data 1";
        let data2 = b"data 2";
        
        let hash1 = hash_data(data1);
        let hash2 = hash_data(data2);
        
        assert_ne!(hash1, hash2, "Different data should produce different hash");
    }

    #[test]
    fn test_hash_length() {
        let data = b"any data";
        let hash = hash_data(data);
        
        // SHA-256 produces 64 hex characters (32 bytes * 2)
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_empty_data() {
        let hash = hash_data(b"");
        
        // Empty data should still produce a valid hash
        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_hash_is_hex() {
        let data = b"some data";
        let hash = hash_data(data);
        
        // All characters should be valid hex
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

// ============================================================================
// Behavioral Correctness Tests
// ============================================================================

mod behavioral_tests {
    use super::*;

    #[test]
    fn test_trust_score_decay() {
        // Test that multiple anomalies compound
        let ac_single = CAntiCheat {
            player_id: "test".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string()],
        };
        
        let ac_double = CAntiCheat {
            player_id: "test".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string(), "impossible_movement".to_string()],
        };
        
        let single_result = validate_player_input(&ac_single);
        let double_result = validate_player_input(&ac_double);
        
        assert!(double_result.trust_score < single_result.trust_score,
            "Multiple anomalies should compound trust reduction");
    }

    #[test]
    fn test_signature_integrity_chain() {
        // Test signing and verifying multiple pieces of data
        let (signing_key, verifying_key) = generate_keypair();
        
        let data1 = b"first message";
        let data2 = b"second message";
        let data3 = b"third message";
        
        let sig1 = generate_signature(data1, &signing_key);
        let sig2 = generate_signature(data2, &signing_key);
        let sig3 = generate_signature(data3, &signing_key);
        
        assert!(verify_signature(data1, &sig1, &verifying_key));
        assert!(verify_signature(data2, &sig2, &verifying_key));
        assert!(verify_signature(data3, &sig3, &verifying_key));
        
        // Cross-verification should fail
        assert!(!verify_signature(data1, &sig2, &verifying_key));
        assert!(!verify_signature(data2, &sig3, &verifying_key));
    }

    #[test]
    fn test_hash_collision_resistance() {
        // Very similar data should produce different hashes
        let data1 = b"test123";
        let data2 = b"test124"; // Just one character different
        
        let hash1 = hash_data(data1);
        let hash2 = hash_data(data2);
        
        assert_ne!(hash1, hash2);
    }
}
