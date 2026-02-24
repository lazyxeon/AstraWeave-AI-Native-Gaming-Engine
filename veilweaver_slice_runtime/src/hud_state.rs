//! Thread HUD widget — stability bar + echo counter.
//!
//! Pure data model for the Veilweaver in-game HUD. The presentation layer
//! (egui / GPU renderer) reads this state each frame to draw:
//!
//! - **Stability bar**: Per-anchor stability (0.0–1.0), color-coded, animated
//! - **Echo counter**: Current echo currency balance with transaction feedback
//!
//! This module contains no rendering code — it is headless-safe.

use std::collections::BTreeMap;

// ── Stability Bar ──────────────────────────────────────────────────────

/// Visual state of a single anchor's stability bar.
#[derive(Debug, Clone)]
pub struct AnchorStabilityEntry {
    /// Unique anchor identifier (e.g., `"Z0_sanctum_core"`).
    pub anchor_id: String,
    /// Current stability value (0.0 = broken, 1.0 = perfect).
    pub stability: f32,
    /// Animated display value (smoothly interpolated toward `stability`).
    pub display_value: f32,
    /// `true` while the anchor is being repaired (drives glow VFX).
    pub is_repairing: bool,
    /// `true` once the anchor has been repaired at least once.
    pub was_repaired: bool,
}

impl AnchorStabilityEntry {
    /// Creates a new entry with immediate display (no animation lag).
    #[must_use]
    pub fn new(anchor_id: impl Into<String>, stability: f32) -> Self {
        let s = stability.clamp(0.0, 1.0);
        Self {
            anchor_id: anchor_id.into(),
            stability: s,
            display_value: s,
            is_repairing: false,
            was_repaired: false,
        }
    }

    /// Updates the target stability and begins animation.
    pub fn set_stability(&mut self, value: f32) {
        self.stability = value.clamp(0.0, 1.0);
    }

    /// Advances the animated display value toward the target.
    ///
    /// `rate` is the interpolation speed (e.g., `5.0` = converge in ~0.2s).
    pub fn animate(&mut self, dt: f32, rate: f32) {
        // Guard against NaN/negative dt to prevent state corruption.
        if !dt.is_finite() || dt < 0.0 || !rate.is_finite() || rate < 0.0 {
            return;
        }
        let diff = self.stability - self.display_value;
        if diff.abs() < 0.001 {
            self.display_value = self.stability;
        } else {
            self.display_value += diff * (rate * dt).min(1.0);
        }
    }

    /// Returns the color tier based on current stability.
    ///
    /// - `>= 0.8` → `Green`
    /// - `>= 0.5` → `Yellow`
    /// - `>= 0.2` → `Orange`
    /// - `< 0.2` → `Red`
    pub fn color_tier(&self) -> StabilityColor {
        match self.display_value {
            v if v >= 0.8 => StabilityColor::Green,
            v if v >= 0.5 => StabilityColor::Yellow,
            v if v >= 0.2 => StabilityColor::Orange,
            _ => StabilityColor::Red,
        }
    }
}

/// Color tier for stability bar rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StabilityColor {
    Green,
    Yellow,
    Orange,
    Red,
}

impl std::fmt::Display for StabilityColor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Green => write!(f, "Green"),
            Self::Yellow => write!(f, "Yellow"),
            Self::Orange => write!(f, "Orange"),
            Self::Red => write!(f, "Red"),
        }
    }
}

// ── Echo Counter ───────────────────────────────────────────────────────

/// Transaction feedback float — small +N / -N indicator that fades out.
#[derive(Debug, Clone, PartialEq)]
pub struct EchoFeedback {
    /// Amount changed (+N or -N).
    pub amount: i32,
    /// Seconds since creation.
    pub age: f32,
    /// Alpha opacity (1.0 → 0.0 over lifetime).
    pub alpha: f32,
}

impl EchoFeedback {
    /// Feedback lifetime in seconds before it expires.
    pub const LIFETIME: f32 = 2.0;

    /// Creates a new feedback float with full opacity.
    #[must_use]
    pub fn new(amount: i32) -> Self {
        Self {
            amount,
            age: 0.0,
            alpha: 1.0,
        }
    }

    /// Advance the feedback animation.
    pub fn tick(&mut self, dt: f32) {
        // Guard against NaN/negative dt to prevent state corruption.
        if !dt.is_finite() || dt < 0.0 {
            return;
        }
        self.age += dt;
        let progress = (self.age / Self::LIFETIME).min(1.0);
        // Fade in first 25%, then fade out.
        self.alpha = if progress < 0.25 {
            progress / 0.25
        } else {
            1.0 - ((progress - 0.25) / 0.75)
        };
    }

    /// Returns `true` if this feedback has expired.
    pub fn is_expired(&self) -> bool {
        self.age >= Self::LIFETIME
    }

