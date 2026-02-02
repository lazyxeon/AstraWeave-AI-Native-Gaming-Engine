//! Comprehensive mutation-killing tests for astraweave-prompts
//!
//! These tests are designed to catch arithmetic and logical mutations
//! by verifying specific expected values and behavioral correctness.

// =============================================================================
// TEMPLATE FORMAT TESTS
// =============================================================================

#[cfg(test)]
mod template_format_tests {
    use crate::TemplateFormat;

    #[test]
    fn template_format_all_returns_three() {
        assert_eq!(TemplateFormat::all().len(), 3);
    }

    #[test]
    fn template_format_names_distinct() {
        let all = TemplateFormat::all();
        for (i, f1) in all.iter().enumerate() {
            for (j, f2) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(f1.name(), f2.name());
                }
            }
        }
    }

    #[test]
    fn template_format_extensions_distinct() {
        let all = TemplateFormat::all();
        for (i, f1) in all.iter().enumerate() {
            for (j, f2) in all.iter().enumerate() {
                if i != j {
                    assert_ne!(f1.extension(), f2.extension());
                }
            }
        }
    }
}

// =============================================================================
// TRUST LEVEL TESTS
// =============================================================================

#[cfg(test)]
mod trust_level_tests {
    use crate::TrustLevel;

    #[test]
    fn trust_level_all_returns_three() {
        assert_eq!(TrustLevel::all().len(), 3);
    }

    #[test]
    fn trust_level_ordering() {
        assert!(TrustLevel::User < TrustLevel::Developer);
        assert!(TrustLevel::Developer < TrustLevel::System);
    }

    #[test]
    fn trust_level_numeric_values() {
        assert_eq!(TrustLevel::User.level(), 0);
        assert_eq!(TrustLevel::Developer.level(), 1);
        assert_eq!(TrustLevel::System.level(), 2);
    }
}

// =============================================================================
// CACHE CONFIG TESTS
// =============================================================================

#[cfg(test)]
mod cache_config_tests {
    use crate::CacheConfig;

    #[test]
    fn cache_config_default_enabled() {
        let config = CacheConfig::default();
        assert!(config.enabled);
    }

    #[test]
    fn cache_config_default_positive_max() {
        let config = CacheConfig::default();
        assert!(config.max_templates > 0);
    }

    #[test]
    fn cache_config_default_positive_ttl() {
        let config = CacheConfig::default();
        assert!(config.ttl_seconds > 0);
    }

    #[test]
    fn cache_config_disabled_state() {
        let config = CacheConfig::disabled();
        assert!(!config.enabled);
        assert_eq!(config.max_templates, 0);
    }
}

// =============================================================================
// VALIDATION CONFIG TESTS
// =============================================================================

#[cfg(test)]
mod validation_config_tests {
    use crate::ValidationConfig;

    #[test]
    fn validation_config_default_enabled() {
        let config = ValidationConfig::default();
        assert!(config.enabled);
    }

    #[test]
    fn validation_config_default_positive_depth() {
        let config = ValidationConfig::default();
        assert!(config.max_recursion_depth > 0);
    }

    #[test]
    fn validation_config_strict_all_enabled() {
        let config = ValidationConfig::strict();
        assert!(config.enabled);
        assert!(config.strict_variables);
        assert!(config.schema_validation);
    }

    #[test]
    fn validation_config_permissive_disabled() {
        let config = ValidationConfig::permissive();
        assert!(!config.enabled);
    }
}

// =============================================================================
// SANITIZATION CONFIG TESTS
// =============================================================================

#[cfg(test)]
mod sanitization_config_tests {
    use crate::SanitizationConfig;

    #[test]
    fn sanitization_config_default_positive_max_input() {
        let config = SanitizationConfig::default();
        assert!(config.max_user_input_length > 0);
    }

    #[test]
    fn sanitization_config_default_positive_max_var_name() {
        let config = SanitizationConfig::default();
        assert!(config.max_variable_name_length > 0);
    }

    #[test]
    fn sanitization_config_default_blocks_injection() {
        let config = SanitizationConfig::default();
        assert!(config.block_injection_patterns);
    }

    #[test]
    fn sanitization_config_strict_smaller_limits() {
        let strict = SanitizationConfig::strict();
        let default = SanitizationConfig::default();
        assert!(strict.max_user_input_length < default.max_user_input_length);
    }
}

// =============================================================================
// BOUNDARY CONDITION TESTS - Test exact boundary values to catch < vs <= mutations
// =============================================================================

#[cfg(test)]
mod boundary_condition_tests {
    use crate::{CacheConfig, ValidationConfig, SanitizationConfig, TrustLevel};

