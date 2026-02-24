//! Post-run metrics panel — recap screen displayed after completing the slice.
//!
//! Collects gameplay metrics throughout the run and presents them as a
//! structured recap. The presentation layer renders this as an animated
//! results screen with category breakdowns.
//!
//! Headless-safe — no rendering code.

use serde::Serialize;
use std::collections::BTreeMap;
use std::fmt;

// ── Metric Category ────────────────────────────────────────────────────

/// Categories for the recap screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub enum MetricCategory {
    /// Combat performance (damage dealt, taken, combos).
    Combat,
    /// Exploration progress (zones visited, secrets found).
    Exploration,
    /// Weaving mechanics (anchors repaired, echoes collected).
    Weaving,
    /// Companion interactions (affinity changes, support actions).
    Companion,
    /// Narrative choices (decisions made, dialogue paths).
    Narrative,
    /// Timing (total run time, boss fight duration).
    Timing,
}

impl fmt::Display for MetricCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Combat => write!(f, "Combat"),
            Self::Exploration => write!(f, "Exploration"),
            Self::Weaving => write!(f, "Weaving"),
            Self::Companion => write!(f, "Companion"),
            Self::Narrative => write!(f, "Narrative"),
            Self::Timing => write!(f, "Timing"),
        }
    }
}

// ── Metric Entry ───────────────────────────────────────────────────────

/// A single metric in the recap.
#[derive(Debug, Clone, Serialize)]
pub struct MetricEntry {
    /// Machine-readable key (e.g., `"damage_dealt"`).
    pub key: String,
    /// Player-facing label (e.g., `"Damage Dealt"`).
    pub label: String,
    /// Numeric value.
    pub value: MetricValue,
    /// Category for grouping in the recap.
    pub category: MetricCategory,
    /// Optional suffix (e.g., `"HP"`, `"%"`, `"s"`).
    pub suffix: String,
}

/// Value type for metrics.
#[derive(Debug, Clone, Serialize)]
pub enum MetricValue {
    /// Integer value (e.g., kill count, echo balance).
    Integer(i64),
    /// Floating-point value (e.g., completion percentage, time).
    Float(f64),
    /// Text value (e.g., storm choice, final rank).
    Text(String),
}

impl fmt::Display for MetricValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(v) => write!(f, "{}", v),
            Self::Float(v) => write!(f, "{:.1}", v),
            Self::Text(v) => write!(f, "{}", v),
        }
    }
}

impl MetricValue {
    /// Returns the numeric value as f64 (for animations). Text returns 0.0.
    pub fn as_f64(&self) -> f64 {
        match self {
            Self::Integer(v) => *v as f64,
            Self::Float(v) => *v,
            Self::Text(_) => 0.0,
        }
    }
}

// ── Rating ─────────────────────────────────────────────────────────────

/// Per-category performance rating.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Rating {
    /// Exceptional performance.
    S,
    /// Great performance.
    A,
    /// Good performance.
    B,
    /// Average performance.
    C,
    /// Below average.
    D,
}

impl fmt::Display for Rating {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::S => write!(f, "S"),
            Self::A => write!(f, "A"),
            Self::B => write!(f, "B"),
            Self::C => write!(f, "C"),
            Self::D => write!(f, "D"),
        }
    }
}

impl Rating {
    /// Color for the rating badge (R, G, B).
    pub fn color(&self) -> (f32, f32, f32) {
        match self {
            Self::S => (1.0, 0.84, 0.0), // Gold
            Self::A => (0.3, 0.8, 0.3),  // Green
            Self::B => (0.3, 0.6, 0.9),  // Blue
            Self::C => (0.7, 0.7, 0.7),  // Gray
            Self::D => (0.8, 0.3, 0.3),  // Red
        }
    }
}

// ── Recap Panel ────────────────────────────────────────────────────────

