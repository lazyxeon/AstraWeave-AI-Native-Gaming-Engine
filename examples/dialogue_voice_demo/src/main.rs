use std::fs;
use astraweave_audio::{AudioEngine, MusicTrack, load_voice_bank, load_dialogue_audio_map, DialoguePlayer};
use astraweave_gameplay::dialogue::{Dialogue, DialogueState};

fn main() -> anyhow::Result<()> {
    let mut audio = AudioEngine::new()?;
    audio.set_master_volume(1.0);

    // soft bgm under VO (if present)
    let _ = audio.play_music(MusicTrack{ path: "assets/audio/bgm.ogg".into(), looped:true }, 0.8);

    // Load dialogue (we used this earlier in your repo)
    let dlg_txt = fs::read_to_string("assets/dialogue_intro.toml")?;
    let dlg: Dialogue = toml::from_str(&dlg_txt)?;
    let mut st = DialogueState::new(&dlg);

    // Load voice bank + optional per-node audio overrides
    let bank = load_voice_bank("assets/voices.toml")?;
    let overrides = load_dialogue_audio_map("assets/dialogue_audio_map.toml").ok();

    // Optional subtitles (stdout)
    let mut subtitles = |speaker: String, text: String| {
        println!("{}: {}", speaker, text);
    };

    let mut player = DialoguePlayer { audio: &mut audio, bank: &bank, tts: None, overrides: overrides.as_ref(), subtitle_out: Some(&mut subtitles) };

    println!("-- Dialogue (voice) --");
    loop {
        let _ = player.speak_current(&dlg, &st)?; // plays node line (VO or beep)
        let node = st.current(&dlg);
        if node.end { break; }
        // For demo, always pick first choice (you can add input here)
        st.choose(&dlg, 0);
    }
    println!("-- End --");
    Ok(())
}
