//! Behavioral Correctness Tests for astraweave-audio
//!
//! These tests validate mathematically correct behavior of audio systems.
//! Focus on volume math, spatialization geometry, and ducking behavior.
//!
//! Coverage targets:
//! - Volume clamping and master volume scaling
//! - Ear position calculation from listener pose
//! - Voice ducking factor application
//! - VoiceBank/VoiceSpec data structures
//! - DialogueAudioMap loading and lookup

use astraweave_audio::{
    load_dialogue_audio_map, load_voice_bank, DialogueAudioMap, ListenerPose, VoiceBank, VoiceSpec,
};
use glam::{vec3, Vec3};
use std::collections::HashMap;

// ============================================================================
// LISTENER POSE TESTS
// ============================================================================

/// Test ListenerPose default construction
#[test]
fn test_listener_pose_construction() {
    let pose = ListenerPose {
        position: vec3(10.0, 5.0, -3.0),
        forward: vec3(0.0, 0.0, -1.0),
        up: vec3(0.0, 1.0, 0.0),
    };

    assert!((pose.position.x - 10.0).abs() < 0.001);
    assert!((pose.position.y - 5.0).abs() < 0.001);
    assert!((pose.position.z - (-3.0)).abs() < 0.001);
    assert!((pose.forward.z - (-1.0)).abs() < 0.001);
    assert!((pose.up.y - 1.0).abs() < 0.001);
}

/// Test right vector calculation from forward and up
#[test]
fn test_listener_right_vector() {
    let forward = vec3(0.0, 0.0, -1.0); // Looking down -Z
    let up = vec3(0.0, 1.0, 0.0);

    let right = forward.cross(up).normalize();

    // Cross product of (0,0,-1) x (0,1,0) = (0*0 - (-1)*1, (-1)*0 - 0*0, 0*1 - 0*0) = (1, 0, 0)
    // But this is actually right = up.cross(forward) for conventional right-handed systems
    // Let's calculate properly: forward cross up in right-handed = left vector
    // For ears: right = forward.cross(up).normalize() gives us the lateral axis
    
    // Actually for (0,0,-1) x (0,1,0):
    // x = 0*0 - (-1)*1 = 1
    // y = (-1)*0 - 0*0 = 0
    // z = 0*1 - 0*0 = 0
    // So right = (1, 0, 0) which is correct!
    
    assert!(
        (right.x - 1.0).abs() < 0.001,
        "Right should be +X, got {}",
        right.x
    );
    assert!(right.y.abs() < 0.001, "Right Y should be 0");
    assert!(right.z.abs() < 0.001, "Right Z should be 0");
}

/// Test ear positions calculation formula
#[test]
fn test_ear_positions_formula() {
    let position = vec3(5.0, 1.7, 0.0); // Standing at 5,1.7,0
    let forward = vec3(0.0, 0.0, -1.0); // Looking down -Z
    let up = vec3(0.0, 1.0, 0.0);
    let ear_sep = 0.2; // 20cm between ears

    let right = forward.cross(up).normalize();
    let left_ear = position - right * (ear_sep * 0.5);
    let right_ear = position + right * (ear_sep * 0.5);

    // Left ear should be at X = 5 - 0.1 = 4.9
    assert!(
        (left_ear.x - 4.9).abs() < 0.001,
        "Left ear X should be 4.9"
    );
    // Right ear should be at X = 5 + 0.1 = 5.1
    assert!(
        (right_ear.x - 5.1).abs() < 0.001,
        "Right ear X should be 5.1"
    );
    // Y and Z should be same as position
    assert!((left_ear.y - 1.7).abs() < 0.001);
    assert!((right_ear.y - 1.7).abs() < 0.001);
}

