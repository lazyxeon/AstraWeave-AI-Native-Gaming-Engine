use anyhow::Result;
use astraweave_audio::{AudioEngine, DialoguePlayer, SimpleSineTts, VoiceBank};
use astraweave_gameplay::dialogue::{Choice, Dialogue, DialogueState, Line, Node};

fn main() -> Result<()> {
    // Construct a minimal dialogue in code
    let dlg = Dialogue {
        id: "cli_demo".into(),
        start: "n0".into(),
        nodes: vec![
            Node {
                id: "n0".into(),
                line: Some(Line {
                    speaker: "ECHO".into(),
                    text: "Hello, Traveler.".into(),
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
                    speaker: "ECHO".into(),
                    text: "The path ahead awaits.".into(),
                    set_vars: vec![],
                }),
                choices: vec![],
                end: true,
            },
        ],
    };
    let mut st = DialogueState::new(&dlg);

    // Audio engine and a voice bank with a TTS-capable speaker (mock)
    let mut audio = AudioEngine::new()?;
    audio.set_master_volume(1.0);
    let mut speakers = std::collections::HashMap::new();
    speakers.insert(
        "ECHO".to_string(),
        astraweave_audio::VoiceSpec {
            folder: "assets/voices/ECHO".into(),
            files: vec![],
            tts_voice: Some("echo_en".into()),
        },
    );
    let bank = VoiceBank { speakers };

    let mut subtitles = |speaker: String, text: String| {
        println!("{}: {}", speaker, text);
    };
    let mut player = DialoguePlayer {
        audio: &mut audio,
        bank: &bank,
        tts: Some(&SimpleSineTts::default()),
        overrides: None,
        subtitle_out: Some(&mut subtitles),
    };

    println!("-- Dialogue Audio CLI --");
    loop {
        player.speak_current(&dlg, &st)?;
        let node = st.current(&dlg);
        if node.end {
            break;
        }
        // move to next node (linear)
        st.choose(&dlg, 0); // if there were choices, we’d pick index here
    }
    println!("-- End --");
    Ok(())
}
