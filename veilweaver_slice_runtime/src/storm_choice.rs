//! Storm choice state machine — tracks the player's critical decision at Z3.
//!
//! At the Loom Crossroads the player chooses whether to **stabilize** or
//! **redirect** the storm.  This decision propagates to Z4's boss arena:
//! - **Stabilize** → clear sightlines, heavier Warden armor
//! - **Redirect**  → reduced visibility, Warden loses fog reinforcements
//!
//! The state machine also resolves which VFX, dialogue branches, and encounter
//! modifiers to apply downstream.

use serde::{Deserialize, Serialize};
use tracing::info;

// ── Storm choice enum ──────────────────────────────────────────────────────

/// The two routing options at the Loom Crossroads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StormChoice {
    /// Calm the storm — clear arena, but Warden gets heavier shields.
    Stabilize,
    /// Redirect the surge — foggy arena, but Warden loses fog reinforcements.
    Redirect,
}

impl StormChoice {
    /// Returns the human-readable label.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Stabilize => "Stabilize the Storm",
            Self::Redirect => "Redirect the Surge",
        }
    }

    /// Returns the VFX name this choice should activate.
    #[must_use]
    pub fn vfx_name(&self) -> &'static str {
        match self {
            Self::Stabilize => "storm_stabilize_blue",
            Self::Redirect => "storm_redirect_amber",
        }
    }

    /// Returns the dialogue branch suffix (used to select dialogue nodes).
    #[must_use]
    pub fn dialogue_suffix(&self) -> &'static str {
        match self {
            Self::Stabilize => "stable",
            Self::Redirect => "redirect",
        }
    }
}

impl std::fmt::Display for StormChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.label())
    }
}

// ── Boss modifiers ─────────────────────────────────────────────────────────

/// Encounter modifiers applied to the boss arena based on the storm choice.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BossArenaModifiers {
    /// Visibility multiplier (1.0 = full, 0.0 = zero visibility).
    pub visibility: f32,
    /// Additional armor percentage for the Warden (0.0 = none, 0.5 = +50%).
    pub warden_armor_bonus: f32,
    /// Whether the Warden can use fog-bank reinforcements.
    pub fog_reinforcements_enabled: bool,
    /// Additional VFX overlay in the arena.
    pub arena_vfx: String,
}

impl BossArenaModifiers {
    /// Returns the modifiers for a given storm choice.
    #[must_use]
    pub fn from_choice(choice: StormChoice) -> Self {
        match choice {
            StormChoice::Stabilize => Self {
                visibility: 1.0,
                warden_armor_bonus: 0.35,
                fog_reinforcements_enabled: true,
                arena_vfx: "storm_stabilize_blue".to_string(),
            },
            StormChoice::Redirect => Self {
                visibility: 0.4,
                warden_armor_bonus: 0.0,
                fog_reinforcements_enabled: false,
                arena_vfx: "storm_redirect_amber".to_string(),
            },
        }
    }
}

// ── State machine ──────────────────────────────────────────────────────────

/// Phase of the storm choice flow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StormPhase {
    /// Player has not yet reached the crossroads.
    PreCrossroads,
    /// Player is at the crossroads; decision prompt is open.
    DecisionPending,
    /// Player has made their choice; effects are being applied.
    ChoiceMade,
    /// Effects fully applied; ready for boss arena.
    Resolved,
}

impl std::fmt::Display for StormPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PreCrossroads => write!(f, "Pre-Crossroads"),
            Self::DecisionPending => write!(f, "Decision Pending"),
            Self::ChoiceMade => write!(f, "Choice Made"),
            Self::Resolved => write!(f, "Resolved"),
        }
    }
}

/// Manages the storm routing decision lifecycle.
#[derive(Debug, Clone)]
pub struct StormChoiceState {
    phase: StormPhase,
    choice: Option<StormChoice>,
    modifiers: Option<BossArenaModifiers>,
}

impl Default for StormChoiceState {
    fn default() -> Self {
        Self::new()
    }
}

impl StormChoiceState {
    /// Creates a new state in the `PreCrossroads` phase.
    #[must_use]
    pub fn new() -> Self {
        Self {
            phase: StormPhase::PreCrossroads,
            choice: None,
            modifiers: None,
        }
    }

    /// Transitions to the `DecisionPending` phase (player entered crossroads).
    pub fn enter_crossroads(&mut self) {
        if self.phase == StormPhase::PreCrossroads {
            info!("Storm choice: player entered crossroads — decision pending");
            self.phase = StormPhase::DecisionPending;
        }
    }

