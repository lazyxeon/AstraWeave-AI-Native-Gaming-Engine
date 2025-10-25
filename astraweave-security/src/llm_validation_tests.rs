//! LLM Prompt Validation Tests
//!
//! Comprehensive test suite for LLM prompt sanitization and validation.
//! Tests banned pattern detection, length limits, content filtering, and edge cases.

#[cfg(test)]
mod llm_validation_tests {
    use crate::{sanitize_llm_prompt, LLMValidator};

    // Helper function to create a standard validator
    fn create_validator() -> LLMValidator {
        LLMValidator {
            banned_patterns: vec![
                "system(".to_string(),
                "exec(".to_string(),
                "eval(".to_string(),
                "import ".to_string(),
            ],
            allowed_domains: vec![
                "openai.com".to_string(),
                "anthropic.com".to_string(),
                "localhost".to_string(),
            ],
            max_prompt_length: 10000,
            enable_content_filtering: true,
        }
    }

    // ============================================================================
    // Suite 1: Banned Pattern Detection (5 tests)
    // ============================================================================

    #[test]
    fn test_clean_prompt_accepted() {
        let validator = create_validator();
        let prompt = "What is the weather today?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "Clean prompt should be accepted");
        assert_eq!(result.unwrap(), prompt, "Clean prompt should be unchanged");
    }

