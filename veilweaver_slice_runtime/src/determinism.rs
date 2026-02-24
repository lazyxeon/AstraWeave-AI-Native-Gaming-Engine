//! Determinism validation — hash-based state fingerprinting for N-run consistency.
//!
//! The Veilweaver vertical slice must produce **identical** game state when given
//! identical inputs. This module provides tools to:
//!
//! 1. Capture a [`StateFingerprint`] from the game loop at any tick.
//! 2. Record a sequence of fingerprints into a [`DeterminismTrace`].
//! 3. Compare two traces for exact match via [`DeterminismTrace::compare`].
//!
//! # Usage
//!
//! ```rust
//! use veilweaver_slice_runtime::determinism::{DeterminismTrace, StateFingerprint};
//!
//! let mut trace_a = DeterminismTrace::new("run_1");
//! let mut trace_b = DeterminismTrace::new("run_2");
//! for tick in 0..5u64 {
//!     let fp = StateFingerprint::new(tick, 42, 43, 44, 45);
//!     trace_a.record(fp.clone());
//!     trace_b.record(fp);
//! }
//! let report = trace_a.compare(&trace_b);
//! assert!(report.is_consistent());
//! ```
//!
//! This module contains no rendering code — it is headless-safe.

use crate::game_loop::GameLoop;
use crate::hud_state::ThreadHud;
use crate::storm_choice::StormPhase;
use crate::telemetry::TelemetryCollector;
use crate::vfx_dispatch::VfxAudioDispatch;

use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

// ── State Fingerprint ──────────────────────────────────────────────────

/// A hash-based snapshot of game state at a single tick.
///
/// Uses a deterministic hasher (FNV-style) so fingerprints are consistent
/// across runs on the same platform.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateFingerprint {
    /// Tick index this fingerprint was captured at.
    pub tick: u64,
    /// Hash of the game loop state (zone, storm phase, active dialogue).
    pub game_loop_hash: u64,
    /// Hash of the HUD state (stability entries, echo balance).
    pub hud_hash: u64,
    /// Hash of the telemetry counters.
    pub telemetry_hash: u64,
    /// Hash of VFX dispatch state (zone, boss flag, thread count).
    pub vfx_hash: u64,
    /// Combined hash of all sub-hashes.
    pub combined: u64,
}

impl StateFingerprint {
    /// Creates a fingerprint from individual sub-system hashes.
    #[must_use]
    pub fn new(
        tick: u64,
        game_loop_hash: u64,
        hud_hash: u64,
        telemetry_hash: u64,
        vfx_hash: u64,
    ) -> Self {
        let combined = Self::combine_hashes(game_loop_hash, hud_hash, telemetry_hash, vfx_hash);
        Self {
            tick,
            game_loop_hash,
            hud_hash,
            telemetry_hash,
            vfx_hash,
            combined,
        }
    }

    /// Combines four sub-hashes into a single deterministic hash.
    fn combine_hashes(a: u64, b: u64, c: u64, d: u64) -> u64 {
        let mut h = FnvHasher::new();
        a.hash(&mut h);
        b.hash(&mut h);
        c.hash(&mut h);
        d.hash(&mut h);
        h.finish()
    }

    /// Returns `true` if this fingerprint matches another exactly.
    pub fn matches(&self, other: &Self) -> bool {
        self.combined == other.combined
            && self.game_loop_hash == other.game_loop_hash
            && self.hud_hash == other.hud_hash
            && self.telemetry_hash == other.telemetry_hash
            && self.vfx_hash == other.vfx_hash
    }