/// Post-run recap panel state.
///
/// Collects metrics during gameplay, then computes category ratings and
/// presents the final summary.
#[derive(Debug, Clone)]
pub struct RecapPanel {
    /// All recorded metrics.
    metrics: Vec<MetricEntry>,
    /// Per-category ratings (computed on finalize).
    ratings: BTreeMap<MetricCategory, Rating>,
    /// Overall rating (computed on finalize).
    overall_rating: Option<Rating>,
    /// Whether the recap has been finalized.
    finalized: bool,
    /// Presentation animation progress (0.0–1.0).
    anim_progress: f32,
    /// Current category being revealed (for staggered animation).
    reveal_index: usize,
}

impl Default for RecapPanel {
    fn default() -> Self {
        Self::new()
    }
}

impl RecapPanel {
    /// Animation duration per category reveal (seconds).
    pub const REVEAL_DURATION: f32 = 0.6;

    /// Creates an empty recap panel ready to receive metrics.
    #[must_use]
    pub fn new() -> Self {
        Self {
            metrics: Vec::new(),
            ratings: BTreeMap::new(),
            overall_rating: None,
            finalized: false,
            anim_progress: 0.0,
            reveal_index: 0,
        }
    }

    // ── Recording ──────────────────────────────────────────────────

    /// Record an integer metric.
    pub fn record_int(
        &mut self,
        key: impl Into<String>,
        label: impl Into<String>,
        value: i64,
        category: MetricCategory,
        suffix: impl Into<String>,
    ) {
        if self.finalized {
            return;
        }
        self.metrics.push(MetricEntry {
            key: key.into(),
            label: label.into(),
            value: MetricValue::Integer(value),
            category,
            suffix: suffix.into(),
        });
    }

    /// Record a floating-point metric.
    pub fn record_float(
        &mut self,
        key: impl Into<String>,
        label: impl Into<String>,
        value: f64,
        category: MetricCategory,
        suffix: impl Into<String>,
    ) {
        if self.finalized {
            return;
        }
        self.metrics.push(MetricEntry {
            key: key.into(),
            label: label.into(),
            value: MetricValue::Float(value),
            category,
            suffix: suffix.into(),
        });
    }

    /// Record a text metric.
    pub fn record_text(
        &mut self,
        key: impl Into<String>,
        label: impl Into<String>,
        value: impl Into<String>,
        category: MetricCategory,
    ) {
        if self.finalized {
            return;
        }
        self.metrics.push(MetricEntry {
            key: key.into(),
            label: label.into(),
            value: MetricValue::Text(value.into()),
            category,
            suffix: String::new(),
        });
    }

    // ── Finalization ───────────────────────────────────────────────

    /// Finalize the recap — compute ratings from recorded metrics.
    ///
    /// After this call, no more metrics can be recorded.
    /// The `rater` closure takes (category, metrics_in_category) and returns a Rating.
    pub fn finalize_with<F>(&mut self, rater: F)
    where
        F: Fn(MetricCategory, &[&MetricEntry]) -> Rating,
    {
        if self.finalized {
            return;
        }
        self.finalized = true;

        // Group metrics by category
        let categories: Vec<MetricCategory> = self
            .metrics
            .iter()
            .map(|m| m.category)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();

        let mut rating_sum = 0u32;
        let mut rating_count = 0u32;

        for cat in &categories {
            let cat_metrics: Vec<&MetricEntry> =
                self.metrics.iter().filter(|m| m.category == *cat).collect();
            let rating = rater(*cat, &cat_metrics);
            self.ratings.insert(*cat, rating);

            rating_sum += match rating {
                Rating::S => 5,
                Rating::A => 4,
                Rating::B => 3,
                Rating::C => 2,
                Rating::D => 1,
            };
            rating_count += 1;
        }

        // Compute overall rating
        if rating_count > 0 {
            let avg = rating_sum as f32 / rating_count as f32;
            self.overall_rating = Some(match avg {
                v if v >= 4.5 => Rating::S,
                v if v >= 3.5 => Rating::A,
                v if v >= 2.5 => Rating::B,
                v if v >= 1.5 => Rating::C,
                _ => Rating::D,
            });
        }
    }

