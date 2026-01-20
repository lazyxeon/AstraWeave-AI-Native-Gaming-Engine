//! Resource cleanup tests for astraweave-audio
//!
//! Tests proper cleanup of audio resources on drop:
//! - Engine drop releases all handles safely
//! - Drop during active playback
//! - Multiple engine instances cleanup
//! - Drop after various operations
//!
//! P0-Critical: Ensures no resource leaks or panics on drop

use astraweave_audio::engine::{AudioEngine, ListenerPose, PanMode};
use glam::vec3;
use std::panic::catch_unwind;

/// Helper to assert that a closure does not panic
fn should_not_panic<F: FnOnce() + std::panic::UnwindSafe>(name: &str, f: F) {
    let result = catch_unwind(f);
    assert!(result.is_ok(), "Operation '{}' panicked: {:?}", name, result.err());
}

// ============================================================================
// Basic Drop Safety Tests
// ============================================================================

#[test]
fn test_engine_drop_immediate_no_panic() {
    should_not_panic("engine drop immediately after creation", || {
        let engine = AudioEngine::new();
        if let Ok(engine) = engine {
            drop(engine);
        }
    });
}

#[test]
fn test_engine_drop_after_beep_no_panic() {
    should_not_panic("engine drop after playing beep", || {
        if let Ok(mut engine) = AudioEngine::new() {
            engine.play_sfx_beep(440.0, 0.1, 0.5);
            drop(engine);
        }
    });
}

#[test]
fn test_engine_drop_during_playback_no_panic() {
    should_not_panic("engine drop during active playback", || {
        if let Ok(mut engine) = AudioEngine::new() {
            // Start multiple sounds
            engine.play_sfx_beep(440.0, 2.0, 0.5);
            engine.play_sfx_beep(880.0, 2.0, 0.5);
            
            // Drop while still playing
            drop(engine);
        }
    });
}

#[test]
fn test_engine_drop_after_tick_no_panic() {
    should_not_panic("engine drop after tick cycles", || {
        if let Ok(mut engine) = AudioEngine::new() {
            engine.play_sfx_beep(440.0, 1.0, 0.5);
            
            // Tick several times
            for _ in 0..60 {
                engine.tick(0.016);
            }
            
            drop(engine);
        }
    });
}

// ============================================================================
// Multiple Instance Cleanup Tests
// ============================================================================

#[test]
fn test_multiple_engines_sequential_drop_no_panic() {
    should_not_panic("multiple engines created and dropped sequentially", || {
        for _ in 0..5 {
            if let Ok(engine) = AudioEngine::new() {
                drop(engine);
            }
        }
    });
}

#[test]
fn test_multiple_engines_nested_drop_no_panic() {
    should_not_panic("nested engine creation and drop", || {
        if let Ok(engine1) = AudioEngine::new() {
            if let Ok(engine2) = AudioEngine::new() {
                if let Ok(engine3) = AudioEngine::new() {
                    drop(engine3);
                }
                drop(engine2);
            }
            drop(engine1);
        }
    });
}

// ============================================================================
// Drop After Various Operations
// ============================================================================

#[test]
fn test_drop_after_set_master_volume_no_panic() {
    should_not_panic("drop after set_master_volume", || {
        if let Ok(mut engine) = AudioEngine::new() {
            engine.set_master_volume(0.5);
            engine.set_master_volume(1.0);
            engine.set_master_volume(0.0);
            drop(engine);
        }
    });
}

#[test]
fn test_drop_after_update_listener_no_panic() {
    should_not_panic("drop after update_listener", || {
        if let Ok(mut engine) = AudioEngine::new() {
            let pose = ListenerPose {
                position: vec3(10.0, 5.0, -3.0),
                forward: vec3(0.0, 0.0, -1.0),
                up: vec3(0.0, 1.0, 0.0),
            };
            engine.update_listener(pose);
            drop(engine);
        }
    });
}

#[test]
fn test_drop_after_set_pan_mode_no_panic() {
    should_not_panic("drop after set_pan_mode", || {
        if let Ok(mut engine) = AudioEngine::new() {
            engine.set_pan_mode(PanMode::StereoAngle);
            engine.set_pan_mode(PanMode::None);
            drop(engine);
        }
    });
}

// ============================================================================
// 3D Spatial Audio Drop Tests
// ============================================================================

#[test]
fn test_drop_after_3d_beep_no_panic() {
    should_not_panic("drop after play_sfx_3d_beep", || {
        if let Ok(mut engine) = AudioEngine::new() {
            // 3D beep requires an emitter ID
            let emitter_id = 1u64;
            let pos = vec3(10.0, 0.0, 0.0);
            let _ = engine.play_sfx_3d_beep(emitter_id, pos, 440.0, 0.5, 0.5);
            drop(engine);
        }
    });
}

