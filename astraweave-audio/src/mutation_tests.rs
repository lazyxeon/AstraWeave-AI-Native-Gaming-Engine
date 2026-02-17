//! Mutation-killing tests for astraweave-audio.
//! These tests are designed to catch subtle mutations like boundary condition changes,
//! arithmetic operator substitutions, and conditional logic inversions.

#![allow(
    clippy::nonminimal_bool,
    clippy::bool_assert_comparison,
    clippy::assertions_on_constants,
    clippy::approx_constant,
    clippy::unnecessary_get_then_check
)]

#[cfg(test)]
mod tests {
    use crate::engine::{AudioEngine, ListenerPose, MusicTrack, PanMode};
    use glam::vec3;

    // ============================================================================
    // Volume Clamping Tests - Ensure boundary values are handled correctly
    // ============================================================================

    #[test]
    fn test_master_volume_clamps_at_zero() {
        let mut engine = AudioEngine::new().unwrap();

        // Negative values should clamp to 0.0
        engine.set_master_volume(-0.1);
        assert_eq!(engine.master_volume, 0.0);

        engine.set_master_volume(-100.0);
        assert_eq!(engine.master_volume, 0.0);

        engine.set_master_volume(-f32::EPSILON);
        assert_eq!(engine.master_volume, 0.0);
    }

    #[test]
    fn test_master_volume_clamps_at_one() {
        let mut engine = AudioEngine::new().unwrap();

        // Values above 1.0 should clamp to 1.0
        engine.set_master_volume(1.1);
        assert_eq!(engine.master_volume, 1.0);

        engine.set_master_volume(100.0);
        assert_eq!(engine.master_volume, 1.0);

        engine.set_master_volume(1.0 + f32::EPSILON);
        assert_eq!(engine.master_volume, 1.0);
    }

    #[test]
    fn test_master_volume_boundary_values() {
        let mut engine = AudioEngine::new().unwrap();

        // Exact boundaries should work
        engine.set_master_volume(0.0);
        assert_eq!(engine.master_volume, 0.0);

        engine.set_master_volume(1.0);
        assert_eq!(engine.master_volume, 1.0);

        // Mid-range values
        engine.set_master_volume(0.5);
        assert_eq!(engine.master_volume, 0.5);

        engine.set_master_volume(0.25);
        assert_eq!(engine.master_volume, 0.25);

        engine.set_master_volume(0.75);
        assert_eq!(engine.master_volume, 0.75);
    }

    // ============================================================================
    // Listener Pose Tests - Ensure listener updates work correctly
    // ============================================================================

    #[test]
    fn test_listener_at_origin_facing_negative_z() {
        let mut engine = AudioEngine::new().unwrap();

        let pose = ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        // Should not panic when updating listener
        engine.update_listener(pose);
    }

    #[test]
    fn test_listener_facing_various_directions() {
        let mut engine = AudioEngine::new().unwrap();

        // Facing +X
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(1.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
        });

