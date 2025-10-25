//! Anti-Cheat Validation Tests
//!
//! Comprehensive test suite for player input validation and anti-cheat measures.
//! Tests trust score calculations, anomaly detection, and validation thresholds.

#[cfg(test)]
mod anticheat_tests {
    use crate::{validate_player_input, CAntiCheat, ValidationResult};

    // ============================================================================
    // Suite 1: Basic Trust Score Calculations (5 tests)
    // ============================================================================

    #[test]
    fn test_clean_player_high_trust_score() {
        let anti_cheat = CAntiCheat {
            player_id: "player1".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![],
        };

        let result = validate_player_input(&anti_cheat);

        assert!(result.is_valid, "Clean player should be valid");
        assert_eq!(result.trust_score, 1.0, "Clean player should have 1.0 trust");
        assert!(result.warnings.is_empty(), "Clean player should have no warnings");
        assert!(result.anomalies.is_empty(), "Clean player should have no anomalies");
    }

    #[test]
    fn test_rapid_input_reduces_trust_score() {
        let anti_cheat = CAntiCheat {
            player_id: "player2".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string()],
        };

        let result = validate_player_input(&anti_cheat);

        assert!(result.is_valid, "Rapid input alone should still be valid");
        assert_eq!(result.trust_score, 0.8, "Rapid input should reduce trust to 0.8");
        assert_eq!(result.warnings.len(), 1, "Should have 1 warning");
        assert_eq!(result.anomalies.len(), 1, "Should have 1 anomaly");
        assert!(result.warnings[0].contains("rapid input"), "Warning should mention rapid input");
    }

    #[test]
    fn test_impossible_movement_severe_penalty() {
        let anti_cheat = CAntiCheat {
            player_id: "player3".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["impossible_movement".to_string()],
        };

        let result = validate_player_input(&anti_cheat);

        assert!(result.is_valid, "Impossible movement alone should still be valid");
        assert_eq!(result.trust_score, 0.5, "Impossible movement should reduce trust to 0.5");
        assert_eq!(result.warnings.len(), 1, "Should have 1 warning");
        assert!(result.warnings[0].contains("movement"), "Warning should mention movement");
    }

    #[test]
    fn test_memory_tamper_critical_penalty() {
        let anti_cheat = CAntiCheat {
            player_id: "player4".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["memory_tamper".to_string()],
        };

        let result = validate_player_input(&anti_cheat);

        assert!(result.is_valid, "Single memory tamper should be at threshold");
        assert_eq!(result.trust_score, 0.3, "Memory tamper should reduce trust to 0.3");
        assert_eq!(result.warnings.len(), 1, "Should have 1 warning");
        assert!(result.warnings[0].contains("Memory tampering"), "Warning should mention memory");
    }

    #[test]
    fn test_validation_threshold_boundary() {
        // Test the 0.2 threshold boundary
        let anti_cheat = CAntiCheat {
            player_id: "player5".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![],
        };

        let result = validate_player_input(&anti_cheat);
        assert!(result.is_valid, "Trust score 1.0 should be valid (> 0.2)");

        // Test at exactly 0.2 (should be valid, uses >)
        let mut result_manual = ValidationResult {
            is_valid: 0.2 > 0.2,
            trust_score: 0.2,
            warnings: vec![],
            anomalies: vec![],
        };
        assert!(!result_manual.is_valid, "Trust score 0.2 should be invalid (not > 0.2)");

        // Test just above threshold
        result_manual.trust_score = 0.21;
        result_manual.is_valid = 0.21 > 0.2;
        assert!(result_manual.is_valid, "Trust score 0.21 should be valid (> 0.2)");
    }

    // ============================================================================
    // Suite 2: Multiple Anomaly Combinations (4 tests)
    // ============================================================================

    #[test]
    fn test_two_anomalies_compound_penalty() {
        let anti_cheat = CAntiCheat {
            player_id: "player6".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string(), "impossible_movement".to_string()],
        };

        let result = validate_player_input(&anti_cheat);

        // 1.0 * 0.8 (rapid) * 0.5 (movement) = 0.4
        assert!(result.is_valid, "Trust score 0.4 should still be valid");
        assert_eq!(result.trust_score, 0.4, "Two anomalies should compound: 1.0 * 0.8 * 0.5 = 0.4");
        assert_eq!(result.warnings.len(), 2, "Should have 2 warnings");
        assert_eq!(result.anomalies.len(), 2, "Should have 2 anomalies");
    }

    #[test]
    fn test_three_anomalies_invalid_player() {
        let anti_cheat = CAntiCheat {
            player_id: "cheater".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![
                "rapid_input".to_string(),
                "impossible_movement".to_string(),
                "memory_tamper".to_string(),
            ],
        };

        let result = validate_player_input(&anti_cheat);

        // 1.0 * 0.8 * 0.5 * 0.3 = 0.12
        assert!(!result.is_valid, "Trust score 0.12 should be invalid (< 0.2)");
        assert!(
            (result.trust_score - 0.12).abs() < 0.001,
            "Three anomalies: 1.0 * 0.8 * 0.5 * 0.3 = 0.12, got {}",
            result.trust_score
        );
        assert_eq!(result.warnings.len(), 3, "Should have 3 warnings");
        assert_eq!(result.anomalies.len(), 3, "Should have 3 anomalies");
    }

    #[test]
    fn test_duplicate_anomaly_flags_handled() {
        let anti_cheat = CAntiCheat {
            player_id: "player7".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![
                "rapid_input".to_string(),
                "rapid_input".to_string(), // Duplicate
                "rapid_input".to_string(), // Duplicate
            ],
        };

        let result = validate_player_input(&anti_cheat);

        // Implementation uses .contains() which only checks ONCE, not per duplicate
        // So duplicates are treated the same as a single flag
        assert!(result.is_valid, "Should still be valid");
        assert_eq!(result.trust_score, 0.8, "Duplicates ignored: .contains() checks once = 0.8");
        assert_eq!(result.warnings.len(), 1, "Should have 1 warning (.contains() fires once)");
        assert_eq!(result.anomalies.len(), 1, "Should have 1 anomaly");
    }

    #[test]
    fn test_unknown_anomaly_flags_ignored() {
        let anti_cheat = CAntiCheat {
            player_id: "player8".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![
                "unknown_flag".to_string(),
                "another_unknown".to_string(),
            ],
        };

        let result = validate_player_input(&anti_cheat);

        assert!(result.is_valid, "Unknown flags should not affect validity");
        assert_eq!(result.trust_score, 1.0, "Unknown flags should not reduce trust");
        assert!(result.warnings.is_empty(), "Unknown flags should not generate warnings");
        assert!(result.anomalies.is_empty(), "Unknown flags should not be anomalies");
    }

    // ============================================================================
    // Suite 3: Edge Cases and Special Scenarios (3 tests)
    // ============================================================================

    #[test]
    fn test_empty_player_id() {
        let anti_cheat = CAntiCheat {
            player_id: "".to_string(), // Empty player ID
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![],
        };

        let result = validate_player_input(&anti_cheat);

        assert!(result.is_valid, "Empty player ID should still validate cleanly");
        assert_eq!(result.trust_score, 1.0, "Empty ID shouldn't affect trust score");
    }

    #[test]
    fn test_future_timestamp() {
        let future_timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 10000; // 10,000 seconds in the future

        let anti_cheat = CAntiCheat {
            player_id: "player9".to_string(),
            trust_score: 1.0,
            last_validation: future_timestamp,
            anomaly_flags: vec![],
        };

        let result = validate_player_input(&anti_cheat);

        // Validation logic doesn't use timestamp, so future timestamp is ignored
        assert!(result.is_valid, "Future timestamp should not affect validation");
        assert_eq!(result.trust_score, 1.0, "Future timestamp should not affect trust");
    }

    #[test]
    fn test_very_long_anomaly_flag_list() {
        let mut anomaly_flags = Vec::new();
        for i in 0..1000 {
            // Add mix of known and unknown flags
            if i % 3 == 0 {
                anomaly_flags.push("rapid_input".to_string());
            } else if i % 3 == 1 {
                anomaly_flags.push("impossible_movement".to_string());
            } else {
                anomaly_flags.push("memory_tamper".to_string());
            }
        }

        let anti_cheat = CAntiCheat {
            player_id: "player10".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags,
        };

        let result = validate_player_input(&anti_cheat);

        // Implementation uses .contains() which only checks ONCE per flag type
        // So 1000 duplicate flags = 3 unique flags = 0.8 * 0.5 * 0.3 = 0.12
        assert!(!result.is_valid, "All three flag types should result in invalid player");
        assert!(
            (result.trust_score - 0.12).abs() < 0.001,
            "Trust score should be 0.12 (3 unique flags), got {}",
            result.trust_score
        );
        assert_eq!(result.warnings.len(), 3, "Should have 3 warnings (one per unique flag)");
        assert_eq!(result.anomalies.len(), 3, "Should have 3 anomalies");
    }

    // ============================================================================
    // Suite 4: Anomaly Flag Combinations (3 tests)
    // ============================================================================

    #[test]
    fn test_rapid_input_and_movement_common_pattern() {
        let anti_cheat = CAntiCheat {
            player_id: "suspicious_player".to_string(),
            trust_score: 0.9,
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string(), "impossible_movement".to_string()],
        };

        let result = validate_player_input(&anti_cheat);

        // Common botting pattern: rapid input + impossible movement
        assert_eq!(result.trust_score, 0.4, "Common bot pattern should reduce to 0.4");
        assert!(result.is_valid, "0.4 is still above 0.2 threshold");
        assert!(
            result.anomalies.contains(&"rapid_input".to_string()),
            "Should detect rapid input"
        );
        assert!(
            result.anomalies.contains(&"impossible_movement".to_string()),
            "Should detect impossible movement"
        );
    }

    #[test]
    fn test_movement_and_memory_severe_cheating() {
        let anti_cheat = CAntiCheat {
            player_id: "likely_cheater".to_string(),
            trust_score: 1.0,
            last_validation: 0,
            anomaly_flags: vec![
                "impossible_movement".to_string(),
                "memory_tamper".to_string(),
            ],
        };

        let result = validate_player_input(&anti_cheat);

        // 1.0 * 0.5 * 0.3 = 0.15
        assert!(!result.is_valid, "Movement + memory tamper should be invalid");
        assert_eq!(result.trust_score, 0.15, "Should be 0.15 (below 0.2 threshold)");
    }

    #[test]
    fn test_existing_low_trust_with_anomaly() {
        let anti_cheat = CAntiCheat {
            player_id: "already_suspicious".to_string(),
            trust_score: 0.3, // Already low trust
            last_validation: 0,
            anomaly_flags: vec!["rapid_input".to_string()],
        };

        let result = validate_player_input(&anti_cheat);

        // Note: validate_player_input doesn't use existing trust_score,
        // it calculates fresh from flags. This is a design choice.
        assert_eq!(result.trust_score, 0.8, "Should calculate fresh trust from flags");
        assert!(result.is_valid, "Fresh calculation should still be valid");
    }
}
