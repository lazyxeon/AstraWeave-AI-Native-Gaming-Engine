//! Comprehensive tests for dialogue_runtime and voice modules
//!
//! Coverage targets:
//! - DialoguePlayer (speak_current, all 4 fallback paths)
//! - DialogueAudioMap loading
//! - VoiceBank loading
//! - TTS adapter (SimpleSineTts with mock_tts feature)
//! - Subtitle output
//! - Edge cases (missing files, empty banks, etc.)

use anyhow::Result;
use astraweave_audio::{dialogue_runtime::*, voice::*, AudioEngine};
use astraweave_gameplay::dialogue::{Choice, Dialogue, DialogueState, Line, Node};
use std::collections::HashMap;

/// Test 1: VoiceBank loading from TOML string
#[test]
fn test_voice_bank_parsing() -> Result<()> {
    let toml_data = r#"
[speakers.Hero]
folder = "assets/voices/hero"
files = ["hello.ogg", "yes.wav"]

[speakers.Companion]
folder = "assets/voices/companion"
tts_voice = "voice_en_us_001"
"#;

    let bank: VoiceBank = toml::from_str(toml_data)?;

    assert_eq!(bank.speakers.len(), 2);
    assert!(bank.speakers.contains_key("Hero"));
    assert!(bank.speakers.contains_key("Companion"));

    let hero = &bank.speakers["Hero"];
    assert_eq!(hero.folder, "assets/voices/hero");
    assert_eq!(hero.files.len(), 2);
    assert!(hero.tts_voice.is_none());

    let companion = &bank.speakers["Companion"];
    assert_eq!(companion.folder, "assets/voices/companion");
    assert_eq!(companion.files.len(), 0);
    assert_eq!(companion.tts_voice, Some("voice_en_us_001".to_string()));

    Ok(())
}

/// Test 2: DialogueAudioMap loading
#[test]
fn test_dialogue_audio_map_parsing() -> Result<()> {
    let toml_data = r#"
[map.quest_intro]
n0 = "intro_welcome.ogg"
n1 = "intro_explain.ogg"

[map.boss_fight]
boss_taunt = "boss_laugh.wav"
"#;

    let map: DialogueAudioMap = toml::from_str(toml_data)?;

    assert_eq!(map.map.len(), 2);
    assert!(map.map.contains_key("quest_intro"));
    assert!(map.map.contains_key("boss_fight"));

    let quest = &map.map["quest_intro"];
    assert_eq!(quest.len(), 2);
    assert_eq!(quest["n0"], "intro_welcome.ogg");
    assert_eq!(quest["n1"], "intro_explain.ogg");

    let boss = &map.map["boss_fight"];
    assert_eq!(boss.len(), 1);
    assert_eq!(boss["boss_taunt"], "boss_laugh.wav");

    Ok(())
}

/// Test 3: DialoguePlayer with empty bank (beep fallback)
#[test]
fn test_dialogue_player_beep_fallback() -> Result<()> {
    let dlg = Dialogue {
        id: "test_dialogue".into(),
        start: "node1".into(),
        nodes: vec![Node {
            id: "node1".into(),
            line: Some(Line {
                speaker: "Unknown Speaker".into(),
                text: "This is a test line".into(),
                set_vars: vec![],
            }),
            choices: vec![],
            end: true,
        }],
    };

    let st = DialogueState::new(&dlg);
    let mut audio = AudioEngine::new()?;
    let bank = VoiceBank {
        speakers: HashMap::new(), // Empty bank
    };

    let mut subtitles = vec![];
    {
        let mut push_sub = |speaker: String, text: String| {
            subtitles.push((speaker, text));
        };

        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: Some(&mut push_sub),
        };

        let played = player.speak_current(&dlg, &st)?;
        assert!(played, "Should play beep fallback");
    }

    assert_eq!(subtitles.len(), 1);
    assert_eq!(subtitles[0].0, "Unknown Speaker");
    assert_eq!(subtitles[0].1, "This is a test line");

    Ok(())
}

