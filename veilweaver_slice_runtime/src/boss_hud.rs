//! Boss health bar with phase indicators.
//!
//! Pure data model for the Oathbound Warden boss HUD. The presentation layer
//! renders this as a wide health bar with:
//!
//! - **Phase segments** — colored sub-bars showing HP thresholds per phase
//! - **Animated damage** — smooth drain effect when boss takes damage
//! - **Telegraph warnings** — icon / text flash for incoming abilities
//! - **Phase name** — current phase label above the bar
//!
//! This module is headless-safe and contains no rendering code.

use std::fmt;

// ── Boss Phase ─────────────────────────────────────────────────────────

/// Warden boss encounter phases (mirrors the director's phases but
/// adds presentation-side data).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum BossPhase {
    /// Phase 1: Boss assesses the player's abilities.
    Assessment,
    /// Phase 2: Boss adapts strategy, gains new abilities.
    FulcrumShift,
    /// Phase 3: Boss enters override mode — most dangerous phase.
    DirectiveOverride,
}

impl fmt::Display for BossPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Assessment => write!(f, "Assessment"),
            Self::FulcrumShift => write!(f, "Fulcrum Shift"),
            Self::DirectiveOverride => write!(f, "Directive Override"),
        }
    }
}

impl BossPhase {
    /// Returns the color (R, G, B) associated with this phase.
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            Self::Assessment => (0.3, 0.8, 0.4),        // Green-ish
            Self::FulcrumShift => (0.9, 0.7, 0.2),      // Gold
            Self::DirectiveOverride => (0.9, 0.2, 0.2), // Red
        }
    }

    /// Phase ordering index (0, 1, 2).
    pub fn index(&self) -> usize {
        match self {
            Self::Assessment => 0,
            Self::FulcrumShift => 1,
            Self::DirectiveOverride => 2,
        }
    }
}

// ── Phase Threshold ────────────────────────────────────────────────────

/// Defines the HP threshold where a phase transition occurs.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct PhaseThreshold {
    /// Which phase this threshold represents.
    pub phase: BossPhase,
    /// HP percentage (0.0–1.0) at which this phase begins.
    /// E.g., Assessment = 1.0 (start), FulcrumShift = 0.6, Override = 0.3.
    pub hp_start: f32,
    /// HP percentage at which this phase ends.
    pub hp_end: f32,
}

// ── Telegraph ──────────────────────────────────────────────────────────

/// A boss telegraph — a brief warning before a major attack.
#[derive(Debug, Clone, PartialEq)]
pub struct TelegraphWarning {
    /// Ability name (e.g., `"Cleave"`, `"Chain Lash"`, `"Anchor Rupture"`).
    pub ability_name: String,
    /// Warning text shown to the player.
    pub warning_text: String,
    /// Time remaining before the attack lands (seconds).
    pub time_remaining: f32,
    /// Total wind-up duration (used for progress bar).
    pub total_duration: f32,
}

impl TelegraphWarning {
    /// Creates a telegraph warning with the given ability name and wind-up duration.
    #[must_use]
    pub fn new(
        ability_name: impl Into<String>,
        warning_text: impl Into<String>,
        duration: f32,
    ) -> Self {
        Self {
            ability_name: ability_name.into(),
            warning_text: warning_text.into(),
            time_remaining: duration,
            total_duration: duration,
        }
    }

    /// Advance the telegraph timer. Returns `true` if expired.
    pub fn tick(&mut self, dt: f32) -> bool {
        self.time_remaining = (self.time_remaining - dt).max(0.0);
        self.time_remaining <= 0.0
    }

    /// Progress (0.0 = just started, 1.0 = about to fire).
    pub fn progress(&self) -> f32 {
        if self.total_duration <= 0.0 {
            return 1.0;
        }
        1.0 - (self.time_remaining / self.total_duration)
    }

    /// Urgency color (yellow → orange → red as progress increases).
    pub fn urgency_color(&self) -> (f32, f32, f32) {
        let p = self.progress();
        if p < 0.5 {
            (1.0, 0.9, 0.2) // Yellow
        } else if p < 0.8 {
            (1.0, 0.5, 0.1) // Orange
        } else {
            (1.0, 0.15, 0.15) // Red
        }
    }
}

// ── Boss Health Bar ────────────────────────────────────────────────────

