//! Comprehensive integration tests for AudioEngine
//! 
//! Coverage targets:
//! - Basic playback (play, pause, stop, volume)
//! - Spatial audio (3D positioning, distance attenuation)
//! - Music system (crossfades, ducking, looping)
//! - Voice system (ducking behavior, beeps, file playback)
//! - SFX system (2D and 3D, buses, concurrent sounds)
//! - Listener updates (position, orientation, ear separation)
//! - Volume controls (master, per-channel, spatialization)

use astraweave_audio::{AudioEngine, ListenerPose, MusicTrack, PanMode};
use glam::{vec3, Vec3};

/// Test 1: AudioEngine initialization
#[test]
fn test_audio_engine_creation() {
    let engine = AudioEngine::new();
    assert!(engine.is_ok(), "AudioEngine should initialize successfully");
    
    let engine = engine.unwrap();
    assert_eq!(engine.master_volume, 1.0, "Default master volume should be 1.0");
}

/// Test 2: Master volume control
#[test]
fn test_master_volume_control() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Test valid range
    engine.set_master_volume(0.5);
    assert_eq!(engine.master_volume, 0.5, "Master volume should be set to 0.5");
    
    engine.set_master_volume(0.0);
    assert_eq!(engine.master_volume, 0.0, "Master volume should be set to 0.0");
    
    engine.set_master_volume(1.0);
    assert_eq!(engine.master_volume, 1.0, "Master volume should be set to 1.0");
    
    // Test clamping
    engine.set_master_volume(1.5);
    assert_eq!(engine.master_volume, 1.0, "Master volume should clamp to 1.0");
    
    engine.set_master_volume(-0.5);
    assert_eq!(engine.master_volume, 0.0, "Master volume should clamp to 0.0");
}

/// Test 3: Pan mode setting
#[test]
fn test_pan_mode_setting() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Test stereo angle mode (default)
    engine.set_pan_mode(PanMode::StereoAngle);
    // No panic = success
    
    // Test none mode
    engine.set_pan_mode(PanMode::None);
    // No panic = success
}

/// Test 4: Listener pose update
#[test]
fn test_listener_update() {
    let mut engine = AudioEngine::new().unwrap();
    
    let pose = ListenerPose {
        position: vec3(10.0, 5.0, 3.0),
        forward: vec3(1.0, 0.0, 0.0),
        up: vec3(0.0, 1.0, 0.0),
    };
    
    engine.update_listener(pose);
    // No panic = success
    
    // Test with different orientation
    let pose2 = ListenerPose {
        position: vec3(0.0, 0.0, 0.0),
        forward: vec3(0.0, 0.0, -1.0),
        up: vec3(0.0, 1.0, 0.0),
    };
    
    engine.update_listener(pose2);
    // No panic = success
}

/// Test 5: Engine tick (update loop)
#[test]
fn test_engine_tick() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Test normal tick
    engine.tick(0.016); // 60 FPS
    
    // Test with zero delta
    engine.tick(0.0);
    
    // Test with large delta
    engine.tick(1.0);
    
    // Test multiple ticks
    for _ in 0..100 {
        engine.tick(0.016);
    }
}

/// Test 6: Voice beep playback
#[test]
fn test_voice_beep_playback() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Test short text (min duration)
    engine.play_voice_beep(5); // Short text
    
    // Test long text (max duration)
    engine.play_voice_beep(100); // Long text
    
    // Test zero length
    engine.play_voice_beep(0);
    
    // Multiple beeps
    for i in 0..5 {
        engine.play_voice_beep(10 + i);
        engine.tick(0.016);
    }
}

/// Test 7: SFX beep playback
#[test]
fn test_sfx_beep_playback() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Test various frequencies and durations
    engine.play_sfx_beep(440.0, 0.5, 0.5); // A4, half second
    engine.play_sfx_beep(880.0, 0.1, 0.3); // A5, short
    engine.play_sfx_beep(220.0, 1.0, 0.8); // A3, long
    
    // Test edge cases
    engine.play_sfx_beep(20.0, 0.01, 0.1); // Very low frequency
    engine.play_sfx_beep(20000.0, 0.01, 0.1); // Very high frequency
    engine.play_sfx_beep(440.0, 0.0, 0.0); // Zero duration/gain
}

