use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[cfg(feature = "mock_tts")]
use std::{f32::consts::PI, path::Path};
#[cfg(feature = "mock_tts")]
use hound::{SampleFormat, WavSpec, WavWriter};

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

/// A tiny, file-only mock TTS that generates a simple sine-wave WAV to the requested path.
/// Enabled with feature `mock_tts` to avoid shipping an extra dependency by default.
#[cfg(feature = "mock_tts")]
pub struct SimpleSineTts {
    pub sample_rate: u32,
    pub base_hz: f32,
}

#[cfg(feature = "mock_tts")]
impl Default for SimpleSineTts {
    fn default() -> Self {
        Self { sample_rate: 22_050, base_hz: 220.0 }
    }
}

#[cfg(feature = "mock_tts")]
impl TtsAdapter for SimpleSineTts {
    fn synth_to_path(&self, _voice_id: &str, text: &str, out_path: &str) -> Result<()> {
        // Derive duration from text length (very rough): 12 chars/sec
        let secs = (text.len() as f32 / 12.0).clamp(0.5, 8.0);
        let total_samples = (secs * self.sample_rate as f32) as u32;
        // Write a mono 16-bit WAV
        if let Some(dir) = Path::new(out_path).parent() {
            if !dir.exists() { std::fs::create_dir_all(dir)?; }
        }
        let spec = WavSpec { channels: 1, sample_rate: self.sample_rate, bits_per_sample: 16, sample_format: SampleFormat::Int };
        let mut writer = WavWriter::create(out_path, spec)?;
        let mut phase = 0.0f32;
        let dt = 1.0f32 / self.sample_rate as f32;
        let hz = self.base_hz;
        for i in 0..total_samples {
            // Amplitude envelope: attack/decay
            let t = i as f32 / total_samples as f32;
            let env = (t.min(0.2) / 0.2) * ((1.0 - t).clamp(0.0, 1.0));
            let s = (phase * 2.0 * PI).sin() * 0.3 * env;
            let sample = (s * i16::MAX as f32) as i16;
            writer.write_sample(sample)?;
            phase = (phase + hz * dt) % 1.0;
        }
        writer.finalize()?;
        Ok(())
    }
}

#[cfg(all(test, feature = "mock_tts"))]
mod tests {
    use super::*;

    #[test]
    fn simple_sine_tts_writes_file() -> Result<()> {
        let tts = SimpleSineTts::default();
        let out = "target/tmp_tts_mock.wav";
        let _ = std::fs::remove_file(out);
        tts.synth_to_path("voiceA", "Hello world", out)?;
        assert!(std::path::Path::new(out).exists());
        Ok(())
    }
}
