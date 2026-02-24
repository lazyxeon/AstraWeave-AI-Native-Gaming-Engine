//! Decision radial UI state — Z3 storm choice widget.
//!
//! Pure data model for a radial decision prompt. The presentation layer renders
//! this as a radial/wheel selector with animated segments, countdown timer,
//! and contextual descriptions.
//!
//! Designed for the Z3 Loom Crossroads storm routing decision, but generic
//! enough for any N-option radial choice.

use std::fmt;

// ── Decision Option ────────────────────────────────────────────────────

/// A single option in the radial decision widget.
#[derive(Debug, Clone, PartialEq)]
pub struct DecisionOption {
    /// Machine-readable key (e.g., `"stabilize"`, `"redirect"`).
    pub key: String,
    /// Player-facing label (e.g., `"Stabilize the Loom"`).
    pub label: String,
    /// Short description shown on hover/focus.
    pub description: String,
    /// Icon identifier for the presentation layer.
    pub icon: String,
    /// Highlight color as (R, G, B) in 0.0–1.0 range.
    pub color: (f32, f32, f32),
}

impl DecisionOption {
    /// Creates a new decision option with key, label, description, icon, and color.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        description: impl Into<String>,
        icon: impl Into<String>,
        color: (f32, f32, f32),
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            description: description.into(),
            icon: icon.into(),
            color,
        }
    }
}

// ── Decision Phase ─────────────────────────────────────────────────────

/// Current phase of the decision radial.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecisionPhase {
    /// Widget is hidden / inactive.
    Hidden,
    /// Opening animation (expanding from center).
    Opening,
    /// Fully visible, awaiting player input.
    Active,
    /// Player has selected an option, closing animation.
    Closing,
    /// Decision is complete and widget should be removed.
    Done,
}

impl fmt::Display for DecisionPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hidden => write!(f, "Hidden"),
            Self::Opening => write!(f, "Opening"),
            Self::Active => write!(f, "Active"),
            Self::Closing => write!(f, "Closing"),
            Self::Done => write!(f, "Done"),
        }
    }
}

// ── Decision Radial ────────────────────────────────────────────────────

/// Radial decision widget state.
///
/// Flow: `Hidden → Opening → Active → Closing → Done`
#[derive(Debug, Clone)]
pub struct DecisionRadial {
    /// The decision's unique identifier (e.g., `"storm_choice"`).
    pub decision_id: String,
    /// Prompt text shown at the center of the radial.
    pub prompt: String,
    /// Available options.
    pub options: Vec<DecisionOption>,
    /// Currently focused/hovered option index (if any).
    pub focused_index: Option<usize>,
    /// The index of the selected option (set on confirm).
    pub selected_index: Option<usize>,
    /// Current phase.
    phase: DecisionPhase,
    /// Animation progress (0.0 → 1.0) for open/close transitions.
    anim_progress: f32,
    /// Optional countdown timer (seconds remaining). `None` = no timer.
    countdown: Option<f32>,
    /// Default option index used when countdown expires.
    default_index: Option<usize>,
}

impl DecisionRadial {
    /// Open/close animation duration in seconds.
    pub const ANIM_DURATION: f32 = 0.35;

    /// Creates a new radial decision widget in the `Hidden` phase.
    #[must_use]
    pub fn new(
        decision_id: impl Into<String>,
        prompt: impl Into<String>,
        options: Vec<DecisionOption>,
    ) -> Self {
        Self {
            decision_id: decision_id.into(),
            prompt: prompt.into(),
            options,
            focused_index: None,
            selected_index: None,
            phase: DecisionPhase::Hidden,
            anim_progress: 0.0,
            countdown: None,
            default_index: None,
        }
    }

    /// Sets an optional countdown timer. When it expires, `default_index` is auto-selected.
    pub fn with_countdown(mut self, seconds: f32, default_index: usize) -> Self {
        self.countdown = Some(seconds);
        self.default_index = Some(default_index);
        self
    }

    // ── Phase transitions ──────────────────────────────────────────

    /// Begin the opening animation (Hidden → Opening).
    pub fn open(&mut self) {
        if self.phase == DecisionPhase::Hidden {
            self.phase = DecisionPhase::Opening;
            self.anim_progress = 0.0;
        }
    }

