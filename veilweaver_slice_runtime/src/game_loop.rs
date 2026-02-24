//! Unified game loop — orchestrates all Phase 2 subsystems per tick.
//!
//! The game loop ties together:
//! - **Zone transitions** (cell streaming)
//! - **Dialogue runner** (TOML-loaded dialogue trees)
//! - **Cinematic player** (RON timeline playback)
//! - **Storm choice** (Z3 decision state machine)
//! - **Trigger dispatch** (action parsing and routing)
//!
//! Each call to [`GameLoop::tick`] processes one frame's worth of events,
//! advancing dialogue, cinematics, and zone state as needed.

use crate::cinematic_player::CinematicPlayer;
use crate::storm_choice::{StormChoice, StormChoiceState, StormPhase};
use crate::zone_transitions::{dispatch_trigger_actions, ZoneRegistry, ZoneTransitionEvent};
use astraweave_dialogue::runner::{DialogueEvent, DialogueRunner, RunnerState};
use astraweave_dialogue::toml_loader::LoadedDialogue;
use astraweave_scene::world_partition::GridCoord;
use std::collections::HashMap;
use tracing::info;

// ── Game-loop events ───────────────────────────────────────────────────────

/// High-level events produced by the game loop each tick.
///
/// The presentation layer / ECS systems consume these to update rendering,
/// audio, UI overlays, etc.
#[derive(Debug, Clone, PartialEq)]
pub enum GameLoopEvent {
    /// A zone is being loaded (cell streaming initiated).
    ZoneLoading { zone_name: String, coord: GridCoord },
    /// Dialogue text to display.
    DialogueDisplay {
        node_id: String,
        text: String,
        choices: Vec<String>,
    },
    /// Dialogue completed.
    DialogueEnded { dialogue_id: String },
    /// A cinematic began playing.
    CinematicStarted { name: String },
    /// A cinematic finished.
    CinematicFinished { name: String },
    /// Storm choice prompt should open.
    StormDecisionPrompt,
    /// Storm choice was made.
    StormDecisionMade { choice: StormChoice },
    /// Storm effects resolved — arena is configured.
    StormResolved { choice: StormChoice },
}

// ── Game loop ──────────────────────────────────────────────────────────────

/// Central game loop state for the Veilweaver vertical slice.
///
/// Holds references to all subsystems and coordinates inter-system communication.
pub struct GameLoop {
    /// Zone name → grid coordinate registry.
    pub zone_registry: ZoneRegistry,
    /// Dialogue runners keyed by dialogue ID.
    pub dialogues: HashMap<String, DialogueRunner>,
    /// Loaded dialogue metadata keyed by dialogue ID.
    pub dialogue_meta: HashMap<String, LoadedDialogue>,
    /// Cinematic player for timeline playback.
    pub cinematics: CinematicPlayer,
    /// Storm choice state machine.
    pub storm_state: StormChoiceState,
    /// Trigger action lookup: trigger_id → action string.
    pub trigger_actions: HashMap<String, String>,
    /// Current active zone coordinate.
    pub active_zone: Option<GridCoord>,
    /// Accumulated events for this tick (drained by the consumer).
    pending_events: Vec<GameLoopEvent>,
    /// Set of triggers that fired "entering" this tick (populated externally).
    entering_triggers: Vec<String>,
    /// Deferred storm choice (set between ticks, consumed during tick).
    deferred_storm_choice: Option<StormChoice>,
}

impl std::fmt::Debug for GameLoop {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("GameLoop")
            .field("zone_registry", &self.zone_registry)
            .field("dialogues", &self.dialogues.keys().collect::<Vec<_>>())
            .field("cinematics", &self.cinematics)
            .field("storm_state", &self.storm_state)
            .field("active_zone", &self.active_zone)
            .field("trigger_actions_count", &self.trigger_actions.len())
            .field("pending_events", &self.pending_events.len())
            .finish_non_exhaustive()
    }
}

impl Default for GameLoop {
    fn default() -> Self {
        Self::new()
    }
}

impl GameLoop {
    /// Creates a new game loop with the default Veilweaver zone registry.
    #[must_use]
    pub fn new() -> Self {
        Self {
            zone_registry: ZoneRegistry::veilweaver_default(),
            dialogues: HashMap::new(),
            dialogue_meta: HashMap::new(),
            cinematics: CinematicPlayer::new(),
            storm_state: StormChoiceState::new(),
            trigger_actions: HashMap::new(),
            active_zone: None,
            pending_events: Vec::new(),
            entering_triggers: Vec::new(),
            deferred_storm_choice: None,
        }
    }

    // ── Setup ──────────────────────────────────────────────────────────