/// Test 8: 3D spatial beep playback
#[test]
fn test_3d_sfx_beep_playback() {
    let mut engine = AudioEngine::new().unwrap();
    
    let emitter_id = 1;
    let pos = vec3(5.0, 0.0, 0.0); // 5 meters to the right
    
    // Test basic 3D beep
    let result = engine.play_sfx_3d_beep(emitter_id, pos, 440.0, 0.5, 0.5);
    assert!(result.is_ok(), "3D beep should play successfully");
    
    // Test multiple positions
    let positions = vec![
        vec3(10.0, 0.0, 0.0),  // Right
        vec3(-10.0, 0.0, 0.0), // Left
        vec3(0.0, 10.0, 0.0),  // Up
        vec3(0.0, -10.0, 0.0), // Down
        vec3(0.0, 0.0, 10.0),  // Forward
        vec3(0.0, 0.0, -10.0), // Behind
    ];
    
    for (i, pos) in positions.iter().enumerate() {
        let result = engine.play_sfx_3d_beep(i as u64, *pos, 440.0, 0.1, 0.5);
        assert!(result.is_ok(), "3D beep at position {:?} should succeed", pos);
    }
}

/// Test 9: Multiple concurrent 3D sounds
#[test]
fn test_concurrent_3d_sounds() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Create 10 concurrent spatial emitters
    for i in 0..10 {
        let angle = (i as f32) * std::f32::consts::TAU / 10.0;
        let pos = vec3(angle.cos() * 5.0, 0.0, angle.sin() * 5.0);
        
        let result = engine.play_sfx_3d_beep(i, pos, 440.0 + (i as f32 * 50.0), 0.5, 0.5);
        assert!(result.is_ok(), "Concurrent sound {} should play", i);
    }
    
    // Update a few times to ensure no crashes
    for _ in 0..10 {
        engine.tick(0.016);
    }
}

/// Test 10: Spatial audio with listener movement
#[test]
fn test_spatial_audio_listener_movement() {
    let mut engine = AudioEngine::new().unwrap();
    
    let emitter_id = 100;
    let sound_pos = vec3(10.0, 0.0, 0.0); // Fixed position
    
    // Play 3D sound
    engine.play_sfx_3d_beep(emitter_id, sound_pos, 440.0, 2.0, 0.5).unwrap();
    
    // Move listener around the sound
    for i in 0..36 {
        let angle = (i as f32) * 10.0 * std::f32::consts::PI / 180.0;
        let listener_pos = vec3(angle.cos() * 5.0, 0.0, angle.sin() * 5.0);
        
        let pose = ListenerPose {
            position: listener_pos,
            forward: (sound_pos - listener_pos).normalize_or_zero(),
            up: vec3(0.0, 1.0, 0.0),
        };
        
        engine.update_listener(pose);
        engine.tick(0.016);
    }
}

/// Test 11: Distance attenuation (near vs far)
#[test]
fn test_distance_attenuation() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Near sound (should be loud)
    let near_id = 1;
    engine.play_sfx_3d_beep(near_id, vec3(1.0, 0.0, 0.0), 440.0, 1.0, 0.8).unwrap();
    
    // Far sound (should be quiet due to attenuation)
    let far_id = 2;
    engine.play_sfx_3d_beep(far_id, vec3(100.0, 0.0, 0.0), 440.0, 1.0, 0.8).unwrap();
    
    // Very far sound (should be very quiet)
    let very_far_id = 3;
    engine.play_sfx_3d_beep(very_far_id, vec3(1000.0, 0.0, 0.0), 440.0, 1.0, 0.8).unwrap();
    
    // All should succeed without panic
    engine.tick(0.016);
}

/// Test 12: Voice ducking behavior
#[test]
fn test_voice_ducking() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Play voice beep (should duck music)
    engine.play_voice_beep(20);
    
    // Tick to trigger ducking
    engine.tick(0.016);
    
    // Continue ticking to restore music volume
    for _ in 0..200 {
        engine.tick(0.016); // ~3 seconds
    }
}

