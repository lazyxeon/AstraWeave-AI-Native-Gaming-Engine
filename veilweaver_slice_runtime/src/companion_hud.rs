//! Companion affinity meter — tracks Aria's bond with the player.
//!
//! Pure data model for the companion affinity HUD widget. Affinity rises
//! through positive interactions (following orders, protecting, strategic play)
//! and decays through negative ones (reckless combat, ignoring advice).
//!
//! The presentation layer renders this as a vertical meter with:
//! - **Fill level** — animated bar matching affinity (0.0–1.0)
//! - **Rank icon** — changes at affinity thresholds
//! - **Pulse effect** — when affinity changes
//! - **Tooltip** — current rank name and pending unlock
//!
//! Headless-safe — no rendering code.

use std::collections::VecDeque;
use std::fmt;

// ── Affinity Rank ──────────────────────────────────────────────────────

/// Companion affinity rank tiers.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum AffinityRank {
    /// 0.0–0.2: Companion is wary, minimal cooperation.
    Wary,
    /// 0.2–0.4: Basic cooperation, follows simple orders.
    Cautious,
    /// 0.4–0.6: Solid teamwork, suggests strategies.
    Cooperative,
    /// 0.6–0.8: Strong bond, initiates combo attacks.
    Bonded,
    /// 0.8–1.0: Full trust, unlocks ultimate sync ability.
    Synced,
}

impl fmt::Display for AffinityRank {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Wary => write!(f, "Wary"),
            Self::Cautious => write!(f, "Cautious"),
            Self::Cooperative => write!(f, "Cooperative"),
            Self::Bonded => write!(f, "Bonded"),
            Self::Synced => write!(f, "Synced"),
        }
    }
}

impl AffinityRank {
    /// Returns the rank for a given affinity value (0.0–1.0).
    pub fn from_affinity(value: f32) -> Self {
        match value {
            v if v >= 0.8 => Self::Synced,
            v if v >= 0.6 => Self::Bonded,
            v if v >= 0.4 => Self::Cooperative,
            v if v >= 0.2 => Self::Cautious,
            _ => Self::Wary,
        }
    }

    /// Returns the affinity threshold at which this rank begins.
    pub fn threshold(&self) -> f32 {
        match self {
            Self::Wary => 0.0,
            Self::Cautious => 0.2,
            Self::Cooperative => 0.4,
            Self::Bonded => 0.6,
            Self::Synced => 0.8,
        }
    }

    /// Returns the next rank (if any).
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Wary => Some(Self::Cautious),
            Self::Cautious => Some(Self::Cooperative),
            Self::Cooperative => Some(Self::Bonded),
            Self::Bonded => Some(Self::Synced),
            Self::Synced => None,
        }
    }

    /// Color for the affinity bar (R, G, B).
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            Self::Wary => (0.6, 0.3, 0.3),        // Dim red
            Self::Cautious => (0.8, 0.6, 0.2),    // Amber
            Self::Cooperative => (0.3, 0.7, 0.4), // Green
            Self::Bonded => (0.2, 0.5, 0.9),      // Blue
            Self::Synced => (0.7, 0.3, 0.9),      // Purple
        }
    }

    /// Description of what this rank unlocks.
    pub fn unlock_description(&self) -> &'static str {
        match self {
            Self::Wary => "Companion follows basic movement orders.",
            Self::Cautious => "Companion uses basic support abilities.",
            Self::Cooperative => "Companion suggests strategies and flanking.",
            Self::Bonded => "Companion initiates combo attacks.",
            Self::Synced => "Ultimate sync ability unlocked: Woven Resonance.",
        }
    }
}

// ── Affinity Event ─────────────────────────────────────────────────────

/// An event that modifies companion affinity.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AffinityEvent {
    /// What caused the change.
    pub reason: String,
    /// Amount of change (+positive or -negative).
    pub delta: f32,
    /// Timestamp (game time) when the event occurred.
    pub time: f32,
}

// ── Companion Affinity Meter ───────────────────────────────────────────

/// Complete companion affinity HUD state.
#[derive(Debug, Clone)]
pub struct CompanionAffinityMeter {
    /// Companion display name.
    pub companion_name: String,
    /// Current affinity value (0.0–1.0).
    affinity: f32,
    /// Animated display value (smoothly follows `affinity`).
    display_value: f32,
    /// Current rank.
    rank: AffinityRank,
    /// Previous rank (for detecting rank-up transitions).
    prev_rank: AffinityRank,
    /// Pulse timer (flashes when affinity changes, 0.0 = no pulse).
    pulse_timer: f32,
    /// Recent events log (last N events for tooltip display).
    recent_events: VecDeque<AffinityEvent>,
    /// Maximum number of recent events to retain.
    max_recent_events: usize,
    /// Animation interpolation rate.
    anim_rate: f32,
}

impl CompanionAffinityMeter {
    /// Pulse duration in seconds.
    pub const PULSE_DURATION: f32 = 0.5;

