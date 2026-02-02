//! Mutation-Resistant Tests: Runtime System
//!
//! Comprehensive tests for RuntimeState, RuntimeStats, RuntimeIssue
//! to achieve ≥92% mutation kill rate.

use aw_editor_lib::{RuntimeState, RuntimeStats};

// =============================================================================
// RUNTIME STATE - CAN_TRANSITION_TO() TESTS
// =============================================================================

mod runtime_state_can_transition_tests {
    use super::*;

    // Editing transitions
    #[test]
    fn editing_can_transition_to_playing() {
        assert!(RuntimeState::Editing.can_transition_to(RuntimeState::Playing));
    }

    #[test]
    fn editing_cannot_transition_to_paused() {
        assert!(!RuntimeState::Editing.can_transition_to(RuntimeState::Paused));
    }

    #[test]
    fn editing_cannot_transition_to_stepping() {
        assert!(!RuntimeState::Editing.can_transition_to(RuntimeState::SteppingOneFrame));
    }

    #[test]
    fn editing_cannot_transition_to_editing() {
        assert!(!RuntimeState::Editing.can_transition_to(RuntimeState::Editing));
    }

    // Playing transitions - can transition to Paused, Editing, AND SteppingOneFrame
    #[test]
    fn playing_can_transition_to_paused() {
        assert!(RuntimeState::Playing.can_transition_to(RuntimeState::Paused));
    }

    #[test]
    fn playing_can_transition_to_editing() {
        assert!(RuntimeState::Playing.can_transition_to(RuntimeState::Editing));
    }

    #[test]
    fn playing_can_transition_to_stepping() {
        assert!(RuntimeState::Playing.can_transition_to(RuntimeState::SteppingOneFrame));
    }

    #[test]
    fn playing_cannot_transition_to_playing() {
        assert!(!RuntimeState::Playing.can_transition_to(RuntimeState::Playing));
    }

    // Paused transitions
    #[test]
    fn paused_can_transition_to_playing() {
        assert!(RuntimeState::Paused.can_transition_to(RuntimeState::Playing));
    }

    #[test]
    fn paused_can_transition_to_editing() {
        assert!(RuntimeState::Paused.can_transition_to(RuntimeState::Editing));
    }

    #[test]
    fn paused_can_transition_to_stepping() {
        assert!(RuntimeState::Paused.can_transition_to(RuntimeState::SteppingOneFrame));
    }

    #[test]
    fn paused_cannot_transition_to_paused() {
        assert!(!RuntimeState::Paused.can_transition_to(RuntimeState::Paused));
    }

    // SteppingOneFrame transitions - can transition to ALL states including itself
    #[test]
    fn stepping_can_transition_to_paused() {
        assert!(RuntimeState::SteppingOneFrame.can_transition_to(RuntimeState::Paused));
    }

    #[test]
    fn stepping_can_transition_to_editing() {
        assert!(RuntimeState::SteppingOneFrame.can_transition_to(RuntimeState::Editing));
    }

    #[test]
    fn stepping_can_transition_to_playing() {
        assert!(RuntimeState::SteppingOneFrame.can_transition_to(RuntimeState::Playing));
    }

    #[test]
    fn stepping_can_transition_to_stepping() {
        // SteppingOneFrame can transition to itself (step again)
        assert!(RuntimeState::SteppingOneFrame.can_transition_to(RuntimeState::SteppingOneFrame));
    }
}

// =============================================================================
// RUNTIME STATE - HAS_SIMULATION() TESTS
// =============================================================================

mod runtime_state_has_simulation_tests {
    use super::*;

    #[test]
    fn editing_has_no_simulation() {
        assert!(!RuntimeState::Editing.has_simulation());
    }

    #[test]
    fn playing_has_simulation() {
        assert!(RuntimeState::Playing.has_simulation());
    }

    #[test]
    fn paused_has_simulation() {
        assert!(RuntimeState::Paused.has_simulation());
    }

    #[test]
    fn stepping_has_simulation() {
        assert!(RuntimeState::SteppingOneFrame.has_simulation());
    }
}

// =============================================================================
// RUNTIME STATE - IS_EDITABLE() TESTS
// =============================================================================

mod runtime_state_is_editable_tests {
    use super::*;

