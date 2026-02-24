//! Cinematic player — high-level wrapper around [`Sequencer`] for tick-based playback.
//!
//! Loads RON-serialized [`Timeline`] files and advances playback each game tick,
//! emitting [`SequencerEvent`]s that the game loop routes to cameras, audio, VFX, etc.

use anyhow::{Context, Result};
use astraweave_cinematics::{SeqError, Sequencer, SequencerEvent, Timeline};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

// ── Playback state ─────────────────────────────────────────────────────────

/// Playback state of the cinematic player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlaybackState {
    /// No cinematic loaded or playback stopped.
    Idle,
    /// Currently playing a cinematic.
    Playing,
    /// Playback is temporarily suspended; resumes from the same position.
    Paused,
    /// Playback finished (reached end of timeline).
    Finished,
}

impl std::fmt::Display for PlaybackState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Idle => write!(f, "Idle"),
            Self::Playing => write!(f, "Playing"),
            Self::Paused => write!(f, "Paused"),
            Self::Finished => write!(f, "Finished"),
        }
    }
}

// ── CinematicPlayer ────────────────────────────────────────────────────────

/// Manages cinematic timeline playback within the game loop.
///
/// Usage:
/// ```text
/// player.load("boss_intro", timeline);
/// player.play("boss_intro");
/// loop { let events = player.tick(dt); ... }
/// ```
pub struct CinematicPlayer {
    /// Library of pre-loaded timelines keyed by name.
    timelines: HashMap<String, Timeline>,
    /// Currently active cinematic name.
    active: Option<String>,
    /// Sequencer state.
    sequencer: Sequencer,
    /// Current playback state.
    state: PlaybackState,
}

impl std::fmt::Debug for CinematicPlayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CinematicPlayer")
            .field("timelines", &self.timelines.keys().collect::<Vec<_>>())
            .field("active", &self.active)
            .field("state", &self.state)
            .finish_non_exhaustive()
    }
}

impl Default for CinematicPlayer {
    fn default() -> Self {
        Self::new()
    }
}

impl CinematicPlayer {
    /// Creates a new cinematic player with no loaded timelines.
    #[must_use]
    pub fn new() -> Self {
        Self {
            timelines: HashMap::new(),
            active: None,
            sequencer: Sequencer::new(),
            state: PlaybackState::Idle,
        }
    }

    // ── Timeline management ────────────────────────────────────────────

    /// Registers a timeline into the library.
    pub fn load(&mut self, name: impl Into<String>, timeline: Timeline) {
        let name = name.into();
        info!(
            "Cinematic loaded: '{}' ({}s, {} tracks)",
            name,
            timeline.duration_secs(),
            timeline.track_count()
        );
        self.timelines.insert(name, timeline);
    }

    /// Loads a timeline from a RON file on disk.
    ///
    /// # Errors
    /// Returns an error if the file cannot be read or deserialized.
    pub fn load_from_ron(&mut self, name: impl Into<String>, path: &Path) -> Result<()> {
        let content = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read cinematic: {}", path.display()))?;
        let timeline: Timeline = ron::de::from_str(&content)
            .with_context(|| format!("Failed to parse cinematic RON: {}", path.display()))?;
        self.load(name, timeline);
        Ok(())
    }

    /// Returns `true` if a timeline with this name is loaded.
    #[must_use]
    pub fn has_timeline(&self, name: &str) -> bool {
        self.timelines.contains_key(name)
    }

    /// Returns the number of loaded timelines.
    #[must_use]
    pub fn timeline_count(&self) -> usize {
        self.timelines.len()
    }

    /// Returns all loaded timeline names.
    #[must_use]
    pub fn timeline_names(&self) -> Vec<&str> {
        self.timelines.keys().map(String::as_str).collect()
    }

    // ── Playback control ───────────────────────────────────────────────

    /// Begins playback of the named cinematic from the start.
    ///
    /// # Errors
    /// Returns an error if the timeline is not loaded.
    pub fn play(&mut self, name: &str) -> Result<()> {
        anyhow::ensure!(
            self.timelines.contains_key(name),
            "Cinematic '{}' not loaded",
            name
        );
        info!("Playing cinematic: '{}'", name);
        self.active = Some(name.to_string());
        self.sequencer = Sequencer::new();
        self.state = PlaybackState::Playing;
        Ok(())
    }