    // --- TrustLevel boundary tests ---
    
    #[test]
    fn trust_level_user_is_level_zero() {
        assert_eq!(TrustLevel::User.level(), 0);
    }

    #[test]
    fn trust_level_system_is_level_two() {
        assert_eq!(TrustLevel::System.level(), 2);
    }

    // --- CacheConfig TTL boundaries ---
    
    #[test]
    fn cache_ttl_display_seconds_below_minute() {
        let mut config = CacheConfig::default();
        config.ttl_seconds = 59;
        let display = config.ttl_display();
        assert!(display.ends_with('s'), "TTL {} should end with 's'", display);
    }

    #[test]
    fn cache_ttl_display_minutes_below_hour() {
        let mut config = CacheConfig::default();
        config.ttl_seconds = 3599; // Just under 1 hour
        let display = config.ttl_display();
        assert!(display.ends_with('m'), "TTL {} should end with 'm'", display);
    }

    #[test]
    fn cache_ttl_display_hours_at_boundary() {
        let mut config = CacheConfig::default();
        config.ttl_seconds = 3600; // Exactly 1 hour
        let display = config.ttl_display();
        assert!(display.ends_with('h'), "TTL {} should end with 'h'", display);
    }

    // --- CacheConfig validity boundaries ---
    
    #[test]
    fn cache_config_valid_when_enabled_with_positive_values() {
        let mut config = CacheConfig::default();
        config.enabled = true;
        config.max_templates = 1;
        config.ttl_seconds = 1;
        assert!(config.is_valid());
    }

    #[test]
    fn cache_config_invalid_when_enabled_with_zero_max() {
        let mut config = CacheConfig::default();
        config.enabled = true;
        config.max_templates = 0;
        config.ttl_seconds = 1;
        assert!(!config.is_valid());
    }

    #[test]
    fn cache_config_invalid_when_enabled_with_zero_ttl() {
        let mut config = CacheConfig::default();
        config.enabled = true;
        config.max_templates = 1;
        config.ttl_seconds = 0;
        assert!(!config.is_valid());
    }

    #[test]
    fn cache_config_valid_when_disabled() {
        let config = CacheConfig::disabled();
        assert!(config.is_valid()); // Disabled is always valid
    }

    // --- ValidationConfig depth boundaries ---
    
    #[test]
    fn validation_default_depth_is_ten() {
        let config = ValidationConfig::default();
        assert_eq!(config.max_recursion_depth, 10);
    }

    #[test]
    fn validation_permissive_depth_is_higher() {
        let permissive = ValidationConfig::permissive();
        let default = ValidationConfig::default();
        assert!(permissive.max_recursion_depth > default.max_recursion_depth);
    }

    // --- SanitizationConfig length boundaries ---
    
    #[test]
    fn sanitization_default_max_input_is_10000() {
        let config = SanitizationConfig::default();
        assert_eq!(config.max_user_input_length, 10_000);
    }

    #[test]
    fn sanitization_default_max_var_name_is_128() {
        let config = SanitizationConfig::default();
        assert_eq!(config.max_variable_name_length, 128);
    }

    #[test]
    fn sanitization_default_nesting_depth_is_10() {
        let config = SanitizationConfig::default();
        assert_eq!(config.max_nesting_depth, 10);
    }
}

// =============================================================================
// COMPARISON OPERATOR TESTS - Test to catch == vs != and < vs > swaps
// =============================================================================

#[cfg(test)]
mod comparison_operator_tests {
    use crate::{TemplateFormat, TrustLevel, CacheConfig, ValidationConfig};

    // --- TemplateFormat equality ---
    
    #[test]
    fn template_format_handlebars_equals_handlebars() {
        assert_eq!(TemplateFormat::Handlebars, TemplateFormat::Handlebars);
    }

    #[test]
    fn template_format_simple_equals_simple() {
        assert_eq!(TemplateFormat::Simple, TemplateFormat::Simple);
    }

    #[test]
    fn template_format_jinja2_equals_jinja2() {
        assert_eq!(TemplateFormat::Jinja2, TemplateFormat::Jinja2);
    }

    #[test]
    fn template_format_handlebars_not_equals_simple() {
        assert_ne!(TemplateFormat::Handlebars, TemplateFormat::Simple);
    }

    #[test]
    fn template_format_simple_not_equals_jinja2() {
        assert_ne!(TemplateFormat::Simple, TemplateFormat::Jinja2);
    }

    // --- TrustLevel ordering ---
    
    #[test]
    fn trust_user_less_than_developer() {
        assert!(TrustLevel::User < TrustLevel::Developer);
    }