    /// Returns `true` if this was a gain (positive amount).
    pub fn is_gain(&self) -> bool {
        self.amount > 0
    }
}

/// Echo counter state — current balance + transaction feedback.
#[derive(Debug, Clone)]
pub struct EchoCounter {
    /// Current echo balance.
    pub balance: u32,
    /// Active feedback floats.
    pub feedbacks: Vec<EchoFeedback>,
}

impl Default for EchoCounter {
    fn default() -> Self {
        Self::new(0)
    }
}

impl EchoCounter {
    /// Creates an echo counter with the given starting balance.
    #[must_use]
    pub fn new(initial_balance: u32) -> Self {
        Self {
            balance: initial_balance,
            feedbacks: Vec::new(),
        }
    }

    /// Record a transaction (gain or spend).
    pub fn transact(&mut self, amount: i32) {
        if amount > 0 {
            self.balance = self.balance.saturating_add(amount as u32);
        } else {
            self.balance = self.balance.saturating_sub(amount.unsigned_abs());
        }
        self.feedbacks.push(EchoFeedback::new(amount));
    }

    /// Advance all feedback animations and prune expired entries.
    pub fn tick(&mut self, dt: f32) {
        for fb in &mut self.feedbacks {
            fb.tick(dt);
        }
        self.feedbacks.retain(|fb| !fb.is_expired());
    }

    /// Number of active (visible) feedback floats.
    pub fn active_feedback_count(&self) -> usize {
        self.feedbacks.len()
    }
}

// ── Unified Thread HUD ────────────────────────────────────────────────

/// Combined HUD state for the Veilweaver thread/weaving overlay.
///
/// Tracks all anchor stability bars and the echo counter in one struct
/// so the presentation layer can read a single snapshot per frame.
#[derive(Debug, Clone)]
pub struct ThreadHud {
    /// Per-anchor stability entries, keyed by anchor ID.
    pub anchors: BTreeMap<String, AnchorStabilityEntry>,
    /// Echo currency display.
    pub echoes: EchoCounter,
    /// Animation interpolation rate (default 5.0).
    pub anim_rate: f32,
}

impl Default for ThreadHud {
    fn default() -> Self {
        Self {
            anchors: BTreeMap::new(),
            echoes: EchoCounter::new(0),
            anim_rate: 5.0,
        }
    }
}

impl ThreadHud {
    /// Create a new ThreadHud with the given initial echo balance.
    #[must_use]
    pub fn new(initial_echoes: u32) -> Self {
        Self {
            anchors: BTreeMap::new(),
            echoes: EchoCounter::new(initial_echoes),
            anim_rate: 5.0,
        }
    }

    /// Register an anchor for HUD tracking.
    pub fn add_anchor(&mut self, anchor_id: impl Into<String>, stability: f32) {
        let id = anchor_id.into();
        self.anchors
            .insert(id.clone(), AnchorStabilityEntry::new(id, stability));
    }

    /// Update an anchor's stability (e.g., after repair or decay).
    pub fn update_anchor_stability(&mut self, anchor_id: &str, stability: f32) {
        if let Some(entry) = self.anchors.get_mut(anchor_id) {
            entry.set_stability(stability);
        }
    }

    /// Mark an anchor as currently repairing (drives glow animation).
    pub fn set_anchor_repairing(&mut self, anchor_id: &str, repairing: bool) {
        if let Some(entry) = self.anchors.get_mut(anchor_id) {
            entry.is_repairing = repairing;
            if !repairing {
                entry.was_repaired = true;
            }
        }
    }

    /// Record an echo transaction.
    pub fn echo_transact(&mut self, amount: i32) {
        self.echoes.transact(amount);
    }

    /// Advance all animations (call once per frame).
    pub fn tick(&mut self, dt: f32) {
        let rate = self.anim_rate;
        for entry in self.anchors.values_mut() {
            entry.animate(dt, rate);
        }
        self.echoes.tick(dt);
    }

    /// Returns the average stability across all tracked anchors.
    ///
    /// Returns `0.0` if no anchors are registered.
    pub fn average_stability(&self) -> f32 {
        if self.anchors.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.anchors.values().map(|a| a.stability).sum();
        sum / self.anchors.len() as f32
    }

