use std::{collections::HashMap, fs};
use serde::Deserialize;
use anyhow::Result;

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceSpec {
    /// Folder where voice assets live (e.g., "assets/voices/Companion")
    pub folder: String,
    /// Optional explicit file list for this speaker (filenames only)
    #[serde(default)]
    pub files: Vec<String>,
    /// Optional TTS voice id to use if file missing / variation needed
    #[serde(default)]
    pub tts_voice: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct VoiceBank {
    pub speakers: HashMap<String, VoiceSpec>,
}

pub fn load_voice_bank(path: &str) -> Result<VoiceBank> {
    let txt = fs::read_to_string(path)?;
    let bank: VoiceBank = toml::from_str(&txt)?;
    Ok(bank)
}

/// Adapter for pluggable TTS backends. Implement this for your engine of choice.
/// For now, we don’t ship an implementation (no external calls). You can wire a local
/// engine (e.g., onnx/cpp) or a cloud API here.
pub trait TtsAdapter: Send + Sync {
    /// Synthesize `text` with the given voice id into `out_path` (wav/ogg).
    fn synth_to_path(&self, voice_id: &str, text: &str, out_path: &str) -> Result<()>;
}
