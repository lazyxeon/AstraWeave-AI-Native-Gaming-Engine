//! Mutation-Resistant Tests: UI Toast System
//!
//! Comprehensive tests for ToastLevel, ToastAction, Toast
//! to achieve ≥92% mutation kill rate.

use aw_editor_lib::ui::toast::{ToastLevel, ToastAction, Toast};

// =============================================================================
// TOAST LEVEL - IS_PROBLEM() TESTS
// =============================================================================

mod toast_level_is_problem_tests {
    use super::*;

    #[test]
    fn info_is_not_problem() {
        assert!(!ToastLevel::Info.is_problem());
    }

    #[test]
    fn success_is_not_problem() {
        assert!(!ToastLevel::Success.is_problem());
    }

    #[test]
    fn warning_is_problem() {
        assert!(ToastLevel::Warning.is_problem());
    }

    #[test]
    fn error_is_problem() {
        assert!(ToastLevel::Error.is_problem());
    }
}

// =============================================================================
// TOAST LEVEL - IS_SUCCESS() TESTS
// =============================================================================

mod toast_level_is_success_tests {
    use super::*;

    #[test]
    fn info_is_not_success() {
        assert!(!ToastLevel::Info.is_success());
    }

    #[test]
    fn success_is_success() {
        assert!(ToastLevel::Success.is_success());
    }

    #[test]
    fn warning_is_not_success() {
        assert!(!ToastLevel::Warning.is_success());
    }

    #[test]
    fn error_is_not_success() {
        assert!(!ToastLevel::Error.is_success());
    }
}

// =============================================================================
// TOAST LEVEL - SEVERITY() TESTS (BOUNDARY CONDITIONS)
// =============================================================================

mod toast_level_severity_tests {
    use super::*;

    #[test]
    fn info_severity_is_0() {
        assert_eq!(ToastLevel::Info.severity(), 0);
    }

    #[test]
    fn success_severity_is_1() {
        assert_eq!(ToastLevel::Success.severity(), 1);
    }

    #[test]
    fn warning_severity_is_2() {
        assert_eq!(ToastLevel::Warning.severity(), 2);
    }

    #[test]
    fn error_severity_is_3() {
        assert_eq!(ToastLevel::Error.severity(), 3);
    }

    #[test]
    fn severity_ordering_is_correct() {
        assert!(ToastLevel::Info.severity() < ToastLevel::Success.severity());
        assert!(ToastLevel::Success.severity() < ToastLevel::Warning.severity());
        assert!(ToastLevel::Warning.severity() < ToastLevel::Error.severity());
    }

    #[test]
    fn max_severity_is_3() {
        let max_sev = [
            ToastLevel::Info.severity(),
            ToastLevel::Success.severity(),
            ToastLevel::Warning.severity(),
            ToastLevel::Error.severity(),
        ].into_iter().max().unwrap();
        assert_eq!(max_sev, 3);
    }

    #[test]
    fn min_severity_is_0() {
        let min_sev = [
            ToastLevel::Info.severity(),
            ToastLevel::Success.severity(),
            ToastLevel::Warning.severity(),
            ToastLevel::Error.severity(),
        ].into_iter().min().unwrap();
        assert_eq!(min_sev, 0);
    }
}

// =============================================================================
// TOAST LEVEL - COLOR() TESTS
// Color32 is [u8; 4] (r, g, b, a) with values 0-255
// =============================================================================

mod toast_level_color_tests {
    use super::*;

    #[test]
    fn info_color_alpha_is_positive() {
        let color = ToastLevel::Info.color();
        assert!(color.a() > 0, "Alpha should be positive");
    }

    #[test]
    fn success_color_alpha_is_positive() {
        let color = ToastLevel::Success.color();
        assert!(color.a() > 0, "Alpha should be positive");
    }

    #[test]
    fn warning_color_alpha_is_positive() {
        let color = ToastLevel::Warning.color();
        assert!(color.a() > 0, "Alpha should be positive");
    }

    #[test]
    fn error_color_alpha_is_positive() {
        let color = ToastLevel::Error.color();
        assert!(color.a() > 0, "Alpha should be positive");
    }