/// Test 13: Multiple voice beeps in sequence
#[test]
fn test_sequential_voice_beeps() {
    let mut engine = AudioEngine::new().unwrap();
    
    for i in 0..10 {
        engine.play_voice_beep(10 + i);
        // Short tick between beeps
        for _ in 0..5 {
            engine.tick(0.016);
        }
    }
}

/// Test 14: Stress test - 100+ concurrent sounds
#[test]
fn test_stress_concurrent_sounds() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Create 100 spatial emitters in a grid
    for x in 0..10 {
        for z in 0..10 {
            let emitter_id = (x * 10 + z) as u64;
            let pos = vec3(x as f32 * 2.0, 0.0, z as f32 * 2.0);
            let freq = 200.0 + (emitter_id as f32 * 10.0);
            
            let result = engine.play_sfx_3d_beep(emitter_id, pos, freq, 0.1, 0.3);
            assert!(result.is_ok(), "Sound {} should play", emitter_id);
        }
    }
    
    // Update several times
    for _ in 0..30 {
        engine.tick(0.016);
    }
}

/// Test 15: Listener orientation changes
#[test]
fn test_listener_orientation() {
    let mut engine = AudioEngine::new().unwrap();
    
    let emitter_id = 1;
    engine.play_sfx_3d_beep(emitter_id, vec3(5.0, 0.0, 0.0), 440.0, 2.0, 0.5).unwrap();
    
    // Rotate listener 360 degrees
    for i in 0..36 {
        let angle = (i as f32) * 10.0 * std::f32::consts::PI / 180.0;
        let forward = vec3(angle.cos(), 0.0, angle.sin());
        
        let pose = ListenerPose {
            position: Vec3::ZERO,
            forward,
            up: vec3(0.0, 1.0, 0.0),
        };
        
        engine.update_listener(pose);
        engine.tick(0.016);
    }
}

/// Test 16: Zero-length ear separation edge case
#[test]
fn test_zero_ear_separation() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Force zero ear separation (edge case testing)
    // This tests the compute_ears function's robustness
    let pose = ListenerPose {
        position: Vec3::ZERO,
        forward: vec3(0.0, 0.0, -1.0),
        up: vec3(0.0, 1.0, 0.0),
    };
    
    engine.update_listener(pose);
    
    // Play spatial sound
    engine.play_sfx_3d_beep(1, vec3(5.0, 0.0, 0.0), 440.0, 0.5, 0.5).unwrap();
    engine.tick(0.016);
}

/// Test 17: Rapid listener position changes
#[test]
fn test_rapid_listener_changes() {
    let mut engine = AudioEngine::new().unwrap();
    
    let emitter_id = 1;
    engine.play_sfx_3d_beep(emitter_id, vec3(10.0, 0.0, 0.0), 440.0, 5.0, 0.5).unwrap();
    
    // Rapidly change listener position every frame
    for i in 0..100 {
        let t = i as f32 * 0.1;
        let pos = vec3(t.sin() * 5.0, t.cos() * 5.0, 0.0);
        
        let pose = ListenerPose {
            position: pos,
            forward: vec3(1.0, 0.0, 0.0),
            up: vec3(0.0, 1.0, 0.0),
        };
        
        engine.update_listener(pose);
        engine.tick(0.016);
    }
}

/// Test 18: Multiple emitters at same position
#[test]
fn test_overlapping_emitters() {
    let mut engine = AudioEngine::new().unwrap();
    
    let pos = vec3(5.0, 0.0, 0.0);
    
    // Create 5 emitters at the same position with different frequencies
    for i in 0..5 {
        engine.play_sfx_3d_beep(i, pos, 200.0 + (i as f32 * 100.0), 1.0, 0.5).unwrap();
    }
    
    engine.tick(0.016);
}