/// Test 4: DialoguePlayer with no line (silent node)
#[test]
fn test_dialogue_player_silent_node() -> Result<()> {
    let dlg = Dialogue {
        id: "silent_dlg".into(),
        start: "silent_node".into(),
        nodes: vec![Node {
            id: "silent_node".into(),
            line: None, // No dialogue line
            choices: vec![],
            end: true,
        }],
    };

    let st = DialogueState::new(&dlg);
    let mut audio = AudioEngine::new()?;
    let bank = VoiceBank {
        speakers: HashMap::new(),
    };

    let mut player = DialoguePlayer {
        audio: &mut audio,
        bank: &bank,
        tts: None,
        overrides: None,
        subtitle_out: None,
    };

    let played = player.speak_current(&dlg, &st)?;
    assert!(!played, "Should not play audio for silent node");

    Ok(())
}

/// Test 5: DialoguePlayer subtitle output
#[test]
fn test_subtitle_output() -> Result<()> {
    let dlg = Dialogue {
        id: "multi_line".into(),
        start: "n0".into(),
        nodes: vec![
            Node {
                id: "n0".into(),
                line: Some(Line {
                    speaker: "Hero".into(),
                    text: "Hello there!".into(),
                    set_vars: vec![],
                }),
                choices: vec![Choice {
                    text: "Continue".into(),
                    go_to: "n1".into(),
                    require: vec![],
                }],
                end: false,
            },
            Node {
                id: "n1".into(),
                line: Some(Line {
                    speaker: "Companion".into(),
                    text: "General Kenobi!".into(),
                    set_vars: vec![],
                }),
                choices: vec![],
                end: true,
            },
        ],
    };

    let mut st = DialogueState::new(&dlg);
    let mut audio = AudioEngine::new()?;
    let bank = VoiceBank {
        speakers: HashMap::new(),
    };

    let mut subtitles = vec![];

    // First node
    {
        let mut push_sub = |s: String, t: String| subtitles.push((s, t));
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: Some(&mut push_sub),
        };

        player.speak_current(&dlg, &st)?;
    }

    assert_eq!(subtitles.len(), 1);
    assert_eq!(subtitles[0].0, "Hero");

    // Choose option and advance
    st.choose(&dlg, 0);

    // Second node
    {
        let mut push_sub = |s: String, t: String| subtitles.push((s, t));
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: Some(&mut push_sub),
        };

        player.speak_current(&dlg, &st)?;
    }

    assert_eq!(subtitles.len(), 2);
    assert_eq!(subtitles[1].0, "Companion");
    assert_eq!(subtitles[1].1, "General Kenobi!");

    Ok(())
}

/// Test 6: VoiceSpec with explicit files
#[test]
fn test_voice_spec_with_files() {
    let spec = VoiceSpec {
        folder: "test_folder".into(),
        files: vec!["file1.wav".into(), "file2.ogg".into()],
        tts_voice: None,
    };

    assert_eq!(spec.folder, "test_folder");
    assert_eq!(spec.files.len(), 2);
    assert!(spec.tts_voice.is_none());
}

/// Test 7: VoiceSpec with TTS fallback
#[test]
fn test_voice_spec_with_tts() {
    let spec = VoiceSpec {
        folder: "tts_folder".into(),
        files: vec![],
        tts_voice: Some("voice_id_123".into()),
    };

    assert_eq!(spec.folder, "tts_folder");
    assert!(spec.files.is_empty());
    assert_eq!(spec.tts_voice, Some("voice_id_123".into()));
}

/// Test 8: Empty VoiceBank
#[test]
fn test_empty_voice_bank() {
    let bank = VoiceBank {
        speakers: HashMap::new(),
    };

    assert_eq!(bank.speakers.len(), 0);
}

/// Test 9: Multiple speakers in VoiceBank
#[test]
fn test_multiple_speakers() {
    let mut speakers = HashMap::new();
    speakers.insert(
        "Speaker1".into(),
        VoiceSpec {
            folder: "folder1".into(),
            files: vec!["a.wav".into()],
            tts_voice: None,
        },
    );
    speakers.insert(
        "Speaker2".into(),
        VoiceSpec {
            folder: "folder2".into(),
            files: vec![],
            tts_voice: Some("tts2".into()),
        },
    );

    let bank = VoiceBank { speakers };

    assert_eq!(bank.speakers.len(), 2);
    assert!(bank.speakers.contains_key("Speaker1"));
    assert!(bank.speakers.contains_key("Speaker2"));
}