    /// Build a fingerprint directly from live subsystem references.
    ///
    /// This is a convenience method that hashes each subsystem and combines
    /// the results into a single fingerprint.
    pub fn capture(
        tick: u64,
        gl: &GameLoop,
        hud: &ThreadHud,
        tc: &TelemetryCollector,
        vfx: &VfxAudioDispatch,
    ) -> Self {
        let active_zone = gl.active_zone.map(|c| [c.x, c.y, c.z]);
        let storm_phase_tag: u8 = match gl.storm_state.phase() {
            StormPhase::PreCrossroads => 0,
            StormPhase::DecisionPending => 1,
            StormPhase::ChoiceMade => 2,
            StormPhase::Resolved => 3,
        };
        let storm_choice_tag: Option<u8> = gl.storm_state.choice().map(|c| match c {
            crate::storm_choice::StormChoice::Stabilize => 0,
            crate::storm_choice::StormChoice::Redirect => 1,
        });
        let gl_hash = hash_game_loop_state(
            active_zone,
            storm_phase_tag,
            storm_choice_tag,
            gl.dialogues.len(),
            gl.trigger_actions.len(),
        );

        let stability_entries: Vec<(String, i32, bool, bool)> = hud
            .anchors
            .values()
            .map(|a| {
                (
                    a.anchor_id.clone(),
                    (a.stability * 1000.0) as i32,
                    a.is_repairing,
                    a.was_repaired,
                )
            })
            .collect();
        let hud_hash = hash_hud_state(hud.echoes.balance as i32, &stability_entries);

        let tel_hash = hash_telemetry_state(
            &tc.zones_visited,
            tc.enemies_defeated,
            tc.anchors_repaired,
            tc.echoes_collected,
            tc.echoes_spent,
            tc.dialogues_completed,
            tc.boss_defeated,
            (tc.damage_dealt * 1000.0) as i64,
            (tc.damage_taken * 1000.0) as i64,
        );

        let sc_tag: Option<u8> = vfx.vfx_scene().storm.as_ref().map(|_| {
            if vfx.audio_scene().pending_stingers.is_empty() {
                0
            } else {
                1
            }
        });
        let vfx_hash = hash_vfx_state(
            vfx.current_zone(),
            vfx.in_boss_encounter(),
            sc_tag,
            vfx.vfx_scene().threads.len(),
            vfx.vfx_scene().anchors.len(),
            vfx.audio_scene().pending_stingers.len(),
        );

        Self::new(tick, gl_hash, hud_hash, tel_hash, vfx_hash)
    }
}

// ── Determinism Trace ──────────────────────────────────────────────────

/// A sequence of fingerprints from one full run of the game loop.
#[derive(Debug, Clone)]
pub struct DeterminismTrace {
    /// Label for this run (e.g., `"run_1"`, `"run_2"`).
    pub label: String,
    /// Ordered fingerprints (tick → fingerprint).
    fingerprints: BTreeMap<u64, StateFingerprint>,
}

