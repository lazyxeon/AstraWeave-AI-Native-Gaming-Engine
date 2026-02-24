//! Performance budget tracker — frame-level timing + percentile analysis.
//!
//! Tracks per-tick wall-clock durations and provides real-time percentile
//! metrics (p50, p95, p99) to verify the game stays within the 16.67 ms
//! frame budget at 60 Hz.
//!
//! # Usage
//!
//! ```rust
//! use veilweaver_slice_runtime::perf_budget::{FrameBudgetTracker, FrameBudgetConfig};
//!
//! let mut budget = FrameBudgetTracker::new(FrameBudgetConfig::default());
//! for _ in 0..10 {
//!     let guard = budget.begin_tick();
//!     // game_loop.tick(dt);
//!     guard.end();    // or let it drop
//! }
//! let report = budget.report();
//! assert!(report.p95_ms < 16.67);
//! ```
//!
//! This module contains no rendering code — it is headless-safe.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

// ── Configuration ──────────────────────────────────────────────────────

/// Configuration for the frame budget tracker.
#[derive(Debug, Clone)]
pub struct FrameBudgetConfig {
    /// Target frame time in milliseconds (default: 16.67 ms = 60 Hz).
    pub target_ms: f64,
    /// Maximum number of samples to retain in the rolling window.
    /// Older samples are evicted when the window is full.
    pub window_size: usize,
    /// Budget violation threshold multiplier (default: 1.5×).
    /// A tick exceeding `target_ms * violation_multiplier` is flagged.
    pub violation_multiplier: f64,
}

impl Default for FrameBudgetConfig {
    fn default() -> Self {
        Self {
            target_ms: 16.67,
            window_size: 3600, // 60s at 60 Hz
            violation_multiplier: 1.5,
        }
    }
}

// ── Tracker ────────────────────────────────────────────────────────────

/// Tracks per-tick durations and computes percentile statistics.
#[derive(Debug)]
pub struct FrameBudgetTracker {
    config: FrameBudgetConfig,
    /// Rolling window of tick durations (milliseconds).
    samples: VecDeque<f64>,
    /// Total number of ticks measured (including evicted samples).
    total_ticks: u64,
    /// Number of ticks that exceeded the violation threshold.
    violation_count: u64,
    /// Peak tick duration observed (milliseconds).
    peak_ms: f64,
    /// Sum of all durations for mean computation.
    sum_ms: f64,
}

impl FrameBudgetTracker {
    /// Creates a new tracker with the given configuration.
    #[must_use]
    pub fn new(config: FrameBudgetConfig) -> Self {
        Self {
            config,
            samples: VecDeque::new(),
            total_ticks: 0,
            violation_count: 0,
            peak_ms: 0.0,
            sum_ms: 0.0,
        }
    }

    /// Creates a tracker with default 60 Hz budget.
    pub fn default_60hz() -> Self {
        Self::new(FrameBudgetConfig::default())
    }