    /// Convenience finalizer with default rating logic.
    ///
    /// Uses a simple scoring heuristic: enough metrics in a category → B,
    /// high-value metrics → A/S.
    pub fn finalize_default(&mut self) {
        self.finalize_with(|_category, metrics| {
            if metrics.is_empty() {
                return Rating::C;
            }
            let count = metrics.len();
            if count >= 5 {
                Rating::A
            } else if count >= 3 {
                Rating::B
            } else {
                Rating::C
            }
        });
    }

    // ── Presentation animation ─────────────────────────────────────

    /// Advance the reveal animation.
    ///
    /// NaN or negative `dt` is silently ignored.
    pub fn tick(&mut self, dt: f32) {
        if !dt.is_finite() || dt < 0.0 {
            return;
        }
        if !self.finalized {
            return;
        }
        let total_categories = self.ratings.len();
        if total_categories == 0 {
            return;
        }

        self.anim_progress += dt / Self::REVEAL_DURATION;
        let target = (self.anim_progress as usize).min(total_categories);
        self.reveal_index = target;
    }

    // ── Queries ────────────────────────────────────────────────────

    /// Returns `true` if the recap has been finalized.
    pub fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Number of categories that have been revealed in the animation.
    pub fn revealed_count(&self) -> usize {
        self.reveal_index
    }

    /// Returns `true` if all categories have been revealed.
    pub fn is_fully_revealed(&self) -> bool {
        self.reveal_index >= self.ratings.len()
    }

    /// Overall rating (available after finalize).
    pub fn overall_rating(&self) -> Option<Rating> {
        self.overall_rating
    }

    /// Per-category ratings.
    pub fn category_ratings(&self) -> &BTreeMap<MetricCategory, Rating> {
        &self.ratings
    }

    /// All recorded metrics.
    pub fn metrics(&self) -> &[MetricEntry] {
        &self.metrics
    }

    /// Metrics for a specific category.
    pub fn metrics_for(&self, category: MetricCategory) -> Vec<&MetricEntry> {
        self.metrics
            .iter()
            .filter(|m| m.category == category)
            .collect()
    }

    /// Look up a metric by key.
    pub fn metric(&self, key: &str) -> Option<&MetricEntry> {
        self.metrics.iter().find(|m| m.key == key)
    }