impl DeterminismTrace {
    /// Creates an empty trace with a label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            fingerprints: BTreeMap::new(),
        }
    }

    /// Records a fingerprint at the given tick.
    pub fn record(&mut self, fingerprint: StateFingerprint) {
        self.fingerprints.insert(fingerprint.tick, fingerprint);
    }

    /// Convenience method: records a fingerprint (by reference) at a specific tick.
    pub fn record_tick(&mut self, tick: u64, fingerprint: &StateFingerprint) {
        // Clone and store with the given tick
        let mut fp = fingerprint.clone();
        fp.tick = tick;
        self.fingerprints.insert(tick, fp);
    }

    /// Returns the number of recorded fingerprints.
    pub fn len(&self) -> usize {
        self.fingerprints.len()
    }

    /// Returns `true` if no fingerprints are recorded.
    pub fn is_empty(&self) -> bool {
        self.fingerprints.is_empty()
    }

    /// Returns the fingerprint at a given tick, if recorded.
    pub fn get(&self, tick: u64) -> Option<&StateFingerprint> {
        self.fingerprints.get(&tick)
    }

    /// Returns all recorded ticks in order.
    pub fn ticks(&self) -> Vec<u64> {
        self.fingerprints.keys().copied().collect()
    }

    /// Compares this trace against another, producing a detailed report.
    pub fn compare(&self, other: &DeterminismTrace) -> DeterminismReport {
        let mut mismatches = Vec::new();
        let mut matched_ticks = 0u64;

        let all_ticks: Vec<u64> = {
            let mut ticks: Vec<u64> = self.fingerprints.keys().copied().collect();
            for tick in other.fingerprints.keys() {
                if !ticks.contains(tick) {
                    ticks.push(*tick);
                }
            }
            ticks.sort_unstable();
            ticks
        };

        for tick in &all_ticks {
            match (self.fingerprints.get(tick), other.fingerprints.get(tick)) {
                (Some(a), Some(b)) => {
                    if a.matches(b) {
                        matched_ticks += 1;
                    } else {
                        mismatches.push(DeterminismMismatch {
                            tick: *tick,
                            subsystem: Self::identify_divergence(a, b),
                            hash_a: a.combined,
                            hash_b: b.combined,
                        });
                    }
                }
                (Some(_), None) => {
                    mismatches.push(DeterminismMismatch {
                        tick: *tick,
                        subsystem: "missing_in_b".to_string(),
                        hash_a: 0,
                        hash_b: 0,
                    });
                }
                (None, Some(_)) => {
                    mismatches.push(DeterminismMismatch {
                        tick: *tick,
                        subsystem: "missing_in_a".to_string(),
                        hash_a: 0,
                        hash_b: 0,
                    });
                }
                (None, None) => {}
            }
        }

        DeterminismReport {
            run_a: self.label.clone(),
            run_b: other.label.clone(),
            total_ticks: all_ticks.len() as u64,
            matched_ticks,
            mismatches,
        }
    }

    /// Identifies which subsystem diverged first between two fingerprints.
    fn identify_divergence(a: &StateFingerprint, b: &StateFingerprint) -> String {
        if a.game_loop_hash != b.game_loop_hash {
            return "game_loop".to_string();
        }
        if a.hud_hash != b.hud_hash {
            return "hud".to_string();
        }
        if a.telemetry_hash != b.telemetry_hash {
            return "telemetry".to_string();
        }
        if a.vfx_hash != b.vfx_hash {
            return "vfx_dispatch".to_string();
        }
        "combined_only".to_string()
    }
}

// ── Report ─────────────────────────────────────────────────────────────

/// Describes a single tick where two runs diverged.
#[derive(Debug, Clone)]
pub struct DeterminismMismatch {
    /// Tick index where the divergence occurred.
    pub tick: u64,
    /// Which subsystem diverged (e.g., `"game_loop"`, `"hud"`).
    pub subsystem: String,
    /// Combined hash from run A.
    pub hash_a: u64,
    /// Combined hash from run B.
    pub hash_b: u64,
}

/// Full comparison report between two determinism traces.
#[derive(Debug, Clone)]
pub struct DeterminismReport {
    /// Label of the first run.
    pub run_a: String,
    /// Label of the second run.
    pub run_b: String,
    /// Total ticks compared.
    pub total_ticks: u64,
    /// Number of ticks that matched exactly.
    pub matched_ticks: u64,
    /// List of mismatched ticks.
    pub mismatches: Vec<DeterminismMismatch>,
}

impl DeterminismReport {
    /// Returns `true` if all ticks matched (zero mismatches).
    pub fn is_consistent(&self) -> bool {
        self.mismatches.is_empty()
    }

    /// Returns the consistency ratio (0.0–1.0).
    pub fn consistency_ratio(&self) -> f32 {
        if self.total_ticks == 0 {
            return 1.0;
        }
        self.matched_ticks as f32 / self.total_ticks as f32
    }

    /// Returns the first tick where divergence occurred, if any.
    pub fn first_divergence_tick(&self) -> Option<u64> {
        self.mismatches.first().map(|m| m.tick)
    }

    /// Produces a human-readable summary string.
    pub fn summary(&self) -> String {
        if self.is_consistent() {
            format!(
                "DETERMINISTIC: {} vs {} — {}/{} ticks match (100%)",
                self.run_a, self.run_b, self.matched_ticks, self.total_ticks
            )
        } else {
            format!(
                "DIVERGED: {} vs {} — {}/{} ticks match ({:.1}%), first divergence at tick {} ({})",
                self.run_a,
                self.run_b,
                self.matched_ticks,
                self.total_ticks,
                self.consistency_ratio() * 100.0,
                self.mismatches.first().map(|m| m.tick).unwrap_or(0),
                self.mismatches
                    .first()
                    .map(|m| m.subsystem.as_str())
                    .unwrap_or("unknown"),
            )
        }
    }
}