    /// Called when the player confirms a selection (Active → Closing).
    ///
    /// Returns `false` if the index is out of range or phase is wrong.
    pub fn select(&mut self, index: usize) -> bool {
        if self.phase != DecisionPhase::Active || index >= self.options.len() {
            return false;
        }
        self.selected_index = Some(index);
        self.phase = DecisionPhase::Closing;
        self.anim_progress = 0.0;
        true
    }

    /// Move focus to an option.
    pub fn focus(&mut self, index: usize) {
        if index < self.options.len() {
            self.focused_index = Some(index);
        }
    }

    /// Clear focus.
    pub fn clear_focus(&mut self) {
        self.focused_index = None;
    }

    // ── Tick ───────────────────────────────────────────────────────

    /// Advance animations and countdown.
    ///
    /// NaN or negative `dt` is silently ignored.
    pub fn tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt < 0.0 {
            return;
        }
        match self.phase {
            DecisionPhase::Opening => {
                self.anim_progress += dt / Self::ANIM_DURATION;
                if self.anim_progress >= 1.0 {
                    self.anim_progress = 1.0;
                    self.phase = DecisionPhase::Active;
                }
            }
            DecisionPhase::Active => {
                // Countdown logic
                if let Some(ref mut remaining) = self.countdown {
                    *remaining -= dt;
                    if *remaining <= 0.0 {
                        *remaining = 0.0;
                        // Auto-select the default option
                        if let Some(default_idx) = self.default_index {
                            self.select(default_idx);
                        }
                    }
                }
            }
            DecisionPhase::Closing => {
                self.anim_progress += dt / Self::ANIM_DURATION;
                if self.anim_progress >= 1.0 {
                    self.anim_progress = 1.0;
                    self.phase = DecisionPhase::Done;
                }
            }
            DecisionPhase::Hidden | DecisionPhase::Done => {}
        }
    }

    // ── Queries ────────────────────────────────────────────────────

    /// Current phase.
    pub fn phase(&self) -> DecisionPhase {
        self.phase
    }

    /// Animation progress (0.0–1.0). Used for opening/closing scale.
    pub fn anim_progress(&self) -> f32 {
        self.anim_progress
    }

    /// Remaining countdown time (if set). `None` = no countdown.
    pub fn countdown_remaining(&self) -> Option<f32> {
        self.countdown
    }

    /// Returns `true` if the widget is visible (Opening, Active, or Closing).
    pub fn is_visible(&self) -> bool {
        matches!(
            self.phase,
            DecisionPhase::Opening | DecisionPhase::Active | DecisionPhase::Closing
        )
    }

    /// Returns `true` if the decision is complete.
    pub fn is_done(&self) -> bool {
        self.phase == DecisionPhase::Done
    }

    /// Returns the selected option's key, if a selection was made.
    pub fn selected_key(&self) -> Option<&str> {
        self.selected_index
            .and_then(|i| self.options.get(i))
            .map(|o| o.key.as_str())
    }

    /// Returns the focused option's description, if focused.
    pub fn focused_description(&self) -> Option<&str> {
        self.focused_index
            .and_then(|i| self.options.get(i))
            .map(|o| o.description.as_str())
    }

    /// Returns the angle (in radians) for the segment at `index`.
    ///
    /// Used by the presentation layer to draw equal radial segments.
    pub fn segment_angle(&self, index: usize) -> Option<(f32, f32)> {
        if index >= self.options.len() || self.options.is_empty() {
            return None;
        }
        let segment = std::f32::consts::TAU / self.options.len() as f32;
        let start = index as f32 * segment;
        Some((start, start + segment))
    }
}