    #[test]
    fn all_colors_are_different() {
        let info = ToastLevel::Info.color();
        let success = ToastLevel::Success.color();
        let warning = ToastLevel::Warning.color();
        let error = ToastLevel::Error.color();

        assert_ne!(info, success);
        assert_ne!(success, warning);
        assert_ne!(warning, error);
    }

    #[test]
    fn error_color_has_red_component() {
        let color = ToastLevel::Error.color();
        // Error is Color32::from_rgb(200, 60, 60)
        assert!(color.r() > 128, "Error should have significant red");
    }

    #[test]
    fn success_color_has_green_component() {
        let color = ToastLevel::Success.color();
        // Success is Color32::from_rgb(40, 160, 80)
        assert!(color.g() > 100, "Success should have green component");
    }

    #[test]
    fn warning_color_has_yellow_orange() {
        let color = ToastLevel::Warning.color();
        // Warning is Color32::from_rgb(200, 140, 40)
        assert!(color.r() > 150, "Warning should have red");
        assert!(color.g() > 100, "Warning should have medium green");
    }
}

// =============================================================================
// TOAST LEVEL - ICON() TESTS
// =============================================================================

mod toast_level_icon_tests {
    use super::*;

    #[test]
    fn info_icon_is_not_empty() {
        assert!(!ToastLevel::Info.icon().is_empty());
    }

    #[test]
    fn success_icon_is_not_empty() {
        assert!(!ToastLevel::Success.icon().is_empty());
    }

    #[test]
    fn warning_icon_is_not_empty() {
        assert!(!ToastLevel::Warning.icon().is_empty());
    }

    #[test]
    fn error_icon_is_not_empty() {
        assert!(!ToastLevel::Error.icon().is_empty());
    }

    #[test]
    fn all_icons_are_different() {
        let info = ToastLevel::Info.icon();
        let success = ToastLevel::Success.icon();
        let warning = ToastLevel::Warning.icon();
        let error = ToastLevel::Error.icon();

        assert_ne!(info, success);
        assert_ne!(success, warning);
        assert_ne!(warning, error);
    }
}

// =============================================================================
// TOAST LEVEL - NAME() TESTS
// =============================================================================

mod toast_level_name_tests {
    use super::*;

    #[test]
    fn info_name_is_info() {
        assert_eq!(ToastLevel::Info.name(), "Info");
    }

    #[test]
    fn success_name_is_success() {
        assert_eq!(ToastLevel::Success.name(), "Success");
    }

    #[test]
    fn warning_name_is_warning() {
        assert_eq!(ToastLevel::Warning.name(), "Warning");
    }

    #[test]
    fn error_name_is_error() {
        assert_eq!(ToastLevel::Error.name(), "Error");
    }
}

// =============================================================================
// TOAST LEVEL - ALL() TESTS
// =============================================================================

mod toast_level_all_tests {
    use super::*;

    #[test]
    fn all_returns_4_levels() {
        assert_eq!(ToastLevel::all().len(), 4);
    }

    #[test]
    fn all_contains_info() {
        assert!(ToastLevel::all().contains(&ToastLevel::Info));
    }

    #[test]
    fn all_contains_success() {
        assert!(ToastLevel::all().contains(&ToastLevel::Success));
    }

    #[test]
    fn all_contains_warning() {
        assert!(ToastLevel::all().contains(&ToastLevel::Warning));
    }

    #[test]
    fn all_contains_error() {
        assert!(ToastLevel::all().contains(&ToastLevel::Error));
    }
}

// =============================================================================
// TOAST ACTION - IS_MUTATING() TESTS
// =============================================================================

mod toast_action_is_mutating_tests {
    use super::*;

    #[test]
    fn undo_is_mutating() {
        assert!(ToastAction::Undo.is_mutating());
    }

    #[test]
    fn view_details_is_not_mutating() {
        let action = ToastAction::ViewDetails("test".to_string());
        assert!(!action.is_mutating());
    }

    #[test]
    fn retry_is_mutating() {
        assert!(ToastAction::Retry.is_mutating());
    }

    #[test]
    fn open_is_not_mutating() {
        let action = ToastAction::Open("/path/to/file".to_string());
        assert!(!action.is_mutating());
    }
}