    /// Begins a tick measurement. Returns a guard that records the duration
    /// when dropped (or when `.end()` is called explicitly).
    pub fn begin_tick(&mut self) -> TickGuard<'_> {
        TickGuard {
            tracker: self,
            start: Instant::now(),
        }
    }

    /// Records a tick duration directly (for testing or manual measurement).
    pub fn record_tick_ms(&mut self, duration_ms: f64) {
        self.total_ticks += 1;
        self.sum_ms += duration_ms;

        if duration_ms > self.peak_ms {
            self.peak_ms = duration_ms;
        }

        let threshold = self.config.target_ms * self.config.violation_multiplier;
        if duration_ms > threshold {
            self.violation_count += 1;
        }

        if self.samples.len() >= self.config.window_size {
            self.samples.pop_front();
        }
        self.samples.push_back(duration_ms);
    }

    /// Records a tick duration from a `std::time::Duration`.
    pub fn record_tick_duration(&mut self, duration: Duration) {
        self.record_tick_ms(duration.as_secs_f64() * 1000.0);
    }

    /// Returns the number of samples in the current window.
    pub fn sample_count(&self) -> usize {
        self.samples.len()
    }

    /// Returns the total number of ticks measured.
    pub fn total_ticks(&self) -> u64 {
        self.total_ticks
    }

    /// Computes the current percentile report.
    pub fn report(&self) -> FrameBudgetReport {
        if self.samples.is_empty() {
            return FrameBudgetReport {
                target_ms: self.config.target_ms,
                total_ticks: 0,
                sample_count: 0,
                mean_ms: 0.0,
                p50_ms: 0.0,
                p95_ms: 0.0,
                p99_ms: 0.0,
                peak_ms: 0.0,
                violation_count: 0,
                violation_rate: 0.0,
                within_budget: true,
            };
        }

        let mut sorted: Vec<f64> = self.samples.iter().copied().collect();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let p50 = Self::percentile(&sorted, 0.50);
        let p95 = Self::percentile(&sorted, 0.95);
        let p99 = Self::percentile(&sorted, 0.99);

        let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
        let violation_rate = if self.total_ticks > 0 {
            self.violation_count as f64 / self.total_ticks as f64
        } else {
            0.0
        };

        FrameBudgetReport {
            target_ms: self.config.target_ms,
            total_ticks: self.total_ticks,
            sample_count: self.samples.len() as u64,
            mean_ms: mean,
            p50_ms: p50,
            p95_ms: p95,
            p99_ms: p99,
            peak_ms: self.peak_ms,
            violation_count: self.violation_count,
            violation_rate,
            within_budget: p95 <= self.config.target_ms,
        }
    }

    /// Computes a percentile from a sorted slice.
    fn percentile(sorted: &[f64], p: f64) -> f64 {
        if sorted.is_empty() {
            return 0.0;
        }
        let idx = (p * (sorted.len() as f64 - 1.0)).round() as usize;
        let idx = idx.min(sorted.len() - 1);
        sorted[idx]
    }

    /// Resets all tracked data.
    pub fn reset(&mut self) {
        self.samples.clear();
        self.total_ticks = 0;
        self.violation_count = 0;
        self.peak_ms = 0.0;
        self.sum_ms = 0.0;
    }
}

// ── Tick Guard ─────────────────────────────────────────────────────────

/// RAII guard that measures a single tick's wall-clock duration.
pub struct TickGuard<'a> {
    tracker: &'a mut FrameBudgetTracker,
    start: Instant,
}

impl<'a> TickGuard<'a> {
    /// Explicitly ends the tick measurement.
    pub fn end(self) {
        // Drop will handle the recording.
    }
}

impl<'a> Drop for TickGuard<'a> {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.tracker.record_tick_duration(elapsed);
    }
}

// ── Report ─────────────────────────────────────────────────────────────

/// Percentile-based performance report.
#[derive(Debug, Clone, Default)]
pub struct FrameBudgetReport {
    /// Target frame budget in milliseconds.
    pub target_ms: f64,
    /// Total ticks measured (all-time).
    pub total_ticks: u64,
    /// Number of samples in the current window.
    pub sample_count: u64,
    /// Mean tick duration in milliseconds.
    pub mean_ms: f64,
    /// 50th percentile (median) tick duration.
    pub p50_ms: f64,
    /// 95th percentile tick duration.
    pub p95_ms: f64,
    /// 99th percentile tick duration.
    pub p99_ms: f64,
    /// Peak (worst) tick duration.
    pub peak_ms: f64,
    /// Number of ticks that exceeded the violation threshold.
    pub violation_count: u64,
    /// Violation rate (0.0–1.0).
    pub violation_rate: f64,
    /// `true` if p95 is within the target budget.
    pub within_budget: bool,
}

impl FrameBudgetReport {
    /// Produces a human-readable summary.
    pub fn summary(&self) -> String {
        format!(
            "Frame Budget: target={:.2}ms, p50={:.2}ms, p95={:.2}ms, p99={:.2}ms, peak={:.2}ms, \
             violations={}/{} ({:.1}%), budget={}",
            self.target_ms,
            self.p50_ms,
            self.p95_ms,
            self.p99_ms,
            self.peak_ms,
            self.violation_count,
            self.total_ticks,
            self.violation_rate * 100.0,
            if self.within_budget { "OK" } else { "EXCEEDED" },
        )
    }
}