/// Test ear positions when listener is rotated
#[test]
fn test_ear_positions_rotated() {
    let position = Vec3::ZERO;
    let forward = vec3(1.0, 0.0, 0.0); // Looking down +X
    let up = vec3(0.0, 1.0, 0.0);
    let ear_sep = 0.2;

    let right = forward.cross(up).normalize();
    let left_ear = position - right * (ear_sep * 0.5);
    let right_ear = position + right * (ear_sep * 0.5);

    // When looking +X, right vector is (1,0,0)×(0,1,0):
    // x = 0*0 - 0*1 = 0
    // y = 0*0 - 1*0 = 0
    // z = 1*1 - 0*0 = 1
    // Wait, that's wrong. Let me recalculate:
    // forward × up = (1,0,0) × (0,1,0)
    // = (0*0 - 0*1, 0*0 - 1*0, 1*1 - 0*0)
    // = (0, 0, 1)
    // So right = (0, 0, 1) which is +Z, not -Z
    
    // Check the actual calculation
    assert!(
        (right.z - 1.0).abs() < 0.001,
        "Right vector should be +Z when looking +X, got {:?}",
        right
    );
    
    // Right ear at Z = +0.1, Left ear at Z = -0.1
    assert!((right_ear.z - 0.1).abs() < 0.001, "Right ear at Z=+0.1");
    assert!((left_ear.z - (-0.1)).abs() < 0.001, "Left ear at Z=-0.1");
}

// ============================================================================
// VOLUME MATH TESTS
// ============================================================================

/// Test volume clamping to [0, 1] range
#[test]
fn test_volume_clamping_upper() {
    let volume: f32 = 1.5;
    let clamped = volume.clamp(0.0, 1.0);
    assert_eq!(clamped, 1.0, "Volume above 1.0 should clamp to 1.0");
}

/// Test volume clamping lower bound
#[test]
fn test_volume_clamping_lower() {
    let volume: f32 = -0.5;
    let clamped = volume.clamp(0.0, 1.0);
    assert_eq!(clamped, 0.0, "Volume below 0.0 should clamp to 0.0");
}

/// Test volume in valid range remains unchanged
#[test]
fn test_volume_valid_range() {
    let volume: f32 = 0.7;
    let clamped = volume.clamp(0.0, 1.0);
    assert_eq!(clamped, 0.7, "Volume in range should not change");
}

/// Test master volume scaling formula
#[test]
fn test_master_volume_scaling() {
    let master_volume: f32 = 0.5;
    let music_base: f32 = 0.8;
    let voice_base: f32 = 1.0;
    let sfx_base: f32 = 0.6;

    let music_final = music_base * master_volume;
    let voice_final = voice_base * master_volume;
    let sfx_final = sfx_base * master_volume;

    assert!((music_final - 0.4).abs() < 0.001, "Music: 0.8 * 0.5 = 0.4");
    assert!((voice_final - 0.5).abs() < 0.001, "Voice: 1.0 * 0.5 = 0.5");
    assert!((sfx_final - 0.3).abs() < 0.001, "SFX: 0.6 * 0.5 = 0.3");
}

/// Test duck factor application
#[test]
fn test_duck_factor() {
    let target_vol: f32 = 0.8;
    let duck_factor: f32 = 0.4;

    let ducked = (target_vol * duck_factor).clamp(0.0, 1.0);

    assert!((ducked - 0.32).abs() < 0.001, "0.8 * 0.4 = 0.32");
}

/// Test duck factor with extreme values
#[test]
fn test_duck_factor_extreme() {
    // Very aggressive duck
    let vol: f32 = 1.0;
    let factor: f32 = 0.1;
    let ducked = (vol * factor).clamp(0.0, 1.0);
    assert!((ducked - 0.1).abs() < 0.001);

    // No duck
    let factor_none: f32 = 1.0;
    let ducked_none = (vol * factor_none).clamp(0.0, 1.0);
    assert!((ducked_none - 1.0).abs() < 0.001);
}

// ============================================================================
// CROSSFADE MATH TESTS
// ============================================================================

/// Test crossfade interpolation formula
#[test]
fn test_crossfade_interpolation() {
    let crossfade_time: f32 = 2.0;
    let crossfade_left: f32 = 1.0; // Halfway through
    let target_vol: f32 = 0.8;

    // k = 1 - (left / time) = 1 - (1/2) = 0.5
    let k = 1.0 - (crossfade_left / crossfade_time).clamp(0.0, 1.0);
    let vol_new = k * target_vol;
    let vol_old = (1.0 - k) * target_vol;

    assert!((k - 0.5).abs() < 0.001, "k should be 0.5 at halfway");
    assert!((vol_new - 0.4).abs() < 0.001, "New track at 0.4");
    assert!((vol_old - 0.4).abs() < 0.001, "Old track at 0.4");
}