/// Constructs the Veilweaver storm choice radial with pre-defined options.
pub fn storm_choice_radial() -> DecisionRadial {
    let options = vec![
        DecisionOption::new(
            "stabilize",
            "Stabilize the Loom",
            "Repair the weave — anchors hold, the arena stays calm. \
             Boss fights without storm interference.",
            "icon_stabilize",
            (0.3, 0.7, 1.0), // Blue
        ),
        DecisionOption::new(
            "redirect",
            "Redirect the Storm",
            "Channel chaos into the arena — environmental hazards \
             empower your abilities but strengthen the boss.",
            "icon_redirect",
            (1.0, 0.4, 0.2), // Orange-red
        ),
    ];

    DecisionRadial::new(
        "storm_choice",
        "The storm surges. What will you do?",
        options,
    )
    .with_countdown(30.0, 0) // Default to Stabilize after 30s
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    fn two_option_radial() -> DecisionRadial {
        let options = vec![
            DecisionOption::new("a", "Option A", "Desc A", "icon_a", (1.0, 0.0, 0.0)),
            DecisionOption::new("b", "Option B", "Desc B", "icon_b", (0.0, 1.0, 0.0)),
        ];
        DecisionRadial::new("test", "Pick one", options)
    }

    #[test]
    fn initial_state_is_hidden() {
        let radial = two_option_radial();
        assert_eq!(radial.phase(), DecisionPhase::Hidden);
        assert!(!radial.is_visible());
        assert!(!radial.is_done());
        assert!(radial.selected_key().is_none());
    }

    #[test]
    fn open_and_animate_to_active() {
        let mut radial = two_option_radial();
        radial.open();
        assert_eq!(radial.phase(), DecisionPhase::Opening);
        assert!(radial.is_visible());

        // Animate past the duration
        radial.tick(0.5);
        assert_eq!(radial.phase(), DecisionPhase::Active);
        assert_eq!(radial.anim_progress(), 1.0);
    }

    #[test]
    fn select_and_close() {
        let mut radial = two_option_radial();
        radial.open();
        radial.tick(0.5); // → Active

        assert!(radial.select(1));
        assert_eq!(radial.phase(), DecisionPhase::Closing);
        assert_eq!(radial.selected_key(), Some("b"));

        radial.tick(0.5); // → Done
        assert!(radial.is_done());
    }

    #[test]
    fn select_out_of_range_fails() {
        let mut radial = two_option_radial();
        radial.open();
        radial.tick(0.5);
        assert!(!radial.select(99));
        assert_eq!(radial.phase(), DecisionPhase::Active);
    }

    #[test]
    fn select_in_wrong_phase_fails() {
        let mut radial = two_option_radial();
        assert!(!radial.select(0)); // Hidden phase
    }

    #[test]
    fn focus_and_description() {
        let mut radial = two_option_radial();
        radial.open();
        radial.tick(0.5);

        radial.focus(0);
        assert_eq!(radial.focused_description(), Some("Desc A"));

        radial.focus(1);
        assert_eq!(radial.focused_description(), Some("Desc B"));

        radial.clear_focus();
        assert!(radial.focused_description().is_none());
    }

    #[test]
    fn countdown_auto_selects() {
        let mut radial = two_option_radial().with_countdown(1.0, 0);
        radial.open();
        radial.tick(0.5); // → Active

        // Not expired yet
        radial.tick(0.5);
        assert_eq!(radial.phase(), DecisionPhase::Active);

        // Expire the countdown
        radial.tick(0.6);
        // Should auto-select default (index 0)
        assert_eq!(radial.phase(), DecisionPhase::Closing);
        assert_eq!(radial.selected_key(), Some("a"));
    }

    #[test]
    fn countdown_remaining_tracks() {
        let mut radial = two_option_radial().with_countdown(5.0, 0);
        assert!((radial.countdown_remaining().unwrap() - 5.0).abs() < 0.01);

        radial.open();
        radial.tick(0.5); // → Active
        radial.tick(2.0);
        let remaining = radial.countdown_remaining().unwrap();
        assert!((remaining - 3.0).abs() < 0.1);
    }

    #[test]
    fn segment_angles_two_options() {
        let radial = two_option_radial();
        let (s0, e0) = radial.segment_angle(0).unwrap();
        let (s1, e1) = radial.segment_angle(1).unwrap();
        assert!((s0 - 0.0).abs() < 0.01);
        assert!((e0 - PI).abs() < 0.01);
        assert!((s1 - PI).abs() < 0.01);
        assert!((e1 - std::f32::consts::TAU).abs() < 0.01);
    }

    #[test]
    fn segment_angle_out_of_range() {
        let radial = two_option_radial();
        assert!(radial.segment_angle(5).is_none());
    }

    #[test]
    fn storm_choice_radial_has_two_options() {
        let radial = storm_choice_radial();
        assert_eq!(radial.options.len(), 2);
        assert_eq!(radial.options[0].key, "stabilize");
        assert_eq!(radial.options[1].key, "redirect");
        assert!(radial.countdown_remaining().is_some());
    }

    #[test]
    fn display_phase_names() {
        assert_eq!(format!("{}", DecisionPhase::Hidden), "Hidden");
        assert_eq!(format!("{}", DecisionPhase::Active), "Active");
        assert_eq!(format!("{}", DecisionPhase::Done), "Done");
    }
}
