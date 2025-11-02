// Audio fixture generator for astraweave-audio integration tests
// 
// Generates 3 test audio files using synthetic sine waves:
// - music_test.ogg (5 sec, 440 Hz) - Music crossfade tests
// - sfx_test.wav (1 sec, 880 Hz) - SFX playback tests  
// - voice_test.wav (2 sec, 220 Hz) - Voice playback tests
//
// Usage: cargo test -p astraweave-audio --test generate_fixtures -- --ignored --nocapture

use std::f32::consts::PI;
use std::fs::File;
use std::io::Write;
use std::path::Path;

const SAMPLE_RATE: u32 = 44100;

fn generate_sine_wave(frequency: f32, duration_secs: f32) -> Vec<i16> {
    let num_samples = (duration_secs * SAMPLE_RATE as f32) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / SAMPLE_RATE as f32;
            let sample = (2.0 * PI * frequency * t).sin();
            (sample * 32767.0) as i16
        })
        .collect()
}

fn write_wav_file(path: &Path, samples: &[i16]) -> std::io::Result<()> {
    let mut file = File::create(path)?;
    let num_samples = samples.len() as u32;
    let byte_rate = SAMPLE_RATE * 2;
    let data_size = num_samples * 2;
    
    // RIFF header
    file.write_all(b"RIFF")?;
    file.write_all(&(36 + data_size).to_le_bytes())?;
    file.write_all(b"WAVE")?;
    
    // fmt chunk
    file.write_all(b"fmt ")?;
    file.write_all(&16u32.to_le_bytes())?;
    file.write_all(&1u16.to_le_bytes())?; // PCM
    file.write_all(&1u16.to_le_bytes())?; // mono
    file.write_all(&SAMPLE_RATE.to_le_bytes())?;
    file.write_all(&byte_rate.to_le_bytes())?;
    file.write_all(&2u16.to_le_bytes())?; // block align
    file.write_all(&16u16.to_le_bytes())?; // bits per sample
    
    // data chunk
    file.write_all(b"data")?;
    file.write_all(&data_size.to_le_bytes())?;
    
    for sample in samples {
        file.write_all(&sample.to_le_bytes())?;
    }
    
    Ok(())
}

#[test]
#[ignore]
fn generate_test_fixtures() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");
    
    std::fs::create_dir_all(&fixtures_dir).expect("Failed to create fixtures directory");
    
    println!("\n🎵 Generating audio test fixtures...");
    println!("   Directory: {}", fixtures_dir.display());
    
    // 1. music_test.ogg - 5 sec, 440 Hz (WAV with .ogg extension, rodio compatible)
    let music_path = fixtures_dir.join("music_test.ogg");
    print!("   [1/3] music_test.ogg (440 Hz, 5 sec)... ");
    let music_samples = generate_sine_wave(440.0, 5.0);
    write_wav_file(&music_path, &music_samples).expect("Failed to write music_test.ogg");
    let size = std::fs::metadata(&music_path).unwrap().len();
    println!("✅ {} bytes", size);
    
    // 2. sfx_test.wav - 1 sec, 880 Hz
    let sfx_path = fixtures_dir.join("sfx_test.wav");
    print!("   [2/3] sfx_test.wav (880 Hz, 1 sec)... ");
    let sfx_samples = generate_sine_wave(880.0, 1.0);
    write_wav_file(&sfx_path, &sfx_samples).expect("Failed to write sfx_test.wav");
    let size = std::fs::metadata(&sfx_path).unwrap().len();
    println!("✅ {} bytes", size);
    
    // 3. voice_test.wav - 2 sec, 220 Hz
    let voice_path = fixtures_dir.join("voice_test.wav");
    print!("   [3/3] voice_test.wav (220 Hz, 2 sec)... ");
    let voice_samples = generate_sine_wave(220.0, 2.0);
    write_wav_file(&voice_path, &voice_samples).expect("Failed to write voice_test.wav");
    let size = std::fs::metadata(&voice_path).unwrap().len();
    println!("✅ {} bytes", size);
    
    println!("\n✅ All fixtures generated successfully!");
    println!("   Total: 3 files, 8 seconds audio");
    println!("\n💡 Next: Run integration tests");
    println!("   cargo test -p astraweave-audio --test integration_tests -- --include-ignored\n");
}
