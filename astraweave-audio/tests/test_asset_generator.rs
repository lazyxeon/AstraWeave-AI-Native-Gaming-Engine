//! Test asset generator for audio tests
//!
//! This module generates minimal test audio files on-the-fly for testing
//! file-based audio APIs without requiring checked-in binary assets.

use anyhow::Result;
use std::fs;
use std::path::Path;

/// Generate a simple sine wave WAV file for testing
pub fn generate_test_beep(
    path: &str,
    frequency: f32,
    duration_secs: f32,
    sample_rate: u32,
) -> Result<()> {
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::f32::consts::PI;

    // Create parent directory if needed
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)?;
    let num_samples = (duration_secs * sample_rate as f32) as u32;

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;
        let sample = (t * frequency * 2.0 * PI).sin();

        // Apply amplitude envelope (fade in/out to avoid clicks)
        let envelope = if t < 0.01 {
            t / 0.01 // Fade in over 10ms
        } else if t > duration_secs - 0.01 {
            (duration_secs - t) / 0.01 // Fade out over 10ms
        } else {
            1.0
        };

        let amplitude = sample * envelope * 0.5; // 50% volume to avoid clipping
        let sample_i16 = (amplitude * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    Ok(())
}

/// Generate a loopable music track (simple chord progression)
pub fn generate_test_music(path: &str, duration_secs: f32, sample_rate: u32) -> Result<()> {
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::f32::consts::PI;

    // Create parent directory if needed
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)?;
    let num_samples = (duration_secs * sample_rate as f32) as u32;

    // Simple C major chord (C4 = 261.63 Hz, E4 = 329.63 Hz, G4 = 392.00 Hz)
    let freqs = [261.63, 329.63, 392.00];

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Mix three sine waves (chord)
        let mut sample = 0.0;
        for freq in &freqs {
            sample += (t * freq * 2.0 * PI).sin();
        }
        sample /= freqs.len() as f32; // Normalize

        // Apply gentle amplitude modulation for "musical" feel
        let modulation = 0.8 + 0.2 * (t * 2.0 * PI / 2.0).sin();
        sample *= modulation;

        // Ensure seamless loop (fade in/out at boundaries)
        let loop_fade = 0.1; // 10% fade at start/end
        let fade_samples = (duration_secs * loop_fade * sample_rate as f32) as u32;
        let envelope = if i < fade_samples {
            i as f32 / fade_samples as f32
        } else if i > num_samples - fade_samples {
            (num_samples - i) as f32 / fade_samples as f32
        } else {
            1.0
        };

        let amplitude = sample * envelope * 0.3; // 30% volume for music
        let sample_i16 = (amplitude * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    Ok(())
}

/// Generate a short voice-like sound (formant synthesis approximation)
pub fn generate_test_voice(path: &str, duration_secs: f32, sample_rate: u32) -> Result<()> {
    use hound::{SampleFormat, WavSpec, WavWriter};
    use std::f32::consts::PI;

    // Create parent directory if needed
    if let Some(parent) = Path::new(path).parent() {
        fs::create_dir_all(parent)?;
    }

    let spec = WavSpec {
        channels: 1,
        sample_rate,
        bits_per_sample: 16,
        sample_format: SampleFormat::Int,
    };

    let mut writer = WavWriter::create(path, spec)?;
    let num_samples = (duration_secs * sample_rate as f32) as u32;

    // Simulate voice with fundamental + harmonics (very simplified formants)
    let f0 = 120.0; // Fundamental frequency (male voice range)
    let formants = [
        (800.0, 0.5),  // First formant (vowel)
        (1200.0, 0.3), // Second formant
        (2400.0, 0.1), // Third formant
    ];

    for i in 0..num_samples {
        let t = i as f32 / sample_rate as f32;

        // Fundamental tone
        let mut sample = (t * f0 * 2.0 * PI).sin();

        // Add formants
        for (freq, amplitude) in &formants {
            sample += (t * freq * 2.0 * PI).sin() * amplitude;
        }
        sample /= 2.0; // Normalize

        // Natural amplitude envelope (attack-decay-sustain-release)
        let attack_time = 0.05;
        let release_time = 0.05;
        let envelope = if t < attack_time {
            t / attack_time
        } else if t > duration_secs - release_time {
            (duration_secs - t) / release_time
        } else {
            1.0
        };

        let amplitude = sample * envelope * 0.4; // 40% volume
        let sample_i16 = (amplitude * i16::MAX as f32) as i16;
        writer.write_sample(sample_i16)?;
    }

    writer.finalize()?;
    Ok(())
}