    /// Creates a new affinity meter for the named companion.
    #[must_use]
    pub fn new(companion_name: impl Into<String>, initial_affinity: f32) -> Self {
        let aff = if initial_affinity.is_finite() {
            initial_affinity.clamp(0.0, 1.0)
        } else {
            0.0
        };
        let rank = AffinityRank::from_affinity(aff);
        Self {
            companion_name: companion_name.into(),
            affinity: aff,
            display_value: aff,
            rank,
            prev_rank: rank,
            pulse_timer: 0.0,
            recent_events: VecDeque::new(),
            max_recent_events: 10,
            anim_rate: 4.0,
        }
    }

    /// Apply an affinity change with a reason string.
    ///
    /// Returns the new rank if a rank transition occurred.
    pub fn apply_event(
        &mut self,
        reason: impl Into<String>,
        delta: f32,
        game_time: f32,
    ) -> Option<AffinityRank> {
        self.prev_rank = self.rank;
        self.affinity = (self.affinity + delta).clamp(0.0, 1.0);
        self.rank = AffinityRank::from_affinity(self.affinity);
        self.pulse_timer = Self::PULSE_DURATION;

        self.recent_events.push_back(AffinityEvent {
            reason: reason.into(),
            delta,
            time: game_time,
        });

        // Trim old events — O(1) front eviction via VecDeque.
        while self.recent_events.len() > self.max_recent_events {
            self.recent_events.pop_front();
        }

        if self.rank != self.prev_rank {
            Some(self.rank)
        } else {
            None
        }
    }

    /// Advance animations.
    pub fn tick(&mut self, dt: f32) {
        // Guard against NaN/negative dt to prevent state corruption.
        if !dt.is_finite() || dt < 0.0 {
            return;
        }

        // Animate display value
        let diff = self.affinity - self.display_value;
        if diff.abs() > 0.001 {
            self.display_value += diff * (self.anim_rate * dt).min(1.0);
        } else {
            self.display_value = self.affinity;
        }

        // Pulse timer
        if self.pulse_timer > 0.0 {
            self.pulse_timer = (self.pulse_timer - dt).max(0.0);
        }
    }

    // ── Queries ────────────────────────────────────────────────────

    /// Current affinity (0.0–1.0).
    pub fn affinity(&self) -> f32 {
        self.affinity
    }

    /// Animated display value (0.0–1.0).
    pub fn display_value(&self) -> f32 {
        self.display_value
    }

    /// Current rank.
    pub fn rank(&self) -> AffinityRank {
        self.rank
    }

    /// Returns `true` if the pulse effect is active.
    pub fn is_pulsing(&self) -> bool {
        self.pulse_timer > 0.0
    }

    /// Pulse alpha (1.0 → 0.0 over duration).
    pub fn pulse_alpha(&self) -> f32 {
        if self.pulse_timer > 0.0 {
            self.pulse_timer / Self::PULSE_DURATION
        } else {
            0.0
        }
    }

    /// Progress toward the next rank (0.0–1.0 within current rank band).
    pub fn rank_progress(&self) -> f32 {
        let current_threshold = self.rank.threshold();
        let next_threshold = self.rank.next().map(|r| r.threshold()).unwrap_or(1.0);
        let band = next_threshold - current_threshold;
        if band <= 0.0 {
            return 1.0;
        }
        ((self.affinity - current_threshold) / band).clamp(0.0, 1.0)
    }

    /// Returns `true` if a rank-up just occurred (rank changed since last event).
    pub fn did_rank_change(&self) -> bool {
        self.rank != self.prev_rank
    }

