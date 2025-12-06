use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq)]
pub struct Time(pub f32); // seconds

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum Track {
    Camera {
        keyframes: Vec<CameraKey>,
    },
    Animation {
        target: u32,
        clip: String,
        start: Time,
    },
    Audio {
        clip: String,
        start: Time,
        volume: f32,
    },
    Fx {
        name: String,
        start: Time,
        params: serde_json::Value,
    },
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
        Self {
            name: name.into(),
            duration: Time(duration),
            tracks: Vec::new(),
        }
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

impl Default for Sequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl Sequencer {
    pub fn new() -> Self {
        Self { t: Time(0.0) }
    }
    pub fn seek(&mut self, t: Time) {
        self.t = t;
    }
    pub fn step(&mut self, dt: f32, tl: &Timeline) -> Result<Vec<SequencerEvent>, SeqError> {
        let next_t = Time(self.t.0 + dt);
        if next_t.0 > tl.duration.0 + 0.001 {
            return Err(SeqError::Range(next_t));
        }
        let from = self.t.0;
        let to = next_t.0;
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
                Track::Animation {
                    target,
                    clip,
                    start,
                } => {
                    if start.0 > from && start.0 <= to {
                        evs.push(SequencerEvent::AnimStart {
                            target: *target,
                            clip: clip.clone(),
                        });
                    }
                }
                Track::Audio {
                    clip,
                    start,
                    volume,
                } => {
                    if start.0 > from && start.0 <= to {
                        evs.push(SequencerEvent::AudioPlay {
                            clip: clip.clone(),
                            volume: *volume,
                        });
                    }
                }
                Track::Fx {
                    name,
                    start,
                    params,
                } => {
                    if start.0 > from && start.0 <= to {
                        evs.push(SequencerEvent::FxTrigger {
                            name: name.clone(),
                            params: params.clone(),
                        });
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
    AnimStart {
        target: u32,
        clip: String,
    },
    AudioPlay {
        clip: String,
        volume: f32,
    },
    FxTrigger {
        name: String,
        params: serde_json::Value,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seq_emits_events() {
        let mut tl = Timeline::new("demo", 3.0);
        tl.tracks.push(Track::Camera {
            keyframes: vec![CameraKey {
                t: Time(1.0),
                pos: (0.0, 1.0, 2.0),
                look_at: (0.0, 0.0, 0.0),
                fov_deg: 60.0,
            }],
        });
        tl.tracks.push(Track::Audio {
            clip: "boom".into(),
            start: Time(2.0),
            volume: 0.8,
        });
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
        tl.tracks.push(Track::Camera {
            keyframes: vec![CameraKey {
                t: Time(0.5),
                pos: (1.0, 2.0, 3.0),
                look_at: (0.0, 1.0, 0.0),
                fov_deg: 70.0,
            }],
        });
        tl.tracks.push(Track::Fx {
            name: "fade-in".into(),
            start: Time(0.0),
            params: serde_json::json!({"duration": 0.25}),
        });
        let s = serde_json::to_string_pretty(&tl).unwrap();
        let de: Timeline = serde_json::from_str(&s).unwrap();
        assert_eq!(tl, de);
    }

    #[test]
    fn sequencer_default() {
        // Test Default implementation
        let seq: Sequencer = Default::default();
        assert_eq!(seq.t.0, 0.0);
    }

    #[test]
    fn sequencer_seek() {
        let mut seq = Sequencer::new();
        assert_eq!(seq.t.0, 0.0);
        
        seq.seek(Time(5.5));
        assert_eq!(seq.t.0, 5.5);
        
        seq.seek(Time(0.0));
        assert_eq!(seq.t.0, 0.0);
    }

    #[test]
    fn sequencer_out_of_range_error() {
        let tl = Timeline::new("short", 1.0);
        let mut seq = Sequencer::new();
        
        // Step past the end
        let result = seq.step(2.0, &tl);
        assert!(result.is_err());
        
        // Check error message
        let err = result.unwrap_err();
        let msg = format!("{}", err);
        assert!(msg.contains("out of range"));
    }

    #[test]
    fn seq_emits_animation_events() {
        let mut tl = Timeline::new("anim_test", 3.0);
        tl.tracks.push(Track::Animation {
            target: 42,
            clip: "walk_cycle".into(),
            start: Time(1.0),
        });
        
        let mut seq = Sequencer::new();
        
        // Before animation start
        let evs0 = seq.step(0.5, &tl).unwrap();
        assert!(evs0.is_empty());
        
        // At animation start
        let evs1 = seq.step(0.6, &tl).unwrap();
        assert_eq!(evs1.len(), 1);
        match &evs1[0] {
            SequencerEvent::AnimStart { target, clip } => {
                assert_eq!(*target, 42);
                assert_eq!(clip, "walk_cycle");
            }
            _ => panic!("Expected AnimStart event"),
        }
    }

    #[test]
    fn seq_emits_fx_events() {
        let mut tl = Timeline::new("fx_test", 3.0);
        tl.tracks.push(Track::Fx {
            name: "explosion".into(),
            start: Time(1.5),
            params: serde_json::json!({"scale": 2.0, "color": "red"}),
        });
        
        let mut seq = Sequencer::new();
        
        // Before FX trigger
        let evs0 = seq.step(1.0, &tl).unwrap();
        assert!(evs0.is_empty());
        
        // At FX trigger
        let evs1 = seq.step(0.6, &tl).unwrap();
        assert_eq!(evs1.len(), 1);
        match &evs1[0] {
            SequencerEvent::FxTrigger { name, params } => {
                assert_eq!(name, "explosion");
                assert_eq!(params["scale"], 2.0);
                assert_eq!(params["color"], "red");
            }
            _ => panic!("Expected FxTrigger event"),
        }
    }

    #[test]
    fn seq_multiple_events_same_frame() {
        let mut tl = Timeline::new("multi", 2.0);
        // All events at t=1.0
        tl.tracks.push(Track::Camera {
            keyframes: vec![CameraKey {
                t: Time(1.0),
                pos: (0.0, 0.0, 0.0),
                look_at: (1.0, 0.0, 0.0),
                fov_deg: 90.0,
            }],
        });
        tl.tracks.push(Track::Audio {
            clip: "sound".into(),
            start: Time(1.0),
            volume: 1.0,
        });
        tl.tracks.push(Track::Animation {
            target: 1,
            clip: "run".into(),
            start: Time(1.0),
        });
        
        let mut seq = Sequencer::new();
        let evs = seq.step(1.5, &tl).unwrap();
        
        // Should have all 3 events
        assert_eq!(evs.len(), 3);
    }

    #[test]
    fn timeline_empty_tracks() {
        let tl = Timeline::new("empty", 5.0);
        let mut seq = Sequencer::new();
        
        let evs = seq.step(1.0, &tl).unwrap();
        assert!(evs.is_empty());
        
        let evs = seq.step(2.0, &tl).unwrap();
        assert!(evs.is_empty());
    }

    #[test]
    fn camera_multiple_keyframes() {
        let mut tl = Timeline::new("camera_test", 5.0);
        tl.tracks.push(Track::Camera {
            keyframes: vec![
                CameraKey {
                    t: Time(1.0),
                    pos: (0.0, 0.0, 0.0),
                    look_at: (1.0, 0.0, 0.0),
                    fov_deg: 60.0,
                },
                CameraKey {
                    t: Time(2.0),
                    pos: (5.0, 0.0, 0.0),
                    look_at: (6.0, 0.0, 0.0),
                    fov_deg: 70.0,
                },
                CameraKey {
                    t: Time(3.0),
                    pos: (10.0, 0.0, 0.0),
                    look_at: (11.0, 0.0, 0.0),
                    fov_deg: 80.0,
                },
            ],
        });
        
        let mut seq = Sequencer::new();
        
        // Hit first keyframe
        let evs1 = seq.step(1.5, &tl).unwrap();
        assert_eq!(evs1.len(), 1);
        
        // Hit second keyframe
        let evs2 = seq.step(1.0, &tl).unwrap();
        assert_eq!(evs2.len(), 1);
        
        // Hit third keyframe
        let evs3 = seq.step(1.0, &tl).unwrap();
        assert_eq!(evs3.len(), 1);
    }

    #[test]
    fn time_struct() {
        // Test Time struct directly
        let t1 = Time(1.5);
        let t2 = Time(1.5);
        let t3 = Time(2.0);
        
        assert_eq!(t1, t2);
        assert_ne!(t1, t3);
        assert_eq!(t1.0, 1.5);
    }

    #[test]
    fn track_variants_equality() {
        // Test track equality
        let camera1 = Track::Camera {
            keyframes: vec![CameraKey {
                t: Time(1.0),
                pos: (0.0, 0.0, 0.0),
                look_at: (1.0, 0.0, 0.0),
                fov_deg: 60.0,
            }],
        };
        let camera2 = Track::Camera {
            keyframes: vec![CameraKey {
                t: Time(1.0),
                pos: (0.0, 0.0, 0.0),
                look_at: (1.0, 0.0, 0.0),
                fov_deg: 60.0,
            }],
        };
        assert_eq!(camera1, camera2);
        
        let anim = Track::Animation {
            target: 1,
            clip: "test".into(),
            start: Time(0.0),
        };
        assert_ne!(camera1, anim);
    }

    #[test]
    fn sequencer_event_equality() {
        let ev1 = SequencerEvent::AudioPlay {
            clip: "test".into(),
            volume: 0.5,
        };
        let ev2 = SequencerEvent::AudioPlay {
            clip: "test".into(),
            volume: 0.5,
        };
        let ev3 = SequencerEvent::AudioPlay {
            clip: "other".into(),
            volume: 0.5,
        };
        
        assert_eq!(ev1, ev2);
        assert_ne!(ev1, ev3);
    }

    #[test]
    fn sequencer_boundary_conditions() {
        let tl = Timeline::new("boundary", 1.0);
        let mut seq = Sequencer::new();
        
        // Step exactly to the end (should work due to 0.001 tolerance)
        let result = seq.step(1.0, &tl);
        assert!(result.is_ok());
        
        // Step tiny bit more
        let result2 = seq.step(0.0005, &tl);
        assert!(result2.is_ok()); // Within tolerance
        
        // Step definitely past
        let result3 = seq.step(0.01, &tl);
        assert!(result3.is_err());
    }

    #[test]
    fn seq_error_display() {
        let err = SeqError::Range(Time(5.5));
        let msg = format!("{}", err);
        assert!(msg.contains("5.5"));
        assert!(msg.contains("out of range"));
    }

    #[test]
    fn camera_key_equality() {
        let key1 = CameraKey {
            t: Time(1.0),
            pos: (1.0, 2.0, 3.0),
            look_at: (0.0, 0.0, 0.0),
            fov_deg: 60.0,
        };
        let key2 = CameraKey {
            t: Time(1.0),
            pos: (1.0, 2.0, 3.0),
            look_at: (0.0, 0.0, 0.0),
            fov_deg: 60.0,
        };
        let key3 = CameraKey {
            t: Time(2.0),
            pos: (1.0, 2.0, 3.0),
            look_at: (0.0, 0.0, 0.0),
            fov_deg: 60.0,
        };
        
        assert_eq!(key1, key2);
        assert_ne!(key1, key3);
    }

    #[test]
    fn timeline_with_all_track_types_roundtrip() {
        let mut tl = Timeline::new("full", 10.0);
        tl.tracks.push(Track::Camera {
            keyframes: vec![
                CameraKey { t: Time(0.0), pos: (0.0, 0.0, 0.0), look_at: (1.0, 0.0, 0.0), fov_deg: 60.0 },
                CameraKey { t: Time(5.0), pos: (5.0, 0.0, 0.0), look_at: (6.0, 0.0, 0.0), fov_deg: 90.0 },
            ],
        });
        tl.tracks.push(Track::Animation {
            target: 100,
            clip: "dance".into(),
            start: Time(2.0),
        });
        tl.tracks.push(Track::Audio {
            clip: "music.ogg".into(),
            start: Time(0.0),
            volume: 0.75,
        });
        tl.tracks.push(Track::Fx {
            name: "particles".into(),
            start: Time(3.0),
            params: serde_json::json!({"count": 1000}),
        });
        
        let json = serde_json::to_string(&tl).unwrap();
        let restored: Timeline = serde_json::from_str(&json).unwrap();
        assert_eq!(tl, restored);
        assert_eq!(restored.tracks.len(), 4);
    }
}