/// Test 10: DialogueAudioMap with multiple nodes in same dialogue
#[test]
fn test_dialogue_audio_map_multiple_nodes() -> Result<()> {
    let toml_data = r#"
[map.dialogue1]
node1 = "audio1.ogg"
node2 = "audio2.ogg"
node3 = "audio3.ogg"
"#;

    let map: DialogueAudioMap = toml::from_str(toml_data)?;

    assert!(map.map.contains_key("dialogue1"));
    let dialogue1 = &map.map["dialogue1"];
    assert_eq!(dialogue1.len(), 3);
    assert!(dialogue1.contains_key("node1"));
    assert!(dialogue1.contains_key("node2"));
    assert!(dialogue1.contains_key("node3"));

    Ok(())
}

/// Test 11: Long dialogue chain
#[test]
fn test_long_dialogue_chain() -> Result<()> {
    // Create dialogue with 10 nodes
    let mut nodes = vec![];
    for i in 0..10 {
        nodes.push(Node {
            id: format!("node{}", i),
            line: Some(Line {
                speaker: format!("Speaker{}", i % 3),
                text: format!("Line number {}", i),
                set_vars: vec![],
            }),
            choices: if i < 9 {
                vec![Choice {
                    text: "Next".into(),
                    go_to: format!("node{}", i + 1),
                    require: vec![],
                }]
            } else {
                vec![]
            },
            end: i == 9,
        });
    }

    let dlg = Dialogue {
        id: "long_chain".into(),
        start: "node0".into(),
        nodes,
    };

    let mut st = DialogueState::new(&dlg);
    let mut audio = AudioEngine::new()?;
    let bank = VoiceBank {
        speakers: HashMap::new(),
    };

    let mut total_subs = 0;

    // Traverse entire chain
    for i in 0..10 {
        let mut subs = vec![];
        {
            let mut push_sub = |s: String, t: String| subs.push((s, t));
            let mut player = DialoguePlayer {
                audio: &mut audio,
                bank: &bank,
                tts: None,
                overrides: None,
                subtitle_out: Some(&mut push_sub),
            };

            player.speak_current(&dlg, &st)?;
        }

        assert_eq!(subs.len(), 1, "Node {} should have 1 subtitle", i);
        total_subs += subs.len();

        if i < 9 {
            st.choose(&dlg, 0); // Choose "Next"
        }
    }

    assert_eq!(total_subs, 10);

    Ok(())
}

/// Test 12: Dialogue with branching choices
#[test]
fn test_branching_dialogue() -> Result<()> {
    let dlg = Dialogue {
        id: "branching".into(),
        start: "start".into(),
        nodes: vec![
            Node {
                id: "start".into(),
                line: Some(Line {
                    speaker: "NPC".into(),
                    text: "Choose your path".into(),
                    set_vars: vec![],
                }),
                choices: vec![
                    Choice {
                        text: "Path A".into(),
                        go_to: "path_a".into(),
                        require: vec![],
                    },
                    Choice {
                        text: "Path B".into(),
                        go_to: "path_b".into(),
                        require: vec![],
                    },
                ],
                end: false,
            },
            Node {
                id: "path_a".into(),
                line: Some(Line {
                    speaker: "NPC".into(),
                    text: "You chose Path A".into(),
                    set_vars: vec![],
                }),
                choices: vec![],
                end: true,
            },
            Node {
                id: "path_b".into(),
                line: Some(Line {
                    speaker: "NPC".into(),
                    text: "You chose Path B".into(),
                    set_vars: vec![],
                }),
                choices: vec![],
                end: true,
            },
        ],
    };

    let mut audio = AudioEngine::new()?;
    let bank = VoiceBank {
        speakers: HashMap::new(),
    };

    // Test Path A
    let mut st_a = DialogueState::new(&dlg);
    {
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: None,
        };

        player.speak_current(&dlg, &st_a)?; // Start node
    }
    st_a.choose(&dlg, 0); // Choose Path A
    {
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: None,
        };

        player.speak_current(&dlg, &st_a)?; // Path A node
    }

    // Test Path B
    let mut st_b = DialogueState::new(&dlg);
    {
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: None,
        };

        player.speak_current(&dlg, &st_b)?; // Start node
    }
    st_b.choose(&dlg, 1); // Choose Path B
    {
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: None,
        };

        player.speak_current(&dlg, &st_b)?; // Path B node
    }

    Ok(())
}