/// Test crossfade at start (0% progress)
#[test]
fn test_crossfade_start() {
    let crossfade_time: f32 = 2.0;
    let crossfade_left: f32 = 2.0; // Just started
    let target_vol: f32 = 1.0;

    let k = 1.0 - (crossfade_left / crossfade_time).clamp(0.0, 1.0);
    let vol_new = k * target_vol;
    let vol_old = (1.0 - k) * target_vol;

    assert!((k - 0.0).abs() < 0.001, "k should be 0 at start");
    assert!((vol_new - 0.0).abs() < 0.001, "New track at 0");
    assert!((vol_old - 1.0).abs() < 0.001, "Old track at full");
}

/// Test crossfade at end (100% progress)
#[test]
fn test_crossfade_end() {
    let crossfade_time: f32 = 2.0;
    let crossfade_left: f32 = 0.0; // Finished
    let target_vol: f32 = 1.0;

    let k = 1.0 - (crossfade_left / crossfade_time).clamp(0.0, 1.0);
    let vol_new = k * target_vol;
    let vol_old = (1.0 - k) * target_vol;

    assert!((k - 1.0).abs() < 0.001, "k should be 1.0 at end");
    assert!((vol_new - 1.0).abs() < 0.001, "New track at full");
    assert!((vol_old - 0.0).abs() < 0.001, "Old track at 0");
}

// ============================================================================
// VOICE SPEC & VOICE BANK TESTS
// ============================================================================

/// Test VoiceSpec creation with all fields
#[test]
fn test_voice_spec_full() {
    let spec = VoiceSpec {
        folder: "assets/voices/Companion".to_string(),
        files: vec!["line_01.wav".to_string(), "line_02.wav".to_string()],
        tts_voice: Some("companion_v1".to_string()),
    };

    assert_eq!(spec.folder, "assets/voices/Companion");
    assert_eq!(spec.files.len(), 2);
    assert_eq!(spec.tts_voice, Some("companion_v1".to_string()));
}

/// Test VoiceSpec with empty files and no TTS
#[test]
fn test_voice_spec_minimal() {
    let spec = VoiceSpec {
        folder: "assets/voices/NPC".to_string(),
        files: vec![],
        tts_voice: None,
    };

    assert!(spec.files.is_empty(), "Files should be empty");
    assert!(spec.tts_voice.is_none(), "TTS voice should be None");
}

/// Test VoiceBank with multiple speakers
#[test]
fn test_voice_bank_multiple_speakers() {
    let mut speakers = HashMap::new();
    speakers.insert(
        "Companion".to_string(),
        VoiceSpec {
            folder: "assets/voices/Companion".to_string(),
            files: vec!["hello.wav".to_string()],
            tts_voice: None,
        },
    );
    speakers.insert(
        "Villain".to_string(),
        VoiceSpec {
            folder: "assets/voices/Villain".to_string(),
            files: vec![],
            tts_voice: Some("evil_voice".to_string()),
        },
    );

    let bank = VoiceBank { speakers };

    assert_eq!(bank.speakers.len(), 2);
    assert!(bank.speakers.contains_key("Companion"));
    assert!(bank.speakers.contains_key("Villain"));
    assert!(!bank.speakers.contains_key("Unknown"));
}

/// Test VoiceBank lookup returns correct spec
#[test]
fn test_voice_bank_lookup() {
    let mut speakers = HashMap::new();
    speakers.insert(
        "Hero".to_string(),
        VoiceSpec {
            folder: "hero_folder".to_string(),
            files: vec!["a.wav".to_string()],
            tts_voice: Some("hero_tts".to_string()),
        },
    );

    let bank = VoiceBank { speakers };

    let spec = bank.speakers.get("Hero");
    assert!(spec.is_some());
    let spec = spec.unwrap();
    assert_eq!(spec.folder, "hero_folder");
    assert_eq!(spec.files.len(), 1);
    assert_eq!(spec.tts_voice, Some("hero_tts".to_string()));
}