/// Complete boss HUD state.
#[derive(Debug, Clone)]
pub struct BossHealthBar {
    /// Boss display name.
    pub boss_name: String,
    /// Maximum HP.
    pub max_hp: f32,
    /// Current HP.
    pub current_hp: f32,
    /// Animated display HP (smoothly follows `current_hp`).
    pub display_hp: f32,
    /// Recent damage (for the delayed "drain" bar effect).
    pub damage_display: f32,
    /// Current phase.
    pub phase: BossPhase,
    /// Phase transition thresholds.
    pub thresholds: Vec<PhaseThreshold>,
    /// Active telegraph warnings.
    pub telegraphs: Vec<TelegraphWarning>,
    /// Flash timer (damage taken flash, 0.0 = no flash).
    pub flash_timer: f32,
    /// Animation interpolation rate.
    anim_rate: f32,
    /// Drain bar animation rate (slower than main bar).
    drain_rate: f32,
}

impl BossHealthBar {
    /// Creates a new boss health bar with default Warden phase thresholds.
    #[must_use]
    pub fn new(boss_name: impl Into<String>, max_hp: f32) -> Self {
        Self {
            boss_name: boss_name.into(),
            max_hp,
            current_hp: max_hp,
            display_hp: max_hp,
            damage_display: max_hp,
            phase: BossPhase::Assessment,
            thresholds: Self::default_thresholds(),
            telegraphs: Vec::new(),
            flash_timer: 0.0,
            anim_rate: 8.0,
            drain_rate: 2.0,
        }
    }

    /// Default Oathbound Warden phase thresholds.
    fn default_thresholds() -> Vec<PhaseThreshold> {
        vec![
            PhaseThreshold {
                phase: BossPhase::Assessment,
                hp_start: 1.0,
                hp_end: 0.6,
            },
            PhaseThreshold {
                phase: BossPhase::FulcrumShift,
                hp_start: 0.6,
                hp_end: 0.3,
            },
            PhaseThreshold {
                phase: BossPhase::DirectiveOverride,
                hp_start: 0.3,
                hp_end: 0.0,
            },
        ]
    }

    /// Set the current HP (triggers animation + damage flash).
    pub fn set_hp(&mut self, hp: f32) {
        let new_hp = hp.clamp(0.0, self.max_hp);
        if new_hp < self.current_hp {
            self.flash_timer = 0.2; // 200ms flash
        }
        self.current_hp = new_hp;
        self.update_phase();
    }

    /// Apply damage to the boss.
    ///
    /// NaN, infinity, and negative values are silently rejected.
    pub fn apply_damage(&mut self, amount: f32) {
        if !amount.is_finite() || amount < 0.0 {
            return;
        }
        self.set_hp(self.current_hp - amount);
    }

    /// Add a telegraph warning.
    pub fn add_telegraph(
        &mut self,
        ability_name: impl Into<String>,
        warning_text: impl Into<String>,
        duration: f32,
    ) {
        self.telegraphs
            .push(TelegraphWarning::new(ability_name, warning_text, duration));
    }

    /// Advance animations and telegraph timers.
    pub fn tick(&mut self, dt: f32) {
        // Guard against NaN/negative dt to prevent state corruption.
        if !dt.is_finite() || dt < 0.0 {
            return;
        }

        // Animate main HP bar
        let hp_diff = self.current_hp - self.display_hp;
        if hp_diff.abs() > 0.1 {
            self.display_hp += hp_diff * (self.anim_rate * dt).min(1.0);
        } else {
            self.display_hp = self.current_hp;
        }

        // Animate drain bar (delayed, slower)
        let drain_diff = self.display_hp - self.damage_display;
        if drain_diff.abs() > 0.1 {
            self.damage_display += drain_diff * (self.drain_rate * dt).min(1.0);
        } else {
            self.damage_display = self.display_hp;
        }

        // Flash timer
        if self.flash_timer > 0.0 {
            self.flash_timer = (self.flash_timer - dt).max(0.0);
        }

        // Advance telegraphs, remove expired
        for t in &mut self.telegraphs {
            t.tick(dt);
        }
        self.telegraphs.retain(|t| t.time_remaining > 0.0);
    }

    // ── Queries ────────────────────────────────────────────────────

    /// HP fraction (0.0–1.0).
    pub fn hp_fraction(&self) -> f32 {
        if !self.max_hp.is_finite() || self.max_hp <= 0.0 {
            return 0.0;
        }
        (self.current_hp / self.max_hp).clamp(0.0, 1.0)
    }