    /// Registers a loaded dialogue (from TOML) into the game loop.
    pub fn register_dialogue(&mut self, loaded: LoadedDialogue) {
        let id = loaded.dialogue_id.clone();
        let runner = DialogueRunner::new(loaded.graph.clone());
        info!(
            "Registered dialogue '{}' ({} nodes, start='{}')",
            id,
            loaded.graph.node_count(),
            loaded.start_node
        );
        self.dialogues.insert(id.clone(), runner);
        self.dialogue_meta.insert(id, loaded);
    }

    /// Registers a trigger action mapping: `trigger_id → action_string`.
    ///
    /// Typically populated from `TriggerZoneSpec.action` at startup.
    pub fn register_trigger_action(
        &mut self,
        trigger_id: impl Into<String>,
        action: impl Into<String>,
    ) {
        self.trigger_actions
            .insert(trigger_id.into(), action.into());
    }

    /// Bulk-registers trigger actions from an iterator of (id, action) pairs.
    pub fn register_trigger_actions(
        &mut self,
        actions: impl IntoIterator<Item = (String, String)>,
    ) {
        for (id, action) in actions {
            self.trigger_actions.insert(id, action);
        }
    }

    /// Sets the initial active zone.
    pub fn set_active_zone(&mut self, coord: GridCoord) {
        self.active_zone = Some(coord);
    }

    // ── Per-tick input ─────────────────────────────────────────────────

    /// Notifies the game loop that triggers with these IDs fired (entering).
    ///
    /// Called by the ECS trigger system before [`tick`].
    pub fn notify_trigger_enter(&mut self, trigger_ids: Vec<String>) {
        self.entering_triggers = trigger_ids;
    }

    /// Notifies the game loop of a player dialogue choice.
    ///
    /// Routes to the active dialogue's runner.
    pub fn notify_dialogue_choice(&mut self, dialogue_id: &str, choice_index: usize) {
        if let Some(runner) = self.dialogues.get_mut(dialogue_id) {
            if let Err(e) = runner.choose(choice_index) {
                tracing::warn!("Dialogue choice error ({}): {}", dialogue_id, e);
            }
        }
    }

    /// Notifies the game loop of the storm decision.
    ///
    /// The choice is deferred and applied at the start of the next [`tick`].
    pub fn notify_storm_choice(&mut self, choice: StormChoice) {
        self.deferred_storm_choice = Some(choice);
    }

    // ── Main tick ──────────────────────────────────────────────────────

    /// Advances the game loop by one frame.
    ///
    /// 1. Process trigger events → dispatch actions
    /// 2. Advance dialogue runners → emit display/end events
    /// 3. Advance cinematic player → (events handled by presentation)
    /// 4. Update storm state
    ///
    /// Returns accumulated events for external consumption.
    pub fn tick(&mut self, dt: f32) -> Vec<GameLoopEvent> {
        self.pending_events.clear();

        // 0. Apply deferred storm choice (set between ticks).
        if let Some(choice) = self.deferred_storm_choice.take() {
            if self.storm_state.make_choice(choice) {
                self.pending_events
                    .push(GameLoopEvent::StormDecisionMade { choice });
            }
        }

        // 1. Process trigger enters → resolve actions.
        self.process_triggers();

        // 2. Drain dialogue runner events.
        self.process_dialogues();

        // 3. Advance cinematics.
        self.process_cinematics(dt);

        // 4. Storm state machine advancement.
        self.process_storm();

        std::mem::take(&mut self.pending_events)
    }

    // ── Internal processing ────────────────────────────────────────────

    fn process_triggers(&mut self) {
        let triggers = std::mem::take(&mut self.entering_triggers);
        if triggers.is_empty() {
            return;
        }

        // Build (trigger_id, action) pairs for fired triggers that have actions.
        let fired: Vec<(String, String)> = triggers
            .iter()
            .filter_map(|id| {
                self.trigger_actions
                    .get(id)
                    .map(|action| (id.clone(), action.clone()))
            })
            .collect();

        let events = dispatch_trigger_actions(&fired, &self.zone_registry);

        for event in events {
            match event {
                ZoneTransitionEvent::ZoneTransition {
                    target_zone,
                    target_coord,
                    ..
                } => {
                    info!(
                        "Game loop: zone transition → {} ({:?})",
                        target_zone, target_coord
                    );
                    self.active_zone = Some(target_coord);
                    self.pending_events.push(GameLoopEvent::ZoneLoading {
                        zone_name: target_zone,
                        coord: target_coord,
                    });
                }
                ZoneTransitionEvent::CinematicTrigger { cinematic_name, .. } => {
                    if self.cinematics.has_timeline(&cinematic_name) {
                        if let Err(e) = self.cinematics.play(&cinematic_name) {
                            tracing::warn!("Cinematic play error: {}", e);
                        } else {
                            self.pending_events.push(GameLoopEvent::CinematicStarted {
                                name: cinematic_name,
                            });
                        }
                    }
                }
                ZoneTransitionEvent::DialogueTrigger { dialogue_name, .. } => {
                    self.start_dialogue(&dialogue_name);
                }
                ZoneTransitionEvent::DecisionTrigger { decision_name, .. } => {
                    if decision_name == "storm_routing" || decision_name == "storm_choice" {
                        self.storm_state.enter_crossroads();
                        self.pending_events.push(GameLoopEvent::StormDecisionPrompt);
                    }
                }
                ZoneTransitionEvent::VfxTrigger { .. }
                | ZoneTransitionEvent::BossTrigger { .. }
                | ZoneTransitionEvent::LegacyAction { .. } => {
                    // VFX, boss, and legacy actions are forwarded to the presentation layer
                    // via the raw ZoneTransitionEvent. The game loop doesn't need to
                    // interpret them further at this level.
                }
            }
        }
    }