// ── Hashable State Extractors ──────────────────────────────────────────

/// Hashes game loop observable state into a u64.
///
/// Captures: active zone, storm phase, number of registered dialogues,
/// number of trigger actions, and storm choice.
pub fn hash_game_loop_state(
    active_zone: Option<[i32; 3]>,
    storm_phase: u8,
    storm_choice: Option<u8>,
    dialogue_count: usize,
    trigger_count: usize,
) -> u64 {
    let mut h = FnvHasher::new();
    active_zone.hash(&mut h);
    storm_phase.hash(&mut h);
    storm_choice.hash(&mut h);
    dialogue_count.hash(&mut h);
    trigger_count.hash(&mut h);
    h.finish()
}

/// Hashes HUD observable state into a u64.
///
/// Captures: echo balance, number of stability entries, each entry's
/// (id, stability × 1000 as integer, is_repairing, was_repaired).
pub fn hash_hud_state(echo_balance: i32, stability_entries: &[(String, i32, bool, bool)]) -> u64 {
    let mut h = FnvHasher::new();
    echo_balance.hash(&mut h);
    stability_entries.len().hash(&mut h);
    for (id, stab, repairing, repaired) in stability_entries {
        id.hash(&mut h);
        stab.hash(&mut h);
        repairing.hash(&mut h);
        repaired.hash(&mut h);
    }
    h.finish()
}

/// Hashes telemetry counters into a u64.
#[allow(clippy::too_many_arguments)]
pub fn hash_telemetry_state(
    zones_visited: &[String],
    enemies_defeated: u32,
    anchors_repaired: u32,
    echoes_collected: u32,
    echoes_spent: u32,
    dialogues_completed: u32,
    boss_defeated: bool,
    damage_dealt_milliunits: i64,
    damage_taken_milliunits: i64,
) -> u64 {
    let mut h = FnvHasher::new();
    zones_visited.len().hash(&mut h);
    for z in zones_visited {
        z.hash(&mut h);
    }
    enemies_defeated.hash(&mut h);
    anchors_repaired.hash(&mut h);
    echoes_collected.hash(&mut h);
    echoes_spent.hash(&mut h);
    dialogues_completed.hash(&mut h);
    boss_defeated.hash(&mut h);
    damage_dealt_milliunits.hash(&mut h);
    damage_taken_milliunits.hash(&mut h);
    h.finish()
}

/// Hashes VFX dispatch observable state into a u64.
pub fn hash_vfx_state(
    current_zone: usize,
    in_boss_encounter: bool,
    storm_choice: Option<u8>,
    thread_count: usize,
    anchor_count: usize,
    stinger_count: usize,
) -> u64 {
    let mut h = FnvHasher::new();
    current_zone.hash(&mut h);
    in_boss_encounter.hash(&mut h);
    storm_choice.hash(&mut h);
    thread_count.hash(&mut h);
    anchor_count.hash(&mut h);
    stinger_count.hash(&mut h);
    h.finish()
}

// ── FNV-1a Hasher (deterministic, no randomization) ────────────────────

/// Fowler–Noll–Vo 1a hasher — platform-stable, no SipHash randomization.
///
/// This ensures hashes are identical across runs (unlike `DefaultHasher`
/// which uses per-process random keys).
struct FnvHasher {
    state: u64,
}

impl FnvHasher {
    const OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;

    fn new() -> Self {
        Self {
            state: Self::OFFSET_BASIS,
        }
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.state
    }

    fn write(&mut self, bytes: &[u8]) {
        for &byte in bytes {
            self.state ^= u64::from(byte);
            self.state = self.state.wrapping_mul(Self::PRIME);
        }
    }
}

// ── Multi-Run Validator ────────────────────────────────────────────────

/// Validates determinism across N runs by collecting traces and comparing.
#[derive(Debug, Clone)]
pub struct MultiRunValidator {
    traces: Vec<DeterminismTrace>,
}

impl Default for MultiRunValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl MultiRunValidator {
    /// Creates a new empty validator.
    #[must_use]
    pub fn new() -> Self {
        Self { traces: Vec::new() }
    }