#[test]
fn test_drop_with_many_3d_emitters_no_panic() {
    should_not_panic("drop with many 3D emitters", || {
        if let Ok(mut engine) = AudioEngine::new() {
            // Create many emitters
            for i in 0..50 {
                let pos = vec3(i as f32 * 2.0, 0.0, 0.0);
                let _ = engine.play_sfx_3d_beep(i as u64, pos, 440.0, 0.1, 0.3);
            }
            drop(engine);
        }
    });
}

// ============================================================================
// Stop and Drop Tests
// ============================================================================

#[test]
fn test_drop_after_stop_music_no_panic() {
    should_not_panic("drop after stop_music", || {
        if let Ok(engine) = AudioEngine::new() {
            engine.stop_music();
            drop(engine);
        }
    });
}

#[test]
fn test_drop_after_voice_beep_no_panic() {
    should_not_panic("drop after voice beep", || {
        if let Ok(mut engine) = AudioEngine::new() {
            engine.play_voice_beep(100); // Simulate 100 character text
            drop(engine);
        }
    });
}

// ============================================================================
// Edge Cases and Error Recovery
// ============================================================================

#[test]
fn test_drop_after_extreme_volumes_no_panic() {
    should_not_panic("drop after extreme volume operations", || {
        if let Ok(mut engine) = AudioEngine::new() {
            // Extreme volumes
            engine.set_master_volume(f32::MAX);
            engine.set_master_volume(f32::MIN);
            engine.set_master_volume(f32::INFINITY);
            engine.set_master_volume(f32::NEG_INFINITY);
            engine.set_master_volume(f32::NAN);
            
            // Reset to sane value
            engine.set_master_volume(0.5);
            
            drop(engine);
        }
    });
}

#[test]
fn test_drop_after_extreme_listener_position_no_panic() {
    should_not_panic("drop after extreme listener positions", || {
        if let Ok(mut engine) = AudioEngine::new() {
            let pose = ListenerPose {
                position: vec3(f32::MAX, f32::MIN, 0.0),
                forward: vec3(0.0, 0.0, -1.0),
                up: vec3(0.0, 1.0, 0.0),
            };
            engine.update_listener(pose);
            drop(engine);
        }
    });
}

#[test]
fn test_drop_after_nan_listener_no_panic() {
    should_not_panic("drop after NaN listener positions", || {
        if let Ok(mut engine) = AudioEngine::new() {
            let pose = ListenerPose {
                position: vec3(f32::NAN, f32::NAN, f32::NAN),
                forward: vec3(f32::NAN, f32::NAN, f32::NAN),
                up: vec3(f32::NAN, f32::NAN, f32::NAN),
            };
            engine.update_listener(pose);
            drop(engine);
        }
    });
}

#[test]
fn test_drop_after_zero_forward_listener_no_panic() {
    should_not_panic("drop after zero-vector listener forward", || {
        if let Ok(mut engine) = AudioEngine::new() {
            let pose = ListenerPose {
                position: vec3(0.0, 0.0, 0.0),
                forward: vec3(0.0, 0.0, 0.0), // Zero forward!
                up: vec3(0.0, 1.0, 0.0),
            };
            engine.update_listener(pose);
            drop(engine);
        }
    });
}

// ============================================================================
// Scope and Ownership Tests
// ============================================================================

#[test]
fn test_drop_from_different_scope_no_panic() {
    should_not_panic("engine passed to inner scope and dropped", || {
        if let Ok(mut engine) = AudioEngine::new() {
            engine.play_sfx_beep(440.0, 0.5, 0.5);
            
            {
                // Move engine to inner scope
                let mut inner_engine = engine;
                inner_engine.tick(0.1);
                // inner_engine dropped here
            }
        }
    });
}

#[test]
fn test_option_drop_some_no_panic() {
    should_not_panic("Option<AudioEngine> drop with Some", || {
        let engine: Option<AudioEngine> = AudioEngine::new().ok();
        if let Some(mut e) = engine {
            e.play_sfx_beep(440.0, 0.1, 0.5);
        }
        // Implicit drop here
    });
}

#[test]
fn test_vec_engines_drop_no_panic() {
    should_not_panic("Vec of engines drop", || {
        let engines: Vec<_> = (0..3)
            .filter_map(|_| AudioEngine::new().ok())
            .collect();
        
        // Drop all at once
        drop(engines);
    });
}

// ============================================================================
// Rapid Create/Drop Stress Tests
// ============================================================================

#[test]
fn test_rapid_create_drop_cycle_no_panic() {
    should_not_panic("rapid create/drop cycle", || {
        for _ in 0..25 {
            if let Ok(mut engine) = AudioEngine::new() {
                engine.play_sfx_beep(440.0, 0.01, 0.1);
            }
        }
    });
}