    /// Display HP fraction (animated, for the main bar).
    pub fn display_fraction(&self) -> f32 {
        if !self.max_hp.is_finite() || self.max_hp <= 0.0 {
            return 0.0;
        }
        (self.display_hp / self.max_hp).clamp(0.0, 1.0)
    }

    /// Damage display fraction (delayed drain bar).
    pub fn drain_fraction(&self) -> f32 {
        if !self.max_hp.is_finite() || self.max_hp <= 0.0 {
            return 0.0;
        }
        (self.damage_display / self.max_hp).clamp(0.0, 1.0)
    }

    /// Returns `true` if the damage flash is active.
    pub fn is_flashing(&self) -> bool {
        self.flash_timer > 0.0
    }

    /// Returns `true` if the boss is defeated (0 HP).
    pub fn is_defeated(&self) -> bool {
        self.current_hp <= 0.0
    }

    /// Current phase label.
    pub fn phase_label(&self) -> &str {
        match self.phase {
            BossPhase::Assessment => "Assessment",
            BossPhase::FulcrumShift => "Fulcrum Shift",
            BossPhase::DirectiveOverride => "Directive Override",
        }
    }

    /// Returns the threshold HP fractions as segment boundaries
    /// for the presentation layer to draw phase markers on the bar.
    pub fn phase_markers(&self) -> Vec<(f32, BossPhase)> {
        self.thresholds
            .iter()
            .map(|t| (t.hp_start, t.phase))
            .collect()
    }

    // ── Internal ───────────────────────────────────────────────────

