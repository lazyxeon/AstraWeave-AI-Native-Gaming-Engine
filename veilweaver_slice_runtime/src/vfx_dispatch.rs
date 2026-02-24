//! VFX & audio event dispatcher — wires game-loop events to presentation specs.
//!
//! This module bridges [`GameLoopEvent`]s and game state to the VFX/audio
//! descriptor layers. Each tick the dispatcher reads the events produced
//! by the game loop and emits the appropriate [`VfxScene`] and
//! [`AudioScene`] updates for the presentation layer.
//!
//! # Usage
//!
//! ```rust
//! use veilweaver_slice_runtime::vfx_dispatch::VfxAudioDispatch;
//!
//! let mut dispatch = VfxAudioDispatch::new();
//! // Each tick:
//! dispatch.process_events(&[], 1.0 / 60.0);
//! let vfx = dispatch.vfx_scene();
//! let audio = dispatch.audio_scene();
//! ```

use crate::audio_specs::{
    AudioScene, BossMusicLayer, SpatialSfxSpec, StingerKind, UiStingerSpec, ZoneAmbienceId,
};
use crate::game_loop::GameLoopEvent;
use crate::palette::PresentationConfig;
use crate::storm_choice::StormChoice;
use crate::vfx_specs::{
    AnchorVfxSpec, EchoBurstSpec, PhaseTransitionVfx, StormVfxSpec, TelegraphVfxSpec,
    ThreadVfxSpec, Vec3f, VfxScene,
};

use serde::Serialize;
use tracing::{debug, info};

// ── Dispatch State ─────────────────────────────────────────────────────

/// Central dispatcher that translates game events into VFX + audio intents.
#[derive(Debug, Clone, Serialize)]
pub struct VfxAudioDispatch {
    /// Current VFX scene description.
    vfx_scene: VfxScene,
    /// Current audio scene description.
    audio_scene: AudioScene,
    /// Current presentation/material config.
    presentation: PresentationConfig,
    /// Current zone index (0–4).
    current_zone: usize,
    /// Whether we are in the boss encounter.
    in_boss_encounter: bool,
    /// Storm choice (if resolved).
    storm_choice: Option<StormChoice>,
    /// Total elapsed time (seconds).
    elapsed: f32,
    /// Tracked telegraph progress values: attack_name → progress.
    telegraph_progress: std::collections::HashMap<String, f32>,
}

impl Default for VfxAudioDispatch {
    fn default() -> Self {
        Self::new()
    }
}

impl VfxAudioDispatch {
    /// Creates a dispatch with default VFX, audio, and presentation layers.
    #[must_use]
    pub fn new() -> Self {
        Self {
            vfx_scene: VfxScene::new(),
            audio_scene: AudioScene::new(),
            presentation: PresentationConfig::for_zone(0),
            current_zone: 0,
            in_boss_encounter: false,
            storm_choice: None,
            elapsed: 0.0,
            telegraph_progress: std::collections::HashMap::new(),
        }
    }

    // ── Accessors ──────────────────────────────────────────────────────

    /// Read the current VFX scene (immutable reference for the renderer).
    pub fn vfx_scene(&self) -> &VfxScene {
        &self.vfx_scene
    }

    /// Read the current audio scene (immutable reference for the audio engine).
    pub fn audio_scene(&self) -> &AudioScene {
        &self.audio_scene
    }

    /// Mutable VFX scene for direct manipulation (e.g., adding anchors).
    pub fn vfx_scene_mut(&mut self) -> &mut VfxScene {
        &mut self.vfx_scene
    }

    /// Mutable audio scene for direct manipulation.
    pub fn audio_scene_mut(&mut self) -> &mut AudioScene {
        &mut self.audio_scene
    }

    /// Current presentation config.
    pub fn presentation(&self) -> &PresentationConfig {
        &self.presentation
    }

    /// Current zone index.
    pub fn current_zone(&self) -> usize {
        self.current_zone
    }

    /// Whether the boss encounter is active.
    pub fn in_boss_encounter(&self) -> bool {
        self.in_boss_encounter
    }