    /// Returns the number of anchors that have been repaired at least once.
    pub fn repaired_count(&self) -> usize {
        self.anchors.values().filter(|a| a.was_repaired).count()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn anchor_stability_entry_basics() {
        let mut entry = AnchorStabilityEntry::new("test_anchor", 0.5);
        assert_eq!(entry.stability, 0.5);
        assert_eq!(entry.display_value, 0.5);
        assert_eq!(entry.color_tier(), StabilityColor::Yellow);

        entry.set_stability(0.9);
        assert_eq!(entry.stability, 0.9);
        // display_value hasn't animated yet
        assert_eq!(entry.display_value, 0.5);
    }

    #[test]
    fn anchor_stability_clamps() {
        let entry = AnchorStabilityEntry::new("a", -0.5);
        assert_eq!(entry.stability, 0.0);

        let entry = AnchorStabilityEntry::new("b", 2.0);
        assert_eq!(entry.stability, 1.0);
    }

    #[test]
    fn anchor_animate_converges() {
        let mut entry = AnchorStabilityEntry::new("a", 0.0);
        entry.set_stability(1.0);
        // Animate in large steps
        for _ in 0..100 {
            entry.animate(0.1, 5.0);
        }
        assert!((entry.display_value - 1.0).abs() < 0.01);
    }

    #[test]
    fn color_tiers_correct() {
        let mut e = AnchorStabilityEntry::new("a", 0.9);
        assert_eq!(e.color_tier(), StabilityColor::Green);

        e.display_value = 0.6;
        assert_eq!(e.color_tier(), StabilityColor::Yellow);

        e.display_value = 0.3;
        assert_eq!(e.color_tier(), StabilityColor::Orange);

        e.display_value = 0.1;
        assert_eq!(e.color_tier(), StabilityColor::Red);
    }

    #[test]
    fn echo_counter_transact() {
        let mut counter = EchoCounter::new(10);
        counter.transact(5);
        assert_eq!(counter.balance, 15);
        counter.transact(-3);
        assert_eq!(counter.balance, 12);
        assert_eq!(counter.active_feedback_count(), 2);
    }

    #[test]
    fn echo_counter_saturating_subtract() {
        let mut counter = EchoCounter::new(5);
        counter.transact(-100);
        assert_eq!(counter.balance, 0);
    }

    #[test]
    fn echo_feedback_lifecycle() {
        let mut fb = EchoFeedback::new(10);
        assert!(fb.is_gain());
        assert!(!fb.is_expired());
        assert_eq!(fb.alpha, 1.0);

        // Advance past lifetime
        fb.tick(2.5);
        assert!(fb.is_expired());
    }

    #[test]
    fn echo_counter_tick_prunes_expired() {
        let mut counter = EchoCounter::new(0);
        counter.transact(5);
        counter.transact(-2);
        assert_eq!(counter.active_feedback_count(), 2);

        // Tick past the entire lifetime
        counter.tick(3.0);
        assert_eq!(counter.active_feedback_count(), 0);
    }

    #[test]
    fn thread_hud_full_lifecycle() {
        let mut hud = ThreadHud::new(20);
        hud.add_anchor("Z0_core", 0.4);
        hud.add_anchor("Z1_left", 0.7);
        hud.add_anchor("Z2_vista", 0.1);

        // Update stability
        hud.update_anchor_stability("Z0_core", 0.95);
        assert_eq!(hud.anchors["Z0_core"].stability, 0.95);

        // Animate a bit
        hud.tick(0.1);

        // Average should reflect actual (not animated) values
        let avg = hud.average_stability();
        let expected = (0.95 + 0.7 + 0.1) / 3.0;
        assert!((avg - expected).abs() < 0.01);

        // Repair tracking
        hud.set_anchor_repairing("Z0_core", true);
        assert!(hud.anchors["Z0_core"].is_repairing);
        hud.set_anchor_repairing("Z0_core", false);
        assert!(hud.anchors["Z0_core"].was_repaired);
        assert_eq!(hud.repaired_count(), 1);

        // Echo transaction
        hud.echo_transact(5);
        assert_eq!(hud.echoes.balance, 25);
    }

    #[test]
    fn thread_hud_average_empty() {
        let hud = ThreadHud::default();
        assert_eq!(hud.average_stability(), 0.0);
    }

    #[test]
    fn thread_hud_unknown_anchor_update_noop() {
        let mut hud = ThreadHud::default();
        hud.update_anchor_stability("nonexistent", 1.0);
        assert!(hud.anchors.is_empty());
    }

    #[test]
    fn animate_nan_dt_does_not_corrupt_state() {
        let mut entry = AnchorStabilityEntry::new("test", 0.8);
        entry.set_stability(0.5);
        let before = entry.display_value;
        entry.animate(f32::NAN, 5.0);
        assert_eq!(entry.display_value, before);
    }

    #[test]
    fn animate_nan_rate_does_not_corrupt_state() {
        let mut entry = AnchorStabilityEntry::new("test", 0.8);
        entry.set_stability(0.5);
        let before = entry.display_value;
        entry.animate(0.016, f32::NAN);
        assert_eq!(entry.display_value, before);
    }

    #[test]
    fn animate_negative_dt_does_not_corrupt_state() {
        let mut entry = AnchorStabilityEntry::new("test", 0.8);
        entry.set_stability(0.5);
        let before = entry.display_value;
        entry.animate(-1.0, 5.0);
        assert_eq!(entry.display_value, before);
    }
}