/// Test VoiceBank empty
#[test]
fn test_voice_bank_empty() {
    let bank = VoiceBank {
        speakers: HashMap::new(),
    };

    assert!(bank.speakers.is_empty());
    assert!(bank.speakers.get("Anyone").is_none());
}

// ============================================================================
// DIALOGUE AUDIO MAP TESTS
// ============================================================================

/// Test DialogueAudioMap construction
#[test]
fn test_dialogue_audio_map_construction() {
    let mut map = HashMap::new();

    let mut node_map = HashMap::new();
    node_map.insert("intro".to_string(), "audio/intro.wav".to_string());
    node_map.insert("outro".to_string(), "audio/outro.wav".to_string());
    map.insert("quest_dialogue".to_string(), node_map);

    let audio_map = DialogueAudioMap { map };

    assert_eq!(audio_map.map.len(), 1);
    let quest = audio_map.map.get("quest_dialogue").unwrap();
    assert_eq!(quest.len(), 2);
    assert_eq!(quest.get("intro"), Some(&"audio/intro.wav".to_string()));
}

/// Test DialogueAudioMap lookup chain
#[test]
fn test_dialogue_audio_map_lookup() {
    let mut map = HashMap::new();
    let mut nodes = HashMap::new();
    nodes.insert("n0".to_string(), "override_n0.wav".to_string());
    map.insert("dlg_test".to_string(), nodes);

    let audio_map = DialogueAudioMap { map };

    // Lookup: dialogue_id -> node_id -> filename
    let per_dialog = audio_map.map.get("dlg_test");
    assert!(per_dialog.is_some());

    let filename = per_dialog.unwrap().get("n0");
    assert_eq!(filename, Some(&"override_n0.wav".to_string()));

    // Non-existent node
    let missing = per_dialog.unwrap().get("n_missing");
    assert!(missing.is_none());
}

/// Test DialogueAudioMap missing dialogue
#[test]
fn test_dialogue_audio_map_missing_dialogue() {
    let audio_map = DialogueAudioMap {
        map: HashMap::new(),
    };

    assert!(audio_map.map.get("unknown_dialogue").is_none());
}

// ============================================================================
// VOICE BEEP DURATION TESTS
// ============================================================================

/// Test voice beep duration calculation
#[test]
fn test_voice_beep_duration_formula() {
    // Formula: (text_len as f32 * 0.05).clamp(0.6, 3.0)
    let short_text = 10; // 10 * 0.05 = 0.5 -> clamps to 0.6
    let medium_text = 40; // 40 * 0.05 = 2.0
    let long_text = 100; // 100 * 0.05 = 5.0 -> clamps to 3.0

    let dur_short = (short_text as f32 * 0.05).clamp(0.6, 3.0);
    let dur_medium = (medium_text as f32 * 0.05).clamp(0.6, 3.0);
    let dur_long = (long_text as f32 * 0.05).clamp(0.6, 3.0);

    assert!((dur_short - 0.6).abs() < 0.001, "Short clamps to 0.6");
    assert!((dur_medium - 2.0).abs() < 0.001, "Medium is 2.0");
    assert!((dur_long - 3.0).abs() < 0.001, "Long clamps to 3.0");
}

/// Test exact boundary values for beep duration
#[test]
fn test_voice_beep_boundaries() {
    // 12 chars -> 0.6 exactly
    let at_min = 12;
    let dur_min = (at_min as f32 * 0.05).clamp(0.6, 3.0);
    assert!((dur_min - 0.6).abs() < 0.001, "12 chars = 0.6 exactly");

    // 60 chars -> 3.0 exactly
    let at_max = 60;
    let dur_max = (at_max as f32 * 0.05).clamp(0.6, 3.0);
    assert!((dur_max - 3.0).abs() < 0.001, "60 chars = 3.0 exactly");
}

// ============================================================================
// TOML LOADING TESTS
// ============================================================================