    /// Total number of recorded metrics.
    pub fn metric_count(&self) -> usize {
        self.metrics.len()
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_recap() -> RecapPanel {
        let mut panel = RecapPanel::new();
        panel.record_int(
            "damage_dealt",
            "Damage Dealt",
            12500,
            MetricCategory::Combat,
            "HP",
        );
        panel.record_int(
            "damage_taken",
            "Damage Taken",
            3200,
            MetricCategory::Combat,
            "HP",
        );
        panel.record_int(
            "enemies_defeated",
            "Enemies Defeated",
            8,
            MetricCategory::Combat,
            "",
        );
        panel.record_int(
            "zones_visited",
            "Zones Visited",
            5,
            MetricCategory::Exploration,
            "",
        );
        panel.record_int(
            "anchors_repaired",
            "Anchors Repaired",
            3,
            MetricCategory::Weaving,
            "",
        );
        panel.record_int(
            "echoes_collected",
            "Echoes Collected",
            42,
            MetricCategory::Weaving,
            "",
        );
        panel.record_text(
            "storm_choice",
            "Storm Decision",
            "Stabilize",
            MetricCategory::Narrative,
        );
        panel.record_float(
            "total_time",
            "Total Time",
            1842.5,
            MetricCategory::Timing,
            "s",
        );
        panel.record_float(
            "boss_time",
            "Boss Fight",
            312.0,
            MetricCategory::Timing,
            "s",
        );
        panel.record_text(
            "final_rank",
            "Companion Rank",
            "Bonded",
            MetricCategory::Companion,
        );
        panel
    }

    #[test]
    fn initial_state() {
        let panel = RecapPanel::new();
        assert!(!panel.is_finalized());
        assert_eq!(panel.metric_count(), 0);
        assert!(panel.overall_rating().is_none());
    }

    #[test]
    fn record_and_query_metrics() {
        let panel = sample_recap();
        assert_eq!(panel.metric_count(), 10);
        assert_eq!(panel.metrics_for(MetricCategory::Combat).len(), 3);
        assert_eq!(panel.metrics_for(MetricCategory::Weaving).len(), 2);

        let damage = panel.metric("damage_dealt").unwrap();
        assert_eq!(damage.value.as_f64(), 12500.0);
        assert_eq!(damage.suffix, "HP");
    }

    #[test]
    fn finalize_computes_ratings() {
        let mut panel = sample_recap();
        panel.finalize_default();

        assert!(panel.is_finalized());
        assert!(panel.overall_rating().is_some());
        assert!(!panel.category_ratings().is_empty());

        // Combat has 3 metrics → B rating with default logic
        assert_eq!(
            *panel
                .category_ratings()
                .get(&MetricCategory::Combat)
                .unwrap(),
            Rating::B
        );
    }

    #[test]
    fn custom_rater() {
        let mut panel = sample_recap();
        panel.finalize_with(|_cat, _metrics| Rating::S);

        assert_eq!(panel.overall_rating(), Some(Rating::S));
        for &rating in panel.category_ratings().values() {
            assert_eq!(rating, Rating::S);
        }
    }

    #[test]
    fn no_recording_after_finalize() {
        let mut panel = sample_recap();
        panel.finalize_default();
        let count_before = panel.metric_count();
        panel.record_int("extra", "Extra", 999, MetricCategory::Combat, "");
        assert_eq!(panel.metric_count(), count_before);
    }

    #[test]
    fn reveal_animation_incremental() {
        let mut panel = sample_recap();
        panel.finalize_default();

        assert_eq!(panel.revealed_count(), 0);
        assert!(!panel.is_fully_revealed());

        // Tick once — should reveal first category
        panel.tick(RecapPanel::REVEAL_DURATION + 0.01);
        assert!(panel.revealed_count() >= 1);
    }

    #[test]
    fn reveal_animation_full() {
        let mut panel = sample_recap();
        panel.finalize_default();
        let num_categories = panel.category_ratings().len();

        // Tick enough to reveal all
        let total_time = num_categories as f32 * RecapPanel::REVEAL_DURATION + 1.0;
        panel.tick(total_time);
        assert!(panel.is_fully_revealed());
    }

    #[test]
    fn metric_value_display() {
        assert_eq!(format!("{}", MetricValue::Integer(42)), "42");
        assert_eq!(format!("{}", MetricValue::Float(2.75)), "2.8");
        assert_eq!(format!("{}", MetricValue::Text("hello".into())), "hello");
    }

    #[test]
    fn rating_display_and_colors() {
        assert_eq!(format!("{}", Rating::S), "S");
        assert_eq!(format!("{}", Rating::D), "D");

        let (r, g, _b) = Rating::S.color();
        assert!(r > 0.9); // Gold R
        assert!(g > 0.8); // Gold G

        let (r, _, _) = Rating::D.color();
        assert!(r > 0.7); // Red
    }

    #[test]
    fn category_display() {
        assert_eq!(format!("{}", MetricCategory::Combat), "Combat");
        assert_eq!(format!("{}", MetricCategory::Timing), "Timing");
    }

    #[test]
    fn empty_finalize() {
        let mut panel = RecapPanel::new();
        panel.finalize_default();
        assert!(panel.is_finalized());
        assert!(panel.overall_rating().is_none()); // No categories → no overall
        assert!(panel.category_ratings().is_empty());
    }

    #[test]
    fn overall_rating_averaging() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.record_int("b", "B", 1, MetricCategory::Exploration, "");

        // Force: Combat → S (5), Exploration → D (1) = avg 3.0 → B
        panel.finalize_with(|cat, _| match cat {
            MetricCategory::Combat => Rating::S,
            _ => Rating::D,
        });
        assert_eq!(panel.overall_rating(), Some(Rating::B));
    }

    #[test]
    fn tick_before_finalize_is_noop() {
        let mut panel = RecapPanel::new();
        panel.tick(1.0);
        assert_eq!(panel.revealed_count(), 0);
    }
}