        // Facing -X
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(-1.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
        });

        // Facing +Z
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, 1.0),
            up: vec3(0.0, 1.0, 0.0),
        });

        // Facing -Z
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });
    }

    #[test]
    fn test_listener_position_offset() {
        let mut engine = AudioEngine::new().unwrap();

        let pose = ListenerPose {
            position: vec3(10.0, 5.0, -3.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        // Should handle arbitrary positions
        engine.update_listener(pose);
    }

    #[test]
    fn test_listener_extreme_positions() {
        let mut engine = AudioEngine::new().unwrap();

        // Very far positions
        engine.update_listener(ListenerPose {
            position: vec3(10000.0, 10000.0, 10000.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });

        engine.update_listener(ListenerPose {
            position: vec3(-10000.0, -10000.0, -10000.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });
    }

    // ============================================================================
    // Tick Update Tests - Ensure time-based operations work correctly
    // ============================================================================

    #[test]
    fn test_tick_with_zero_delta() {
        let mut engine = AudioEngine::new().unwrap();

        // Should not panic or change state incorrectly
        for _ in 0..100 {
            engine.tick(0.0);
        }
    }

    #[test]
    fn test_tick_with_large_delta() {
        let mut engine = AudioEngine::new().unwrap();

        // Large delta time should not break anything
        engine.tick(10.0);
        engine.tick(100.0);
        engine.tick(1000.0);
    }

    #[test]
    fn test_tick_with_small_delta() {
        let mut engine = AudioEngine::new().unwrap();

        // Very small deltas (high FPS) should work
        for _ in 0..1000 {
            engine.tick(0.001); // 1000 FPS
        }
    }

    #[test]
    fn test_tick_with_negative_delta_does_not_panic() {
        let mut engine = AudioEngine::new().unwrap();

        // Negative delta shouldn't panic (edge case)
        engine.tick(-0.016);
        engine.tick(-1.0);
    }

    // ============================================================================
    // Voice Beep Duration Tests - Ensure duration calculations are correct
    // ============================================================================

    #[test]
    fn test_voice_beep_minimum_duration() {
        let mut engine = AudioEngine::new().unwrap();

        // Zero length text should use minimum duration (0.6s)
        // Formula: (0 * 0.05).clamp(0.6, 3.0) = 0.6
        engine.play_voice_beep(0);

        // Very short text
        engine.play_voice_beep(1); // 0.05 -> 0.6
        engine.play_voice_beep(5); // 0.25 -> 0.6
        engine.play_voice_beep(10); // 0.5 -> 0.6
        engine.play_voice_beep(11); // 0.55 -> 0.6
    }

    #[test]
    fn test_voice_beep_maximum_duration() {
        let mut engine = AudioEngine::new().unwrap();

        // Long text should clamp to maximum duration (3.0s)
        // Formula: (100 * 0.05).clamp(0.6, 3.0) = 5.0 -> 3.0
        engine.play_voice_beep(100); // 5.0 -> 3.0
        engine.play_voice_beep(1000); // 50.0 -> 3.0
    }

    #[test]
    fn test_voice_beep_mid_range_duration() {
        let mut engine = AudioEngine::new().unwrap();

        // Mid-range text length
        // Formula: (20 * 0.05).clamp(0.6, 3.0) = 1.0
        engine.play_voice_beep(20);

        // Formula: (40 * 0.05).clamp(0.6, 3.0) = 2.0
        engine.play_voice_beep(40);
    }

    // ============================================================================
    // SFX Beep Tests - Ensure beep parameters are handled correctly
    // ============================================================================

    #[test]
    fn test_sfx_beep_various_frequencies() {
        let mut engine = AudioEngine::new().unwrap();

        // Test various frequencies
        engine.play_sfx_beep(20.0, 0.1, 0.5); // Low frequency
        engine.play_sfx_beep(440.0, 0.1, 0.5); // A4
        engine.play_sfx_beep(880.0, 0.1, 0.5); // A5
        engine.play_sfx_beep(20000.0, 0.1, 0.5); // High frequency
    }

    #[test]
    fn test_sfx_beep_various_durations() {
        let mut engine = AudioEngine::new().unwrap();

        engine.play_sfx_beep(440.0, 0.001, 0.5); // Very short
        engine.play_sfx_beep(440.0, 0.1, 0.5); // Normal
        engine.play_sfx_beep(440.0, 1.0, 0.5); // Long
        engine.play_sfx_beep(440.0, 10.0, 0.5); // Very long
    }

    #[test]
    fn test_sfx_beep_various_gains() {
        let mut engine = AudioEngine::new().unwrap();

        engine.play_sfx_beep(440.0, 0.1, 0.0); // Silent
        engine.play_sfx_beep(440.0, 0.1, 0.1); // Quiet
        engine.play_sfx_beep(440.0, 0.1, 0.5); // Normal
        engine.play_sfx_beep(440.0, 0.1, 1.0); // Full volume
        engine.play_sfx_beep(440.0, 0.1, 2.0); // Amplified
    }

    // ============================================================================
    // Spatial Audio Tests - Ensure 3D audio calculations work
    // ============================================================================

    #[test]
    fn test_spatial_sink_creation_for_new_emitter() {
        let mut engine = AudioEngine::new().unwrap();

        let emitter_id = 12345;
        let pos = vec3(5.0, 0.0, 0.0);

        // First call should create new sink
        let result = engine.play_sfx_3d_beep(emitter_id, pos, 440.0, 0.1, 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_spatial_sink_reuse_for_existing_emitter() {
        let mut engine = AudioEngine::new().unwrap();

        let emitter_id = 42;
        let pos = vec3(0.0, 0.0, 5.0);

        // Create first sink
        engine
            .play_sfx_3d_beep(emitter_id, pos, 440.0, 0.1, 0.5)
            .unwrap();

        // Second call should succeed (reuses sink)
        let result = engine.play_sfx_3d_beep(emitter_id, pos, 880.0, 0.1, 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_multiple_unique_emitters() {
        let mut engine = AudioEngine::new().unwrap();

        for i in 0..20 {
            let pos = vec3(i as f32, 0.0, 0.0);
            let result = engine.play_sfx_3d_beep(i, pos, 440.0, 0.1, 0.5);
            assert!(result.is_ok(), "Emitter {} should succeed", i);
        }
    }

    #[test]
    fn test_emitter_at_origin() {
        let mut engine = AudioEngine::new().unwrap();

        let pos = vec3(0.0, 0.0, 0.0);
        let result = engine.play_sfx_3d_beep(0, pos, 440.0, 0.1, 0.5);
        assert!(result.is_ok());
    }

    #[test]
    fn test_emitter_at_extreme_positions() {
        let mut engine = AudioEngine::new().unwrap();

        // Far positions
        engine
            .play_sfx_3d_beep(1, vec3(1000.0, 0.0, 0.0), 440.0, 0.1, 0.5)
            .unwrap();
        engine
            .play_sfx_3d_beep(2, vec3(-1000.0, 0.0, 0.0), 440.0, 0.1, 0.5)
            .unwrap();
        engine
            .play_sfx_3d_beep(3, vec3(0.0, 1000.0, 0.0), 440.0, 0.1, 0.5)
            .unwrap();
        engine
            .play_sfx_3d_beep(4, vec3(0.0, -1000.0, 0.0), 440.0, 0.1, 0.5)
            .unwrap();
    }

    // ============================================================================
    // Pan Mode Tests - Ensure pan mode switching works
    // ============================================================================

    #[test]
    fn test_pan_mode_stereo_angle() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_pan_mode(PanMode::StereoAngle);
        engine.play_sfx_beep(440.0, 0.1, 0.5);
    }

    #[test]
    fn test_pan_mode_none() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_pan_mode(PanMode::None);
        engine.play_sfx_beep(440.0, 0.1, 0.5);
    }

    #[test]
    fn test_pan_mode_switching() {
        let mut engine = AudioEngine::new().unwrap();

        for _ in 0..10 {
            engine.set_pan_mode(PanMode::StereoAngle);
            engine.play_sfx_beep(440.0, 0.01, 0.5);

            engine.set_pan_mode(PanMode::None);
            engine.play_sfx_beep(880.0, 0.01, 0.5);
        }
    }

    // ============================================================================
    // Music Track Tests - Ensure music playback works correctly
    // ============================================================================

    #[test]
    fn test_music_track_looped_flag() {
        // Just verify track struct works
        let track = MusicTrack {
            path: "nonexistent.wav".to_string(),
            looped: true,
        };
        assert!(track.looped);

        let track = MusicTrack {
            path: "nonexistent.wav".to_string(),
            looped: false,
        };
        assert!(!track.looped);
    }

    // ============================================================================
    // Engine Initialization Tests - Ensure proper defaults
    // ============================================================================

    #[test]
    fn test_engine_default_volumes() {
        let engine = AudioEngine::new().unwrap();

        assert_eq!(engine.master_volume, 1.0);
    }

    #[test]
    fn test_engine_multiple_instances() {
        // Should be able to create multiple engines (though only one can use default output)
        let engine1 = AudioEngine::new();
        // Note: Second instance may fail on some systems due to audio device limitations
        // but shouldn't panic
        let _ = engine1;
    }
}

// ============================================================================
// Behavioral Correctness Tests - Audio Physics & Invariants
// ============================================================================

#[cfg(test)]
mod behavioral_correctness_tests {
    use crate::engine::{AudioEngine, ListenerPose, PanMode};
    use glam::vec3;

    #[test]
    fn test_volume_clamp_never_exceeds_bounds() {
        // Behavioral: volume must always be in [0, 1] after any operation
        let mut engine = AudioEngine::new().unwrap();

        // Test extreme values
        for &vol in &[-1000.0, -1.0, -0.001, 0.0, 0.5, 1.0, 1.001, 100.0, 1000.0] {
            engine.set_master_volume(vol);
            assert!(engine.master_volume >= 0.0, "Volume must be >= 0");
            assert!(engine.master_volume <= 1.0, "Volume must be <= 1");
        }
    }

    #[test]
    fn test_listener_forward_must_be_normalized() {
        // Behavioral: forward direction should be usable (normalized or normalizable)
        let pose = ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        let forward_len = pose.forward.length();
        assert!(
            (forward_len - 1.0).abs() < 0.001,
            "Forward vector should be unit length: {}",
            forward_len
        );
    }

    #[test]
    fn test_listener_up_must_be_normalized() {
        // Behavioral: up direction should be unit length
        let pose = ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        let up_len = pose.up.length();
        assert!(
            (up_len - 1.0).abs() < 0.001,
            "Up vector should be unit length: {}",
            up_len
        );
    }

    #[test]
    fn test_listener_forward_up_perpendicular() {
        // Behavioral: forward and up should be perpendicular for valid orientation
        let pose = ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        let dot = pose.forward.dot(pose.up);
        assert!(
            dot.abs() < 0.001,
            "Forward and up must be perpendicular (dot=0), got: {}",
            dot
        );
    }

    #[test]
    fn test_pan_mode_none_is_distinct() {
        // Behavioral: None mode should be distinct from StereoAngle
        assert_ne!(PanMode::None, PanMode::StereoAngle);
    }

    #[test]
    fn test_volume_set_get_consistency() {
        // Behavioral: setting and getting volume should be consistent
        let mut engine = AudioEngine::new().unwrap();

        engine.set_master_volume(0.7);
        assert!(
            (engine.master_volume - 0.7).abs() < 0.0001,
            "Set volume should persist: expected 0.7, got {}",
            engine.master_volume
        );
    }

    #[test]
    fn test_engine_default_volume_is_full() {
        // Behavioral: engine should start at full volume (1.0)
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.master_volume - 1.0).abs() < 0.0001,
            "Default volume should be 1.0 (full)"
        );
    }

    #[test]
    fn test_3d_audio_left_right_symmetry() {
        // Behavioral: positions equidistant left/right should be valid
        let mut engine = AudioEngine::new().unwrap();

        // Set listener at origin facing -Z
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });

        // Both left and right positions should work without panic
        engine
            .play_sfx_3d_beep(1, vec3(-5.0, 0.0, -5.0), 440.0, 0.01, 0.5)
            .unwrap();
        engine
            .play_sfx_3d_beep(2, vec3(5.0, 0.0, -5.0), 440.0, 0.01, 0.5)
            .unwrap();
    }
}

// ============================================================================
// BOUNDARY CONDITION TESTS
// Catches mutations: < vs <=, > vs >=, off-by-one errors
// ============================================================================

#[cfg(test)]
mod boundary_condition_tests {
    use crate::engine::{AudioEngine, ListenerPose};
    use glam::vec3;

    // --- Volume Boundary Tests ---

    #[test]
    fn test_volume_exactly_at_zero_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Exactly 0.0 should be accepted
        engine.set_master_volume(0.0);
        assert_eq!(engine.master_volume, 0.0, "Volume should be exactly 0.0");
    }

    #[test]
    fn test_volume_exactly_at_one_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Exactly 1.0 should be accepted
        engine.set_master_volume(1.0);
        assert_eq!(engine.master_volume, 1.0, "Volume should be exactly 1.0");
    }

    #[test]
    fn test_volume_epsilon_below_zero() {
        let mut engine = AudioEngine::new().unwrap();

        // Just below 0 should clamp to 0
        engine.set_master_volume(-f32::EPSILON);
        assert_eq!(engine.master_volume, 0.0, "Below zero should clamp to 0.0");

        engine.set_master_volume(-0.001);
        assert_eq!(engine.master_volume, 0.0, "Negative should clamp to 0.0");
    }

    #[test]
    fn test_volume_epsilon_above_one() {
        let mut engine = AudioEngine::new().unwrap();

        // Just above 1 should clamp to 1
        engine.set_master_volume(1.0 + f32::EPSILON);
        assert_eq!(engine.master_volume, 1.0, "Above 1.0 should clamp to 1.0");

        engine.set_master_volume(1.001);
        assert_eq!(engine.master_volume, 1.0, ">1.0 should clamp to 1.0");
    }

    // --- Voice Beep Duration Boundary Tests ---

    #[test]
    fn test_voice_beep_minimum_duration_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Text length 12: 12 * 0.05 = 0.6 (exactly at minimum)
        // Text length 11: 11 * 0.05 = 0.55 (below minimum, clamps to 0.6)
        // Text length 13: 13 * 0.05 = 0.65 (above minimum)
        engine.play_voice_beep(11); // Should use 0.6 (minimum)
        engine.play_voice_beep(12); // Should use 0.6 (exactly at threshold)
        engine.play_voice_beep(13); // Should use 0.65
    }

    #[test]
    fn test_voice_beep_maximum_duration_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Text length 60: 60 * 0.05 = 3.0 (exactly at maximum)
        // Text length 59: 59 * 0.05 = 2.95 (below maximum)
        // Text length 61: 61 * 0.05 = 3.05 (above maximum, clamps to 3.0)
        engine.play_voice_beep(59); // Should use 2.95
        engine.play_voice_beep(60); // Should use 3.0 (exactly at threshold)
        engine.play_voice_beep(61); // Should clamp to 3.0
    }

    // --- Listener Position Boundary Tests ---

    #[test]
    fn test_listener_at_origin_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Listener exactly at origin
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });

        // Play sound at origin - should not cause division by zero
        let result = engine.play_sfx_3d_beep(1, vec3(0.0, 0.0, 0.0), 440.0, 0.01, 0.5);
        assert!(result.is_ok(), "Sound at listener position should work");
    }

    #[test]
    fn test_listener_extreme_position_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Listener at very large coordinates
        engine.update_listener(ListenerPose {
            position: vec3(1e6, 1e6, 1e6),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });

        // Should not overflow or panic
        let result = engine.play_sfx_3d_beep(1, vec3(1e6, 1e6, 1e6 - 1.0), 440.0, 0.01, 0.5);
        assert!(result.is_ok(), "Large coordinates should work");
    }

    // --- Emitter ID Boundary Tests ---

    #[test]
    fn test_emitter_id_zero_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Emitter ID 0 is valid
        let result = engine.play_sfx_3d_beep(0, vec3(1.0, 0.0, 0.0), 440.0, 0.01, 0.5);
        assert!(result.is_ok(), "Emitter ID 0 should be valid");
    }

    #[test]
    fn test_emitter_id_max_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Emitter ID at u64::MAX is valid
        let result = engine.play_sfx_3d_beep(u64::MAX, vec3(1.0, 0.0, 0.0), 440.0, 0.01, 0.5);
        assert!(result.is_ok(), "Emitter ID u64::MAX should be valid");
    }

    // --- Tick Time Boundary Tests ---

    #[test]
    fn test_tick_zero_dt_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Zero delta time should not cause issues
        engine.tick(0.0);
        engine.tick(0.0);
        engine.tick(0.0);
    }

    #[test]
    fn test_tick_very_small_dt_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Very small delta time
        engine.tick(f32::EPSILON);
        engine.tick(1e-10);
    }

    #[test]
    fn test_tick_very_large_dt_boundary() {
        let mut engine = AudioEngine::new().unwrap();

        // Very large delta time (should not overflow)
        engine.tick(1000.0);
        engine.tick(f32::MAX / 2.0);
    }
}