    fn update_phase(&mut self) {
        let frac = self.hp_fraction();
        if frac <= 0.0 {
            // Defeated — keep last phase
            return;
        }
        // Phase determined by which threshold range the HP falls into
        for threshold in &self.thresholds {
            if frac <= threshold.hp_start && frac > threshold.hp_end {
                self.phase = threshold.phase;
                return;
            }
        }
        // Edge case: at exactly 0.0, use the last phase
        if let Some(last) = self.thresholds.last() {
            self.phase = last.phase;
        }
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_boss_bar_full_hp() {
        let bar = BossHealthBar::new("Oathbound Warden", 1000.0);
        assert_eq!(bar.current_hp, 1000.0);
        assert_eq!(bar.hp_fraction(), 1.0);
        assert_eq!(bar.phase, BossPhase::Assessment);
        assert!(!bar.is_defeated());
    }

    #[test]
    fn damage_triggers_phase_transition() {
        let mut bar = BossHealthBar::new("Warden", 1000.0);

        // Stay in Assessment above 60%
        bar.set_hp(700.0);
        assert_eq!(bar.phase, BossPhase::Assessment);

        // Drop into FulcrumShift (60%-30%)
        bar.set_hp(500.0);
        assert_eq!(bar.phase, BossPhase::FulcrumShift);

        // Drop into DirectiveOverride (<30%)
        bar.set_hp(200.0);
        assert_eq!(bar.phase, BossPhase::DirectiveOverride);
    }

    #[test]
    fn apply_damage_accumulates() {
        let mut bar = BossHealthBar::new("Warden", 1000.0);
        bar.apply_damage(300.0);
        assert_eq!(bar.current_hp, 700.0);
        bar.apply_damage(500.0);
        assert_eq!(bar.current_hp, 200.0);
    }

    #[test]
    fn damage_flash_activates() {
        let mut bar = BossHealthBar::new("Warden", 1000.0);
        bar.set_hp(800.0);
        assert!(bar.is_flashing());

        // Tick past flash duration
        bar.tick(0.3);
        assert!(!bar.is_flashing());
    }

    #[test]
    fn hp_clamps_to_zero() {
        let mut bar = BossHealthBar::new("Warden", 1000.0);
        bar.apply_damage(2000.0);
        assert_eq!(bar.current_hp, 0.0);
        assert!(bar.is_defeated());
    }

    #[test]
    fn display_hp_animates_toward_current() {
        let mut bar = BossHealthBar::new("Warden", 1000.0);
        bar.apply_damage(500.0);
        // display_hp still at 1000.0
        assert_eq!(bar.display_hp, 1000.0);

        // Animate
        for _ in 0..100 {
            bar.tick(0.05);
        }
        assert!((bar.display_hp - 500.0).abs() < 1.0);
    }

    #[test]
    fn drain_bar_lags_behind() {
        let mut bar = BossHealthBar::new("Warden", 1000.0);
        bar.apply_damage(500.0);

        // A single tick — damage_display should still be near max because
        // the drain bar follows display_hp which itself is still animating
        bar.tick(0.016);
        assert!(bar.damage_display > 900.0);
    }

    #[test]
    fn telegraph_lifecycle() {
        let mut bar = BossHealthBar::new("Warden", 1000.0);
        bar.add_telegraph("Cleave", "Dodge left!", 2.0);
        assert_eq!(bar.telegraphs.len(), 1);
        assert!((bar.telegraphs[0].progress() - 0.0).abs() < 0.01);

        // Advance halfway
        bar.tick(1.0);
        assert!((bar.telegraphs[0].progress() - 0.5).abs() < 0.01);

        // Expire
        bar.tick(1.5);
        assert_eq!(bar.telegraphs.len(), 0);
    }

    #[test]
    fn telegraph_urgency_colors() {
        let mut t = TelegraphWarning::new("test", "watch out", 1.0);
        // At start: yellow
        let (r, _, _) = t.urgency_color();
        assert_eq!(r, 1.0);

        // At 60%: orange
        t.tick(0.6);
        let color = t.urgency_color();
        assert!((color.1 - 0.5).abs() < 0.01);

        // At 90%: red
        t.tick(0.3);
        let color = t.urgency_color();
        assert!(color.1 < 0.2);
    }

    #[test]
    fn phase_markers_correct() {
        let bar = BossHealthBar::new("Warden", 1000.0);
        let markers = bar.phase_markers();
        assert_eq!(markers.len(), 3);
        assert_eq!(markers[0].0, 1.0);
        assert_eq!(markers[0].1, BossPhase::Assessment);
        assert_eq!(markers[1].0, 0.6);
        assert_eq!(markers[2].0, 0.3);
    }

    #[test]
    fn phase_display_names() {
        assert_eq!(format!("{}", BossPhase::Assessment), "Assessment");
        assert_eq!(format!("{}", BossPhase::FulcrumShift), "Fulcrum Shift");
        assert_eq!(
            format!("{}", BossPhase::DirectiveOverride),
            "Directive Override"
        );
    }

    #[test]
    fn phase_colors_and_indices() {
        assert_eq!(BossPhase::Assessment.index(), 0);
        assert_eq!(BossPhase::FulcrumShift.index(), 1);
        assert_eq!(BossPhase::DirectiveOverride.index(), 2);

        let (r, _, _) = BossPhase::DirectiveOverride.color();
        assert!(r > 0.8); // Red-ish
    }

    #[test]
    fn zero_max_hp_fractions() {
        let bar = BossHealthBar::new("Zero", 0.0);
        assert_eq!(bar.hp_fraction(), 0.0);
        assert_eq!(bar.display_fraction(), 0.0);
        assert_eq!(bar.drain_fraction(), 0.0);
    }

    #[test]
    fn nan_max_hp_returns_zero_fractions() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.max_hp = f32::NAN;
        assert_eq!(bar.hp_fraction(), 0.0);
        assert_eq!(bar.display_fraction(), 0.0);
        assert_eq!(bar.drain_fraction(), 0.0);
    }

    #[test]
    fn infinity_max_hp_returns_zero_fractions() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.max_hp = f32::INFINITY;
        assert_eq!(bar.hp_fraction(), 0.0);
        assert_eq!(bar.display_fraction(), 0.0);
        assert_eq!(bar.drain_fraction(), 0.0);
    }

    #[test]
    fn nan_dt_tick_does_not_corrupt_state() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(30.0);
        let hp_before = bar.display_hp;
        bar.tick(f32::NAN);
        // State should be unchanged — NaN dt is rejected.
        assert_eq!(bar.display_hp, hp_before);
    }

    #[test]
    fn negative_dt_tick_does_not_corrupt_state() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(30.0);
        let hp_before = bar.display_hp;
        bar.tick(-1.0);
        assert_eq!(bar.display_hp, hp_before);
    }

    #[test]
    fn fractions_clamped_to_0_1() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.current_hp = 150.0; // Over-heal
        assert!(bar.hp_fraction() <= 1.0);
        bar.current_hp = -10.0; // Under zero
        assert!(bar.hp_fraction() >= 0.0);
    }
}