    /// Elapsed time in seconds.
    pub fn elapsed(&self) -> f32 {
        self.elapsed
    }

    // ── Event Processing ───────────────────────────────────────────────

    /// Process a batch of game-loop events and advance time.
    pub fn process_events(&mut self, events: &[GameLoopEvent], dt: f32) {
        self.elapsed += dt;

        for event in events {
            self.handle_event(event);
        }

        // Tick VFX animations (phase transitions, anchor repairs)
        self.vfx_scene.tick(dt);
    }

    fn handle_event(&mut self, event: &GameLoopEvent) {
        match event {
            GameLoopEvent::ZoneLoading { zone_name, coord } => {
                self.on_zone_loading(zone_name, coord.x.max(0) as usize);
            }
            GameLoopEvent::StormDecisionPrompt => {
                self.on_storm_prompt();
            }
            GameLoopEvent::StormDecisionMade { choice } => {
                self.on_storm_decision(*choice);
            }
            GameLoopEvent::StormResolved { choice } => {
                self.on_storm_resolved(*choice);
            }
            GameLoopEvent::CinematicStarted { name } => {
                debug!("VFX dispatch: cinematic started '{}'", name);
            }
            GameLoopEvent::CinematicFinished { name } => {
                debug!("VFX dispatch: cinematic finished '{}'", name);
            }
            GameLoopEvent::DialogueDisplay { .. } => {}
            GameLoopEvent::DialogueEnded { .. } => {}
        }
    }

    fn on_zone_loading(&mut self, zone_name: &str, zone_x: usize) {
        let zone_index = zone_x.saturating_sub(100); // coords start at 100
        if zone_index == self.current_zone {
            return;
        }
        self.current_zone = zone_index;
        info!(
            "VFX dispatch: switching to zone {} ({})",
            zone_index, zone_name
        );

        // Update audio ambience
        if let Some(zone_id) = ZoneAmbienceId::from_zone_index(zone_index) {
            self.audio_scene.set_zone(zone_id);
        }

        // Update presentation config
        self.presentation = if self.in_boss_encounter {
            PresentationConfig::boss_encounter(zone_index)
        } else {
            PresentationConfig::for_zone(zone_index)
        };
    }

