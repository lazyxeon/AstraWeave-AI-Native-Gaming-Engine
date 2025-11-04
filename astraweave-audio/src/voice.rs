use anyhow::Result;
use serde::Deserialize;
use std::{collections::HashMap, fs};

#[cfg(feature = "mock_tts")]
use hound::{SampleFormat, WavSpec, WavWriter};
#[cfg(feature = "mock_tts")]
use std::{f32::consts::PI, path::Path};

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
        Self {
            sample_rate: 22_050,
            base_hz: 220.0,
        }
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
            if !dir.exists() {
                std::fs::create_dir_all(dir)?;
            }
        }
        let spec = WavSpec {
            channels: 1,
            sample_rate: self.sample_rate,
            bits_per_sample: 16,
            sample_format: SampleFormat::Int,
        };
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

#[cfg(test)]
mod core_tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_voice_spec_creation() {
        let spec = VoiceSpec {
            folder: "assets/voices/TestSpeaker".to_string(),
            files: vec!["greeting.wav".to_string(), "farewell.wav".to_string()],
            tts_voice: Some("voice_1".to_string()),
        };
        assert_eq!(spec.folder, "assets/voices/TestSpeaker");
        assert_eq!(spec.files.len(), 2);
        assert_eq!(spec.tts_voice, Some("voice_1".to_string()));
    }

    #[test]
    fn test_voice_spec_no_tts() {
        let spec = VoiceSpec {
            folder: "assets/voices/NoTTS".to_string(),
            files: vec![],
            tts_voice: None,
        };
        assert!(spec.files.is_empty());
        assert!(spec.tts_voice.is_none());
    }

    #[test]
    fn test_voice_bank_creation() {
        let mut speakers = HashMap::new();
        speakers.insert(
            "Companion".to_string(),
            VoiceSpec {
                folder: "assets/voices/Companion".to_string(),
                files: vec!["line_01.wav".to_string()],
                tts_voice: None,
            },
        );
        speakers.insert(
            "Player".to_string(),
            VoiceSpec {
                folder: "assets/voices/Player".to_string(),
                files: vec![],
                tts_voice: Some("player_voice".to_string()),
            },
        );

        let bank = VoiceBank { speakers };
        assert_eq!(bank.speakers.len(), 2);
        assert!(bank.speakers.contains_key("Companion"));
        assert!(bank.speakers.contains_key("Player"));
    }

    #[test]
    fn test_voice_bank_empty() {
        let bank = VoiceBank {
            speakers: HashMap::new(),
        };
        assert_eq!(bank.speakers.len(), 0);
    }

    #[test]
    fn test_load_voice_bank_valid_toml() -> Result<()> {
        // Ensure target directory exists
        std::fs::create_dir_all("target")?;

        // Create a temporary TOML file
        let test_toml = "target/test_voices.toml";
        std::fs::write(
            test_toml,
            r#"
[speakers.TestSpeaker]
folder = "assets/voices/TestSpeaker"
files = ["line_01.wav", "line_02.wav"]
tts_voice = "test_voice"

[speakers.AnotherSpeaker]
folder = "assets/voices/Another"
"#,
        )?;

        let bank = load_voice_bank(test_toml)?;
        assert_eq!(bank.speakers.len(), 2);

        let test_spec = bank
            .speakers
            .get("TestSpeaker")
            .expect("TestSpeaker should exist");
        assert_eq!(test_spec.folder, "assets/voices/TestSpeaker");
        assert_eq!(test_spec.files.len(), 2);
        assert_eq!(test_spec.tts_voice, Some("test_voice".to_string()));

        let another_spec = bank
            .speakers
            .get("AnotherSpeaker")
            .expect("AnotherSpeaker should exist");
        assert_eq!(another_spec.folder, "assets/voices/Another");
        assert_eq!(another_spec.files.len(), 0); // Default empty
        assert!(another_spec.tts_voice.is_none()); // Default None

        std::fs::remove_file(test_toml)?;
        Ok(())
    }

    #[test]
    fn test_load_voice_bank_missing_file() {
        let result = load_voice_bank("nonexistent_file.toml");
        assert!(result.is_err(), "Should fail when file doesn't exist");
    }

    #[test]
    fn test_load_voice_bank_invalid_toml() -> Result<()> {
        // Ensure target directory exists
        std::fs::create_dir_all("target")?;

        let test_toml = "target/test_invalid.toml";
        std::fs::write(
            test_toml,
            r#"
[speakers.Broken
this is not valid TOML
"#,
        )?;

        let result = load_voice_bank(test_toml);
        assert!(result.is_err(), "Should fail when TOML is invalid");

        std::fs::remove_file(test_toml)?;
        Ok(())
    }
}
