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

impl Default for CutsceneState {
    fn default() -> Self {
        Self::new()
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cutscene_state_default() {
        let state = CutsceneState::new();
        assert_eq!(state.idx, 0);
        assert_eq!(state.t, 0.0);
    }

    #[test]
    fn test_camera_to_cue_during_transition() {
        let mut state = CutsceneState::new();
        let timeline = Timeline {
            cues: vec![Cue::CameraTo {
                pos: Vec3::new(1.0, 2.0, 3.0),
                yaw: 45.0,
                pitch: -10.0,
                time: 2.0,
            }],
        };

        // Tick before completion
        let (cam, text, done) = state.tick(0.5, &timeline);
        assert!(cam.is_some());
        let (pos, yaw, pitch) = cam.unwrap();
        assert_eq!(pos, Vec3::new(1.0, 2.0, 3.0));
        assert_eq!(yaw, 45.0);
        assert_eq!(pitch, -10.0);
        assert!(text.is_none());
        assert!(!done);
        assert_eq!(state.idx, 0);
        assert_eq!(state.t, 0.5);
    }

    #[test]
    fn test_camera_to_cue_completes() {
        let mut state = CutsceneState::new();
        let timeline = Timeline {
            cues: vec![Cue::CameraTo {
                pos: Vec3::new(5.0, 6.0, 7.0),
                yaw: 90.0,
                pitch: 0.0,
                time: 1.0,
            }],
        };

        // Tick to completion
        let (cam, text, done) = state.tick(1.5, &timeline);
        assert!(cam.is_some());
        assert!(text.is_none());
        assert!(done); // Only one cue, should be done
        assert_eq!(state.idx, 1);
        assert_eq!(state.t, 0.0);
    }

    #[test]
    fn test_title_cue_displays_text() {
        let mut state = CutsceneState::new();
        let timeline = Timeline {
            cues: vec![Cue::Title {
                text: "Chapter 1".to_string(),
                time: 3.0,
            }],
        };

        // Tick before completion
        let (cam, text, done) = state.tick(1.0, &timeline);
        assert!(cam.is_none());
        assert_eq!(text, Some("Chapter 1".to_string()));
        assert!(!done);
        assert_eq!(state.idx, 0);

        // Tick to completion
        let (cam, text, done) = state.tick(2.5, &timeline);
        assert!(cam.is_none());
        assert_eq!(text, Some("Chapter 1".to_string()));
        assert!(done);
        assert_eq!(state.idx, 1);
    }

    #[test]
    fn test_wait_cue_progression() {
        let mut state = CutsceneState::new();
        let timeline = Timeline {
            cues: vec![Cue::Wait { time: 2.0 }],
        };

        // Tick before completion
        let (cam, text, done) = state.tick(1.0, &timeline);
        assert!(cam.is_none());
        assert!(text.is_none());
        assert!(!done);

        // Tick to completion
        let (cam, text, done) = state.tick(1.5, &timeline);
        assert!(cam.is_none());
        assert!(text.is_none());
        assert!(done);
        assert_eq!(state.idx, 1);
    }

    #[test]
    fn test_multiple_cues_progression() {
        let mut state = CutsceneState::new();
        let timeline = Timeline {
            cues: vec![
                Cue::Title {
                    text: "Intro".to_string(),
                    time: 1.0,
                },
                Cue::Wait { time: 0.5 },
                Cue::CameraTo {
                    pos: Vec3::ZERO,
                    yaw: 0.0,
                    pitch: 0.0,
                    time: 1.0,
                },
            ],
        };

        // Complete first cue (Title)
        let (_, text, done) = state.tick(1.5, &timeline);
        assert_eq!(text, Some("Intro".to_string()));
        assert!(!done);
        assert_eq!(state.idx, 1);

        // Complete second cue (Wait)
        let (cam, text, done) = state.tick(0.6, &timeline);
        assert!(cam.is_none());
        assert!(text.is_none());
        assert!(!done);
        assert_eq!(state.idx, 2);

        // Complete third cue (CameraTo)
        let (cam, _, done) = state.tick(1.1, &timeline);
        assert!(cam.is_some());
        assert!(done);
        assert_eq!(state.idx, 3);
    }

    #[test]
    fn test_empty_timeline_returns_done() {
        let mut state = CutsceneState::new();
        let timeline = Timeline { cues: vec![] };

        let (cam, text, done) = state.tick(1.0, &timeline);
        assert!(cam.is_none());
        assert!(text.is_none());
        assert!(done);
    }

    #[test]
    fn test_tick_after_completion_stays_done() {
        let mut state = CutsceneState::new();
        let timeline = Timeline {
            cues: vec![Cue::Wait { time: 1.0 }],
        };

        // Complete the timeline
        state.tick(2.0, &timeline);
        assert_eq!(state.idx, 1);

        // Tick again after completion
        let (cam, text, done) = state.tick(1.0, &timeline);
        assert!(cam.is_none());
        assert!(text.is_none());
        assert!(done);
        assert_eq!(state.idx, 1); // Stays at end
    }

    #[test]
    fn test_state_timer_resets_between_cues() {
        let mut state = CutsceneState::new();
        let timeline = Timeline {
            cues: vec![
                Cue::Wait { time: 1.0 },
                Cue::Wait { time: 1.0 },
            ],
        };

        // Complete first cue
        state.tick(1.5, &timeline);
        assert_eq!(state.idx, 1);
        assert_eq!(state.t, 0.0); // Timer reset

        // Verify timer accumulates for second cue
        state.tick(0.5, &timeline);
        assert_eq!(state.t, 0.5);
    }
}