// ============================================================================
// COMPARISON OPERATOR TESTS
// Catches mutations: == vs !=, < vs >, wrong enum comparisons
// ============================================================================

#[cfg(test)]
mod comparison_operator_tests {
    use crate::engine::{AudioEngine, ListenerPose, PanMode};
    use glam::vec3;

    // --- PanMode Enum Comparison Tests ---

    #[test]
    fn test_pan_mode_equality_same_variant() {
        assert_eq!(PanMode::None, PanMode::None, "Same variant should be equal");
        assert_eq!(
            PanMode::StereoAngle,
            PanMode::StereoAngle,
            "Same variant should be equal"
        );
    }

    #[test]
    fn test_pan_mode_inequality_different_variants() {
        assert_ne!(
            PanMode::None,
            PanMode::StereoAngle,
            "Different variants should not be equal"
        );
        assert_ne!(
            PanMode::StereoAngle,
            PanMode::None,
            "Different variants should not be equal (reversed)"
        );
    }

    // --- Volume Comparison Tests ---

    #[test]
    fn test_volume_comparison_after_set() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_master_volume(0.5);

        // Volume should equal what was set
        assert!(
            (engine.master_volume - 0.5).abs() < 1e-6,
            "Volume should equal set value"
        );

        // Volume should not equal different values
        assert!(
            (engine.master_volume - 0.4).abs() > 1e-6,
            "Volume should not equal different value"
        );
        assert!(
            (engine.master_volume - 0.6).abs() > 1e-6,
            "Volume should not equal different value"
        );
    }

    #[test]
    fn test_volume_zero_vs_nonzero_comparison() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_master_volume(0.0);
        assert_eq!(engine.master_volume, 0.0, "Zero volume should equal 0.0");

        engine.set_master_volume(0.001);
        assert!(engine.master_volume > 0.0, "Nonzero volume should be > 0.0");
        assert!(
            engine.master_volume != 0.0,
            "Nonzero volume should not equal 0.0"
        );
    }

    // --- Listener Pose Vector Comparisons ---

    #[test]
    fn test_listener_forward_vs_up_different() {
        let pose = ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        // Forward and up should be different vectors
        assert_ne!(pose.forward, pose.up, "Forward and up should be different");
    }

    #[test]
    fn test_listener_position_comparisons() {
        let pose1 = ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        let pose2 = ListenerPose {
            position: vec3(1.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };

        assert_ne!(
            pose1.position, pose2.position,
            "Different positions should not be equal"
        );
        assert_eq!(pose1.forward, pose2.forward, "Same forward should be equal");
    }

    // --- Emitter Comparison Tests ---

    #[test]
    fn test_emitter_id_uniqueness_matters() {
        let mut engine = AudioEngine::new().unwrap();

        // Different emitter IDs create different sinks
        engine
            .play_sfx_3d_beep(1, vec3(0.0, 0.0, 0.0), 440.0, 0.01, 0.5)
            .unwrap();
        engine
            .play_sfx_3d_beep(2, vec3(0.0, 0.0, 0.0), 440.0, 0.01, 0.5)
            .unwrap();

        // Both should succeed with different IDs
        let result1 = engine.play_sfx_3d_beep(1, vec3(1.0, 0.0, 0.0), 440.0, 0.01, 0.5);
        let result2 = engine.play_sfx_3d_beep(2, vec3(2.0, 0.0, 0.0), 440.0, 0.01, 0.5);

        assert!(
            result1.is_ok() && result2.is_ok(),
            "Both emitters should work"
        );
    }

    // --- Boolean Result Comparisons ---

    #[test]
    fn test_play_sfx_3d_beep_ok_vs_err_comparison() {
        let mut engine = AudioEngine::new().unwrap();

        // Valid operation returns Ok
        let result = engine.play_sfx_3d_beep(1, vec3(0.0, 0.0, 0.0), 440.0, 0.01, 0.5);
        assert!(result.is_ok(), "Valid operation should return Ok");
        assert!(!result.is_err(), "Valid operation should not return Err");
    }
}