    #[test]
    fn editing_is_editable() {
        assert!(RuntimeState::Editing.is_editable());
    }

    #[test]
    fn playing_is_not_editable() {
        assert!(!RuntimeState::Playing.is_editable());
    }

    #[test]
    fn paused_is_not_editable() {
        assert!(!RuntimeState::Paused.is_editable());
    }

    #[test]
    fn stepping_is_not_editable() {
        assert!(!RuntimeState::SteppingOneFrame.is_editable());
    }
}

// =============================================================================
// RUNTIME STATE - IS_ACTIVE() TESTS
// =============================================================================

mod runtime_state_is_active_tests {
    use super::*;

    #[test]
    fn editing_is_not_active() {
        assert!(!RuntimeState::Editing.is_active());
    }

    #[test]
    fn playing_is_active() {
        assert!(RuntimeState::Playing.is_active());
    }

    #[test]
    fn paused_is_not_active() {
        assert!(!RuntimeState::Paused.is_active());
    }

    #[test]
    fn stepping_is_active() {
        assert!(RuntimeState::SteppingOneFrame.is_active());
    }
}

// =============================================================================
// RUNTIME STATE - VALID_TRANSITIONS() TESTS
// =============================================================================

mod runtime_state_valid_transitions_tests {
    use super::*;

    #[test]
    fn editing_has_one_valid_transition() {
        let transitions = RuntimeState::Editing.valid_transitions();
        assert_eq!(transitions.len(), 1);
        assert!(transitions.contains(&RuntimeState::Playing));
    }

    #[test]
    fn playing_has_three_valid_transitions() {
        let transitions = RuntimeState::Playing.valid_transitions();
        // Playing can go to: Paused, Editing, SteppingOneFrame
        assert_eq!(transitions.len(), 3);
        assert!(transitions.contains(&RuntimeState::Paused));
        assert!(transitions.contains(&RuntimeState::Editing));
        assert!(transitions.contains(&RuntimeState::SteppingOneFrame));
    }

    #[test]
    fn paused_has_three_valid_transitions() {
        let transitions = RuntimeState::Paused.valid_transitions();
        assert_eq!(transitions.len(), 3);
        assert!(transitions.contains(&RuntimeState::Playing));
        assert!(transitions.contains(&RuntimeState::Editing));
        assert!(transitions.contains(&RuntimeState::SteppingOneFrame));
    }

    #[test]
    fn stepping_has_four_valid_transitions() {
        let transitions = RuntimeState::SteppingOneFrame.valid_transitions();
        // SteppingOneFrame can go to: Paused, Editing, Playing, SteppingOneFrame (all)
        assert_eq!(transitions.len(), 4);
        assert!(transitions.contains(&RuntimeState::Paused));
        assert!(transitions.contains(&RuntimeState::Editing));
        assert!(transitions.contains(&RuntimeState::Playing));
        assert!(transitions.contains(&RuntimeState::SteppingOneFrame));
    }
}

// =============================================================================
// RUNTIME STATE - ICON() TESTS
// =============================================================================

mod runtime_state_icon_tests {
    use super::*;

    #[test]
    fn editing_icon_not_empty() {
        assert!(!RuntimeState::Editing.icon().is_empty());
    }

    #[test]
    fn playing_icon_not_empty() {
        assert!(!RuntimeState::Playing.icon().is_empty());
    }

    #[test]
    fn paused_icon_not_empty() {
        assert!(!RuntimeState::Paused.icon().is_empty());
    }

    #[test]
    fn stepping_icon_not_empty() {
        assert!(!RuntimeState::SteppingOneFrame.icon().is_empty());
    }

    #[test]
    fn different_states_have_different_icons() {
        assert_ne!(RuntimeState::Editing.icon(), RuntimeState::Playing.icon());
        assert_ne!(RuntimeState::Playing.icon(), RuntimeState::Paused.icon());
    }
}

// =============================================================================
// RUNTIME STATE - DESCRIPTION() TESTS
// =============================================================================

mod runtime_state_description_tests {
    use super::*;

    #[test]
    fn editing_description_not_empty() {
        assert!(!RuntimeState::Editing.description().is_empty());
    }