    fn start_dialogue(&mut self, dialogue_name: &str) {
        if let Some(meta) = self.dialogue_meta.get(dialogue_name).cloned() {
            if let Some(runner) = self.dialogues.get_mut(dialogue_name) {
                if let Err(e) = runner.start(&meta.start_node) {
                    tracing::warn!("Failed to start dialogue '{}': {}", dialogue_name, e);
                }
            }
        }
    }

    fn process_dialogues(&mut self) {
        let dialogue_ids: Vec<String> = self.dialogues.keys().cloned().collect();

        for id in &dialogue_ids {
            if let Some(runner) = self.dialogues.get_mut(id) {
                let events = runner.drain_events();
                for event in events {
                    match event {
                        DialogueEvent::NodeEntered {
                            node_id,
                            text,
                            choices,
                        } => {
                            self.pending_events.push(GameLoopEvent::DialogueDisplay {
                                node_id,
                                text,
                                choices,
                            });
                        }
                        DialogueEvent::Ended { .. } => {
                            self.pending_events.push(GameLoopEvent::DialogueEnded {
                                dialogue_id: id.clone(),
                            });
                        }
                        DialogueEvent::ChoiceMade {
                            next_node_id,
                            choice_text,
                            ..
                        } => {
                            // Check if this choice triggers a storm decision.
                            if choice_text.contains("Stabilize")
                                || choice_text.contains("stabilize")
                            {
                                if self.storm_state.phase() == StormPhase::DecisionPending {
                                    self.notify_storm_choice(StormChoice::Stabilize);
                                }
                            } else if (choice_text.contains("Redirect")
                                || choice_text.contains("redirect")
                                || choice_text.contains("chaos"))
                                && self.storm_state.phase() == StormPhase::DecisionPending
                            {
                                self.notify_storm_choice(StormChoice::Redirect);
                            }

                            // Check if next node leads into storm branch.
                            if let Some(ref next_id) = next_node_id {
                                if next_id == "storm_stabilize"
                                    && self.storm_state.phase() == StormPhase::DecisionPending
                                {
                                    self.notify_storm_choice(StormChoice::Stabilize);
                                } else if next_id == "storm_redirect"
                                    && self.storm_state.phase() == StormPhase::DecisionPending
                                {
                                    self.notify_storm_choice(StormChoice::Redirect);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn process_cinematics(&mut self, dt: f32) {
        let was_playing = self.cinematics.is_playing();
        let active_name = self.cinematics.active_cinematic().map(String::from);

        let _events = self.cinematics.tick(dt);
        // Sequencer events (camera keys, audio cues, etc.) are consumed by
        // the presentation layer — we only track lifecycle here.

        if was_playing && self.cinematics.is_finished() {
            if let Some(name) = active_name {
                self.pending_events
                    .push(GameLoopEvent::CinematicFinished { name });
            }
        }
    }

    fn process_storm(&mut self) {
        if self.storm_state.phase() == StormPhase::ChoiceMade {
            // Auto-resolve once the choice is made — presentation picks up the
            // modifiers from `storm_state.modifiers()`.
            let choice = match self.storm_state.choice() {
                Some(c) => c,
                None => {
                    // Defensive: ChoiceMade phase without a choice should not occur,
                    // but we handle it gracefully rather than panicking.
                    tracing::warn!(
                        "process_storm: ChoiceMade phase but no choice set — skipping resolve"
                    );
                    return;
                }
            };
            self.storm_state.resolve();
            self.pending_events
                .push(GameLoopEvent::StormResolved { choice });
        }
    }

    // ── Queries ────────────────────────────────────────────────────────

    /// Returns the current active zone coordinate.
    #[must_use]
    pub fn active_zone(&self) -> Option<GridCoord> {
        self.active_zone
    }

    /// Returns `true` if a cinematic is currently playing.
    #[must_use]
    pub fn is_cinematic_playing(&self) -> bool {
        self.cinematics.is_playing()
    }

    /// Returns the storm choice state.
    #[must_use]
    pub fn storm_state(&self) -> &StormChoiceState {
        &self.storm_state
    }

    /// Returns `true` if any dialogue runner is waiting for input.
    #[must_use]
    pub fn is_dialogue_active(&self) -> bool {
        self.dialogues
            .values()
            .any(|r| r.state() == RunnerState::WaitingForChoice)
    }

    /// Returns the ID of the active (waiting) dialogue, if any.
    #[must_use]
    pub fn active_dialogue_id(&self) -> Option<&str> {
        self.dialogues
            .iter()
            .find(|(_, r)| r.state() == RunnerState::WaitingForChoice)
            .map(|(id, _)| id.as_str())
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use astraweave_cinematics::{CameraKey, Time, Timeline};
    use astraweave_dialogue::toml_loader::load_dialogue_from_toml;

    fn test_dialogue() -> LoadedDialogue {
        let toml = r#"
id = "intro"
start = "n0"

[[nodes]]
id = "n0"
line = { speaker = "A", text = "Hello." }
choices = [{ text = "Hi", go_to = "n1" }]

[[nodes]]
id = "n1"
line = { speaker = "A", text = "Bye." }
end = true
"#;
        load_dialogue_from_toml(toml).unwrap()
    }

    fn test_timeline() -> Timeline {
        let mut tl = Timeline::new("boss_intro", 2.0);
        tl.add_camera_track(vec![CameraKey::new(
            Time::from_secs(0.5),
            (0.0, 5.0, 10.0),
            (0.0, 0.0, 0.0),
            60.0,
        )]);
        tl
    }

    #[test]
    fn basic_tick_empty() {
        let mut gl = GameLoop::new();
        let events = gl.tick(1.0 / 60.0);
        assert!(events.is_empty());
    }

    #[test]
    fn zone_transition_via_trigger() {
        let mut gl = GameLoop::new();
        gl.register_trigger_action("zone_exit_boss", "zone.transition:Z4_boss_courtyard");

        gl.notify_trigger_enter(vec!["zone_exit_boss".to_string()]);
        let events = gl.tick(1.0 / 60.0);

        assert!(events.iter().any(|e| matches!(
            e,
            GameLoopEvent::ZoneLoading { zone_name, .. } if zone_name == "Z4_boss_courtyard"
        )));
        assert_eq!(gl.active_zone(), Some(GridCoord::new(104, 0, 0)));
    }

    #[test]
    fn dialogue_trigger_and_advance() {
        let mut gl = GameLoop::new();
        let loaded = test_dialogue();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("entry_trigger", "dialogue.play:intro");

        gl.notify_trigger_enter(vec!["entry_trigger".to_string()]);
        let events = gl.tick(1.0 / 60.0);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::DialogueDisplay { .. })));
        assert!(gl.is_dialogue_active());

        gl.notify_dialogue_choice("intro", 0);
        let events = gl.tick(1.0 / 60.0);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::DialogueEnded { .. })));
    }

    #[test]
    fn cinematic_trigger_and_finish() {
        let mut gl = GameLoop::new();
        gl.cinematics.load("boss_intro", test_timeline());
        gl.register_trigger_action("cinematic_trigger", "cinematic.play:boss_intro");

        gl.notify_trigger_enter(vec!["cinematic_trigger".to_string()]);
        let events = gl.tick(1.0 / 60.0);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::CinematicStarted { .. })));
        assert!(gl.is_cinematic_playing());

        // Advance past the timeline end.
        let _events = gl.tick(1.0);
        let events = gl.tick(1.5);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::CinematicFinished { .. })));
    }

    #[test]
    fn storm_decision_flow() {
        let mut gl = GameLoop::new();
        gl.register_trigger_action("choice_prompt", "decision.open:storm_routing");

        gl.notify_trigger_enter(vec!["choice_prompt".to_string()]);
        let events = gl.tick(1.0 / 60.0);
        assert!(events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::StormDecisionPrompt)));

        gl.notify_storm_choice(StormChoice::Redirect);
        let events = gl.tick(1.0 / 60.0);
        assert!(events.iter().any(|e| matches!(
            e,
            GameLoopEvent::StormDecisionMade { choice } if *choice == StormChoice::Redirect
        )));
        assert!(events.iter().any(|e| matches!(
            e,
            GameLoopEvent::StormResolved { choice } if *choice == StormChoice::Redirect
        )));
        assert!(gl.storm_state().is_resolved());
    }

    #[test]
    fn trigger_without_action_is_silently_ignored() {
        let mut gl = GameLoop::new();
        gl.notify_trigger_enter(vec!["random_unregistered_trigger".to_string()]);
        let events = gl.tick(1.0 / 60.0);
        assert!(events.is_empty());
    }
}