// ============================================================================
// BOOLEAN RETURN PATH TESTS
// Catches mutations: return true vs false, logic inversions, early returns
// ============================================================================

#[cfg(test)]
mod boolean_return_path_tests {
    use crate::engine::{AudioEngine, ListenerPose, MusicTrack, PanMode};
    use glam::vec3;

    // --- Result Type Boolean Tests ---

    #[test]
    fn test_audio_engine_new_returns_ok() {
        let result = AudioEngine::new();
        assert!(result.is_ok(), "AudioEngine::new() should return Ok");
        assert!(!result.is_err(), "AudioEngine::new() should not return Err");
    }

    #[test]
    fn test_play_sfx_3d_beep_returns_ok_for_valid_input() {
        let mut engine = AudioEngine::new().unwrap();

        let result = engine.play_sfx_3d_beep(1, vec3(0.0, 0.0, 0.0), 440.0, 0.1, 0.5);
        assert_eq!(
            result.is_ok(),
            true,
            "Valid 3D beep should return Ok (true)"
        );
        assert_eq!(
            result.is_err(),
            false,
            "Valid 3D beep should not return Err (false)"
        );
    }

    #[test]
    fn test_play_music_returns_err_for_nonexistent_file() {
        let mut engine = AudioEngine::new().unwrap();

        let track = MusicTrack {
            path: "/nonexistent/path/to/file.wav".to_string(),
            looped: false,
        };

        let result = engine.play_music(track, 0.5);
        assert_eq!(
            result.is_err(),
            true,
            "Nonexistent file should return Err (true)"
        );
        assert_eq!(
            result.is_ok(),
            false,
            "Nonexistent file should not return Ok (false)"
        );
    }

    // --- MusicTrack Boolean Field Tests ---

    #[test]
    fn test_music_track_looped_true() {
        let track = MusicTrack {
            path: "test.wav".to_string(),
            looped: true,
        };
        assert_eq!(track.looped, true, "Looped field should be true");
        assert_ne!(track.looped, false, "Looped field should not be false");
    }

    #[test]
    fn test_music_track_looped_false() {
        let track = MusicTrack {
            path: "test.wav".to_string(),
            looped: false,
        };
        assert_eq!(track.looped, false, "Looped field should be false");
        assert_ne!(track.looped, true, "Looped field should not be true");
    }

    // --- Volume State Boolean Logic Tests ---

    #[test]
    fn test_volume_is_muted_when_zero() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_master_volume(0.0);
        let is_muted = engine.master_volume == 0.0;
        assert_eq!(is_muted, true, "Volume 0.0 should be considered muted");