#[test]
fn test_rapid_create_play_tick_drop_no_panic() {
    should_not_panic("rapid create/play/tick/drop cycle", || {
        for i in 0..10 {
            if let Ok(mut engine) = AudioEngine::new() {
                let freq = 220.0 + (i as f32 * 20.0);
                engine.play_sfx_beep(freq, 0.05, 0.2);
                engine.tick(0.016);
            }
        }
    });
}

// ============================================================================
// Memory Safety During Drop
// ============================================================================

#[test]
fn test_drop_does_not_double_free() {
    should_not_panic("verify no double-free on drop", || {
        if let Ok(mut engine) = AudioEngine::new() {
            // Create complex state
            engine.play_sfx_beep(440.0, 0.5, 0.3);
            engine.play_sfx_beep(880.0, 0.5, 0.3);
            engine.play_voice_beep(50);
            
            // Set various states
            engine.set_master_volume(0.7);
            let pose = ListenerPose {
                position: vec3(5.0, 2.0, -1.0),
                forward: vec3(0.0, 0.0, -1.0),
                up: vec3(0.0, 1.0, 0.0),
            };
            engine.update_listener(pose);
            
            // Drop should clean up without double-free
            drop(engine);
        }
    });
}

#[test]
fn test_drop_with_muted_state_no_panic() {
    should_not_panic("drop with muted playback state", || {
        if let Ok(mut engine) = AudioEngine::new() {
            engine.play_sfx_beep(440.0, 1.0, 0.5);
            engine.set_master_volume(0.0); // Effectively mute
            drop(engine);
        }
    });
}

// ============================================================================
// Multiple Drop Cycles Stability
// ============================================================================

#[test]
fn test_multiple_drop_cycles_stable() {
    should_not_panic("multiple drop cycles stability test", || {
        for cycle in 0..10 {
            if let Ok(mut engine) = AudioEngine::new() {
                // More activity in later cycles
                for i in 0..(cycle + 1) {
                    let freq = 440.0 + (i as f32 * 20.0);
                    engine.play_sfx_beep(freq, 0.02, 0.1);
                }
                
                // Tick a bit
                for _ in 0..5 {
                    engine.tick(0.016);
                }
            }
        }
    });
}

#[test]
fn test_drop_without_any_operations_no_panic() {
    should_not_panic("drop engine that was never used", || {
        if let Ok(engine) = AudioEngine::new() {
            // No operations at all
            drop(engine);
        }
    });
}

// ============================================================================
// Complex Lifecycle Tests
// ============================================================================

#[test]
fn test_full_lifecycle_drop_no_panic() {
    should_not_panic("full lifecycle then drop", || {
        if let Ok(mut engine) = AudioEngine::new() {
            // Initialize
            engine.set_master_volume(0.8);
            engine.set_pan_mode(PanMode::StereoAngle);
            
            // Update listener multiple times
            for i in 0..10 {
                let pose = ListenerPose {
                    position: vec3(i as f32, 0.0, 0.0),
                    forward: vec3(0.0, 0.0, -1.0),
                    up: vec3(0.0, 1.0, 0.0),
                };
                engine.update_listener(pose);
            }
            
            // Play various sounds
            engine.play_sfx_beep(440.0, 0.2, 0.5);
            engine.play_voice_beep(25);
            
            // Tick through
            for _ in 0..30 {
                engine.tick(0.016);
            }
            
            // Stop music
            engine.stop_music();
            
            // Change volume
            engine.set_master_volume(0.5);
            
            // Final tick
            engine.tick(0.016);
            
            // Drop
            drop(engine);
        }
    });
}

#[test]
fn test_3d_audio_lifecycle_drop_no_panic() {
    should_not_panic("3D audio lifecycle then drop", || {
        if let Ok(mut engine) = AudioEngine::new() {
            // Setup listener
            let pose = ListenerPose {
                position: vec3(0.0, 0.0, 0.0),
                forward: vec3(0.0, 0.0, -1.0),
                up: vec3(0.0, 1.0, 0.0),
            };
            engine.update_listener(pose);
            
            // Play 3D sounds at various positions
            for i in 0..20 {
                let pos = vec3(
                    (i as f32 * 2.0) - 20.0,
                    0.0,
                    -(i as f32),
                );
                let _ = engine.play_sfx_3d_beep(i as u64, pos, 440.0 + (i as f32 * 10.0), 0.1, 0.3);
            }
            
            // Move listener through space
            for i in 0..20 {
                let pose = ListenerPose {
                    position: vec3(i as f32 - 10.0, 0.0, 0.0),
                    forward: vec3(0.0, 0.0, -1.0),
                    up: vec3(0.0, 1.0, 0.0),
                };
                engine.update_listener(pose);
                engine.tick(0.016);
            }
            
            // Drop with all 3D sounds active
            drop(engine);
        }
    });
}
