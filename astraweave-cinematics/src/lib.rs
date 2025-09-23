use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Time(pub f32); // seconds

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Track {
    Camera { keyframes: Vec<CameraKey> },
    Animation { target: u32, clip: String, start: Time },
    Audio { clip: String, start: Time, volume: f32 },
    Fx { name: String, start: Time, params: serde_json::Value },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CameraKey {
    pub t: Time,
    pub pos: (f32, f32, f32),
    pub look_at: (f32, f32, f32),
    pub fov_deg: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Timeline {
    pub name: String,
    pub duration: Time,
    pub tracks: Vec<Track>,
}

impl Timeline {
    pub fn new(name: &str, duration: f32) -> Self {
        Self { name: name.into(), duration: Time(duration), tracks: Vec::new() }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum SeqError {
    #[error("timeline out of range: {0:?}")]
    Range(Time),
}

pub struct Sequencer {
    pub t: Time,
}

impl Sequencer {
    pub fn new() -> Self { Self { t: Time(0.0) } }
    pub fn seek(&mut self, t: Time) { self.t = t; }
    pub fn step(&mut self, dt: f32, tl: &Timeline) -> Result<Vec<SequencerEvent>, SeqError> {
        let next_t = Time(self.t.0 + dt);
        if next_t.0 > tl.duration.0 + 0.001 { return Err(SeqError::Range(next_t)); }
        let from = self.t.0; let to = next_t.0;
        self.t = next_t;
        // very simple: emit events whose start is in (from..=to]
        let mut evs = Vec::new();
        for tr in &tl.tracks {
            match tr {
                Track::Camera { keyframes } => {
                    for k in keyframes {
                        if k.t.0 > from && k.t.0 <= to {
                            evs.push(SequencerEvent::CameraKey(k.clone()));
                        }
                    }
                }
                Track::Animation { target, clip, start } => {
                    if start.0 > from && start.0 <= to {
                        evs.push(SequencerEvent::AnimStart { target: *target, clip: clip.clone() });
                    }
                }
                Track::Audio { clip, start, volume } => {
                    if start.0 > from && start.0 <= to {
                        evs.push(SequencerEvent::AudioPlay { clip: clip.clone(), volume: *volume });
                    }
                }
                Track::Fx { name, start, params } => {
                    if start.0 > from && start.0 <= to {
                        evs.push(SequencerEvent::FxTrigger { name: name.clone(), params: params.clone() });
                    }
                }
            }
        }
        Ok(evs)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum SequencerEvent {
    CameraKey(CameraKey),
    AnimStart { target: u32, clip: String },
    AudioPlay { clip: String, volume: f32 },
    FxTrigger { name: String, params: serde_json::Value },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seq_emits_events() {
        let mut tl = Timeline::new("demo", 3.0);
        tl.tracks.push(Track::Camera { keyframes: vec![CameraKey{ t: Time(1.0), pos:(0.0,1.0,2.0), look_at:(0.0,0.0,0.0), fov_deg:60.0 }]});
        tl.tracks.push(Track::Audio { clip: "boom".into(), start: Time(2.0), volume: 0.8 });
        let mut seq = Sequencer::new();
        let evs0 = seq.step(0.5, &tl).unwrap();
        assert!(evs0.is_empty());
        let evs1 = seq.step(0.6, &tl).unwrap();
        assert!(matches!(evs1[0], SequencerEvent::CameraKey(_)));
    let evs2 = seq.step(0.9, &tl).unwrap();
    assert!(matches!(evs2[0], SequencerEvent::AudioPlay { .. }));
    let evs3 = seq.step(1.0, &tl).unwrap();
    assert!(evs3.is_empty());
    }

    #[test]
    fn timeline_json_roundtrip() {
        let mut tl = Timeline::new("roundtrip", 2.0);
        tl.tracks.push(Track::Camera { keyframes: vec![
            CameraKey { t: Time(0.5), pos:(1.0,2.0,3.0), look_at:(0.0,1.0,0.0), fov_deg: 70.0 },
        ]});
        tl.tracks.push(Track::Fx { name: "fade-in".into(), start: Time(0.0), params: serde_json::json!({"duration": 0.25}) });
        let s = serde_json::to_string_pretty(&tl).unwrap();
        let de: Timeline = serde_json::from_str(&s).unwrap();
        assert_eq!(tl, de);
    }
}