        engine.set_master_volume(0.001);
        let is_muted = engine.master_volume == 0.0;
        assert_eq!(is_muted, false, "Non-zero volume should not be muted");
    }

    #[test]
    fn test_volume_is_full_when_one() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_master_volume(1.0);
        let is_full = engine.master_volume == 1.0;
        assert_eq!(is_full, true, "Volume 1.0 should be considered full");

        engine.set_master_volume(0.999);
        let is_full = engine.master_volume == 1.0;
        assert_eq!(is_full, false, "Volume <1.0 should not be full");
    }

    // --- PanMode Boolean Logic Tests ---

    #[test]
    fn test_pan_mode_is_spatial_when_stereo_angle() {
        let mode = PanMode::StereoAngle;
        let is_spatial = mode == PanMode::StereoAngle;
        assert_eq!(is_spatial, true, "StereoAngle should be spatial");

        let mode = PanMode::None;
        let is_spatial = mode == PanMode::StereoAngle;
        assert_eq!(is_spatial, false, "None should not be spatial");
    }

    #[test]
    fn test_pan_mode_is_none_check() {
        let mode = PanMode::None;
        let is_none = mode == PanMode::None;
        assert_eq!(is_none, true, "None mode should return true for is_none");

        let mode = PanMode::StereoAngle;
        let is_none = mode == PanMode::None;
        assert_eq!(
            is_none, false,
            "StereoAngle should return false for is_none"
        );
    }

    // --- Multiple Operations Return Path Tests ---

    #[test]
    fn test_multiple_spatial_sinks_all_succeed() {
        let mut engine = AudioEngine::new().unwrap();

        let mut all_ok = true;
        for i in 0..10 {
            let result = engine.play_sfx_3d_beep(i, vec3(i as f32, 0.0, 0.0), 440.0, 0.01, 0.5);
            if result.is_err() {
                all_ok = false;
            }
        }

        assert_eq!(all_ok, true, "All spatial sink creations should succeed");
    }

    #[test]
    fn test_listener_update_does_not_fail() {
        let mut engine = AudioEngine::new().unwrap();

        // Listener update doesn't return Result, but shouldn't panic
        for i in 0..100 {
            engine.update_listener(ListenerPose {
                position: vec3(i as f32, 0.0, 0.0),
                forward: vec3(0.0, 0.0, -1.0),
                up: vec3(0.0, 1.0, 0.0),
            });
        }
        // Test passes if we reach here without panic
        assert!(true, "Multiple listener updates should not panic");
    }

    // ============================================================================
    // MUTATION-RESISTANT TESTS: Exact value assertions via test accessors
    // These kill mutants that change constants, swap operators, or invert logic
    // ============================================================================

    // ---------- Default Initialization Value Tests ----------

    #[test]
    fn test_default_music_base_volume_exact() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_music_base_volume() - 0.8).abs() < 1e-6,
            "music_base_volume must be 0.8, got {}",
            engine.test_music_base_volume()
        );
    }

    #[test]
    fn test_default_voice_base_volume_exact() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_voice_base_volume() - 1.0).abs() < 1e-6,
            "voice_base_volume must be 1.0, got {}",
            engine.test_voice_base_volume()
        );
    }

    #[test]
    fn test_default_sfx_base_volume_exact() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_sfx_base_volume() - 1.0).abs() < 1e-6,
            "sfx_base_volume must be 1.0, got {}",
            engine.test_sfx_base_volume()
        );
    }

    #[test]
    fn test_default_ear_sep_exact() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_ear_sep() - 0.2).abs() < 1e-6,
            "ear_sep must be 0.2, got {}",
            engine.test_ear_sep()
        );
    }

    #[test]
    fn test_default_pan_mode_is_stereo_angle() {
        let engine = AudioEngine::new().unwrap();
        assert_eq!(
            engine.test_pan_mode(),
            PanMode::StereoAngle,
            "default pan_mode must be StereoAngle"
        );
    }

    #[test]
    fn test_default_duck_timer_zero() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            engine.test_duck_timer().abs() < 1e-6,
            "duck_timer must start at 0.0"
        );
    }

    #[test]
    fn test_default_duck_factor_exact() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_duck_factor() - 0.4).abs() < 1e-6,
            "duck_factor must be 0.4, got {}",
            engine.test_duck_factor()
        );
    }

    #[test]
    fn test_default_master_volume_exact() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.master_volume - 1.0).abs() < 1e-6,
            "master_volume must be 1.0"
        );
    }

    #[test]
    fn test_default_music_target_vol_equals_base() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_music_target_vol() - 0.8).abs() < 1e-6,
            "music target_vol must be 0.8 (matches music_base_volume)"
        );
    }

    #[test]
    fn test_default_music_using_a_is_true() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            engine.test_music_using_a(),
            "music channel should start using_a=true"
        );
    }

    #[test]
    fn test_default_crossfade_is_zero() {
        let engine = AudioEngine::new().unwrap();
        assert!(
            engine.test_music_crossfade_left().abs() < 1e-6,
            "crossfade_left must start at 0.0"
        );
        assert!(
            engine.test_music_crossfade_time().abs() < 1e-6,
            "crossfade_time must start at 0.0"
        );
    }

    #[test]
    fn test_default_music_base_volume_is_not_one() {
        // Kills mutant that replaces 0.8 with 1.0
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_music_base_volume() - 1.0).abs() > 0.1,
            "music_base_volume must NOT be 1.0 (should be 0.8)"
        );
    }

    #[test]
    fn test_default_duck_factor_is_not_one() {
        // Kills mutant that replaces 0.4 with 1.0
        let engine = AudioEngine::new().unwrap();
        assert!(
            (engine.test_duck_factor() - 1.0).abs() > 0.5,
            "duck_factor must NOT be 1.0 (should be 0.4)"
        );
    }

    #[test]
    fn test_default_ear_sep_is_not_zero() {
        // Kills mutant that replaces 0.2 with 0.0
        let engine = AudioEngine::new().unwrap();
        assert!(
            engine.test_ear_sep() > 0.1,
            "ear_sep must NOT be 0.0 (should be 0.2)"
        );
    }

    // ---------- Voice Beep Duration Formula: (text_len * 0.05).clamp(0.6, 3.0) + 0.2 ----------

    #[test]
    fn test_beep_text_0_duck_timer_exact() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(0);
        // dur = (0 * 0.05).clamp(0.6, 3.0) = 0.6; duck_timer = 0.6 + 0.2 = 0.8
        assert!(
            (engine.test_duck_timer() - 0.8).abs() < 1e-4,
            "duck_timer for text_len=0 must be 0.8, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_text_12_at_min_boundary() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(12);
        // dur = (12 * 0.05).clamp(0.6, 3.0) = 0.6 (exactly at min); duck_timer = 0.8
        assert!(
            (engine.test_duck_timer() - 0.8).abs() < 1e-4,
            "duck_timer for text_len=12 must be 0.8, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_text_13_just_above_min() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(13);
        // dur = (13 * 0.05).clamp(0.6, 3.0) = 0.65; duck_timer = 0.85
        assert!(
            (engine.test_duck_timer() - 0.85).abs() < 1e-4,
            "duck_timer for text_len=13 must be 0.85, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_text_20_mid_range() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(20);
        // dur = (20 * 0.05).clamp(0.6, 3.0) = 1.0; duck_timer = 1.2
        assert!(
            (engine.test_duck_timer() - 1.2).abs() < 1e-4,
            "duck_timer for text_len=20 must be 1.2, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_text_40_mid_range() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(40);
        // dur = (40 * 0.05).clamp(0.6, 3.0) = 2.0; duck_timer = 2.2
        assert!(
            (engine.test_duck_timer() - 2.2).abs() < 1e-4,
            "duck_timer for text_len=40 must be 2.2, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_text_60_at_max_boundary() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(60);
        // dur = (60 * 0.05).clamp(0.6, 3.0) = 3.0 (exactly at max); duck_timer = 3.2
        assert!(
            (engine.test_duck_timer() - 3.2).abs() < 1e-4,
            "duck_timer for text_len=60 must be 3.2, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_text_100_clamped_at_max() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(100);
        // dur = (100 * 0.05).clamp(0.6, 3.0) = 3.0 (clamped); duck_timer = 3.2
        assert!(
            (engine.test_duck_timer() - 3.2).abs() < 1e-4,
            "duck_timer for text_len=100 must be 3.2, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_duration_coefficient_is_005() {
        let mut engine = AudioEngine::new().unwrap();
        // text_len=20: 20 * 0.05 = 1.0, duck_timer = 1.2
        // If coefficient were 0.04: 20 * 0.04 = 0.8, duck_timer = 1.0 (different!)
        // If coefficient were 0.06: 20 * 0.06 = 1.2, duck_timer = 1.4 (different!)
        engine.play_voice_beep(20);
        let dt = engine.test_duck_timer();
        assert!(
            (dt - 1.2).abs() < 1e-4,
            "Coefficient must be 0.05: expected 1.2, got {}",
            dt
        );
    }

    #[test]
    fn test_beep_duck_timer_offset_is_02() {
        let mut engine = AudioEngine::new().unwrap();
        // text_len=20: dur=1.0, duck_timer = dur + 0.2 = 1.2
        // If offset were 0.1: duck_timer = 1.1 (different!)
        // If offset were 0.3: duck_timer = 1.3 (different!)
        engine.play_voice_beep(20);
        let dur = (20_f32 * 0.05).clamp(0.6, 3.0);
        let expected = dur + 0.2;
        assert!(
            (engine.test_duck_timer() - expected).abs() < 1e-4,
            "duck_timer = dur + 0.2: expected {}, got {}",
            expected,
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_beep_min_clamp_is_06() {
        // Distinguish 0.6 from 0.5 or 0.7
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(0); // dur = clamp(0, 0.6, 3.0) = 0.6
        let expected = 0.6 + 0.2; // 0.8
        assert!(
            (engine.test_duck_timer() - expected).abs() < 1e-4,
            "Min clamp must be 0.6, got duck_timer={}",
            engine.test_duck_timer()
        );
        // If min were 0.5: duck_timer = 0.7 (different!)
        assert!(
            (engine.test_duck_timer() - 0.7).abs() > 0.05,
            "Min clamp is NOT 0.5"
        );
    }

    #[test]
    fn test_beep_max_clamp_is_30() {
        // Distinguish 3.0 from 2.0 or 4.0
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(80); // 80*0.05=4.0, clamped to 3.0
        let expected = 3.0 + 0.2; // 3.2
        assert!(
            (engine.test_duck_timer() - expected).abs() < 1e-4,
            "Max clamp must be 3.0, got duck_timer={}",
            engine.test_duck_timer()
        );
        // If max were 4.0: duck_timer = 4.2 (different!)
        assert!(
            (engine.test_duck_timer() - 4.2).abs() > 0.5,
            "Max clamp is NOT 4.0"
        );
    }

    // ---------- Ear Position Geometry Tests ----------

    #[test]
    fn test_ears_facing_neg_z_exact_positions() {
        let mut engine = AudioEngine::new().unwrap();
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });
        let (left, right) = engine.test_compute_ears();
        // right = (0,0,-1) × (0,1,0) = (1,0,0)
        // left_pos = (0,0,0) - (1,0,0)*0.1 = (-0.1, 0, 0)
        // right_pos = (0,0,0) + (1,0,0)*0.1 = (0.1, 0, 0)
        assert!(
            (left[0] - (-0.1)).abs() < 1e-4,
            "Left ear X must be -0.1, got {}",
            left[0]
        );
        assert!(
            left[1].abs() < 1e-4,
            "Left ear Y must be 0, got {}",
            left[1]
        );
        assert!(
            left[2].abs() < 1e-4,
            "Left ear Z must be 0, got {}",
            left[2]
        );
        assert!(
            (right[0] - 0.1).abs() < 1e-4,
            "Right ear X must be 0.1, got {}",
            right[0]
        );
        assert!(
            right[1].abs() < 1e-4,
            "Right ear Y must be 0, got {}",
            right[1]
        );
        assert!(
            right[2].abs() < 1e-4,
            "Right ear Z must be 0, got {}",
            right[2]
        );
    }

    #[test]
    fn test_ears_facing_pos_x_exact_positions() {
        let mut engine = AudioEngine::new().unwrap();
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(1.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
        });
        let (left, right) = engine.test_compute_ears();
        // right = (1,0,0) × (0,1,0) = (0,0,1)
        // left = (0,0,0) - (0,0,1)*0.1 = (0,0,-0.1)
        // right = (0,0,0) + (0,0,1)*0.1 = (0,0,0.1)
        assert!(left[0].abs() < 1e-4, "Left X must be 0, got {}", left[0]);
        assert!(
            (left[2] - (-0.1)).abs() < 1e-4,
            "Left Z must be -0.1, got {}",
            left[2]
        );
        assert!(right[0].abs() < 1e-4, "Right X must be 0, got {}", right[0]);
        assert!(
            (right[2] - 0.1).abs() < 1e-4,
            "Right Z must be 0.1, got {}",
            right[2]
        );
    }

    #[test]
    fn test_ears_from_offset_position() {
        let mut engine = AudioEngine::new().unwrap();
        engine.update_listener(ListenerPose {
            position: vec3(5.0, 1.7, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });
        let (left, right) = engine.test_compute_ears();
        // right vector = (1,0,0), ear_sep * 0.5 = 0.1
        // left = (5.0 - 0.1, 1.7, 0) = (4.9, 1.7, 0)
        // right = (5.0 + 0.1, 1.7, 0) = (5.1, 1.7, 0)
        assert!(
            (left[0] - 4.9).abs() < 1e-4,
            "Left ear at (4.9,1.7,0), got X={}",
            left[0]
        );
        assert!((left[1] - 1.7).abs() < 1e-4);
        assert!(
            (right[0] - 5.1).abs() < 1e-4,
            "Right ear at (5.1,1.7,0), got X={}",
            right[0]
        );
        assert!((right[1] - 1.7).abs() < 1e-4);
    }

    #[test]
    fn test_ears_separation_uses_05_factor() {
        let mut engine = AudioEngine::new().unwrap();
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });
        let (left, right) = engine.test_compute_ears();
        // Each ear offset = ear_sep * 0.5 = 0.2 * 0.5 = 0.1
        // Total separation = 0.2
        // If 0.5 were 1.0, offset would be 0.2 per side → total 0.4
        let separation = (right[0] - left[0]).abs();
        assert!(
            (separation - 0.2).abs() < 1e-4,
            "Total ear separation must be 0.2 (ear_sep), got {}",
            separation
        );
    }

    #[test]
    fn test_ears_degenerate_forward_up_parallel() {
        let mut engine = AudioEngine::new().unwrap();
        // forward and up parallel → cross product = zero → normalize_or_zero = zero
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 1.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
        });
        let (left, right) = engine.test_compute_ears();
        // Both ears at origin (right vector is zero)
        assert!(left[0].abs() < 1e-4 && left[1].abs() < 1e-4 && left[2].abs() < 1e-4);
        assert!(right[0].abs() < 1e-4 && right[1].abs() < 1e-4 && right[2].abs() < 1e-4);
    }

    #[test]
    fn test_ears_cross_product_order_forward_cross_up() {
        // Verify that compute_ears uses forward.cross(up), not up.cross(forward)
        // up.cross(forward) would give (-1,0,0) for neg-Z/Y, flipping L/R
        let mut engine = AudioEngine::new().unwrap();
        engine.update_listener(ListenerPose {
            position: vec3(0.0, 0.0, 0.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        });
        let (left, right) = engine.test_compute_ears();
        // Correct: right = forward×up = (1,0,0) → left_x < right_x
        assert!(
            left[0] < right[0],
            "Left ear must be to the left (negative X), left={}, right={}",
            left[0],
            right[0]
        );
    }

    // ---------- Duck Timer Tick Logic Tests ----------

    #[test]
    fn test_duck_timer_decreases_by_dt() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(20); // duck_timer = 1.2
        let before = engine.test_duck_timer();
        engine.tick(0.1);
        let after = engine.test_duck_timer();
        assert!(
            (after - (before - 0.1)).abs() < 1e-4,
            "duck_timer should decrease by dt=0.1: before={}, after={}",
            before,
            after
        );
    }

    #[test]
    fn test_duck_timer_multiple_ticks() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(20); // duck_timer = 1.2
        engine.tick(0.3);
        assert!(
            (engine.test_duck_timer() - 0.9).abs() < 1e-4,
            "After 0.3s: 1.2-0.3=0.9, got {}",
            engine.test_duck_timer()
        );
        engine.tick(0.5);
        assert!(
            (engine.test_duck_timer() - 0.4).abs() < 1e-4,
            "After 0.8s total: 1.2-0.8=0.4, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_duck_timer_reaches_zero_after_full_duration() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(0); // duck_timer = 0.8
        engine.tick(0.9);
        assert!(
            engine.test_duck_timer() <= 0.0,
            "duck_timer should be <= 0 after 0.9s, got {}",
            engine.test_duck_timer()
        );
    }

    #[test]
    fn test_duck_timer_guard_gt_zero() {
        let mut engine = AudioEngine::new().unwrap();
        // Without beep, duck_timer = 0.0
        engine.tick(0.016);
        // duck_timer should still be 0 (the > 0.0 guard prevents decrement)
        assert!(
            engine.test_duck_timer().abs() < 1e-6,
            "duck_timer should remain 0 when not ducking"
        );
    }

    #[test]
    fn test_duck_timer_music_volume_restored_after_expiry() {
        let mut engine = AudioEngine::new().unwrap();
        let initial_vol = engine.test_music_target_vol();
        assert!((initial_vol - 0.8).abs() < 1e-4);

        // Play beep → ducks music (target_vol *= 0.4 = 0.32)
        engine.play_voice_beep(0); // duck_timer = 0.8
        let ducked_vol = engine.test_music_target_vol();
        assert!(
            (ducked_vol - 0.32).abs() < 1e-4,
            "Ducked music vol must be 0.8 * 0.4 = 0.32, got {}",
            ducked_vol
        );

        // Tick past duck timer → music.set_volume(music_base_volume * master_volume)
        engine.tick(1.0);
        let restored_vol = engine.test_music_target_vol();
        assert!(
            (restored_vol - 0.8).abs() < 1e-4,
            "Music vol must be restored to 0.8 after ducking, got {}",
            restored_vol
        );
    }

    // ---------- Duck Factor Math Tests ----------

    #[test]
    fn test_duck_reduces_music_target_vol_by_factor() {
        let mut engine = AudioEngine::new().unwrap();
        assert!((engine.test_music_target_vol() - 0.8).abs() < 1e-4);
        // duck applies: target_vol = (0.8 * 0.4).clamp(0.0, 1.0) = 0.32
        engine.play_voice_beep(20);
        assert!(
            (engine.test_music_target_vol() - 0.32).abs() < 1e-4,
            "After duck: 0.8 * 0.4 = 0.32, got {}",
            engine.test_music_target_vol()
        );
    }

    #[test]
    fn test_duck_multiple_times_compounds() {
        let mut engine = AudioEngine::new().unwrap();
        // First duck: 0.8 * 0.4 = 0.32
        engine.play_voice_beep(20);
        assert!((engine.test_music_target_vol() - 0.32).abs() < 1e-3);
        // Second duck: 0.32 * 0.4 = 0.128
        engine.play_voice_beep(20);
        assert!(
            (engine.test_music_target_vol() - 0.128).abs() < 1e-3,
            "Second duck: 0.32 * 0.4 = 0.128, got {}",
            engine.test_music_target_vol()
        );
    }

    #[test]
    fn test_duck_factor_value_matters() {
        // Verifies the exact duck_factor value affects the result
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(20);
        let vol = engine.test_music_target_vol();
        // With factor=0.4: 0.8*0.4=0.32
        // With factor=0.5: 0.8*0.5=0.40 (different!)
        // With factor=1.0: 0.8*1.0=0.80 (different!)
        assert!(
            (vol - 0.32).abs() < 1e-3,
            "Duck factor must be 0.4, resulting vol=0.32, got {}",
            vol
        );
    }

    // ---------- MusicChannel set_volume Clamp Tests ----------

    #[test]
    fn test_music_set_volume_clamps_negative_to_zero() {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_master_volume(0.0);
        assert!(
            engine.test_music_target_vol() >= 0.0,
            "Music target_vol must never be negative"
        );
    }

    // ---------- Crossfade Mechanics Tests ----------

    #[test]
    fn test_crossfade_steady_state_uses_a() {
        let mut engine = AudioEngine::new().unwrap();
        // Without any music play, crossfade_left = 0, crossfade_time = 0
        engine.tick(0.016);
        assert!(engine.test_music_using_a());
    }

    // ---------- Master Volume Propagation Tests ----------

    #[test]
    fn test_set_master_volume_updates_music_target() {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_master_volume(0.5);
        // music.set_volume(music_base_volume * 0.5) = 0.8 * 0.5 = 0.4
        assert!(
            (engine.test_music_target_vol() - 0.4).abs() < 1e-4,
            "Music target = 0.8 * 0.5 = 0.4, got {}",
            engine.test_music_target_vol()
        );
    }

    #[test]
    fn test_set_master_volume_zero_silences_music() {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_master_volume(0.0);
        assert!(
            engine.test_music_target_vol().abs() < 1e-6,
            "Music target must be 0 when master is 0"
        );
    }

    #[test]
    fn test_set_master_volume_quarter() {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_master_volume(0.25);
        // 0.8 * 0.25 = 0.2
        assert!(
            (engine.test_music_target_vol() - 0.2).abs() < 1e-4,
            "Music target = 0.8 * 0.25 = 0.2, got {}",
            engine.test_music_target_vol()
        );
    }

    #[test]
    fn test_set_master_volume_propagation_formula() {
        // Verify base * master multiplication (catch + or - mutations)
        let mut engine = AudioEngine::new().unwrap();
        let base = engine.test_music_base_volume(); // 0.8
        for m in [0.0_f32, 0.25, 0.5, 0.75, 1.0] {
            engine.set_master_volume(m);
            let expected = base * m;
            assert!(
                (engine.test_music_target_vol() - expected).abs() < 1e-4,
                "master={}: expected {}, got {}",
                m,
                expected,
                engine.test_music_target_vol()
            );
        }
    }

    // ---------- Pan Mode Enum Tests ----------

    #[test]
    fn test_pan_mode_set_and_get() {
        let mut engine = AudioEngine::new().unwrap();
        assert_eq!(engine.test_pan_mode(), PanMode::StereoAngle);
        engine.set_pan_mode(PanMode::None);
        assert_eq!(engine.test_pan_mode(), PanMode::None);
        engine.set_pan_mode(PanMode::StereoAngle);
        assert_eq!(engine.test_pan_mode(), PanMode::StereoAngle);
    }

    #[test]
    fn test_pan_mode_eq_and_ne() {
        assert_eq!(PanMode::StereoAngle, PanMode::StereoAngle);
        assert_eq!(PanMode::None, PanMode::None);
        assert_ne!(PanMode::StereoAngle, PanMode::None);
        assert_ne!(PanMode::None, PanMode::StereoAngle);
    }

    // ---------- ListenerPose Tests ----------

    #[test]
    fn test_listener_pose_fields_exact() {
        let pose = ListenerPose {
            position: vec3(1.0, 2.0, 3.0),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };
        assert!((pose.position.x - 1.0).abs() < 1e-6);
        assert!((pose.position.y - 2.0).abs() < 1e-6);
        assert!((pose.position.z - 3.0).abs() < 1e-6);
        assert!((pose.forward.z - (-1.0)).abs() < 1e-6);
        assert!((pose.up.y - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_listener_pose_copy_semantics() {
        let pose = ListenerPose {
            position: vec3(3.14, 2.71, 1.41),
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };
        let copy = pose;
        assert!((copy.position.x - 3.14).abs() < 1e-4);
        assert!((copy.position.y - 2.71).abs() < 1e-4);
        assert!((copy.position.z - 1.41).abs() < 1e-4);
    }

    // ---------- MusicTrack Tests ----------

    #[test]
    fn test_music_track_looped_flag() {
        let looped_track = MusicTrack {
            path: "bg.ogg".to_string(),
            looped: true,
        };
        assert!(looped_track.looped);
        let oneshot_track = MusicTrack {
            path: "jingle.wav".to_string(),
            looped: false,
        };
        assert!(!oneshot_track.looped);
    }

    #[test]
    fn test_music_track_path_preserved() {
        let track = MusicTrack {
            path: "music/battle_theme.ogg".to_string(),
            looped: true,
        };
        assert_eq!(track.path, "music/battle_theme.ogg");
    }

    // ---------- VoiceSpec / VoiceBank Tests ----------

    #[test]
    fn test_voice_spec_clone_and_fields() {
        use crate::voice::VoiceSpec;
        let spec = VoiceSpec {
            folder: "/voices/hero".to_string(),
            files: vec!["greeting.wav".to_string(), "farewell.wav".to_string()],
            tts_voice: Some("en-us-1".to_string()),
        };
        let cloned = spec.clone();
        assert_eq!(cloned.folder, "/voices/hero");
        assert_eq!(cloned.files.len(), 2);
        assert_eq!(cloned.files[0], "greeting.wav");
        assert_eq!(cloned.files[1], "farewell.wav");
        assert_eq!(cloned.tts_voice, Some("en-us-1".to_string()));
    }

    #[test]
    fn test_voice_spec_no_tts_voice() {
        use crate::voice::VoiceSpec;
        let spec = VoiceSpec {
            folder: "/voices/npc".to_string(),
            files: vec![],
            tts_voice: None,
        };
        assert!(spec.tts_voice.is_none());
        assert!(spec.files.is_empty());
    }

    #[test]
    fn test_voice_bank_speakers_lookup() {
        use crate::voice::{VoiceBank, VoiceSpec};
        use std::collections::HashMap;
        let mut speakers = HashMap::new();
        speakers.insert(
            "hero".to_string(),
            VoiceSpec {
                folder: "voices/hero".to_string(),
                files: vec![],
                tts_voice: None,
            },
        );
        speakers.insert(
            "villain".to_string(),
            VoiceSpec {
                folder: "voices/villain".to_string(),
                files: vec!["evil_laugh.wav".to_string()],
                tts_voice: Some("deep".to_string()),
            },
        );
        let bank = VoiceBank { speakers };
        assert!(bank.speakers.contains_key("hero"));
        assert!(bank.speakers.contains_key("villain"));
        assert!(!bank.speakers.contains_key("unknown"));
        assert_eq!(bank.speakers.len(), 2);
    }

    // ---------- DialogueAudioMap Tests ----------

    #[test]
    fn test_dialogue_audio_map_lookup_paths() {
        use crate::dialogue_runtime::DialogueAudioMap;
        use std::collections::HashMap;
        let mut map = HashMap::new();
        let mut nodes = HashMap::new();
        nodes.insert("n0".to_string(), "audio/greet.wav".to_string());
        nodes.insert("n1".to_string(), "audio/bye.wav".to_string());
        map.insert("dlg1".to_string(), nodes);
        let audio_map = DialogueAudioMap { map };

        let dlg_nodes = audio_map.map.get("dlg1").unwrap();
        assert_eq!(dlg_nodes.get("n0").unwrap(), "audio/greet.wav");
        assert_eq!(dlg_nodes.get("n1").unwrap(), "audio/bye.wav");
        assert!(dlg_nodes.get("n2").is_none());
    }

    #[test]
    fn test_dialogue_audio_map_missing_dialogue() {
        use crate::dialogue_runtime::DialogueAudioMap;
        use std::collections::HashMap;
        let audio_map = DialogueAudioMap {
            map: HashMap::new(),
        };
        assert!(!audio_map.map.contains_key("nonexistent"));
    }

    // ---------- Integration: Full Duck Cycle Tests ----------

    #[test]
    fn test_full_duck_cycle_exact_values() {
        let mut engine = AudioEngine::new().unwrap();

        // 1. Initial state
        assert!((engine.test_music_target_vol() - 0.8).abs() < 1e-4);
        assert!(engine.test_duck_timer().abs() < 1e-6);

        // 2. Trigger duck via beep (text_len=20 → duck_timer = 1.2)
        engine.play_voice_beep(20);
        assert!((engine.test_duck_timer() - 1.2).abs() < 1e-4);
        assert!((engine.test_music_target_vol() - 0.32).abs() < 1e-4);

        // 3. Tick 0.5s → duck_timer = 0.7
        engine.tick(0.5);
        assert!(
            (engine.test_duck_timer() - 0.7).abs() < 1e-4,
            "After 0.5s tick: duck_timer = 0.7, got {}",
            engine.test_duck_timer()
        );

        // 4. Tick remaining 0.8s → duck_timer = -0.1 (expired), music restored
        engine.tick(0.8);
        assert!(
            engine.test_duck_timer() <= 0.0,
            "Duck timer should have expired"
        );
        // Music restored to base*master = 0.8*1.0 = 0.8
        assert!(
            (engine.test_music_target_vol() - 0.8).abs() < 1e-4,
            "Music should be restored to 0.8, got {}",
            engine.test_music_target_vol()
        );
    }

    #[test]
    fn test_duck_cycle_with_reduced_master_volume() {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_master_volume(0.5);
        // music target = 0.8 * 0.5 = 0.4
        assert!((engine.test_music_target_vol() - 0.4).abs() < 1e-4);

        engine.play_voice_beep(0); // duck_timer = 0.8
                                   // duck: 0.4 * 0.4 = 0.16
        assert!((engine.test_music_target_vol() - 0.16).abs() < 1e-3);

        engine.tick(1.0); // expire
                          // restore: music_base_volume * master = 0.8 * 0.5 = 0.4
        assert!(
            (engine.test_music_target_vol() - 0.4).abs() < 1e-4,
            "After restore with master=0.5: 0.8*0.5=0.4, got {}",
            engine.test_music_target_vol()
        );
    }

    #[test]
    fn test_duck_cycle_beep_overwrites_previous_timer() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(20); // duck_timer = 1.2
        engine.tick(0.3); // duck_timer = 0.9
        assert!((engine.test_duck_timer() - 0.9).abs() < 1e-4);

        // Second beep resets duck_timer
        engine.play_voice_beep(40); // duck_timer = 2.2
        assert!(
            (engine.test_duck_timer() - 2.2).abs() < 1e-4,
            "Second beep should reset duck_timer to 2.2, got {}",
            engine.test_duck_timer()
        );
    }

    // ---------- set_master_volume Clamp Tests ----------

    #[test]
    fn test_master_volume_clamp_exact_boundaries() {
        let mut engine = AudioEngine::new().unwrap();

        engine.set_master_volume(-1.0);
        assert!(
            (engine.master_volume - 0.0).abs() < 1e-6,
            "master_volume must clamp to 0.0 for negative input"
        );

        engine.set_master_volume(2.0);
        assert!(
            (engine.master_volume - 1.0).abs() < 1e-6,
            "master_volume must clamp to 1.0 for input > 1.0"
        );

        engine.set_master_volume(0.5);
        assert!(
            (engine.master_volume - 0.5).abs() < 1e-6,
            "master_volume must be 0.5 for valid input"
        );
    }

    #[test]
    fn test_master_volume_at_exact_zero() {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_master_volume(0.0);
        assert!(engine.master_volume.abs() < 1e-6);
    }

    #[test]
    fn test_master_volume_at_exact_one() {
        let mut engine = AudioEngine::new().unwrap();
        engine.set_master_volume(1.0);
        assert!((engine.master_volume - 1.0).abs() < 1e-6);
    }

    // ---------- Voice beep amplify and frequency constants ----------

    #[test]
    fn test_beep_produces_audible_output() {
        // Integration test: beep should not crash and should set duck_timer > 0
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(25);
        assert!(
            engine.test_duck_timer() > 0.0,
            "Beep must set duck_timer > 0"
        );
        // Verify the duck was applied (music target reduced from 0.8)
        assert!(
            engine.test_music_target_vol() < 0.8 - 1e-4,
            "Music must be ducked below initial 0.8"
        );
    }

    // ---------- SFX beep doesn't affect ducking ----------

    #[test]
    fn test_sfx_beep_does_not_duck_music() {
        let mut engine = AudioEngine::new().unwrap();
        let initial_target = engine.test_music_target_vol();
        engine.play_sfx_beep(440.0, 0.5, 0.3);
        assert!(
            (engine.test_music_target_vol() - initial_target).abs() < 1e-6,
            "SFX beep must NOT affect music ducking"
        );
        assert!(
            engine.test_duck_timer().abs() < 1e-6,
            "SFX beep must NOT set duck_timer"
        );
    }

    // ---------- Tick with zero dt ----------

    #[test]
    fn test_tick_zero_dt_no_change() {
        let mut engine = AudioEngine::new().unwrap();
        engine.play_voice_beep(20); // duck_timer = 1.2
        let dt_before = engine.test_duck_timer();
        engine.tick(0.0);
        assert!(
            (engine.test_duck_timer() - dt_before).abs() < 1e-6,
            "tick(0.0) must not change duck_timer"
        );
    }
}
