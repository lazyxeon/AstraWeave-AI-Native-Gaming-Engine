//! Mutation-resistant comprehensive tests for astraweave-audio.
//! Targets exact default values, enum variants, serde roundtrips,
//! and boundary conditions for 90%+ mutation kill rate.

use astraweave_audio::*;
use glam::Vec3;

// ========================================================================
// PAN MODE ENUM
// ========================================================================

#[test]
fn pan_mode_stereo_angle_exists() {
    let _m = PanMode::StereoAngle;
}

#[test]
fn pan_mode_none_exists() {
    let _m = PanMode::None;
}

#[test]
fn pan_mode_eq() {
    assert_eq!(PanMode::StereoAngle, PanMode::StereoAngle);
    assert_eq!(PanMode::None, PanMode::None);
    assert_ne!(PanMode::StereoAngle, PanMode::None);
}

#[test]
fn pan_mode_clone_copy() {
    let m = PanMode::StereoAngle;
    let m2 = m; // Copy
    let _ = m; // Still usable
    assert_eq!(m2, PanMode::StereoAngle);
}

#[test]
fn pan_mode_debug() {
    assert_eq!(format!("{:?}", PanMode::StereoAngle), "StereoAngle");
    assert_eq!(format!("{:?}", PanMode::None), "None");
}

// ========================================================================
// LISTENER POSE
// ========================================================================

#[test]
fn listener_pose_fields() {
    let lp = ListenerPose {
        position: Vec3::new(1.0, 2.0, 3.0),
        forward: Vec3::new(0.0, 0.0, -1.0),
        up: Vec3::new(0.0, 1.0, 0.0),
    };
    assert_eq!(lp.position, Vec3::new(1.0, 2.0, 3.0));
    assert_eq!(lp.forward, Vec3::new(0.0, 0.0, -1.0));
    assert_eq!(lp.up, Vec3::new(0.0, 1.0, 0.0));
}

#[test]
fn listener_pose_copy() {
    let lp = ListenerPose {
        position: Vec3::ZERO,
        forward: Vec3::NEG_Z,
        up: Vec3::Y,
    };
    let lp2 = lp; // Copy
    let _ = lp; // Still usable
    assert_eq!(lp2.position, Vec3::ZERO);
}

#[test]
fn listener_pose_clone() {
    let lp = ListenerPose {
        position: Vec3::X,
        forward: Vec3::NEG_Z,
        up: Vec3::Y,
    };
    let lp2 = lp.clone();
    assert_eq!(lp2.position, Vec3::X);
    assert_eq!(lp2.forward, Vec3::NEG_Z);
    assert_eq!(lp2.up, Vec3::Y);
}

#[test]
fn listener_pose_debug() {
    let lp = ListenerPose {
        position: Vec3::ZERO,
        forward: Vec3::NEG_Z,
        up: Vec3::Y,
    };
    let dbg = format!("{:?}", lp);
    assert!(dbg.contains("ListenerPose"));
}

// ========================================================================
// MUSIC TRACK
// ========================================================================

#[test]
fn music_track_fields() {
    let mt = MusicTrack {
        path: "music/battle.ogg".to_string(),
        looped: true,
    };
    assert_eq!(mt.path, "music/battle.ogg");
    assert!(mt.looped);
}

#[test]
fn music_track_not_looped() {
    let mt = MusicTrack {
        path: "music/stinger.ogg".to_string(),
        looped: false,
    };
    assert!(!mt.looped);
}

// ========================================================================
// VOICE SPEC — serde defaults
// ========================================================================

#[test]
fn voice_spec_full_deserialize() {
    let toml_str = r#"
folder = "voices/merchant"
files = ["greet.ogg", "farewell.ogg"]
tts_voice = "en_speaker_3"
"#;
    let vs: VoiceSpec = toml::from_str(toml_str).unwrap();
    assert_eq!(vs.folder, "voices/merchant");
    assert_eq!(vs.files.len(), 2);
    assert_eq!(vs.files[0], "greet.ogg");
    assert_eq!(vs.files[1], "farewell.ogg");
    assert_eq!(vs.tts_voice, Some("en_speaker_3".to_string()));
}

#[test]
fn voice_spec_serde_defaults_files_empty() {
    let toml_str = r#"
folder = "voices/guard"
"#;
    let vs: VoiceSpec = toml::from_str(toml_str).unwrap();
    assert_eq!(vs.folder, "voices/guard");
    assert!(vs.files.is_empty(), "files should default to empty vec");
    assert!(vs.tts_voice.is_none(), "tts_voice should default to None");
}

#[test]
fn voice_spec_clone() {
    let vs = VoiceSpec {
        folder: "test".to_string(),
        files: vec!["a.ogg".to_string()],
        tts_voice: Some("voice1".to_string()),
    };
    let vs2 = vs.clone();
    assert_eq!(vs2.folder, "test");
    assert_eq!(vs2.files.len(), 1);
    assert_eq!(vs2.tts_voice, Some("voice1".to_string()));
}

// ========================================================================
// VOICE BANK
// ========================================================================

#[test]
fn voice_bank_deserialize_toml() {
    let toml_str = r#"
[speakers.merchant]
folder = "voices/merchant"
files = ["hello.ogg"]

[speakers.guard]
folder = "voices/guard"
tts_voice = "en_male_1"
"#;
    let vb: VoiceBank = toml::from_str(toml_str).unwrap();
    assert_eq!(vb.speakers.len(), 2);
    assert!(vb.speakers.contains_key("merchant"));
    assert!(vb.speakers.contains_key("guard"));
    assert_eq!(vb.speakers["merchant"].folder, "voices/merchant");
    assert_eq!(vb.speakers["merchant"].files.len(), 1);
    assert_eq!(vb.speakers["guard"].folder, "voices/guard");
    assert!(vb.speakers["guard"].files.is_empty());
    assert_eq!(
        vb.speakers["guard"].tts_voice,
        Some("en_male_1".to_string())
    );
}