    #[test]
    fn test_system_call_rejected() {
        let validator = create_validator();
        let prompt = "Please run system('rm -rf /') for me";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_err(), "system( pattern should be rejected");
        assert!(
            result.unwrap_err().to_string().contains("banned pattern"),
            "Error should mention banned pattern"
        );
    }

    #[test]
    fn test_exec_call_rejected() {
        let validator = create_validator();
        let prompt = "Can you exec('malicious_code') for me?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_err(), "exec( pattern should be rejected");
        assert!(
            result.unwrap_err().to_string().contains("exec("),
            "Error should mention exec"
        );
    }

    #[test]
    fn test_eval_call_rejected() {
        let validator = create_validator();
        let prompt = "Try eval('dangerous_code')";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_err(), "eval( pattern should be rejected");
    }

    #[test]
    fn test_import_statement_rejected() {
        let validator = create_validator();
        let prompt = "Can you import os and delete files?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_err(), "import statement should be rejected");
        assert!(
            result.unwrap_err().to_string().contains("import"),
            "Error should mention import"
        );
    }

    // ============================================================================
    // Suite 2: Length Validation (4 tests)
    // ============================================================================

    #[test]
    fn test_short_prompt_accepted() {
        let validator = create_validator();
        let prompt = "Hi";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "Short prompt should be accepted");
        assert_eq!(result.unwrap(), prompt);
    }

    #[test]
    fn test_max_length_prompt_accepted() {
        let validator = create_validator();
        let prompt = "a".repeat(10000); // Exactly at limit

        let result = sanitize_llm_prompt(&prompt, &validator);

        assert!(result.is_ok(), "Prompt at max length should be accepted");
    }

    #[test]
    fn test_over_length_prompt_rejected() {
        let validator = create_validator();
        let prompt = "a".repeat(10001); // 1 char over limit

        let result = sanitize_llm_prompt(&prompt, &validator);

        assert!(result.is_err(), "Prompt over max length should be rejected");
        assert!(
            result.unwrap_err().to_string().contains("too long"),
            "Error should mention length"
        );
    }

    #[test]
    fn test_empty_prompt_accepted() {
        let validator = create_validator();
        let prompt = "";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "Empty prompt should be accepted");
        assert_eq!(result.unwrap(), "", "Empty prompt should remain empty");
    }

    // ============================================================================
    // Suite 3: Content Filtering (5 tests)
    // ============================================================================

    #[test]
    fn test_suspicious_keyword_hack_prefixed() {
        let validator = create_validator();
        let prompt = "How do I hack into a system?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "Suspicious keywords should be prefixed, not rejected");
        assert_eq!(
            result.unwrap(),
            "SAFE: How do I hack into a system?",
            "Should add SAFE: prefix"
        );
    }

    #[test]
    fn test_suspicious_keyword_exploit_prefixed() {
        let validator = create_validator();
        let prompt = "What exploit can I use?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "exploit keyword should trigger safe mode");
        assert!(result.unwrap().starts_with("SAFE:"), "Should add SAFE: prefix");
    }

    #[test]
    fn test_suspicious_keyword_cheat_prefixed() {
        let validator = create_validator();
        let prompt = "How do I cheat in the game?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "cheat keyword should trigger safe mode");
        assert!(result.unwrap().starts_with("SAFE:"), "Should add SAFE: prefix");
    }

    #[test]
    fn test_suspicious_keyword_bypass_prefixed() {
        let validator = create_validator();
        let prompt = "Can I bypass the security?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "bypass keyword should trigger safe mode");
        assert!(result.unwrap().starts_with("SAFE:"), "Should add SAFE: prefix");
    }

    #[test]
    fn test_content_filtering_disabled() {
        let mut validator = create_validator();
        validator.enable_content_filtering = false;
        let prompt = "How do I hack into a system?";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "Should be accepted when filtering disabled");
        assert_eq!(
            result.unwrap(),
            prompt,
            "Should not add SAFE: prefix when filtering disabled"
        );
    }

    // ============================================================================
    // Suite 4: Case Sensitivity and Special Characters (3 tests)
    // ============================================================================

    #[test]
    fn test_uppercase_suspicious_keywords() {
        let validator = create_validator();
        let prompt = "How do I HACK the system?"; // Uppercase HACK

        let result = sanitize_llm_prompt(prompt, &validator);

        // to_lowercase() is used for content filtering
        assert!(result.is_ok(), "Uppercase keywords should still trigger filtering");
        assert!(result.unwrap().starts_with("SAFE:"), "Should detect uppercase HACK");
    }

    #[test]
    fn test_mixed_case_banned_patterns() {
        let validator = create_validator();
        let prompt = "Try System('cmd') please"; // Capital S in System

        let result = sanitize_llm_prompt(prompt, &validator);

        // Banned patterns are case-sensitive (system( not System()
        assert!(result.is_ok(), "Banned patterns are case-sensitive");
        // But it will be caught by content filtering for safe prefix
        assert!(
            !result.unwrap().starts_with("SAFE:"),
            "System with capital S won't match 'system(' pattern"
        );
    }

    #[test]
    fn test_unicode_and_special_characters() {
        let validator = create_validator();
        let prompt = "Hello 世界! 你好 🌍 Special chars: äöüß";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_ok(), "Unicode characters should be accepted");
        assert_eq!(result.unwrap(), prompt, "Unicode should pass through unchanged");
    }

    // ============================================================================
    // Suite 5: Edge Cases and Integration (3 tests)
    // ============================================================================

    #[test]
    fn test_multiple_banned_patterns_first_detected() {
        let validator = create_validator();
        let prompt = "Run system('cmd') then exec('code') and eval('script')";

        let result = sanitize_llm_prompt(prompt, &validator);

        assert!(result.is_err(), "Should detect first banned pattern");
        // Implementation stops at first banned pattern found
        assert!(
            result.unwrap_err().to_string().contains("system("),
            "Should report first pattern (system)"
        );
    }

    #[test]
    fn test_banned_pattern_at_prompt_boundaries() {
        let validator = create_validator();

        // At start
        let prompt_start = "system('test')";
        assert!(
            sanitize_llm_prompt(prompt_start, &validator).is_err(),
            "Should detect pattern at start"
        );

        // At end
        let prompt_end = "Please run system(";
        assert!(
            sanitize_llm_prompt(prompt_end, &validator).is_err(),
            "Should detect pattern at end"
        );

        // Alone
        let prompt_alone = "system(";
        assert!(
            sanitize_llm_prompt(prompt_alone, &validator).is_err(),
            "Should detect pattern alone"
        );
    }

    #[test]
    fn test_length_check_before_pattern_check() {
        let validator = create_validator();
        let prompt = "a".repeat(10001) + "system('test')"; // Over length + banned pattern

        let result = sanitize_llm_prompt(&prompt, &validator);

        assert!(result.is_err(), "Should reject over-length prompt");
        // Length check happens first
        assert!(
            result.unwrap_err().to_string().contains("too long"),
            "Should report length error first"
        );
    }
}