    /// Recent affinity events (newest last).
    pub fn recent_events(&self) -> &VecDeque<AffinityEvent> {
        &self.recent_events
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initial_state() {
        let meter = CompanionAffinityMeter::new("Aria", 0.3);
        assert_eq!(meter.affinity(), 0.3);
        assert_eq!(meter.rank(), AffinityRank::Cautious);
        assert!(!meter.is_pulsing());
        assert!(meter.recent_events().is_empty());
    }

    #[test]
    fn rank_from_affinity_boundaries() {
        assert_eq!(AffinityRank::from_affinity(0.0), AffinityRank::Wary);
        assert_eq!(AffinityRank::from_affinity(0.19), AffinityRank::Wary);
        assert_eq!(AffinityRank::from_affinity(0.2), AffinityRank::Cautious);
        assert_eq!(AffinityRank::from_affinity(0.4), AffinityRank::Cooperative);
        assert_eq!(AffinityRank::from_affinity(0.6), AffinityRank::Bonded);
        assert_eq!(AffinityRank::from_affinity(0.8), AffinityRank::Synced);
        assert_eq!(AffinityRank::from_affinity(1.0), AffinityRank::Synced);
    }

    #[test]
    fn apply_event_changes_affinity() {
        let mut meter = CompanionAffinityMeter::new("Aria", 0.3);
        meter.apply_event("Protected player", 0.1, 1.0);
        assert!((meter.affinity() - 0.4).abs() < 0.01);
        assert_eq!(meter.recent_events().len(), 1);
        assert!(meter.is_pulsing());
    }

    #[test]
    fn rank_up_detected() {
        let mut meter = CompanionAffinityMeter::new("Aria", 0.38);
        let rank_change = meter.apply_event("Great combo", 0.05, 2.0);
        assert_eq!(rank_change, Some(AffinityRank::Cooperative));
        assert!(meter.did_rank_change());
    }

    #[test]
    fn rank_down_detected() {
        let mut meter = CompanionAffinityMeter::new("Aria", 0.41);
        let rank_change = meter.apply_event("Reckless attack", -0.05, 3.0);
        assert_eq!(rank_change, Some(AffinityRank::Cautious));
    }

    #[test]
    fn affinity_clamps() {
        let mut meter = CompanionAffinityMeter::new("Aria", 0.95);
        meter.apply_event("Overcharge", 0.5, 1.0);
        assert_eq!(meter.affinity(), 1.0);

        let mut meter = CompanionAffinityMeter::new("Aria", 0.05);
        meter.apply_event("Total betrayal", -0.5, 1.0);
        assert_eq!(meter.affinity(), 0.0);
    }

    #[test]
    fn rank_progress_within_band() {
        let meter = CompanionAffinityMeter::new("Aria", 0.5);
        // Cooperative band: 0.4 → 0.6, at 0.5 = 50% through
        let progress = meter.rank_progress();
        assert!((progress - 0.5).abs() < 0.01);
    }

    #[test]
    fn rank_progress_at_max() {
        let meter = CompanionAffinityMeter::new("Aria", 1.0);
        assert_eq!(meter.rank(), AffinityRank::Synced);
        assert_eq!(meter.rank_progress(), 1.0);
    }

    #[test]
    fn animation_converges() {
        let mut meter = CompanionAffinityMeter::new("Aria", 0.0);
        meter.apply_event("Boost", 0.8, 1.0);
        for _ in 0..200 {
            meter.tick(0.016);
        }
        assert!((meter.display_value() - 0.8).abs() < 0.01);
    }

    #[test]
    fn pulse_decays() {
        let mut meter = CompanionAffinityMeter::new("Aria", 0.5);
        meter.apply_event("Test", 0.1, 0.0);
        assert!(meter.is_pulsing());
        assert!(meter.pulse_alpha() > 0.9);

        meter.tick(0.6);
        assert!(!meter.is_pulsing());
        assert_eq!(meter.pulse_alpha(), 0.0);
    }

    #[test]
    fn recent_events_capped() {
        let mut meter = CompanionAffinityMeter::new("Aria", 0.5);
        for i in 0..20 {
            meter.apply_event(format!("event_{}", i), 0.001, i as f32);
        }
        assert_eq!(meter.recent_events().len(), 10);
    }

    #[test]
    fn rank_display_names() {
        assert_eq!(format!("{}", AffinityRank::Wary), "Wary");
        assert_eq!(format!("{}", AffinityRank::Synced), "Synced");
    }

    #[test]
    fn rank_next_chain() {
        let mut rank = AffinityRank::Wary;
        let mut chain = vec![rank];
        while let Some(next) = rank.next() {
            chain.push(next);
            rank = next;
        }
        assert_eq!(chain.len(), 5);
        assert_eq!(*chain.last().unwrap(), AffinityRank::Synced);
    }

    #[test]
    fn rank_colors_and_descriptions() {
        for rank in &[
            AffinityRank::Wary,
            AffinityRank::Cautious,
            AffinityRank::Cooperative,
            AffinityRank::Bonded,
            AffinityRank::Synced,
        ] {
            let (r, g, b) = rank.color();
            assert!((0.0..=1.0).contains(&r));
            assert!((0.0..=1.0).contains(&g));
            assert!((0.0..=1.0).contains(&b));
            assert!(!rank.unlock_description().is_empty());
        }
    }

    #[test]
    fn nan_initial_affinity_defaults_to_zero() {
        let meter = CompanionAffinityMeter::new("Lira", f32::NAN);
        assert_eq!(meter.affinity(), 0.0);
        assert_eq!(meter.display_value(), 0.0);
        assert_eq!(meter.rank(), AffinityRank::Wary);
    }

    #[test]
    fn nan_dt_tick_does_not_corrupt_state() {
        let mut meter = CompanionAffinityMeter::new("Lira", 0.5);
        let val_before = meter.display_value();
        meter.tick(f32::NAN);
        assert_eq!(meter.display_value(), val_before);
    }

    #[test]
    fn negative_dt_tick_does_not_corrupt_state() {
        let mut meter = CompanionAffinityMeter::new("Lira", 0.5);
        let val_before = meter.display_value();
        meter.tick(-5.0);
        assert_eq!(meter.display_value(), val_before);
    }

    #[test]
    fn infinity_initial_affinity_clamps_to_one() {
        let meter = CompanionAffinityMeter::new("Lira", f32::INFINITY);
        // Infinity is not finite, so it defaults to 0.0.
        assert_eq!(meter.affinity(), 0.0);
    }
}