    /// Stops playback and returns to idle.
    pub fn stop(&mut self) {
        if let Some(name) = &self.active {
            info!("Stopped cinematic: '{}'", name);
        }
        self.active = None;
        self.state = PlaybackState::Idle;
    }

    /// Pauses playback at the current time position.
    ///
    /// While paused, [`tick`] returns an empty event list and the
    /// sequencer clock does not advance.  Call [`resume`] to continue.
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            if let Some(name) = &self.active {
                info!(
                    "Paused cinematic: '{}' at {:.2}s",
                    name,
                    self.current_time()
                );
            }
            self.state = PlaybackState::Paused;
        }
    }

    /// Resumes playback from the paused position.
    pub fn resume(&mut self) {
        if self.state == PlaybackState::Paused {
            if let Some(name) = &self.active {
                info!(
                    "Resumed cinematic: '{}' at {:.2}s",
                    name,
                    self.current_time()
                );
            }
            self.state = PlaybackState::Playing;
        }
    }

    /// Advances playback by `dt` seconds and returns events from this step.
    ///
    /// Returns an empty vec if no cinematic is playing.
    pub fn tick(&mut self, dt: f32) -> Vec<SequencerEvent> {
        if self.state != PlaybackState::Playing {
            return Vec::new();
        }

        let timeline_name = match &self.active {
            Some(name) => name.clone(),
            None => return Vec::new(),
        };

        let timeline = match self.timelines.get(&timeline_name) {
            Some(tl) => tl,
            None => return Vec::new(),
        };

        match self.sequencer.step(dt, timeline) {
            Ok(events) => events,
            Err(SeqError::Range(_)) => {
                // Timeline finished.
                info!("Cinematic '{}' finished", timeline_name);
                self.state = PlaybackState::Finished;
                self.active = None;
                Vec::new()
            }
            Err(_) => {
                // Unknown sequencer error — stop playback gracefully.
                tracing::warn!("Cinematic '{}' sequencer error", timeline_name);
                self.state = PlaybackState::Finished;
                self.active = None;
                Vec::new()
            }
        }
    }

    // ── Queries ────────────────────────────────────────────────────────

    /// Returns the current playback state.
    #[must_use]
    pub fn state(&self) -> PlaybackState {
        self.state
    }

    /// Returns `true` if a cinematic is currently playing.
    #[must_use]
    pub fn is_playing(&self) -> bool {
        self.state == PlaybackState::Playing
    }

    /// Returns `true` if playback has finished.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.state == PlaybackState::Finished
    }

    /// Returns `true` if playback is paused.
    #[must_use]
    pub fn is_paused(&self) -> bool {
        self.state == PlaybackState::Paused
    }

    /// Returns the name of the currently active cinematic, if any.
    #[must_use]
    pub fn active_cinematic(&self) -> Option<&str> {
        self.active.as_deref()
    }

    /// Returns the current playback time in seconds.
    #[must_use]
    pub fn current_time(&self) -> f32 {
        self.sequencer.t.as_secs()
    }

    /// Returns the duration of the active cinematic, or 0 if none.
    #[must_use]
    pub fn active_duration(&self) -> f32 {
        self.active
            .as_ref()
            .and_then(|name| self.timelines.get(name))
            .map(|tl| tl.duration_secs())
            .unwrap_or(0.0)
    }

    /// Returns the playback progress as 0.0..=1.0.
    #[must_use]
    pub fn progress(&self) -> f32 {
        let dur = self.active_duration();
        if dur > 0.0 {
            (self.current_time() / dur).clamp(0.0, 1.0)
        } else {
            0.0
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_cinematics::{CameraKey, Time};

    fn test_timeline() -> Timeline {
        let mut tl = Timeline::new("test", 2.0);
        tl.add_camera_track(vec![
            CameraKey::new(
                Time::from_secs(0.5),
                (0.0, 5.0, 10.0),
                (0.0, 0.0, 0.0),
                60.0,
            ),
            CameraKey::new(Time::from_secs(1.5), (5.0, 3.0, 8.0), (0.0, 0.0, 0.0), 55.0),
        ]);
        tl.add_audio_track("ambient_boss", Time::from_secs(0.2), 0.8);
        tl
    }

    #[test]
    fn load_and_play() {
        let mut player = CinematicPlayer::new();
        player.load("test", test_timeline());
        assert!(player.has_timeline("test"));
        assert_eq!(player.timeline_count(), 1);

        player.play("test").unwrap();
        assert!(player.is_playing());
        assert_eq!(player.active_cinematic(), Some("test"));
    }

    #[test]
    fn tick_emits_events() {
        let mut player = CinematicPlayer::new();
        player.load("test", test_timeline());
        player.play("test").unwrap();

        // Step past the 0.2s audio cue.
        let events = player.tick(0.3);
        assert!(!events.is_empty());
        assert!(events.iter().any(|e| e.is_audio_play()));
    }

    #[test]
    fn tick_past_end_finishes() {
        let mut player = CinematicPlayer::new();
        player.load("test", test_timeline());
        player.play("test").unwrap();

        player.tick(1.0);
        player.tick(1.0);
        let _events = player.tick(0.5); // Beyond 2.0s duration
        assert!(player.is_finished());
    }

    #[test]
    fn stop_returns_to_idle() {
        let mut player = CinematicPlayer::new();
        player.load("test", test_timeline());
        player.play("test").unwrap();
        player.stop();
        assert_eq!(player.state(), PlaybackState::Idle);
        assert!(player.active_cinematic().is_none());
    }

    #[test]
    fn play_unknown_errors() {
        let mut player = CinematicPlayer::new();
        assert!(player.play("nonexistent").is_err());
    }

    #[test]
    fn progress_tracking() {
        let mut player = CinematicPlayer::new();
        player.load("test", test_timeline());
        player.play("test").unwrap();
        player.tick(1.0);
        let p = player.progress();
        assert!(p > 0.4 && p < 0.6);
    }

    #[test]
    fn idle_tick_returns_empty() {
        let mut player = CinematicPlayer::new();
        let events = player.tick(0.5);
        assert!(events.is_empty());
    }

    #[test]
    fn pause_and_resume() {
        let mut player = CinematicPlayer::new();
        player.load("test", test_timeline());
        player.play("test").unwrap();

        // Advance slightly so we're mid-playback.
        player.tick(0.5);
        assert!(player.is_playing());

        // Pause freezes the clock.
        player.pause();
        assert!(player.is_paused());
        assert_eq!(player.state(), PlaybackState::Paused);

        let time_before = player.current_time();
        let events = player.tick(1.0);
        assert!(events.is_empty(), "paused tick should emit no events");
        assert!(
            (player.current_time() - time_before).abs() < f32::EPSILON,
            "clock should not advance while paused"
        );

        // Resume continues from the paused position.
        player.resume();
        assert!(player.is_playing());
        let events = player.tick(0.5);
        // May or may not emit events depending on timeline cues,
        // but the clock should have advanced.
        assert!(player.current_time() > time_before);
        let _ = events;
    }

    #[test]
    fn pause_noop_when_not_playing() {
        let mut player = CinematicPlayer::new();
        // Pausing while idle should be a no-op.
        player.pause();
        assert_eq!(player.state(), PlaybackState::Idle);

        // Pausing after finish should be a no-op.
        player.load("test", test_timeline());
        player.play("test").unwrap();
        player.tick(3.0); // past the 2.0s timeline
        assert!(player.is_finished());
        player.pause();
        assert!(player.is_finished());
    }

    #[test]
    fn resume_noop_when_not_paused() {
        let mut player = CinematicPlayer::new();
        player.load("test", test_timeline());
        player.play("test").unwrap();

        // Resuming while already playing should be a no-op.
        player.resume();
        assert!(player.is_playing());

        // Resuming while idle should be a no-op.
        player.stop();
        player.resume();
        assert_eq!(player.state(), PlaybackState::Idle);
    }
}