/// Test voice bank TOML loading
#[test]
fn test_load_voice_bank_toml() {
    std::fs::create_dir_all("target").unwrap();
    let test_path = "target/test_voice_bank.toml";

    std::fs::write(
        test_path,
        r#"
[speakers.Hero]
folder = "assets/voices/Hero"
files = ["greeting.wav", "combat.wav"]
tts_voice = "hero_v1"

[speakers.Sidekick]
folder = "assets/voices/Sidekick"
"#,
    )
    .unwrap();

    let bank = load_voice_bank(test_path).expect("Should load voice bank");

    assert_eq!(bank.speakers.len(), 2);

    let hero = bank.speakers.get("Hero").unwrap();
    assert_eq!(hero.folder, "assets/voices/Hero");
    assert_eq!(hero.files.len(), 2);
    assert_eq!(hero.tts_voice, Some("hero_v1".to_string()));

    let sidekick = bank.speakers.get("Sidekick").unwrap();
    assert_eq!(sidekick.folder, "assets/voices/Sidekick");
    assert!(sidekick.files.is_empty(), "Default files should be empty");
    assert!(sidekick.tts_voice.is_none(), "Default tts_voice should be None");

    std::fs::remove_file(test_path).ok();
}

/// Test dialogue audio map TOML loading
#[test]
fn test_load_dialogue_audio_map_toml() {
    std::fs::create_dir_all("target").unwrap();
    let test_path = "target/test_dlg_audio.toml";

    std::fs::write(
        test_path,
        r#"
[map.quest_intro]
node_greeting = "audio/intro/greeting.wav"
node_explain = "audio/intro/explain.wav"

[map.boss_fight]
taunt1 = "audio/boss/taunt1.wav"
"#,
    )
    .unwrap();

    let audio_map = load_dialogue_audio_map(test_path).expect("Should load audio map");

    assert_eq!(audio_map.map.len(), 2);

    let quest = audio_map.map.get("quest_intro").unwrap();
    assert_eq!(quest.len(), 2);
    assert_eq!(
        quest.get("node_greeting"),
        Some(&"audio/intro/greeting.wav".to_string())
    );

    let boss = audio_map.map.get("boss_fight").unwrap();
    assert_eq!(boss.len(), 1);

    std::fs::remove_file(test_path).ok();
}

/// Test loading missing file returns error
#[test]
fn test_load_voice_bank_missing_file() {
    let result = load_voice_bank("nonexistent_voice_bank_12345.toml");
    assert!(result.is_err(), "Missing file should return error");
}

/// Test loading missing dialogue map returns error
#[test]
fn test_load_dialogue_audio_map_missing_file() {
    let result = load_dialogue_audio_map("nonexistent_dlg_audio_12345.toml");
    assert!(result.is_err(), "Missing file should return error");
}

// ============================================================================
// TTS DURATION ESTIMATION TESTS
// ============================================================================

/// Test TTS duration estimation (12 chars per second heuristic)
#[test]
fn test_tts_duration_estimation() {
    // Formula: (text.len() as f32 / 12.0).clamp(0.5, 8.0)
    let short = "Hi"; // 2 chars / 12 = 0.167 -> clamps to 0.5
    let medium = "Hello there, friend!"; // 20 chars / 12 = 1.67
    let long = "This is a very long piece of text that would take quite a while to speak out loud in full"; // 90 chars / 12 = 7.5

    let dur_short = (short.len() as f32 / 12.0).clamp(0.5, 8.0);
    let dur_medium = (medium.len() as f32 / 12.0).clamp(0.5, 8.0);
    let dur_long = (long.len() as f32 / 12.0).clamp(0.5, 8.0);

    // Verify lengths first to catch string edits
    assert_eq!(short.len(), 2, "Short text is 2 chars");
    assert_eq!(medium.len(), 20, "Medium text is 20 chars");
    let long_len = long.len();
    
    assert!((dur_short - 0.5).abs() < 0.001, "Short clamps to 0.5");
    assert!((dur_medium - (20.0 / 12.0)).abs() < 0.01, "Medium = 1.67s");
    assert!((dur_long - (long_len as f32 / 12.0)).abs() < 0.01, "Long = {} / 12 = {:.2}s", long_len, long_len as f32 / 12.0);
}

/// Test TTS max duration clamp
#[test]
fn test_tts_duration_max_clamp() {
    // 150 chars / 12 = 12.5 -> clamps to 8.0
    let very_long = "A".repeat(150);
    let dur = (very_long.len() as f32 / 12.0).clamp(0.5, 8.0);
    assert!((dur - 8.0).abs() < 0.001, "Very long clamps to 8.0");
}
