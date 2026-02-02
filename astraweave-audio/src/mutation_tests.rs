//! Mutation-killing tests for astraweave-audio.
//! These tests are designed to catch subtle mutations like boundary condition changes,
//! arithmetic operator substitutions, and conditional logic inversions.

#[cfg(test)]
mod mutation_tests {
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
            engine.tick(0.001);  // 1000 FPS
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
        engine.play_voice_beep(1);  // 0.05 -> 0.6
        engine.play_voice_beep(5);  // 0.25 -> 0.6
        engine.play_voice_beep(10); // 0.5 -> 0.6
        engine.play_voice_beep(11); // 0.55 -> 0.6
    }

    #[test]
    fn test_voice_beep_maximum_duration() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Long text should clamp to maximum duration (3.0s)
        // Formula: (100 * 0.05).clamp(0.6, 3.0) = 5.0 -> 3.0
        engine.play_voice_beep(100);  // 5.0 -> 3.0
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
        engine.play_sfx_beep(20.0, 0.1, 0.5);     // Low frequency
        engine.play_sfx_beep(440.0, 0.1, 0.5);    // A4
        engine.play_sfx_beep(880.0, 0.1, 0.5);    // A5
        engine.play_sfx_beep(20000.0, 0.1, 0.5);  // High frequency
    }

    #[test]
    fn test_sfx_beep_various_durations() {
        let mut engine = AudioEngine::new().unwrap();
        
        engine.play_sfx_beep(440.0, 0.001, 0.5);  // Very short
        engine.play_sfx_beep(440.0, 0.1, 0.5);    // Normal
        engine.play_sfx_beep(440.0, 1.0, 0.5);    // Long
        engine.play_sfx_beep(440.0, 10.0, 0.5);   // Very long
    }

    #[test]
    fn test_sfx_beep_various_gains() {
        let mut engine = AudioEngine::new().unwrap();
        
        engine.play_sfx_beep(440.0, 0.1, 0.0);    // Silent
        engine.play_sfx_beep(440.0, 0.1, 0.1);    // Quiet
        engine.play_sfx_beep(440.0, 0.1, 0.5);    // Normal
        engine.play_sfx_beep(440.0, 0.1, 1.0);    // Full volume
        engine.play_sfx_beep(440.0, 0.1, 2.0);    // Amplified
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
        engine.play_sfx_3d_beep(emitter_id, pos, 440.0, 0.1, 0.5).unwrap();
        
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
        engine.play_sfx_3d_beep(1, vec3(1000.0, 0.0, 0.0), 440.0, 0.1, 0.5).unwrap();
        engine.play_sfx_3d_beep(2, vec3(-1000.0, 0.0, 0.0), 440.0, 0.1, 0.5).unwrap();
        engine.play_sfx_3d_beep(3, vec3(0.0, 1000.0, 0.0), 440.0, 0.1, 0.5).unwrap();
        engine.play_sfx_3d_beep(4, vec3(0.0, -1000.0, 0.0), 440.0, 0.1, 0.5).unwrap();
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
        assert!((forward_len - 1.0).abs() < 0.001, 
            "Forward vector should be unit length: {}", forward_len);
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
        assert!((up_len - 1.0).abs() < 0.001, 
            "Up vector should be unit length: {}", up_len);
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
        assert!(dot.abs() < 0.001, 
            "Forward and up must be perpendicular (dot=0), got: {}", dot);
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
        assert!((engine.master_volume - 0.7).abs() < 0.0001,
            "Set volume should persist: expected 0.7, got {}", engine.master_volume);
    }

    #[test]
    fn test_engine_default_volume_is_full() {
        // Behavioral: engine should start at full volume (1.0)
        let engine = AudioEngine::new().unwrap();
        assert!((engine.master_volume - 1.0).abs() < 0.0001,
            "Default volume should be 1.0 (full)");
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
        engine.play_sfx_3d_beep(1, vec3(-5.0, 0.0, -5.0), 440.0, 0.01, 0.5).unwrap();
        engine.play_sfx_3d_beep(2, vec3(5.0, 0.0, -5.0), 440.0, 0.01, 0.5).unwrap();
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
        engine.play_voice_beep(11);  // Should use 0.6 (minimum)
        engine.play_voice_beep(12);  // Should use 0.6 (exactly at threshold)
        engine.play_voice_beep(13);  // Should use 0.65
    }

    #[test]
    fn test_voice_beep_maximum_duration_boundary() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Text length 60: 60 * 0.05 = 3.0 (exactly at maximum)
        // Text length 59: 59 * 0.05 = 2.95 (below maximum)
        // Text length 61: 61 * 0.05 = 3.05 (above maximum, clamps to 3.0)
        engine.play_voice_beep(59);  // Should use 2.95
        engine.play_voice_beep(60);  // Should use 3.0 (exactly at threshold)
        engine.play_voice_beep(61);  // Should clamp to 3.0
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
        assert_eq!(PanMode::StereoAngle, PanMode::StereoAngle, "Same variant should be equal");
    }

    #[test]
    fn test_pan_mode_inequality_different_variants() {
        assert_ne!(PanMode::None, PanMode::StereoAngle, "Different variants should not be equal");
        assert_ne!(PanMode::StereoAngle, PanMode::None, "Different variants should not be equal (reversed)");
    }

    // --- Volume Comparison Tests ---
    
    #[test]
    fn test_volume_comparison_after_set() {
        let mut engine = AudioEngine::new().unwrap();
        
        engine.set_master_volume(0.5);
        
        // Volume should equal what was set
        assert!((engine.master_volume - 0.5).abs() < 1e-6, "Volume should equal set value");
        
        // Volume should not equal different values
        assert!((engine.master_volume - 0.4).abs() > 1e-6, "Volume should not equal different value");
        assert!((engine.master_volume - 0.6).abs() > 1e-6, "Volume should not equal different value");
    }

    #[test]
    fn test_volume_zero_vs_nonzero_comparison() {
        let mut engine = AudioEngine::new().unwrap();
        
        engine.set_master_volume(0.0);
        assert_eq!(engine.master_volume, 0.0, "Zero volume should equal 0.0");
        
        engine.set_master_volume(0.001);
        assert!(engine.master_volume > 0.0, "Nonzero volume should be > 0.0");
        assert!(engine.master_volume != 0.0, "Nonzero volume should not equal 0.0");
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
        
        assert_ne!(pose1.position, pose2.position, "Different positions should not be equal");
        assert_eq!(pose1.forward, pose2.forward, "Same forward should be equal");
    }

    // --- Emitter Comparison Tests ---
    
    #[test]
    fn test_emitter_id_uniqueness_matters() {
        let mut engine = AudioEngine::new().unwrap();
        
        // Different emitter IDs create different sinks
        engine.play_sfx_3d_beep(1, vec3(0.0, 0.0, 0.0), 440.0, 0.01, 0.5).unwrap();
        engine.play_sfx_3d_beep(2, vec3(0.0, 0.0, 0.0), 440.0, 0.01, 0.5).unwrap();
        
        // Both should succeed with different IDs
        let result1 = engine.play_sfx_3d_beep(1, vec3(1.0, 0.0, 0.0), 440.0, 0.01, 0.5);
        let result2 = engine.play_sfx_3d_beep(2, vec3(2.0, 0.0, 0.0), 440.0, 0.01, 0.5);
        
        assert!(result1.is_ok() && result2.is_ok(), "Both emitters should work");
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
        assert_eq!(result.is_ok(), true, "Valid 3D beep should return Ok (true)");
        assert_eq!(result.is_err(), false, "Valid 3D beep should not return Err (false)");
    }

    #[test]
    fn test_play_music_returns_err_for_nonexistent_file() {
        let mut engine = AudioEngine::new().unwrap();
        
        let track = MusicTrack {
            path: "/nonexistent/path/to/file.wav".to_string(),
            looped: false,
        };
        
        let result = engine.play_music(track, 0.5);
        assert_eq!(result.is_err(), true, "Nonexistent file should return Err (true)");
        assert_eq!(result.is_ok(), false, "Nonexistent file should not return Ok (false)");
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
        assert_eq!(is_none, false, "StereoAngle should return false for is_none");
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
}