/// Generate all test assets needed for comprehensive audio testing
pub fn setup_all_test_assets() -> Result<()> {
    // When running tests, cargo sets the working directory to the crate root
    // So "tests/assets" is correct for the astraweave-audio crate
    let assets_dir = "tests/assets";
    fs::create_dir_all(assets_dir)?;

    // Short beep for SFX testing (440 Hz "A" note, 0.5 seconds)
    generate_test_beep(
        &format!("{}/test_beep_440hz.wav", assets_dir),
        440.0,
        0.5,
        22050,
    )?;

    // Low frequency beep for bass testing (200 Hz, 0.5 seconds)
    generate_test_beep(
        &format!("{}/test_beep_200hz.wav", assets_dir),
        200.0,
        0.5,
        22050,
    )?;

    // High frequency beep (1000 Hz, 0.3 seconds)
    generate_test_beep(
        &format!("{}/test_beep_1000hz.wav", assets_dir),
        1000.0,
        0.3,
        22050,
    )?;

    // Very short beep for rapid-fire testing (0.1 seconds)
    generate_test_beep(
        &format!("{}/test_beep_short.wav", assets_dir),
        440.0,
        0.1,
        22050,
    )?;

    // Long beep for duration testing (3 seconds)
    generate_test_beep(
        &format!("{}/test_beep_long.wav", assets_dir),
        440.0,
        3.0,
        22050,
    )?;

    // Music track for crossfade testing (5 seconds, loopable)
    generate_test_music(&format!("{}/test_music_5sec.wav", assets_dir), 5.0, 22050)?;

    // Shorter music for quick crossfade tests (2 seconds)
    generate_test_music(&format!("{}/test_music_2sec.wav", assets_dir), 2.0, 22050)?;

    // Voice samples for dialogue testing
    generate_test_voice(&format!("{}/test_voice_short.wav", assets_dir), 1.0, 22050)?;
    generate_test_voice(&format!("{}/test_voice_medium.wav", assets_dir), 2.0, 22050)?;
    generate_test_voice(&format!("{}/test_voice_long.wav", assets_dir), 3.0, 22050)?;

    Ok(())
}

/// Clean up all generated test assets
pub fn cleanup_test_assets() -> Result<()> {
    let assets_dir = "tests/assets";
    if Path::new(assets_dir).exists() {
        fs::remove_dir_all(assets_dir)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_generate_beep() -> Result<()> {
        let path = "tests/assets/test_generator_beep.wav";
        generate_test_beep(path, 440.0, 0.5, 22050)?;

        assert!(Path::new(path).exists());

        // Verify file is valid WAV (basic check)
        let metadata = fs::metadata(path)?;
        assert!(
            metadata.len() > 44,
            "WAV file should be larger than header (44 bytes)"
        );

        // Expected size: 44 bytes header + (22050 samples/sec * 0.5 sec * 2 bytes/sample)
        let expected_data_size = 22050.0 * 0.5 * 2.0;
        let expected_total = 44.0 + expected_data_size;
        assert!(
            (metadata.len() as f32 - expected_total).abs() < 100.0,
            "File size should be close to expected"
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_generate_music() -> Result<()> {
        let path = "tests/assets/test_generator_music.wav";
        generate_test_music(path, 2.0, 22050)?;

        assert!(Path::new(path).exists());

        let metadata = fs::metadata(path)?;
        assert!(
            metadata.len() > 44,
            "Music file should be larger than header"
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_generate_voice() -> Result<()> {
        let path = "tests/assets/test_generator_voice.wav";
        generate_test_voice(path, 1.0, 22050)?;

        assert!(Path::new(path).exists());

        let metadata = fs::metadata(path)?;
        assert!(
            metadata.len() > 44,
            "Voice file should be larger than header"
        );

        Ok(())
    }

    #[test]
    #[serial]
    fn test_setup_all_assets() -> Result<()> {
        setup_all_test_assets()?;

        // Verify all expected files exist
        let expected_files = vec![
            "tests/assets/test_beep_440hz.wav",
            "tests/assets/test_beep_200hz.wav",
            "tests/assets/test_beep_1000hz.wav",
            "tests/assets/test_beep_short.wav",
            "tests/assets/test_beep_long.wav",
            "tests/assets/test_music_5sec.wav",
            "tests/assets/test_music_2sec.wav",
            "tests/assets/test_voice_short.wav",
            "tests/assets/test_voice_medium.wav",
            "tests/assets/test_voice_long.wav",
        ];

        for file in expected_files {
            assert!(Path::new(file).exists(), "Expected file {} to exist", file);
        }

        Ok(())
    }

    #[test]
    #[serial]
    fn test_cleanup_assets() -> Result<()> {
        setup_all_test_assets()?;
        cleanup_test_assets()?;

        assert!(
            !Path::new("tests/assets").exists(),
            "Assets directory should be removed"
        );

        Ok(())
    }
}