// ── Per-Subsystem Timing ───────────────────────────────────────────────

/// Tracks per-subsystem timing breakdown within a frame.
#[derive(Debug, Clone, Default)]
pub struct SubsystemTimings {
    /// Timing entries: subsystem name → duration in ms.
    entries: Vec<(String, f64)>,
}

impl SubsystemTimings {
    /// Creates an empty subsystem timing collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Records a subsystem's duration.
    pub fn record(&mut self, name: impl Into<String>, duration_ms: f64) {
        self.entries.push((name.into(), duration_ms));
    }

    /// Records a subsystem's duration from a `Duration`.
    pub fn record_duration(&mut self, name: impl Into<String>, duration: Duration) {
        self.record(name, duration.as_secs_f64() * 1000.0);
    }

    /// Returns all recorded entries.
    pub fn entries(&self) -> &[(String, f64)] {
        &self.entries
    }

    /// Returns the total duration of all subsystems.
    pub fn total_ms(&self) -> f64 {
        self.entries.iter().map(|(_, ms)| ms).sum()
    }

    /// Returns the subsystem with the highest duration.
    pub fn bottleneck(&self) -> Option<(&str, f64)> {
        self.entries
            .iter()
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(name, ms)| (name.as_str(), *ms))
    }

    /// Clears all entries for the next frame.
    pub fn clear(&mut self) {
        self.entries.clear();
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = FrameBudgetConfig::default();
        assert!((config.target_ms - 16.67).abs() < 0.01);
        assert_eq!(config.window_size, 3600);
    }

    #[test]
    fn record_and_report() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        for i in 0..100 {
            tracker.record_tick_ms(f64::from(i) * 0.1 + 1.0);
        }
        let report = tracker.report();
        assert_eq!(report.total_ticks, 100);
        assert_eq!(report.sample_count, 100);
        assert!(report.within_budget);
        assert!(report.p50_ms > 0.0);
        assert!(report.p95_ms > report.p50_ms);
    }

    #[test]
    fn violation_detection() {
        let mut tracker = FrameBudgetTracker::new(FrameBudgetConfig {
            target_ms: 10.0,
            window_size: 100,
            violation_multiplier: 1.5,
        });
        // 9 ticks under budget
        for _ in 0..9 {
            tracker.record_tick_ms(5.0);
        }
        // 1 tick over violation threshold (10 * 1.5 = 15)
        tracker.record_tick_ms(20.0);

        let report = tracker.report();
        assert_eq!(report.violation_count, 1);
        assert!((report.violation_rate - 0.1).abs() < 0.001);
    }

    #[test]
    fn peak_tracking() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(5.0);
        tracker.record_tick_ms(15.0);
        tracker.record_tick_ms(3.0);
        assert!((tracker.report().peak_ms - 15.0).abs() < 0.001);
    }

    #[test]
    fn window_eviction() {
        let mut tracker = FrameBudgetTracker::new(FrameBudgetConfig {
            target_ms: 16.67,
            window_size: 5,
            violation_multiplier: 1.5,
        });
        for i in 0..10 {
            tracker.record_tick_ms(f64::from(i));
        }
        assert_eq!(tracker.sample_count(), 5);
        assert_eq!(tracker.total_ticks(), 10);
    }

    #[test]
    fn empty_report() {
        let tracker = FrameBudgetTracker::default_60hz();
        let report = tracker.report();
        assert_eq!(report.total_ticks, 0);
        assert!(report.within_budget);
        assert!((report.p50_ms).abs() < 0.001);
    }

    #[test]
    fn reset_clears() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(5.0);
        tracker.record_tick_ms(10.0);
        tracker.reset();
        assert_eq!(tracker.sample_count(), 0);
        assert_eq!(tracker.total_ticks(), 0);
    }

    #[test]
    fn report_summary_format() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(8.0);
        let report = tracker.report();
        let summary = report.summary();
        assert!(summary.contains("target="));
        assert!(summary.contains("p50="));
        assert!(summary.contains("budget=OK"));
    }

    #[test]
    fn report_budget_exceeded() {
        let mut tracker = FrameBudgetTracker::new(FrameBudgetConfig {
            target_ms: 5.0,
            window_size: 100,
            violation_multiplier: 1.5,
        });
        for _ in 0..100 {
            tracker.record_tick_ms(10.0); // All above 5ms p95 = 10ms > 5ms
        }
        let report = tracker.report();
        assert!(!report.within_budget);
        assert!(report.summary().contains("EXCEEDED"));
    }

    #[test]
    fn subsystem_timings_basic() {
        let mut timings = SubsystemTimings::new();
        timings.record("game_loop", 3.5);
        timings.record("vfx", 1.2);
        timings.record("hud", 0.8);
        assert_eq!(timings.entries().len(), 3);
        assert!((timings.total_ms() - 5.5).abs() < 0.001);
    }

    #[test]
    fn subsystem_bottleneck() {
        let mut timings = SubsystemTimings::new();
        timings.record("a", 1.0);
        timings.record("b", 5.0);
        timings.record("c", 2.0);
        let (name, ms) = timings.bottleneck().unwrap();
        assert_eq!(name, "b");
        assert!((ms - 5.0).abs() < 0.001);
    }

    #[test]
    fn subsystem_empty_bottleneck() {
        let timings = SubsystemTimings::new();
        assert!(timings.bottleneck().is_none());
    }

    #[test]
    fn subsystem_clear() {
        let mut timings = SubsystemTimings::new();
        timings.record("a", 1.0);
        timings.clear();
        assert!(timings.entries().is_empty());
        assert!((timings.total_ms()).abs() < 0.001);
    }

    #[test]
    fn tick_guard_records_duration() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        {
            let _guard = tracker.begin_tick();
            // Simulate minimal work
            std::hint::black_box(42);
        }
        assert_eq!(tracker.total_ticks(), 1);
        assert!(tracker.report().p50_ms >= 0.0);
    }

    #[test]
    fn percentile_single_sample() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(8.0);
        let report = tracker.report();
        assert!((report.p50_ms - 8.0).abs() < 0.001);
        assert!((report.p95_ms - 8.0).abs() < 0.001);
        assert!((report.p99_ms - 8.0).abs() < 0.001);
    }

    #[test]
    fn record_tick_duration_works() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_duration(Duration::from_millis(10));
        assert_eq!(tracker.total_ticks(), 1);
        assert!((tracker.report().p50_ms - 10.0).abs() < 0.1);
    }

    #[test]
    fn subsystem_record_duration() {
        let mut timings = SubsystemTimings::new();
        timings.record_duration("test", Duration::from_millis(5));
        assert_eq!(timings.entries().len(), 1);
        assert!((timings.entries()[0].1 - 5.0).abs() < 0.1);
    }

    #[test]
    fn vecdeque_eviction_is_fifo() {
        let config = FrameBudgetConfig {
            target_ms: 16.67,
            window_size: 3,
            violation_multiplier: 1.5,
        };
        let mut tracker = FrameBudgetTracker::new(config);
        tracker.record_tick_ms(1.0);
        tracker.record_tick_ms(2.0);
        tracker.record_tick_ms(3.0);
        tracker.record_tick_ms(4.0); // Evicts 1.0
        let report = tracker.report();
        assert_eq!(report.sample_count, 3);
        assert_eq!(tracker.total_ticks(), 4);
        // p50 of [2, 3, 4] = 3.0
        assert!((report.p50_ms - 3.0).abs() < 0.01);
    }

    #[test]
    fn empty_tracker_report_is_safe() {
        let tracker = FrameBudgetTracker::default_60hz();
        let report = tracker.report();
        assert_eq!(report.total_ticks, 0);
        assert_eq!(report.p50_ms, 0.0);
        assert!(report.within_budget);
    }
}