    #[test]
    fn playing_description_not_empty() {
        assert!(!RuntimeState::Playing.description().is_empty());
    }

    #[test]
    fn paused_description_not_empty() {
        assert!(!RuntimeState::Paused.description().is_empty());
    }

    #[test]
    fn stepping_description_not_empty() {
        assert!(!RuntimeState::SteppingOneFrame.description().is_empty());
    }
}

// =============================================================================
// RUNTIME STATE - ALL() TESTS
// =============================================================================

mod runtime_state_all_tests {
    use super::*;

    #[test]
    fn all_returns_4_states() {
        assert_eq!(RuntimeState::all().len(), 4);
    }

    #[test]
    fn all_contains_editing() {
        assert!(RuntimeState::all().contains(&RuntimeState::Editing));
    }

    #[test]
    fn all_contains_playing() {
        assert!(RuntimeState::all().contains(&RuntimeState::Playing));
    }

    #[test]
    fn all_contains_paused() {
        assert!(RuntimeState::all().contains(&RuntimeState::Paused));
    }

    #[test]
    fn all_contains_stepping() {
        assert!(RuntimeState::all().contains(&RuntimeState::SteppingOneFrame));
    }
}

// =============================================================================
// RUNTIME STATS - IS_FRAME_TIME_HEALTHY() TESTS (BOUNDARY CONDITIONS)
// =============================================================================

mod runtime_stats_is_frame_time_healthy_tests {
    use super::*;

    #[test]
    fn frame_time_below_threshold_is_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 15.0,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_frame_time_healthy(16.67));
    }

    #[test]
    fn frame_time_at_threshold_is_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 16.67,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_frame_time_healthy(16.67));
    }

    #[test]
    fn frame_time_above_threshold_is_not_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 16.68,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!(!stats.is_frame_time_healthy(16.67));
    }

    #[test]
    fn frame_time_zero_is_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 0.0,
            entity_count: 100,
            tick_count: 60,
            fps: 1000.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_frame_time_healthy(16.67));
    }

    #[test]
    fn frame_time_very_high_is_not_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 100.0,
            entity_count: 100,
            tick_count: 60,
            fps: 10.0,
            fixed_steps_last_tick: 1,
        };
        assert!(!stats.is_frame_time_healthy(16.67));
    }
}

// =============================================================================
// RUNTIME STATS - IS_FPS_HEALTHY() TESTS (BOUNDARY CONDITIONS)
// =============================================================================

mod runtime_stats_is_fps_healthy_tests {
    use super::*;

    #[test]
    fn fps_above_minimum_is_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 16.0,
            entity_count: 100,
            tick_count: 60,
            fps: 61.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_fps_healthy(60.0));
    }

    #[test]
    fn fps_at_minimum_is_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 16.67,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_fps_healthy(60.0));
    }

    #[test]
    fn fps_below_minimum_is_not_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 17.0,
            entity_count: 100,
            tick_count: 60,
            fps: 59.0,
            fixed_steps_last_tick: 1,
        };
        assert!(!stats.is_fps_healthy(60.0));
    }

    #[test]
    fn fps_zero_is_not_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: f32::INFINITY,
            entity_count: 100,
            tick_count: 60,
            fps: 0.0,
            fixed_steps_last_tick: 1,
        };
        assert!(!stats.is_fps_healthy(60.0));
    }

    #[test]
    fn fps_very_high_is_healthy() {
        let stats = RuntimeStats {
            frame_time_ms: 1.0,
            entity_count: 100,
            tick_count: 60,
            fps: 1000.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_fps_healthy(60.0));
    }
}

// =============================================================================
// RUNTIME STATS - PERFORMANCE_GRADE() TESTS (BOUNDARY CONDITIONS)
// Actual thresholds from runtime.rs:
//   0..=14 => "Critical"
//   15..=29 => "Poor"
//   30..=44 => "Fair"
//   45..=59 => "Good"
//   _ => "Excellent" (60+)
// =============================================================================

mod runtime_stats_performance_grade_tests {
    use super::*;