    #[test]
    fn trust_developer_less_than_system() {
        assert!(TrustLevel::Developer < TrustLevel::System);
    }

    #[test]
    fn trust_user_not_greater_than_system() {
        assert!(!(TrustLevel::User > TrustLevel::System));
    }

    // --- TrustLevel equality ---
    
    #[test]
    fn trust_user_equals_user() {
        assert_eq!(TrustLevel::User, TrustLevel::User);
    }

    #[test]
    fn trust_developer_not_equals_system() {
        assert_ne!(TrustLevel::Developer, TrustLevel::System);
    }

    // --- CacheConfig comparisons ---
    
    #[test]
    fn cache_default_ttl_greater_than_zero() {
        let config = CacheConfig::default();
        assert!(config.ttl_seconds > 0);
    }

    #[test]
    fn cache_default_max_greater_than_zero() {
        let config = CacheConfig::default();
        assert!(config.max_templates > 0);
    }

    // --- ValidationConfig strictness comparison ---
    
    #[test]
    fn strict_validation_depth_equals_default_depth() {
        let strict = ValidationConfig::strict();
        let default = ValidationConfig::default();
        assert_eq!(strict.max_recursion_depth, default.max_recursion_depth);
    }
}

// =============================================================================
// BOOLEAN RETURN PATH TESTS - Test all paths through boolean-returning functions
// =============================================================================

#[cfg(test)]
mod boolean_return_path_tests {
    use crate::{TemplateFormat, TrustLevel, CacheConfig, ValidationConfig};

    // --- TemplateFormat.is_handlebars() paths ---
    
    #[test]
    fn is_handlebars_returns_true() {
        assert!(TemplateFormat::Handlebars.is_handlebars());
    }

    #[test]
    fn is_handlebars_returns_false_for_simple() {
        assert!(!TemplateFormat::Simple.is_handlebars());
    }

    #[test]
    fn is_handlebars_returns_false_for_jinja2() {
        assert!(!TemplateFormat::Jinja2.is_handlebars());
    }

    // --- TemplateFormat.is_simple() paths ---
    
    #[test]
    fn is_simple_returns_true() {
        assert!(TemplateFormat::Simple.is_simple());
    }

    #[test]
    fn is_simple_returns_false_for_handlebars() {
        assert!(!TemplateFormat::Handlebars.is_simple());
    }

    // --- TemplateFormat.is_jinja2() paths ---
    
    #[test]
    fn is_jinja2_returns_true() {
        assert!(TemplateFormat::Jinja2.is_jinja2());
    }

    #[test]
    fn is_jinja2_returns_false_for_handlebars() {
        assert!(!TemplateFormat::Handlebars.is_jinja2());
    }

    // --- TemplateFormat.supports_helpers() paths ---
    
    #[test]
    fn supports_helpers_true_for_handlebars() {
        assert!(TemplateFormat::Handlebars.supports_helpers());
    }

    #[test]
    fn supports_helpers_true_for_jinja2() {
        assert!(TemplateFormat::Jinja2.supports_helpers());
    }

    #[test]
    fn supports_helpers_false_for_simple() {
        assert!(!TemplateFormat::Simple.supports_helpers());
    }

    // --- TemplateFormat.supports_partials() paths ---
    
    #[test]
    fn supports_partials_true_for_handlebars() {
        assert!(TemplateFormat::Handlebars.supports_partials());
    }

    #[test]
    fn supports_partials_false_for_simple() {
        assert!(!TemplateFormat::Simple.supports_partials());
    }

    // --- TrustLevel.is_user() paths ---
    
    #[test]
    fn is_user_returns_true() {
        assert!(TrustLevel::User.is_user());
    }

    #[test]
    fn is_user_returns_false_for_developer() {
        assert!(!TrustLevel::Developer.is_user());
    }

    #[test]
    fn is_user_returns_false_for_system() {
        assert!(!TrustLevel::System.is_user());
    }

    // --- TrustLevel.is_developer() paths ---
    
    #[test]
    fn is_developer_returns_true() {
        assert!(TrustLevel::Developer.is_developer());
    }

    #[test]
    fn is_developer_returns_false_for_user() {
        assert!(!TrustLevel::User.is_developer());
    }

    // --- TrustLevel.is_system() paths ---
    
    #[test]
    fn is_system_returns_true() {
        assert!(TrustLevel::System.is_system());
    }

    #[test]
    fn is_system_returns_false_for_developer() {
        assert!(!TrustLevel::Developer.is_system());
    }

    // --- TrustLevel.is_trusted() paths ---
    
    #[test]
    fn is_trusted_false_for_user() {
        assert!(!TrustLevel::User.is_trusted());
    }