    fn on_storm_prompt(&mut self) {
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::DecisionPrompt));
    }

    fn on_storm_decision(&mut self, choice: StormChoice) {
        self.storm_choice = Some(choice);
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::StormChoiceMade));
    }

    fn on_storm_resolved(&mut self, choice: StormChoice) {
        self.storm_choice = Some(choice);
        let storm_vfx = StormVfxSpec::from_choice(&choice);
        self.vfx_scene.storm = Some(storm_vfx);

        // Add storm ambient SFX
        self.audio_scene
            .spatial_sfx
            .push(SpatialSfxSpec::storm_crackle(Vec3f::ZERO));
    }

    // ── Manual VFX Commands ────────────────────────────────────────────

    /// Add a weaving thread VFX.
    pub fn add_thread(&mut self, spec: ThreadVfxSpec) {
        self.vfx_scene.threads.push(spec);
    }

    /// Remove a thread by ID.
    pub fn remove_thread(&mut self, thread_id: &str) {
        self.vfx_scene.threads.retain(|t| t.thread_id != thread_id);
    }

    /// Add or update an anchor VFX. If the anchor_id already exists, update it.
    pub fn set_anchor(&mut self, spec: AnchorVfxSpec) {
        if let Some(existing) = self
            .vfx_scene
            .anchors
            .iter_mut()
            .find(|a| a.anchor_id == spec.anchor_id)
        {
            *existing = spec;
        } else {
            self.vfx_scene.anchors.push(spec);
        }
    }

    /// Update an anchor's stability.
    pub fn update_anchor_stability(&mut self, anchor_id: &str, stability: f32) {
        if let Some(anchor) = self
            .vfx_scene
            .anchors
            .iter_mut()
            .find(|a| a.anchor_id == anchor_id)
        {
            anchor.set_stability(stability);
        }
    }

    /// Start anchor repair animation.
    pub fn begin_anchor_repair(&mut self, anchor_id: &str) {
        if let Some(anchor) = self
            .vfx_scene
            .anchors
            .iter_mut()
            .find(|a| a.anchor_id == anchor_id)
        {
            anchor.begin_repair();
        }
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::AnchorStabilized));
    }

    /// Emit an echo collection burst.
    pub fn emit_echo_burst(&mut self, position: Vec3f, large: bool) {
        let burst = if large {
            EchoBurstSpec::large(position)
        } else {
            EchoBurstSpec::standard(position)
        };
        self.vfx_scene.echo_bursts.push(burst);
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::EchoCollected));
    }

    /// Add a boss telegraph VFX.
    pub fn add_telegraph(&mut self, spec: TelegraphVfxSpec) {
        self.telegraph_progress
            .insert(spec.attack_name.clone(), 0.0);
        self.vfx_scene.telegraphs.push(spec);
    }

    /// Update a telegraph's progress by name.
    pub fn set_telegraph_progress(&mut self, attack_name: &str, progress: f32) {
        self.telegraph_progress
            .insert(attack_name.to_string(), progress);
        if let Some(t) = self
            .vfx_scene
            .telegraphs
            .iter_mut()
            .find(|t| t.attack_name == attack_name)
        {
            t.set_progress(progress);
        }
    }

    /// Remove a telegraph (attack fired or cancelled).
    pub fn remove_telegraph(&mut self, attack_name: &str) {
        self.telegraph_progress.remove(attack_name);
        self.vfx_scene
            .telegraphs
            .retain(|t| t.attack_name != attack_name);
    }

    /// Trigger a boss phase transition VFX.
    pub fn trigger_phase_transition(&mut self, from_phase: u32, to_phase: u32, origin: Vec3f) {
        let vfx = PhaseTransitionVfx::new(from_phase, to_phase, origin);
        self.vfx_scene.phase_transition = Some(vfx);

        // Audio: queue stinger + switch music layer
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::PhaseTransition));
        let layer = match to_phase {
            1 => BossMusicLayer::FulcrumShift,
            2 => BossMusicLayer::DirectiveOverride,
            _ => BossMusicLayer::Assessment,
        };
        self.audio_scene.set_boss_layer(layer);
    }

    /// Enter boss encounter mode.
    pub fn enter_boss_encounter(&mut self) {
        self.in_boss_encounter = true;
        self.presentation = PresentationConfig::boss_encounter(self.current_zone);
        self.audio_scene.set_boss_layer(BossMusicLayer::Assessment);
    }

    /// Boss was defeated.
    pub fn boss_defeated(&mut self) {
        self.in_boss_encounter = false;
        self.audio_scene.set_boss_layer(BossMusicLayer::Victory);
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::BossDefeated));
        // Clear telegraphs
        self.vfx_scene.telegraphs.clear();
        self.telegraph_progress.clear();
    }

    /// Weaving succeeded stinger.
    pub fn weaving_success(&mut self) {
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::WeavingSuccess));
    }

    /// Weaving failed stinger.
    pub fn weaving_failure(&mut self) {
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::WeavingFailure));
    }

    /// Companion affinity rank up stinger.
    pub fn affinity_rank_up(&mut self) {
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::AffinityRankUp));
    }

    /// Ability unlock stinger.
    pub fn ability_unlock(&mut self) {
        self.audio_scene
            .queue_stinger(UiStingerSpec::new(StingerKind::AbilityUnlock));
    }

    // ── Drain ──────────────────────────────────────────────────────────

    /// Drain echo bursts after the renderer has consumed them.
    pub fn drain_echo_bursts(&mut self) -> Vec<EchoBurstSpec> {
        self.vfx_scene.drain_bursts()
    }

    /// Drain pending stingers after the audio engine has processed them.
    pub fn drain_stingers(&mut self) -> Vec<UiStingerSpec> {
        self.audio_scene.drain_stingers()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_loop::GameLoopEvent;
    use crate::vfx_specs::AnchorVfxState;
    use astraweave_scene::world_partition::GridCoord;

    fn coord(x: i32) -> GridCoord {
        GridCoord { x, y: 0, z: 0 }
    }

    #[test]
    fn dispatch_initial_state() {
        let d = VfxAudioDispatch::new();
        assert_eq!(d.current_zone(), 0);
        assert!(!d.in_boss_encounter());
        assert!(d.vfx_scene().threads.is_empty());
        assert!(d.audio_scene().ambience.is_none());
    }

    #[test]
    fn dispatch_zone_loading_updates_ambience() {
        let mut d = VfxAudioDispatch::new();
        let events = vec![GameLoopEvent::ZoneLoading {
            zone_name: "Threadhollow Ruins".into(),
            coord: coord(101),
        }];
        d.process_events(&events, 0.016);
        assert_eq!(d.current_zone(), 1);
        assert!(d.audio_scene().ambience.is_some());
    }

    #[test]
    fn dispatch_duplicate_zone_noop() {
        let mut d = VfxAudioDispatch::new();
        let events = vec![GameLoopEvent::ZoneLoading {
            zone_name: "Loomspire Sanctum".into(),
            coord: coord(100),
        }];
        d.process_events(&events, 0.016);
        // Should still be zone 0
        assert_eq!(d.current_zone(), 0);
    }

    #[test]
    fn dispatch_storm_prompt_stinger() {
        let mut d = VfxAudioDispatch::new();
        let events = vec![GameLoopEvent::StormDecisionPrompt];
        d.process_events(&events, 0.016);
        let stingers = d.drain_stingers();
        assert_eq!(stingers.len(), 1);
        assert_eq!(stingers[0].kind, StingerKind::DecisionPrompt);
    }

    #[test]
    fn dispatch_storm_decision_and_resolve() {
        let mut d = VfxAudioDispatch::new();
        let events = vec![
            GameLoopEvent::StormDecisionMade {
                choice: StormChoice::Stabilize,
            },
            GameLoopEvent::StormResolved {
                choice: StormChoice::Stabilize,
            },
        ];
        d.process_events(&events, 0.016);
        assert!(d.vfx_scene().storm.is_some());
        let stingers = d.drain_stingers();
        // StormDecisionMade queues one stinger; StormResolved configures VFX only
        assert_eq!(stingers.len(), 1);
        assert_eq!(stingers[0].kind, StingerKind::StormChoiceMade);
    }

    #[test]
    fn dispatch_echo_burst() {
        let mut d = VfxAudioDispatch::new();
        d.emit_echo_burst(Vec3f::ZERO, false);
        assert_eq!(d.vfx_scene().echo_bursts.len(), 1);
        let drained = d.drain_echo_bursts();
        assert_eq!(drained.len(), 1);
        assert!(d.vfx_scene().echo_bursts.is_empty());
    }

    #[test]
    fn dispatch_thread_add_remove() {
        let mut d = VfxAudioDispatch::new();
        let spec = ThreadVfxSpec::stable("t1", Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0));
        d.add_thread(spec);
        assert_eq!(d.vfx_scene().threads.len(), 1);

        d.remove_thread("t1");
        assert!(d.vfx_scene().threads.is_empty());
    }

    #[test]
    fn dispatch_anchor_set_and_update() {
        let mut d = VfxAudioDispatch::new();
        let spec = AnchorVfxSpec::new("a1", Vec3f::ZERO, 0.8);
        d.set_anchor(spec);
        assert_eq!(d.vfx_scene().anchors.len(), 1);
        assert_eq!(d.vfx_scene().anchors[0].vfx_state, AnchorVfxState::Stable);

        d.update_anchor_stability("a1", 0.1);
        assert_eq!(d.vfx_scene().anchors[0].vfx_state, AnchorVfxState::Critical);
    }

    #[test]
    fn dispatch_anchor_upsert() {
        let mut d = VfxAudioDispatch::new();
        d.set_anchor(AnchorVfxSpec::new("a1", Vec3f::ZERO, 0.8));
        d.set_anchor(AnchorVfxSpec::new("a1", Vec3f::new(5.0, 0.0, 0.0), 0.5));
        // Should still be 1 anchor, updated
        assert_eq!(d.vfx_scene().anchors.len(), 1);
        assert!((d.vfx_scene().anchors[0].position.x - 5.0).abs() < 0.01);
    }

    #[test]
    fn dispatch_anchor_repair() {
        let mut d = VfxAudioDispatch::new();
        d.set_anchor(AnchorVfxSpec::new("a1", Vec3f::ZERO, 0.5));
        d.begin_anchor_repair("a1");
        assert!(d.vfx_scene().anchors[0].is_repairing);

        // Tick to advance repair timer
        d.process_events(&[], 1.0);
        assert!(d.vfx_scene().anchors[0].repair_time > 0.0);
    }

    #[test]
    fn dispatch_telegraph_lifecycle() {
        let mut d = VfxAudioDispatch::new();
        let t = TelegraphVfxSpec::cleave(Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0));
        d.add_telegraph(t);
        assert_eq!(d.vfx_scene().telegraphs.len(), 1);

        d.set_telegraph_progress("Oathbound Cleave", 0.5);
        assert!((d.vfx_scene().telegraphs[0].progress - 0.5).abs() < 0.01);

        d.remove_telegraph("Oathbound Cleave");
        assert!(d.vfx_scene().telegraphs.is_empty());
    }

    #[test]
    fn dispatch_phase_transition() {
        let mut d = VfxAudioDispatch::new();
        d.enter_boss_encounter();
        d.trigger_phase_transition(0, 1, Vec3f::ZERO);
        assert!(d.vfx_scene().phase_transition.is_some());

        let stingers = d.drain_stingers();
        assert!(stingers
            .iter()
            .any(|s| s.kind == StingerKind::PhaseTransition));

        // After enough ticks, it should auto-clear
        for _ in 0..300 {
            d.process_events(&[], 1.0 / 60.0);
        }
        assert!(d.vfx_scene().phase_transition.is_none());
    }

    #[test]
    fn dispatch_boss_encounter_lifecycle() {
        let mut d = VfxAudioDispatch::new();
        d.enter_boss_encounter();
        assert!(d.in_boss_encounter());
        assert!(d.audio_scene().boss_music.is_some());

        d.boss_defeated();
        assert!(!d.in_boss_encounter());
        let stingers = d.drain_stingers();
        assert!(stingers.iter().any(|s| s.kind == StingerKind::BossDefeated));
    }

    #[test]
    fn dispatch_weaving_stingers() {
        let mut d = VfxAudioDispatch::new();
        d.weaving_success();
        d.weaving_failure();
        let stingers = d.drain_stingers();
        assert_eq!(stingers.len(), 2);
        assert_eq!(stingers[0].kind, StingerKind::WeavingSuccess);
        assert_eq!(stingers[1].kind, StingerKind::WeavingFailure);
    }

    #[test]
    fn dispatch_affinity_and_ability_stingers() {
        let mut d = VfxAudioDispatch::new();
        d.affinity_rank_up();
        d.ability_unlock();
        let stingers = d.drain_stingers();
        assert_eq!(stingers.len(), 2);
    }

    #[test]
    fn dispatch_elapsed_time() {
        let mut d = VfxAudioDispatch::new();
        d.process_events(&[], 0.5);
        d.process_events(&[], 0.5);
        assert!((d.elapsed() - 1.0).abs() < 0.001);
    }

    #[test]
    fn dispatch_presentation_boss_overrides() {
        let mut d = VfxAudioDispatch::new();
        // Enter boss zone
        let events = vec![GameLoopEvent::ZoneLoading {
            zone_name: "Boss Courtyard".into(),
            coord: coord(104),
        }];
        d.process_events(&events, 0.016);
        let normal_vignette = d.presentation().vignette;

        // Enter boss encounter — should increase vignette
        d.enter_boss_encounter();
        let boss_vignette = d.presentation().vignette;
        assert!(boss_vignette > normal_vignette);
    }
}
