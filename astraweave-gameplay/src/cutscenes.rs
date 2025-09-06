use glam::Vec3;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Cue {
    CameraTo {
        pos: Vec3,
        yaw: f32,
        pitch: f32,
        time: f32,
    },
    Title {
        text: String,
        time: f32,
    },
    Wait {
        time: f32,
    },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timeline {
    pub cues: Vec<Cue>,
}

pub struct CutsceneState {
    pub idx: usize,
    pub t: f32,
}

impl CutsceneState {
    pub fn new() -> Self {
        Self { idx: 0, t: 0.0 }
    }

    /// Advances timeline; returns camera override (pos, yaw, pitch) and optional text
    pub fn tick(
        &mut self,
        dt: f32,
        tl: &Timeline,
    ) -> (Option<(Vec3, f32, f32)>, Option<String>, bool) {
        if self.idx >= tl.cues.len() {
            return (None, None, true);
        }
        self.t += dt;
        match &tl.cues[self.idx] {
            Cue::CameraTo {
                pos,
                yaw,
                pitch,
                time,
            } => {
                if self.t >= *time {
                    self.idx += 1;
                    self.t = 0.0;
                    (Some((*pos, *yaw, *pitch)), None, self.idx >= tl.cues.len())
                } else {
                    (Some((*pos, *yaw, *pitch)), None, false)
                }
            }
            Cue::Title { text, time } => {
                let done = self.t >= *time;
                if done {
                    self.idx += 1;
                    self.t = 0.0;
                }
                (None, Some(text.clone()), self.idx >= tl.cues.len())
            }
            Cue::Wait { time } => {
                let done = self.t >= *time;
                if done {
                    self.idx += 1;
                    self.t = 0.0;
                }
                (None, None, self.idx >= tl.cues.len())
            }
        }
    }
}