    #[test]
    fn is_trusted_true_for_developer() {
        assert!(TrustLevel::Developer.is_trusted());
    }

    #[test]
    fn is_trusted_true_for_system() {
        assert!(TrustLevel::System.is_trusted());
    }

    // --- TrustLevel.requires_sanitization() paths ---
    
    #[test]
    fn requires_sanitization_true_for_user() {
        assert!(TrustLevel::User.requires_sanitization());
    }

    #[test]
    fn requires_sanitization_false_for_developer() {
        assert!(!TrustLevel::Developer.requires_sanitization());
    }

    #[test]
    fn requires_sanitization_false_for_system() {
        assert!(!TrustLevel::System.requires_sanitization());
    }

    // --- CacheConfig.is_enabled() paths ---
    
    #[test]
    fn is_enabled_true_for_default() {
        let config = CacheConfig::default();
        assert!(config.is_enabled());
    }

    #[test]
    fn is_enabled_false_for_disabled() {
        let config = CacheConfig::disabled();
        assert!(!config.is_enabled());
    }

    // --- CacheConfig.is_valid() paths ---
    
    #[test]
    fn is_valid_true_for_default() {
        let config = CacheConfig::default();
        assert!(config.is_valid());
    }

    #[test]
    fn is_valid_true_for_disabled() {
        let config = CacheConfig::disabled();
        assert!(config.is_valid());
    }

    // --- ValidationConfig.is_enabled() paths ---
    
    #[test]
    fn validation_is_enabled_true_for_default() {
        let config = ValidationConfig::default();
        assert!(config.is_enabled());
    }

    #[test]
    fn validation_is_enabled_false_for_permissive() {
        let config = ValidationConfig::permissive();
        assert!(!config.is_enabled());
    }

    // --- ValidationConfig.is_strict() paths ---
    
    #[test]
    fn validation_is_strict_false_for_default() {
        let config = ValidationConfig::default();
        assert!(!config.is_strict());
    }

    #[test]
    fn validation_is_strict_true_for_strict() {
        let config = ValidationConfig::strict();
        assert!(config.is_strict());
    }

    // --- ValidationConfig.has_schema_validation() paths ---
    
    #[test]
    fn has_schema_validation_false_for_default() {
        let config = ValidationConfig::default();
        assert!(!config.has_schema_validation());
    }

    #[test]
    fn has_schema_validation_true_for_strict() {
        let config = ValidationConfig::strict();
        assert!(config.has_schema_validation());
    }
}

// =============================================================================
// BEHAVIORAL CORRECTNESS TESTS
// =============================================================================

#[cfg(test)]
mod behavioral_correctness_tests {
    use crate::{TemplateFormat, TrustLevel, ValidationConfig};

    #[test]
    fn template_format_all_contains_all_variants() {
        let all = TemplateFormat::all();
        assert!(all.contains(&TemplateFormat::Handlebars));
        assert!(all.contains(&TemplateFormat::Simple));
        assert!(all.contains(&TemplateFormat::Jinja2));
    }

    #[test]
    fn trust_level_all_contains_all_variants() {
        let all = TrustLevel::all();
        assert!(all.contains(&TrustLevel::User));
        assert!(all.contains(&TrustLevel::Developer));
        assert!(all.contains(&TrustLevel::System));
    }

    #[test]
    fn trust_level_order_is_correct() {
        // Verify the ordering is User < Developer < System
        assert!(TrustLevel::User.level() < TrustLevel::Developer.level());
        assert!(TrustLevel::Developer.level() < TrustLevel::System.level());
    }

    #[test]
    fn validation_strictness_levels_distinct() {
        let default = ValidationConfig::default();
        let strict = ValidationConfig::strict();
        let permissive = ValidationConfig::permissive();
        
        assert_ne!(default.strictness_level(), strict.strictness_level());
        assert_ne!(strict.strictness_level(), permissive.strictness_level());
    }

    #[test]
    fn template_format_display_not_empty() {
        for format in TemplateFormat::all() {
            assert!(!format.to_string().is_empty());
        }
    }

    #[test]
    fn trust_level_display_not_empty() {
        for level in TrustLevel::all() {
            assert!(!level.to_string().is_empty());
        }
    }

    #[test]
    fn template_format_icon_is_char() {
        for format in TemplateFormat::all() {
            let icon = format.icon();
            // Icon should be a valid char
            assert!(icon as u32 > 0);
        }
    }

    #[test]
    fn trust_level_icon_not_empty() {
        for level in TrustLevel::all() {
            assert!(!level.icon().is_empty());
        }
    }
}