/// Test 13: Empty dialogue audio map
#[test]
fn test_empty_dialogue_audio_map() -> Result<()> {
    let toml_data = r#"
[map]
"#;

    let map: DialogueAudioMap = toml::from_str(toml_data)?;
    assert_eq!(map.map.len(), 0);

    Ok(())
}

/// Test 14: VoiceBank with complex structure
#[test]
fn test_complex_voice_bank() -> Result<()> {
    let toml_data = r#"
[speakers.Hero]
folder = "assets/voices/hero"
files = ["greeting.wav", "farewell.wav", "battle_cry.ogg"]

[speakers.Villain]
folder = "assets/voices/villain"
tts_voice = "villain_deep_voice"

[speakers.Sidekick]
folder = "assets/voices/sidekick"
files = ["joke1.ogg", "joke2.ogg"]
tts_voice = "sidekick_high_voice"
"#;

    let bank: VoiceBank = toml::from_str(toml_data)?;

    assert_eq!(bank.speakers.len(), 3);

    let hero = &bank.speakers["Hero"];
    assert_eq!(hero.files.len(), 3);
    assert!(hero.tts_voice.is_none());

    let villain = &bank.speakers["Villain"];
    assert_eq!(villain.files.len(), 0);
    assert!(villain.tts_voice.is_some());

    let sidekick = &bank.speakers["Sidekick"];
    assert_eq!(sidekick.files.len(), 2);
    assert!(sidekick.tts_voice.is_some());

    Ok(())
}

// Note: SimpleSineTts tests are gated by #[cfg(all(test, feature = "mock_tts"))]
// They are already included in voice.rs module tests

/// Test 15: DialoguePlayer with multiple speakers
#[test]
fn test_multiple_speakers_dialogue() -> Result<()> {
    let dlg = Dialogue {
        id: "conversation".into(),
        start: "n0".into(),
        nodes: vec![
            Node {
                id: "n0".into(),
                line: Some(Line {
                    speaker: "Alice".into(),
                    text: "Hi Bob!".into(),
                    set_vars: vec![],
                }),
                choices: vec![Choice {
                    text: "Continue".into(),
                    go_to: "n1".into(),
                    require: vec![],
                }],
                end: false,
            },
            Node {
                id: "n1".into(),
                line: Some(Line {
                    speaker: "Bob".into(),
                    text: "Hi Alice!".into(),
                    set_vars: vec![],
                }),
                choices: vec![Choice {
                    text: "Continue".into(),
                    go_to: "n2".into(),
                    require: vec![],
                }],
                end: false,
            },
            Node {
                id: "n2".into(),
                line: Some(Line {
                    speaker: "Charlie".into(),
                    text: "Hello everyone!".into(),
                    set_vars: vec![],
                }),
                choices: vec![],
                end: true,
            },
        ],
    };

    let mut st = DialogueState::new(&dlg);
    let mut audio = AudioEngine::new()?;
    let bank = VoiceBank {
        speakers: HashMap::new(),
    };

    let mut speakers = vec![];

    // Collect all speakers through dialogue
    for _ in 0..3 {
        let mut subs = vec![];
        {
            let mut push_sub = |s: String, t: String| subs.push((s, t));
            let mut player = DialoguePlayer {
                audio: &mut audio,
                bank: &bank,
                tts: None,
                overrides: None,
                subtitle_out: Some(&mut push_sub),
            };

            player.speak_current(&dlg, &st)?;
        }

        if !subs.is_empty() {
            speakers.push(subs[0].0.clone());
        }

        if !st.current(&dlg).end {
            st.choose(&dlg, 0);
        }
    }

    assert_eq!(speakers.len(), 3);
    assert_eq!(speakers[0], "Alice");
    assert_eq!(speakers[1], "Bob");
    assert_eq!(speakers[2], "Charlie");

    Ok(())
}