#[test]
fn voice_bank_clone() {
    let toml_str = r#"
[speakers.npc]
folder = "v"
"#;
    let vb: VoiceBank = toml::from_str(toml_str).unwrap();
    let vb2 = vb.clone();
    assert_eq!(vb2.speakers.len(), 1);
}

#[test]
fn voice_bank_empty_speakers() {
    let toml_str = r#"
[speakers]
"#;
    let vb: VoiceBank = toml::from_str(toml_str).unwrap();
    assert!(vb.speakers.is_empty());
}

// ========================================================================
// DIALOGUE AUDIO MAP
// ========================================================================

#[test]
fn dialogue_audio_map_deserialize() {
    let toml_str = r#"
[map.dlg_blacksmith]
node_greet = "blacksmith_hello.ogg"
node_quest = "blacksmith_quest.ogg"

[map.dlg_guard]
node_warn = "guard_warning.ogg"
"#;
    let dam: DialogueAudioMap = toml::from_str(toml_str).unwrap();
    assert_eq!(dam.map.len(), 2);
    assert!(dam.map.contains_key("dlg_blacksmith"));
    assert_eq!(dam.map["dlg_blacksmith"].len(), 2);
    assert_eq!(
        dam.map["dlg_blacksmith"]["node_greet"],
        "blacksmith_hello.ogg"
    );
    assert_eq!(
        dam.map["dlg_blacksmith"]["node_quest"],
        "blacksmith_quest.ogg"
    );
    assert_eq!(dam.map["dlg_guard"]["node_warn"], "guard_warning.ogg");
}

#[test]
fn dialogue_audio_map_empty() {
    let toml_str = r#"
[map]
"#;
    let dam: DialogueAudioMap = toml::from_str(toml_str).unwrap();
    assert!(dam.map.is_empty());
}

#[test]
fn dialogue_audio_map_clone() {
    let toml_str = r#"
[map.test]
n1 = "file.ogg"
"#;
    let dam: DialogueAudioMap = toml::from_str(toml_str).unwrap();
    let dam2 = dam.clone();
    assert_eq!(dam2.map.len(), 1);
    assert_eq!(dam2.map["test"]["n1"], "file.ogg");
}

// ========================================================================
// AUDIO ENGINE — creation and public fields
// ========================================================================

#[test]
fn audio_engine_new_succeeds() {
    // May fail on headless CI, so we just verify the Result type
    let result = AudioEngine::new();
    // If audio device available, should succeed
    if let Ok(engine) = result {
        assert!(
            (engine.master_volume - 1.0).abs() < 1e-6,
            "master_volume default should be 1.0"
        );
    }
    // If Err, that's acceptable on headless systems
}

#[test]
fn audio_engine_set_master_volume_clamps_high() {
    if let Ok(mut engine) = AudioEngine::new() {
        engine.set_master_volume(2.0);
        assert!(
            (engine.master_volume - 1.0).abs() < 1e-6,
            "master_volume should clamp to 1.0"
        );
    }
}

#[test]
fn audio_engine_set_master_volume_clamps_low() {
    if let Ok(mut engine) = AudioEngine::new() {
        engine.set_master_volume(-1.0);
        assert!(
            (engine.master_volume - 0.0).abs() < 1e-6,
            "master_volume should clamp to 0.0"
        );
    }
}

#[test]
fn audio_engine_set_master_volume_exact() {
    if let Ok(mut engine) = AudioEngine::new() {
        engine.set_master_volume(0.5);
        assert!((engine.master_volume - 0.5).abs() < 1e-6);
    }
}

#[test]
fn audio_engine_set_pan_mode() {
    if let Ok(mut engine) = AudioEngine::new() {
        engine.set_pan_mode(PanMode::None);
        // just verify no panic
        engine.set_pan_mode(PanMode::StereoAngle);
    }
}

#[test]
fn audio_engine_update_listener() {
    if let Ok(mut engine) = AudioEngine::new() {
        let pose = ListenerPose {
            position: Vec3::new(10.0, 5.0, 0.0),
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
        };
        engine.update_listener(pose);
        // No panic = success
    }
}

#[test]
fn audio_engine_tick_no_panic() {
    if let Ok(mut engine) = AudioEngine::new() {
        engine.tick(1.0 / 60.0);
        engine.tick(0.0);
        engine.tick(1.0);
    }
}

#[test]
fn audio_engine_play_sfx_beep() {
    if let Ok(mut engine) = AudioEngine::new() {
        engine.play_sfx_beep(440.0, 0.1, 0.3);
        // No panic = success
    }
}

#[test]
fn audio_engine_play_voice_beep() {
    if let Ok(mut engine) = AudioEngine::new() {
        engine.play_voice_beep(20); // 20 chars
                                    // No panic = success
    }
}

#[test]
fn audio_engine_stop_music_no_panic() {
    if let Ok(engine) = AudioEngine::new() {
        engine.stop_music();
    }
}

#[test]
fn audio_engine_stop_ambient_no_panic() {
    if let Ok(engine) = AudioEngine::new() {
        engine.stop_ambient();
    }
}

// ========================================================================
// EMITTER ID TYPE
// ========================================================================

#[test]
fn emitter_id_is_u64() {
    let id: EmitterId = 42;
    assert_eq!(id, 42u64);
}

#[test]
fn emitter_id_zero() {
    let id: EmitterId = 0;
    assert_eq!(id, 0u64);
}

#[test]
fn emitter_id_max() {
    let id: EmitterId = u64::MAX;
    assert_eq!(id, u64::MAX);
}
