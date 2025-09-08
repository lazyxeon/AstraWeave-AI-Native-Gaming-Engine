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
                        let path = pool.choose(&mut rng).unwrap().clone();
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
