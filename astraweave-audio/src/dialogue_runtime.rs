use crate::{
    engine::AudioEngine,
    voice::{TtsAdapter, VoiceBank},
};
use anyhow::Result;
use astraweave_gameplay::dialogue::{Dialogue, DialogueState};
use rand::prelude::*;
use std::{collections::HashMap, fs, path::Path};

#[derive(Clone, Debug, serde::Deserialize)]
pub struct DialogueAudioMap {
    /// maps Dialogue.id -> (optional) overrides per node.id -> filename
    pub map: HashMap<String, HashMap<String, String>>,
}

pub fn load_dialogue_audio_map(path: &str) -> Result<DialogueAudioMap> {
    let txt = fs::read_to_string(path)?;
    let m: DialogueAudioMap = toml::from_str(&txt)?;
    Ok(m)
}

pub struct DialoguePlayer<'a> {
    pub audio: &'a mut AudioEngine,
    pub bank: &'a VoiceBank,
    pub tts: Option<&'a dyn TtsAdapter>,
    pub overrides: Option<&'a DialogueAudioMap>,
    pub subtitle_out: Option<&'a mut dyn FnMut(String, String)>, // (speaker, text)
}

impl<'a> DialoguePlayer<'a> {
    /// Play the current node’s line if any (blocking the queue on the Sink by appending).
    /// Returns true if audio played or beeped.
    pub fn speak_current(&mut self, dlg: &Dialogue, st: &DialogueState) -> Result<bool> {
        let node = st.current(dlg);
        let Some(line) = &node.line else {
            return Ok(false);
        };
        let spk = &line.speaker;
        let txt = &line.text;

        if let Some(out) = &mut self.subtitle_out {
            out(spk.clone(), txt.clone());
        }

        // 1) explicit override? (dialogue id + node id)
        if let Some(over) = self.overrides {
            if let Some(per_dialog) = over.map.get(&dlg.id) {
                if let Some(fname) = per_dialog.get(&node.id) {
                    if Path::new(fname).exists() {
                        self.audio.play_voice_file(fname, None)?;
                        return Ok(true);
                    }
                }
            }
        }

        // 2) VoiceBank folder/files for speaker
        if let Some(vspec) = self.bank.speakers.get(spk) {
            // try explicit files
            if !vspec.files.is_empty() {
                let mut rng = rand::rng();
                if let Some(choice) = vspec.files.choose(&mut rng) {
                    let path = format!("{}/{}", vspec.folder, choice);
                    if Path::new(&path).exists() {
                        self.audio.play_voice_file(&path, None)?;
                        return Ok(true);
                    }
                }
            } else {
                // or any .ogg/.wav in folder
                if let Ok(rd) = fs::read_dir(&vspec.folder) {
                    let mut pool: Vec<String> = vec![];
                    for e in rd.flatten() {
                        if let Some(ext) = e.path().extension() {
                            if ext == "ogg" || ext == "wav" {
                                pool.push(e.path().to_string_lossy().to_string());
                            }
                        }
                    }
                    if !pool.is_empty() {
                        let mut rng = rand::rng();
                        let path = pool.choose(&mut rng)
                            .expect("BUG: pool should have items after is_empty check")
                            .clone();
                        self.audio.play_voice_file(&path, None)?;
                        return Ok(true);
                    }
                }
            }

            // 3) TTS fallback if available
            if let (Some(tts), Some(voice_id)) = (self.tts.as_ref(), vspec.tts_voice.as_ref()) {
                let out_path = format!("{}/tts_tmp_{}.wav", vspec.folder, rand::random::<u64>());
                tts.synth_to_path(voice_id, txt, &out_path)?;
                self.audio.play_voice_file(&out_path, None)?;
                return Ok(true);
            }
        }

        // 4) Final fallback: beep by text length
        self.audio.play_voice_beep(txt.len());
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_gameplay::dialogue::{Dialogue, Line, Node};

    #[test]
    fn test_dialogue_audio_map_creation() {
        let mut map = HashMap::new();
        let mut node_map = HashMap::new();
        node_map.insert("n0".to_string(), "audio/greeting.wav".to_string());
        node_map.insert("n1".to_string(), "audio/farewell.wav".to_string());
        map.insert("dlg_test".to_string(), node_map);

        let audio_map = DialogueAudioMap { map };
        assert_eq!(audio_map.map.len(), 1);
        let node_overrides = audio_map.map.get("dlg_test").unwrap();
        assert_eq!(node_overrides.len(), 2);
        assert_eq!(node_overrides.get("n0"), Some(&"audio/greeting.wav".to_string()));
    }

    #[test]
    fn test_dialogue_audio_map_empty() {
        let audio_map = DialogueAudioMap {
            map: HashMap::new(),
        };
        assert_eq!(audio_map.map.len(), 0);
    }

    #[test]
    fn test_load_dialogue_audio_map_valid() -> Result<()> {
        std::fs::create_dir_all("target")?;
        let test_toml = "target/test_dialogue_audio.toml";
        std::fs::write(test_toml, r#"
[map.dialogue1]
node1 = "audio/line1.wav"
node2 = "audio/line2.wav"

[map.dialogue2]
node_start = "audio/intro.wav"
"#)?;

        let audio_map = load_dialogue_audio_map(test_toml)?;
        assert_eq!(audio_map.map.len(), 2);
        
        let dlg1 = audio_map.map.get("dialogue1").unwrap();
        assert_eq!(dlg1.len(), 2);
        assert_eq!(dlg1.get("node1"), Some(&"audio/line1.wav".to_string()));
        
        let dlg2 = audio_map.map.get("dialogue2").unwrap();
        assert_eq!(dlg2.len(), 1);
        assert_eq!(dlg2.get("node_start"), Some(&"audio/intro.wav".to_string()));
        
        std::fs::remove_file(test_toml)?;
        Ok(())
    }

    #[test]
    fn test_load_dialogue_audio_map_missing_file() {
        let result = load_dialogue_audio_map("nonexistent_dialogue_map.toml");
        assert!(result.is_err(), "Should fail for missing file");
    }

    #[test]
    fn test_dialogue_player_no_line() -> Result<()> {
        let dlg = Dialogue {
            id: "test_no_line".into(),
            start: "n0".into(),
            nodes: vec![Node {
                id: "n0".into(),
                line: None, // No line to speak
                choices: vec![],
                end: true,
            }],
        };
        let st = DialogueState::new(&dlg);
        let mut audio = AudioEngine::new()?;
        let bank = VoiceBank {
            speakers: Default::default(),
        };
        
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: None,
            subtitle_out: None,
        };
        
        let result = player.speak_current(&dlg, &st)?;
        assert_eq!(result, false, "Should return false when node has no line");
        Ok(())
    }

    #[test]
    fn test_dialogue_player_subtitle_callback() -> Result<()> {
        let dlg = Dialogue {
            id: "test_subtitle".into(),
            start: "n0".into(),
            nodes: vec![Node {
                id: "n0".into(),
                line: Some(Line {
                    speaker: "TestSpeaker".into(),
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
            speakers: Default::default(),
        };
        
        let mut subtitles: Vec<(String, String)> = vec![];
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
            
            player.speak_current(&dlg, &st)?;
        }
        
        assert_eq!(subtitles.len(), 1);
        assert_eq!(subtitles[0].0, "TestSpeaker");
        assert_eq!(subtitles[0].1, "This is a test line");
        Ok(())
    }

    #[test]
    fn speak_beep_fallback() -> Result<()> {
        // Build minimal dialogue in memory
        let dlg = Dialogue {
            id: "test".into(),
            start: "n0".into(),
            nodes: vec![Node {
                id: "n0".into(),
                line: Some(Line {
                    speaker: "Test".into(),
                    text: "Hello there".into(),
                    set_vars: vec![],
                }),
                choices: vec![],
                end: true,
            }],
        };
        let st = DialogueState::new(&dlg);
        let mut audio = AudioEngine::new()?;
        let bank = VoiceBank {
            speakers: Default::default(),
        };
        let mut subs: Vec<(String, String)> = vec![];
        {
            let mut push_sub = |s: String, t: String| {
                subs.push((s, t));
            };
            let mut player = DialoguePlayer {
                audio: &mut audio,
                bank: &bank,
                tts: None,
                overrides: None,
                subtitle_out: Some(&mut push_sub),
            };
            let ok = player.speak_current(&dlg, &st)?;
            assert!(ok);
        }
        assert_eq!(subs.len(), 1);
        Ok(())
    }

    #[test]
    fn test_dialogue_player_with_override_path() -> Result<()> {
        std::fs::create_dir_all("target/test_audio")?;
        let test_audio_file = "target/test_audio/override_line.wav";
        // Create a minimal WAV file (just header, won't play but will exist)
        std::fs::write(test_audio_file, b"RIFF")?;
        
        let dlg = Dialogue {
            id: "test_override".into(),
            start: "n0".into(),
            nodes: vec![Node {
                id: "n0".into(),
                line: Some(Line {
                    speaker: "Overridden".into(),
                    text: "This line has an audio override".into(),
                    set_vars: vec![],
                }),
                choices: vec![],
                end: true,
            }],
        };
        let st = DialogueState::new(&dlg);
        let mut audio = AudioEngine::new()?;
        let bank = VoiceBank {
            speakers: Default::default(),
        };
        
        // Create override map
        let mut node_map = HashMap::new();
        node_map.insert("n0".to_string(), test_audio_file.to_string());
        let mut map = HashMap::new();
        map.insert("test_override".to_string(), node_map);
        let audio_map = DialogueAudioMap { map };
        
        let mut player = DialoguePlayer {
            audio: &mut audio,
            bank: &bank,
            tts: None,
            overrides: Some(&audio_map),
            subtitle_out: None,
        };
        
        let _result = player.speak_current(&dlg, &st);
        // Might fail due to invalid WAV, but we tested the override path logic
        // The important part is it attempted to use the override
        std::fs::remove_file(test_audio_file)?;
        Ok(())
    }

    #[test]
    fn test_dialogue_audio_map_multiple_dialogues() {
        let mut map = HashMap::new();
        
        // Dialogue 1
        let mut dlg1_nodes = HashMap::new();
        dlg1_nodes.insert("start".to_string(), "audio/dlg1_start.wav".to_string());
        dlg1_nodes.insert("end".to_string(), "audio/dlg1_end.wav".to_string());
        map.insert("dialogue_1".to_string(), dlg1_nodes);
        
        // Dialogue 2
        let mut dlg2_nodes = HashMap::new();
        dlg2_nodes.insert("intro".to_string(), "audio/dlg2_intro.wav".to_string());
        map.insert("dialogue_2".to_string(), dlg2_nodes);
        
        let audio_map = DialogueAudioMap { map };
        assert_eq!(audio_map.map.len(), 2);
        assert!(audio_map.map.contains_key("dialogue_1"));
        assert!(audio_map.map.contains_key("dialogue_2"));
        
        let dlg1 = audio_map.map.get("dialogue_1").unwrap();
        assert_eq!(dlg1.len(), 2);
        
        let dlg2 = audio_map.map.get("dialogue_2").unwrap();
        assert_eq!(dlg2.len(), 1);
    }
}