// =============================================================================
// TOAST ACTION - LABEL() TESTS
// =============================================================================

mod toast_action_label_tests {
    use super::*;

    #[test]
    fn undo_label_not_empty() {
        assert!(!ToastAction::Undo.label().is_empty());
    }

    #[test]
    fn view_details_label_not_empty() {
        let action = ToastAction::ViewDetails("error details".to_string());
        assert!(!action.label().is_empty());
    }

    #[test]
    fn retry_label_not_empty() {
        assert!(!ToastAction::Retry.label().is_empty());
    }

    #[test]
    fn open_label_not_empty() {
        let action = ToastAction::Open("/path".to_string());
        assert!(!action.label().is_empty());
    }
}

// =============================================================================
// TOAST ACTION - ICON() TESTS
// =============================================================================

mod toast_action_icon_tests {
    use super::*;

    #[test]
    fn undo_icon_not_empty() {
        assert!(!ToastAction::Undo.icon().is_empty());
    }

    #[test]
    fn view_details_icon_not_empty() {
        let action = ToastAction::ViewDetails("details".to_string());
        assert!(!action.icon().is_empty());
    }

    #[test]
    fn retry_icon_not_empty() {
        assert!(!ToastAction::Retry.icon().is_empty());
    }

    #[test]
    fn open_icon_not_empty() {
        let action = ToastAction::Open("/path".to_string());
        assert!(!action.icon().is_empty());
    }
}

// =============================================================================
// TOAST - CREATION TESTS
// Toast::new(message, level) - message first, level second
// =============================================================================

mod toast_creation_tests {
    use super::*;

    #[test]
    fn new_toast_has_correct_level() {
        let toast = Toast::new("Test message", ToastLevel::Info);
        assert_eq!(toast.level, ToastLevel::Info);
    }

    #[test]
    fn new_toast_has_correct_message() {
        let toast = Toast::new("Warning message", ToastLevel::Warning);
        assert_eq!(toast.message, "Warning message");
    }

    #[test]
    fn info_toast_creation() {
        let toast = Toast::new("Info message", ToastLevel::Info);
        assert_eq!(toast.level, ToastLevel::Info);
    }

    #[test]
    fn success_toast_creation() {
        let toast = Toast::new("Success message", ToastLevel::Success);
        assert_eq!(toast.level, ToastLevel::Success);
    }

    #[test]
    fn warning_toast_creation() {
        let toast = Toast::new("Warning message", ToastLevel::Warning);
        assert_eq!(toast.level, ToastLevel::Warning);
    }

    #[test]
    fn error_toast_creation() {
        let toast = Toast::new("Error message", ToastLevel::Error);
        assert_eq!(toast.level, ToastLevel::Error);
    }
}

// =============================================================================
// TOAST - DURATION TESTS
// =============================================================================

mod toast_duration_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn with_duration_sets_custom_duration() {
        let toast = Toast::new("Test", ToastLevel::Info).with_duration(Duration::from_secs(10));
        assert_eq!(toast.duration, Duration::from_secs(10));
    }

    #[test]
    fn default_duration_is_positive() {
        let toast = Toast::new("Test", ToastLevel::Info);
        assert!(toast.duration > Duration::ZERO);
    }

    #[test]
    fn default_duration_is_4_seconds() {
        let toast = Toast::new("Test", ToastLevel::Info);
        assert_eq!(toast.duration, Duration::from_secs(4));
    }
}

// =============================================================================
// TOAST - ACTION TESTS
// Toast uses actions: Vec<ToastAction> (plural), not Option<ToastAction>
// =============================================================================

mod toast_action_chaining_tests {
    use super::*;

    #[test]
    fn with_action_adds_action() {
        let toast = Toast::new("Test", ToastLevel::Info).with_action(ToastAction::Undo);
        assert_eq!(toast.actions.len(), 1);
    }

    #[test]
    fn default_has_no_actions() {
        let toast = Toast::new("Test", ToastLevel::Info);
        assert!(toast.actions.is_empty());
    }

    #[test]
    fn action_is_correct_variant() {
        let toast = Toast::new("Test", ToastLevel::Info).with_action(ToastAction::Retry);
        assert!(matches!(toast.actions.first(), Some(ToastAction::Retry)));
    }

