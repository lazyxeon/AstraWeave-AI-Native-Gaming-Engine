use std::fs;
use astraweave_gameplay::*;

fn main() -> anyhow::Result<()> {
    // Load dialogue
    let dlg_txt = fs::read_to_string("assets/dialogue_intro.toml")?;
    let dialogue: dialogue::Dialogue = toml::from_str(&dlg_txt)?;
    let mut dlg_state = dialogue::DialogueState::new(&dialogue);

    println!("-- Dialogue start --");
    loop {
        let node = dlg_state.current(&dialogue);
        if let Some(line) = &node.line {
            println!("{}: {}", line.speaker, line.text);
        }
        if node.end { break; }
        for (i,c) in node.choices.iter().enumerate() { println!("  [{}] {}", i, c.text); }
        // auto-pick first choice for demo:
        dlg_state.choose(&dialogue, 0);
    }

    // Quest
    let q_txt = fs::read_to_string("assets/quests_main.toml")?;
    let mut q: quests::Quest = toml::from_str(&q_txt)?;
    let mut log = quests::QuestLog::default();
    log.add(q.clone());
    println!("Quest added: {}", q.title);

    // Progress: gathered 2 crystals, then 1
    log.progress_gather("q_tutorial", "Crystal", 2);
    log.progress_gather("q_tutorial", "Crystal", 1);
    println!("Quest completed? {}", log.is_done("q_tutorial"));

    // Tiny cutscene
    let tl = cutscene::Timeline {
        cues: vec![
            cutscene::Cue::Title{ text: "Threads Awaken".into(), time: 1.5 },
            cutscene::Cue::Wait { time: 0.5 },
            cutscene::Cue::CameraTo { pos: glam::vec3(2.0, 3.0, 6.0), yaw: -1.57, pitch:-0.4, time: 2.0 },
        ]
    };
    let mut cs = cutscene::CutsceneState::new();
    let mut t = 0.0;
    while t < 4.0 {
        let (cam, title, done) = cs.tick(0.5, &tl);
        if let Some(txt) = title { println!("[Cutscene Title] {}", txt); }
        if let Some((pos,yaw,pitch)) = cam { println!("[Cutscene Camera] to {:?} yaw={:.2} pitch={:.2}", pos, yaw, pitch); }
        if done { break; }
        t += 0.5;
    }

    Ok(())
}

let banter = r#"
[Companion] Threads hum in the fog.
-> mood=curious
? mood == curious : goto n1
[Companion] Or maybe I'm just cold.
"#;
let dialog2 = dialogue::compile_banter_to_nodes("banter", banter);
let mut ds2 = dialogue::DialogueState::new(&dialog2);
println!("Banter start: {}", dialog2.current(&dialog2).line.as_ref().unwrap().text);
ds2.choose(&dialog2, 0);