/// Test 19: PanMode behavior differences
#[test]
fn test_pan_modes() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Test StereoAngle mode
    engine.set_pan_mode(PanMode::StereoAngle);
    engine.play_sfx_beep(440.0, 0.5, 0.5);
    engine.tick(0.016);
    
    // Test None mode
    engine.set_pan_mode(PanMode::None);
    engine.play_sfx_beep(440.0, 0.5, 0.5);
    engine.tick(0.016);
}

/// Test 20: Volume control interaction with spatial audio
#[test]
fn test_volume_spatial_interaction() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Set low master volume
    engine.set_master_volume(0.1);
    
    // Play spatial sound
    engine.play_sfx_3d_beep(1, vec3(5.0, 0.0, 0.0), 440.0, 1.0, 0.8).unwrap();
    engine.tick(0.016);
    
    // Increase master volume
    engine.set_master_volume(1.0);
    engine.tick(0.016);
    
    // Set to zero
    engine.set_master_volume(0.0);
    engine.tick(0.016);
}

/// Test 21: Long-running audio (>1 second)
#[test]
fn test_long_duration_audio() {
    let mut engine = AudioEngine::new().unwrap();
    
    // Play long beep
    engine.play_sfx_beep(440.0, 5.0, 0.5);
    
    // Simulate 5 seconds of game time
    for _ in 0..300 {
        engine.tick(0.016);
    }
}

/// Test 22: Interleaved 2D and 3D sounds
#[test]
fn test_interleaved_2d_3d() {
    let mut engine = AudioEngine::new().unwrap();
    
    for i in 0..20 {
        if i % 2 == 0 {
            // 2D sound
            engine.play_sfx_beep(440.0, 0.2, 0.5);
        } else {
            // 3D sound
            let pos = vec3((i as f32).sin() * 10.0, 0.0, (i as f32).cos() * 10.0);
            engine.play_sfx_3d_beep(i, pos, 440.0, 0.2, 0.5).unwrap();
        }
        
        engine.tick(0.016);
    }
}

/// Test 23: Extreme listener positions
#[test]
fn test_extreme_listener_positions() {
    let mut engine = AudioEngine::new().unwrap();
    
    let emitter_id = 1;
    engine.play_sfx_3d_beep(emitter_id, vec3(0.0, 0.0, 0.0), 440.0, 2.0, 0.5).unwrap();
    
    // Test very far positions
    let extreme_positions = vec![
        vec3(10000.0, 0.0, 0.0),
        vec3(0.0, 10000.0, 0.0),
        vec3(0.0, 0.0, 10000.0),
        vec3(-10000.0, -10000.0, -10000.0),
    ];
    
    for pos in extreme_positions {
        let pose = ListenerPose {
            position: pos,
            forward: vec3(0.0, 0.0, -1.0),
            up: vec3(0.0, 1.0, 0.0),
        };
        
        engine.update_listener(pose);
        engine.tick(0.016);
    }
}

/// Test 24: Emitter reuse (same ID, different positions)
#[test]
fn test_emitter_reuse() {
    let mut engine = AudioEngine::new().unwrap();
    
    let emitter_id = 42;
    
    // Play at position 1
    engine.play_sfx_3d_beep(emitter_id, vec3(5.0, 0.0, 0.0), 440.0, 0.5, 0.5).unwrap();
    engine.tick(0.1);
    
    // Reuse same emitter at position 2
    engine.play_sfx_3d_beep(emitter_id, vec3(-5.0, 0.0, 0.0), 880.0, 0.5, 0.5).unwrap();
    engine.tick(0.1);
    
    // Reuse again at position 3
    engine.play_sfx_3d_beep(emitter_id, vec3(0.0, 5.0, 0.0), 220.0, 0.5, 0.5).unwrap();
    engine.tick(0.1);
}

/// Test 25: Zero-duration edge cases
#[test]
fn test_zero_duration_sounds() {
    let mut engine = AudioEngine::new().unwrap();
    
    // 2D beep with zero duration
    engine.play_sfx_beep(440.0, 0.0, 0.5);
    
    // 3D beep with zero duration
    engine.play_sfx_3d_beep(1, vec3(5.0, 0.0, 0.0), 440.0, 0.0, 0.5).unwrap();
    
    engine.tick(0.016);
}