    #[test]
    fn multiple_actions_can_be_added() {
        let toast = Toast::new("Test", ToastLevel::Info)
            .with_action(ToastAction::Undo)
            .with_action(ToastAction::Retry);
        assert_eq!(toast.actions.len(), 2);
    }

    #[test]
    fn action_order_is_preserved() {
        let toast = Toast::new("Test", ToastLevel::Info)
            .with_action(ToastAction::Undo)
            .with_action(ToastAction::Retry);
        assert!(matches!(toast.actions[0], ToastAction::Undo));
        assert!(matches!(toast.actions[1], ToastAction::Retry));
    }
}

// =============================================================================
// TOAST - EXPIRY TESTS
// Toast uses should_remove() and age() methods
// =============================================================================

mod toast_expiry_tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn new_toast_should_not_be_removed() {
        let toast = Toast::new("Test", ToastLevel::Info);
        assert!(!toast.should_remove());
    }

    #[test]
    fn dismissed_toast_should_be_removed() {
        let mut toast = Toast::new("Test", ToastLevel::Info);
        toast.dismissed = true;
        assert!(toast.should_remove());
    }

    #[test]
    fn age_is_positive_for_new_toast() {
        let toast = Toast::new("Test", ToastLevel::Info);
        // Age is measured from creation, should be very small
        assert!(toast.age() >= 0.0);
        assert!(toast.age() < 1.0); // Less than 1 second
    }

    #[test]
    fn hovered_toast_should_not_timeout() {
        let mut toast = Toast::new("Test", ToastLevel::Info).with_duration(Duration::ZERO);
        toast.hovered = true;
        // Even with zero duration, hovered toast shouldn't be removed
        assert!(!toast.should_remove());
    }
}

// =============================================================================
// TOAST - ANIMATION STATE TESTS
// =============================================================================

mod toast_animation_tests {
    use super::*;

    #[test]
    fn new_toast_animation_progress_is_low() {
        let toast = Toast::new("Test", ToastLevel::Info);
        // Brand new toast should be at start of animation
        assert!(toast.animation_progress() < 0.5);
    }

    #[test]
    fn animation_progress_is_clamped_0_to_1() {
        let toast = Toast::new("Test", ToastLevel::Info);
        let progress = toast.animation_progress();
        assert!(progress >= 0.0);
        assert!(progress <= 1.0);
    }
}

// =============================================================================
// TOAST LEVEL - COMPARISON TESTS (CATCH COMPARISON OPERATORS)
// =============================================================================

mod toast_level_comparison_tests {
    use super::*;

    #[test]
    fn info_equals_info() {
        assert_eq!(ToastLevel::Info, ToastLevel::Info);
    }

    #[test]
    fn success_equals_success() {
        assert_eq!(ToastLevel::Success, ToastLevel::Success);
    }

    #[test]
    fn warning_equals_warning() {
        assert_eq!(ToastLevel::Warning, ToastLevel::Warning);
    }

    #[test]
    fn error_equals_error() {
        assert_eq!(ToastLevel::Error, ToastLevel::Error);
    }

    #[test]
    fn info_not_equals_success() {
        assert_ne!(ToastLevel::Info, ToastLevel::Success);
    }

    #[test]
    fn success_not_equals_warning() {
        assert_ne!(ToastLevel::Success, ToastLevel::Warning);
    }

    #[test]
    fn warning_not_equals_error() {
        assert_ne!(ToastLevel::Warning, ToastLevel::Error);
    }

    #[test]
    fn info_not_equals_error() {
        assert_ne!(ToastLevel::Info, ToastLevel::Error);
    }
}

// =============================================================================
// TOAST LEVEL - CLONE TESTS
// =============================================================================

mod toast_level_clone_tests {
    use super::*;

    #[test]
    fn clone_info_equals_original() {
        let original = ToastLevel::Info;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn clone_success_equals_original() {
        let original = ToastLevel::Success;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn clone_warning_equals_original() {
        let original = ToastLevel::Warning;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }

    #[test]
    fn clone_error_equals_original() {
        let original = ToastLevel::Error;
        let cloned = original.clone();
        assert_eq!(original, cloned);
    }
}