    /// Records the player's choice and computes arena modifiers.
    ///
    /// Only succeeds in the `DecisionPending` phase.
    pub fn make_choice(&mut self, choice: StormChoice) -> bool {
        if self.phase != StormPhase::DecisionPending {
            return false;
        }
        info!("Storm choice: player chose '{}'", choice);
        self.choice = Some(choice);
        self.modifiers = Some(BossArenaModifiers::from_choice(choice));
        self.phase = StormPhase::ChoiceMade;
        true
    }

    /// Marks the effects as fully applied (VFX spawned, dialogue routed).
    pub fn resolve(&mut self) {
        if self.phase == StormPhase::ChoiceMade {
            info!("Storm choice: effects resolved");
            self.phase = StormPhase::Resolved;
        }
    }

    // ── Queries ────────────────────────────────────────────────────────

    /// Returns the current phase.
    #[must_use]
    pub fn phase(&self) -> StormPhase {
        self.phase
    }

    /// Returns the player's choice, if made.
    #[must_use]
    pub fn choice(&self) -> Option<StormChoice> {
        self.choice
    }

    /// Returns the boss arena modifiers, if computed.
    #[must_use]
    pub fn modifiers(&self) -> Option<&BossArenaModifiers> {
        self.modifiers.as_ref()
    }

    /// Returns `true` if the choice has been made (regardless of resolution).
    #[must_use]
    pub fn is_decided(&self) -> bool {
        self.choice.is_some()
    }

    /// Returns `true` if the full storm flow is resolved.
    #[must_use]
    pub fn is_resolved(&self) -> bool {
        self.phase == StormPhase::Resolved
    }

    /// Returns `true` if the player chose to stabilize.
    #[must_use]
    pub fn chose_stabilize(&self) -> bool {
        self.choice == Some(StormChoice::Stabilize)
    }

    /// Returns `true` if the player chose to redirect.
    #[must_use]
    pub fn chose_redirect(&self) -> bool {
        self.choice == Some(StormChoice::Redirect)
    }

    /// Resets to `PreCrossroads` (used for replay/testing).
    pub fn reset(&mut self) {
        *self = Self::new();
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let state = StormChoiceState::new();
        assert_eq!(state.phase(), StormPhase::PreCrossroads);
        assert!(state.choice().is_none());
        assert!(!state.is_decided());
    }

    #[test]
    fn full_lifecycle_stabilize() {
        let mut state = StormChoiceState::new();
        state.enter_crossroads();
        assert_eq!(state.phase(), StormPhase::DecisionPending);

        assert!(state.make_choice(StormChoice::Stabilize));
        assert_eq!(state.phase(), StormPhase::ChoiceMade);
        assert!(state.chose_stabilize());

        let mods = state.modifiers().unwrap();
        assert_eq!(mods.visibility, 1.0);
        assert!(mods.fog_reinforcements_enabled);
        assert!(mods.warden_armor_bonus > 0.0);

        state.resolve();
        assert!(state.is_resolved());
    }

    #[test]
    fn full_lifecycle_redirect() {
        let mut state = StormChoiceState::new();
        state.enter_crossroads();
        assert!(state.make_choice(StormChoice::Redirect));
        assert!(state.chose_redirect());

        let mods = state.modifiers().unwrap();
        assert!(mods.visibility < 1.0);
        assert!(!mods.fog_reinforcements_enabled);
        assert_eq!(mods.warden_armor_bonus, 0.0);
    }

    #[test]
    fn cannot_choose_before_entering_crossroads() {
        let mut state = StormChoiceState::new();
        assert!(!state.make_choice(StormChoice::Stabilize));
        assert!(!state.is_decided());
    }

    #[test]
    fn cannot_choose_twice() {
        let mut state = StormChoiceState::new();
        state.enter_crossroads();
        assert!(state.make_choice(StormChoice::Stabilize));
        assert!(!state.make_choice(StormChoice::Redirect)); // Second choice rejected
        assert!(state.chose_stabilize()); // First choice persists
    }

    #[test]
    fn reset_clears_everything() {
        let mut state = StormChoiceState::new();
        state.enter_crossroads();
        state.make_choice(StormChoice::Redirect);
        state.resolve();
        state.reset();
        assert_eq!(state.phase(), StormPhase::PreCrossroads);
        assert!(!state.is_decided());
    }

    #[test]
    fn vfx_and_dialogue_suffix() {
        assert_eq!(StormChoice::Stabilize.vfx_name(), "storm_stabilize_blue");
        assert_eq!(StormChoice::Redirect.dialogue_suffix(), "redirect");
    }

    #[test]
    fn boss_modifiers_stabilize() {
        let m = BossArenaModifiers::from_choice(StormChoice::Stabilize);
        assert_eq!(m.arena_vfx, "storm_stabilize_blue");
    }

    #[test]
    fn boss_modifiers_redirect() {
        let m = BossArenaModifiers::from_choice(StormChoice::Redirect);
        assert_eq!(m.arena_vfx, "storm_redirect_amber");
    }
}