    #[test]
    fn fps_above_60_is_excellent() {
        let stats = RuntimeStats {
            frame_time_ms: 16.0,
            entity_count: 100,
            tick_count: 60,
            fps: 65.0,
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Excellent");
    }

    #[test]
    fn fps_at_60_is_excellent() {
        let stats = RuntimeStats {
            frame_time_ms: 16.67,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0, // 60+ is Excellent
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Excellent");
    }

    #[test]
    fn fps_at_45_is_good() {
        let stats = RuntimeStats {
            frame_time_ms: 22.22,
            entity_count: 100,
            tick_count: 60,
            fps: 45.0, // 45-59 is Good
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Good");
    }

    #[test]
    fn fps_at_30_is_fair() {
        let stats = RuntimeStats {
            frame_time_ms: 33.33,
            entity_count: 100,
            tick_count: 60,
            fps: 30.0, // 30-44 is Fair
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Fair");
    }

    #[test]
    fn fps_at_15_is_poor() {
        let stats = RuntimeStats {
            frame_time_ms: 66.67,
            entity_count: 100,
            tick_count: 60,
            fps: 15.0, // 15-29 is Poor
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Poor");
    }

    #[test]
    fn fps_below_15_is_critical() {
        let stats = RuntimeStats {
            frame_time_ms: 100.0,
            entity_count: 100,
            tick_count: 60,
            fps: 10.0, // 0-14 is Critical
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Critical");
    }

    // Boundary tests
    #[test]
    fn fps_at_59_is_good() {
        let stats = RuntimeStats {
            frame_time_ms: 16.95,
            entity_count: 100,
            tick_count: 60,
            fps: 59.0, // 45-59 = Good
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Good");
    }

    #[test]
    fn fps_at_44_is_fair() {
        let stats = RuntimeStats {
            frame_time_ms: 22.72,
            entity_count: 100,
            tick_count: 60,
            fps: 44.0, // 30-44 = Fair
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Fair");
    }

    #[test]
    fn fps_at_29_is_poor() {
        let stats = RuntimeStats {
            frame_time_ms: 34.48,
            entity_count: 100,
            tick_count: 60,
            fps: 29.0, // 15-29 = Poor
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Poor");
    }

    #[test]
    fn fps_at_14_is_critical() {
        let stats = RuntimeStats {
            frame_time_ms: 71.43,
            entity_count: 100,
            tick_count: 60,
            fps: 14.0, // 0-14 = Critical
            fixed_steps_last_tick: 1,
        };
        assert_eq!(stats.performance_grade(), "Critical");
    }
}

// =============================================================================
// RUNTIME STATS - FRAME_BUDGET_PERCENTAGE() TESTS
// =============================================================================

mod runtime_stats_frame_budget_percentage_tests {
    use super::*;

    #[test]
    fn frame_time_0_is_0_percent() {
        let stats = RuntimeStats {
            frame_time_ms: 0.0,
            entity_count: 100,
            tick_count: 60,
            fps: 1000.0,
            fixed_steps_last_tick: 1,
        };
        assert!((stats.frame_budget_percentage() - 0.0).abs() < 0.01);
    }

    #[test]
    fn frame_time_16_67_is_100_percent() {
        let stats = RuntimeStats {
            frame_time_ms: 16.67,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!((stats.frame_budget_percentage() - 100.0).abs() < 1.0);
    }

    #[test]
    fn frame_time_8_33_is_50_percent() {
        let stats = RuntimeStats {
            frame_time_ms: 8.33,
            entity_count: 100,
            tick_count: 60,
            fps: 120.0,
            fixed_steps_last_tick: 1,
        };
        assert!((stats.frame_budget_percentage() - 50.0).abs() < 1.0);
    }

    #[test]
    fn frame_time_33_34_is_200_percent() {
        let stats = RuntimeStats {
            frame_time_ms: 33.34,
            entity_count: 100,
            tick_count: 60,
            fps: 30.0,
            fixed_steps_last_tick: 1,
        };
        assert!((stats.frame_budget_percentage() - 200.0).abs() < 1.0);
    }
}

// =============================================================================
// RUNTIME STATS - IS_RUNNING_SMOOTHLY() TESTS
// =============================================================================

mod runtime_stats_is_running_smoothly_tests {
    use super::*;

    #[test]
    fn smooth_when_fps_60_and_frame_time_16() {
        let stats = RuntimeStats {
            frame_time_ms: 16.0,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_running_smoothly());
    }

    #[test]
    fn not_smooth_when_fps_below_60() {
        let stats = RuntimeStats {
            frame_time_ms: 16.0,
            entity_count: 100,
            tick_count: 60,
            fps: 59.0,
            fixed_steps_last_tick: 1,
        };
        assert!(!stats.is_running_smoothly());
    }

    #[test]
    fn not_smooth_when_frame_time_above_16_67() {
        let stats = RuntimeStats {
            frame_time_ms: 17.0,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!(!stats.is_running_smoothly());
    }

    #[test]
    fn smooth_when_fps_high_and_frame_time_low() {
        let stats = RuntimeStats {
            frame_time_ms: 8.0,
            entity_count: 100,
            tick_count: 60,
            fps: 120.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_running_smoothly());
    }
}

// =============================================================================
// RUNTIME STATS - IS_FRAME_TIME_CRITICAL() TESTS
// =============================================================================

mod runtime_stats_is_frame_time_critical_tests {
    use super::*;

    #[test]
    fn frame_time_below_33_33_is_not_critical() {
        let stats = RuntimeStats {
            frame_time_ms: 33.0,
            entity_count: 100,
            tick_count: 60,
            fps: 30.0,
            fixed_steps_last_tick: 1,
        };
        assert!(!stats.is_frame_time_critical());
    }

    #[test]
    fn frame_time_above_33_33_is_critical() {
        let stats = RuntimeStats {
            frame_time_ms: 34.0,
            entity_count: 100,
            tick_count: 60,
            fps: 29.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_frame_time_critical());
    }

    #[test]
    fn frame_time_at_boundary_33_33() {
        let stats = RuntimeStats {
            frame_time_ms: 33.33,
            entity_count: 100,
            tick_count: 60,
            fps: 30.0,
            fixed_steps_last_tick: 1,
        };
        // Just at threshold - not critical (<=)
        assert!(!stats.is_frame_time_critical());
    }

    #[test]
    fn frame_time_100_is_critical() {
        let stats = RuntimeStats {
            frame_time_ms: 100.0,
            entity_count: 100,
            tick_count: 60,
            fps: 10.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.is_frame_time_critical());
    }
}

// =============================================================================
// RUNTIME STATS - VALIDATE() TESTS
// =============================================================================

mod runtime_stats_validate_tests {
    use super::*;

    #[test]
    fn valid_stats_return_empty_issues() {
        let stats = RuntimeStats {
            frame_time_ms: 16.0,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0,
            fixed_steps_last_tick: 1,
        };
        assert!(stats.validate().is_empty());
    }

    #[test]
    fn low_fps_returns_issue() {
        let stats = RuntimeStats {
            frame_time_ms: 50.0,
            entity_count: 100,
            tick_count: 60,
            fps: 20.0,
            fixed_steps_last_tick: 1,
        };
        let issues = stats.validate();
        assert!(!issues.is_empty());
    }

    #[test]
    fn high_frame_time_returns_issue() {
        let stats = RuntimeStats {
            frame_time_ms: 50.0,
            entity_count: 100,
            tick_count: 60,
            fps: 60.0, // fps still ok but frame time is bad
            fixed_steps_last_tick: 1,
        };
        let issues = stats.validate();
        assert!(!issues.is_empty());
    }
}

// =============================================================================
// RUNTIME STATS - DEFAULT TESTS
// =============================================================================

mod runtime_stats_default_tests {
    use super::*;

    #[test]
    fn default_frame_time_is_zero() {
        let stats = RuntimeStats::default();
        assert_eq!(stats.frame_time_ms, 0.0);
    }

    #[test]
    fn default_entity_count_is_zero() {
        let stats = RuntimeStats::default();
        assert_eq!(stats.entity_count, 0);
    }

    #[test]
    fn default_tick_count_is_zero() {
        let stats = RuntimeStats::default();
        assert_eq!(stats.tick_count, 0);
    }

    #[test]
    fn default_fps_is_zero() {
        let stats = RuntimeStats::default();
        assert_eq!(stats.fps, 0.0);
    }

    #[test]
    fn default_fixed_steps_is_zero() {
        let stats = RuntimeStats::default();
        assert_eq!(stats.fixed_steps_last_tick, 0);
    }
}