    /// Creates a new validator with a capacity hint.
    pub fn with_capacity(expected_runs: usize) -> Self {
        Self {
            traces: Vec::with_capacity(expected_runs),
        }
    }

    /// Adds a completed trace from one run.
    pub fn add_trace(&mut self, trace: DeterminismTrace) {
        self.traces.push(trace);
    }

    /// Alias for [`add_trace`](Self::add_trace).
    pub fn add_run(&mut self, trace: DeterminismTrace) {
        self.add_trace(trace);
    }

    /// Returns the number of traces collected.
    pub fn run_count(&self) -> usize {
        self.traces.len()
    }

    /// Validates all traces against the first run (baseline).
    ///
    /// Returns one report per comparison (baseline vs run_2, baseline vs run_3, …).
    pub fn validate(&self) -> Vec<DeterminismReport> {
        if self.traces.len() < 2 {
            return Vec::new();
        }
        let baseline = &self.traces[0];
        self.traces[1..]
            .iter()
            .map(|trace| baseline.compare(trace))
            .collect()
    }

    /// Returns `true` if all runs are consistent with the baseline.
    pub fn all_consistent(&self) -> bool {
        self.validate().iter().all(|r| r.is_consistent())
    }

    /// Produces a human-readable validation summary.
    pub fn summary(&self) -> String {
        let reports = self.validate();
        if reports.is_empty() {
            return "No comparisons possible (need at least 2 runs)".to_string();
        }
        let mut lines = Vec::new();
        lines.push(format!(
            "Determinism validation: {} runs",
            self.traces.len()
        ));
        for report in &reports {
            lines.push(format!("  {}", report.summary()));
        }
        let all_ok = reports.iter().all(|r| r.is_consistent());
        lines.push(format!(
            "Result: {}",
            if all_ok {
                "ALL RUNS DETERMINISTIC"
            } else {
                "DIVERGENCE DETECTED"
            }
        ));
        lines.join("\n")
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_fingerprint(tick: u64) -> StateFingerprint {
        StateFingerprint::new(tick, 100, 200, 300, 400)
    }

    fn divergent_fingerprint(tick: u64) -> StateFingerprint {
        StateFingerprint::new(tick, 100, 999, 300, 400) // HUD differs
    }

    #[test]
    fn fingerprint_matches_identical() {
        let a = sample_fingerprint(0);
        let b = sample_fingerprint(0);
        assert!(a.matches(&b));
    }

    #[test]
    fn fingerprint_detects_difference() {
        let a = sample_fingerprint(0);
        let b = divergent_fingerprint(0);
        assert!(!a.matches(&b));
    }

    #[test]
    fn trace_record_and_len() {
        let mut trace = DeterminismTrace::new("test");
        assert!(trace.is_empty());
        trace.record(sample_fingerprint(0));
        trace.record(sample_fingerprint(1));
        assert_eq!(trace.len(), 2);
    }

    #[test]
    fn trace_compare_identical() {
        let mut a = DeterminismTrace::new("run_1");
        let mut b = DeterminismTrace::new("run_2");
        for tick in 0..10 {
            a.record(sample_fingerprint(tick));
            b.record(sample_fingerprint(tick));
        }
        let report = a.compare(&b);
        assert!(report.is_consistent());
        assert_eq!(report.matched_ticks, 10);
        assert!(report.consistency_ratio() > 0.999);
    }

    #[test]
    fn trace_compare_diverged() {
        let mut a = DeterminismTrace::new("run_1");
        let mut b = DeterminismTrace::new("run_2");
        for tick in 0..10 {
            a.record(sample_fingerprint(tick));
            if tick == 5 {
                b.record(divergent_fingerprint(tick));
            } else {
                b.record(sample_fingerprint(tick));
            }
        }
        let report = a.compare(&b);
        assert!(!report.is_consistent());
        assert_eq!(report.mismatches.len(), 1);
        assert_eq!(report.first_divergence_tick(), Some(5));
        assert_eq!(report.mismatches[0].subsystem, "hud");
    }

    #[test]
    fn trace_compare_missing_tick() {
        let mut a = DeterminismTrace::new("run_1");
        let mut b = DeterminismTrace::new("run_2");
        a.record(sample_fingerprint(0));
        a.record(sample_fingerprint(1));
        b.record(sample_fingerprint(0));
        // b is missing tick 1
        let report = a.compare(&b);
        assert!(!report.is_consistent());
        assert_eq!(report.mismatches[0].subsystem, "missing_in_b");
    }

    #[test]
    fn multi_run_validator_all_consistent() {
        let mut validator = MultiRunValidator::new();
        for run_idx in 0..3 {
            let mut trace = DeterminismTrace::new(format!("run_{}", run_idx));
            for tick in 0..20 {
                trace.record(sample_fingerprint(tick));
            }
            validator.add_trace(trace);
        }
        assert_eq!(validator.run_count(), 3);
        assert!(validator.all_consistent());
        let summary = validator.summary();
        assert!(summary.contains("ALL RUNS DETERMINISTIC"));
    }

    #[test]
    fn multi_run_validator_detects_divergence() {
        let mut validator = MultiRunValidator::new();

        let mut trace1 = DeterminismTrace::new("run_1");
        let mut trace2 = DeterminismTrace::new("run_2");
        for tick in 0..10 {
            trace1.record(sample_fingerprint(tick));
            if tick == 7 {
                trace2.record(divergent_fingerprint(tick));
            } else {
                trace2.record(sample_fingerprint(tick));
            }
        }
        validator.add_trace(trace1);
        validator.add_trace(trace2);
        assert!(!validator.all_consistent());
        let summary = validator.summary();
        assert!(summary.contains("DIVERGENCE DETECTED"));
    }

    #[test]
    fn multi_run_validator_empty() {
        let validator = MultiRunValidator::new();
        assert!(validator.all_consistent()); // Vacuously true — no comparisons
        assert!(validator.validate().is_empty());
    }

    #[test]
    fn multi_run_validator_single_run() {
        let mut validator = MultiRunValidator::new();
        let mut trace = DeterminismTrace::new("solo");
        trace.record(sample_fingerprint(0));
        validator.add_trace(trace);
        assert!(validator.all_consistent()); // Vacuously true
    }

    #[test]
    fn hash_game_loop_deterministic() {
        let h1 = hash_game_loop_state(Some([100, 0, 0]), 2, Some(1), 3, 5);
        let h2 = hash_game_loop_state(Some([100, 0, 0]), 2, Some(1), 3, 5);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_game_loop_differs_on_zone() {
        let h1 = hash_game_loop_state(Some([100, 0, 0]), 2, Some(1), 3, 5);
        let h2 = hash_game_loop_state(Some([101, 0, 0]), 2, Some(1), 3, 5);
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_hud_deterministic() {
        let entries = vec![
            ("anchor_1".to_string(), 750, false, true),
            ("anchor_2".to_string(), 500, true, false),
        ];
        let h1 = hash_hud_state(42, &entries);
        let h2 = hash_hud_state(42, &entries);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_hud_differs_on_balance() {
        let entries = vec![("a".to_string(), 750, false, true)];
        let h1 = hash_hud_state(42, &entries);
        let h2 = hash_hud_state(43, &entries);
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_telemetry_deterministic() {
        let zones = vec!["Z0".to_string(), "Z1".to_string()];
        let h1 = hash_telemetry_state(&zones, 5, 2, 10, 3, 1, false, 1500, 300);
        let h2 = hash_telemetry_state(&zones, 5, 2, 10, 3, 1, false, 1500, 300);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_telemetry_differs() {
        let zones = vec!["Z0".to_string()];
        let h1 = hash_telemetry_state(&zones, 5, 2, 10, 3, 1, false, 1500, 300);
        let h2 = hash_telemetry_state(&zones, 6, 2, 10, 3, 1, false, 1500, 300);
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_vfx_deterministic() {
        let h1 = hash_vfx_state(2, true, Some(1), 4, 3, 2);
        let h2 = hash_vfx_state(2, true, Some(1), 4, 3, 2);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_vfx_differs() {
        let h1 = hash_vfx_state(2, true, Some(1), 4, 3, 2);
        let h2 = hash_vfx_state(3, true, Some(1), 4, 3, 2);
        assert_ne!(h1, h2);
    }

    #[test]
    fn fnv_hasher_deterministic() {
        let mut h1 = FnvHasher::new();
        42u64.hash(&mut h1);
        let mut h2 = FnvHasher::new();
        42u64.hash(&mut h2);
        assert_eq!(h1.finish(), h2.finish());
    }

    #[test]
    fn fnv_hasher_collision_resistant() {
        let mut h1 = FnvHasher::new();
        1u64.hash(&mut h1);
        let mut h2 = FnvHasher::new();
        2u64.hash(&mut h2);
        assert_ne!(h1.finish(), h2.finish());
    }

    #[test]
    fn report_summary_consistent() {
        let report = DeterminismReport {
            run_a: "a".to_string(),
            run_b: "b".to_string(),
            total_ticks: 100,
            matched_ticks: 100,
            mismatches: Vec::new(),
        };
        let s = report.summary();
        assert!(s.contains("DETERMINISTIC"));
        assert!(s.contains("100/100"));
    }

    #[test]
    fn report_summary_diverged() {
        let report = DeterminismReport {
            run_a: "a".to_string(),
            run_b: "b".to_string(),
            total_ticks: 10,
            matched_ticks: 9,
            mismatches: vec![DeterminismMismatch {
                tick: 5,
                subsystem: "game_loop".to_string(),
                hash_a: 111,
                hash_b: 222,
            }],
        };
        let s = report.summary();
        assert!(s.contains("DIVERGED"));
        assert!(s.contains("tick 5"));
    }

    #[test]
    fn consistency_ratio_zero_ticks() {
        let report = DeterminismReport {
            run_a: "a".to_string(),
            run_b: "b".to_string(),
            total_ticks: 0,
            matched_ticks: 0,
            mismatches: Vec::new(),
        };
        assert!((report.consistency_ratio() - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn trace_get_tick() {
        let mut trace = DeterminismTrace::new("test");
        trace.record(sample_fingerprint(42));
        assert!(trace.get(42).is_some());
        assert!(trace.get(43).is_none());
    }

    #[test]
    fn trace_ticks_ordered() {
        let mut trace = DeterminismTrace::new("test");
        trace.record(sample_fingerprint(5));
        trace.record(sample_fingerprint(1));
        trace.record(sample_fingerprint(3));
        assert_eq!(trace.ticks(), vec![1, 3, 5]);
    }

    #[test]
    fn report_summary_with_empty_mismatches_does_not_panic() {
        // Manually construct a report with is_consistent = true.
        let report = DeterminismReport {
            run_a: "a".to_string(),
            run_b: "b".to_string(),
            total_ticks: 100,
            matched_ticks: 100,
            mismatches: vec![],
        };
        assert!(report.is_consistent());
        let summary = report.summary();
        assert!(summary.contains("DETERMINISTIC"));
    }

    #[test]
    fn report_summary_diverged_uses_first_safely() {
        let report = DeterminismReport {
            run_a: "a".to_string(),
            run_b: "b".to_string(),
            total_ticks: 10,
            matched_ticks: 8,
            mismatches: vec![
                DeterminismMismatch {
                    tick: 3,
                    subsystem: "game_loop".to_string(),
                    hash_a: 111,
                    hash_b: 222,
                },
                DeterminismMismatch {
                    tick: 7,
                    subsystem: "hud".to_string(),
                    hash_a: 333,
                    hash_b: 444,
                },
            ],
        };
        let summary = report.summary();
        assert!(summary.contains("DIVERGED"));
        assert!(summary.contains("tick 3"));
        assert!(summary.contains("game_loop"));
    }

    #[test]
    fn multi_run_validator_single_trace_no_panic() {
        let mut validator = MultiRunValidator::new();
        let trace = DeterminismTrace::new("solo");
        validator.add_trace(trace);
        let reports = validator.validate();
        assert!(reports.is_empty());
        // summary should not panic on 0 comparisons.
        let summary = validator.summary();
        assert!(summary.contains("No comparisons possible"));
    }
}
