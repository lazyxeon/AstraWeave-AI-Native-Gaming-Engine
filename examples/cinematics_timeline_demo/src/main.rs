use anyhow::Result;
use astraweave_cinematics::*;

fn main() -> Result<()> {
    let mut tl = Timeline::new("cutscene", 5.0);
    tl.tracks.push(Track::Camera {
        keyframes: vec![
            CameraKey {
                t: Time(1.0),
                pos: (0.0, 1.5, 3.0),
                look_at: (0.0, 1.0, 0.0),
                fov_deg: 60.0,
            },
            CameraKey {
                t: Time(3.0),
                pos: (2.0, 2.0, 3.0),
                look_at: (0.0, 1.0, 0.0),
                fov_deg: 55.0,
            },
        ],
    });
    tl.tracks.push(Track::Audio {
        clip: "music:start".into(),
        start: Time(0.2),
        volume: 0.7,
    });
    tl.tracks.push(Track::Fx {
        name: "fade-in".into(),
        start: Time(0.0),
        params: serde_json::json!({"duration": 0.5}),
    });

    let mut seq = Sequencer::new();
    let mut t = 0.0f32;
    while t < 5.0 {
        let evs = seq.step(0.5, &tl)?;
        for e in evs {
            println!("{:4.1}s: {:?}", seq.t.0, e);
        }
        t += 0.5;
    }
    Ok(())
}
