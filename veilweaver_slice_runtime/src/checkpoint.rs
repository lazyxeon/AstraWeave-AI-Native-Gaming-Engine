//! Checkpoint / save system for the Veilweaver vertical slice.
//!
//! Captures a serialisable snapshot of the full runtime state — game loop,
//! HUD, telemetry, and VFX/audio — so that a run can be saved to disk and
//! resumed later, or compared across determinism runs.
//!
//! # Design
//!
//! Because several data-bearing types in the crate (`AffinityRank`,
//! `BossPhase`, inner dialogue-runner state) do **not** derive
//! `serde::Serialize`/`Deserialize`, the checkpoint uses *proxy* structs
//! that store only plain-data equivalents.  The `capture_*` helpers
//! convert live objects into their proxy form.
//!
//! Serialisation uses **serde_json** so checkpoints are human-readable
//! and diffable.
//!
//! # Usage
//!
//! ```rust,no_run
//! use veilweaver_slice_runtime::checkpoint::SliceCheckpoint;
//! use veilweaver_slice_runtime::game_loop::GameLoop;
//! use veilweaver_slice_runtime::hud_state::ThreadHud;
//! use veilweaver_slice_runtime::telemetry::TelemetryCollector;
//! use veilweaver_slice_runtime::vfx_dispatch::VfxAudioDispatch;
//!
//! # fn main() -> anyhow::Result<()> {
//! let game_loop = GameLoop::new();
//! let hud = ThreadHud::new(0);
//! let telemetry = TelemetryCollector::new();
//! let vfx = VfxAudioDispatch::new();
//!
//! let snap = SliceCheckpoint::capture(0, "quicksave", &game_loop, &hud, &telemetry, &vfx);
//! let json = snap.save_to_json()?;
//! std::fs::write("checkpoint.json", &json)?;
//!
//! let loaded = SliceCheckpoint::load_from_json(&json)?;
//! assert_eq!(snap.header.tick, loaded.header.tick);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::boss_hud::BossHealthBar;
use crate::combat::{CombatEncounter, EncounterPhase};
use crate::companion_hud::CompanionAffinityMeter;
use crate::decision_ui::{DecisionPhase, DecisionRadial};
use crate::game_loop::GameLoop;
use crate::hud_state::ThreadHud;
use crate::player_state::PlayerState;
use crate::recap_panel::RecapPanel;
use crate::storm_choice::{StormChoice, StormPhase};
use crate::telemetry::TelemetryCollector;
use crate::vfx_dispatch::VfxAudioDispatch;
use crate::walkthrough::{SliceOrchestrator, WalkthroughBeat};

// ── Checkpoint header ──────────────────────────────────────────────────

/// Lightweight metadata attached to every checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CheckpointHeader {
    /// Semantic version of the checkpoint format.
    pub format_version: String,
    /// Monotonic tick counter at save time.
    pub tick: u64,
    /// Wall-clock ISO-8601 timestamp (or empty if unavailable).
    pub timestamp: String,
    /// Optional human label ("autosave", "zone transition", …).
    pub label: String,
}

impl CheckpointHeader {
    /// Create a new header for the given tick.
    #[must_use]
    pub fn new(tick: u64, label: impl Into<String>) -> Self {
        Self {
            format_version: "1.0.0".to_string(),
            tick,
            timestamp: String::new(), // headless-safe — no system clock dep
            label: label.into(),
        }
    }
}

// ── Game-loop proxy ────────────────────────────────────────────────────

/// Serialisable representation of [`GameLoop`] state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GameLoopSnapshot {
    /// Active zone coordinate (x, y, z) if any.
    pub active_zone: Option<[i32; 3]>,
    /// Storm-choice phase as a string tag.
    pub storm_phase: String,
    /// Resolved storm choice, if made.
    pub storm_choice: Option<StormChoice>,
    /// IDs of dialogues that have been started / completed.
    pub dialogue_ids: Vec<String>,
    /// Number of registered trigger actions.
    pub trigger_action_count: usize,
}

impl GameLoopSnapshot {
    /// Build from a live [`GameLoop`].
    pub fn capture(gl: &GameLoop) -> Self {
        let active_zone = gl.active_zone.map(|c| [c.x, c.y, c.z]);

        let storm_phase = match gl.storm_state.phase() {
            StormPhase::PreCrossroads => "PreCrossroads",
            StormPhase::DecisionPending => "DecisionPending",
            StormPhase::ChoiceMade => "ChoiceMade",
            StormPhase::Resolved => "Resolved",
        }
        .to_string();

        let storm_choice = gl.storm_state.choice();

        let mut dialogue_ids: Vec<String> = gl.dialogues.keys().cloned().collect();
        dialogue_ids.sort();

        Self {
            active_zone,
            storm_phase,
            storm_choice,
            dialogue_ids,
            trigger_action_count: gl.trigger_actions.len(),
        }
    }
}

// ── HUD proxy ──────────────────────────────────────────────────────────

/// Serialisable anchor stability entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnchorSnapshot {
    pub anchor_id: String,
    pub stability: f32,
    pub display_value: f32,
    pub is_repairing: bool,
    pub was_repaired: bool,
}

/// Serialisable representation of [`ThreadHud`] state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HudSnapshot {
    /// Per-anchor stability data.
    pub anchors: Vec<AnchorSnapshot>,
    /// Echo currency balance.
    pub echo_balance: u32,
    /// Number of active echo feedback floats.
    pub active_feedbacks: usize,
}

impl HudSnapshot {
    /// Build from a live [`ThreadHud`].
    pub fn capture(hud: &ThreadHud) -> Self {
        let anchors = hud
            .anchors
            .values()
            .map(|a| AnchorSnapshot {
                anchor_id: a.anchor_id.clone(),
                stability: a.stability,
                display_value: a.display_value,
                is_repairing: a.is_repairing,
                was_repaired: a.was_repaired,
            })
            .collect();

        Self {
            anchors,
            echo_balance: hud.echoes.balance,
            active_feedbacks: hud.echoes.active_feedback_count(),
        }
    }
}

// ── Telemetry proxy ────────────────────────────────────────────────────

/// Serialisable representation of [`TelemetryCollector`] state.
///
/// Fields that reference non-serde enums (`AffinityRank`, `BossPhase`)
/// are stored as string tags.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TelemetrySnapshot {
    // Combat
    pub damage_dealt: f32,
    pub damage_taken: f32,
    pub enemies_defeated: u32,
    pub deaths: u32,
    pub combos_landed: u32,
    // Exploration
    pub zones_visited: Vec<String>,
    pub secrets_found: u32,
    // Weaving
    pub anchors_repaired: u32,
    pub echoes_collected: u32,
    pub echoes_spent: u32,
    // Companion
    pub companion_support_actions: u32,
    /// String tag for `AffinityRank` if set (e.g. "Bonded").
    pub final_affinity_rank: Option<String>,
    // Narrative
    pub storm_choice: Option<StormChoice>,
    pub dialogues_completed: u32,
    // Boss
    /// String tags for each `BossPhase` seen.
    pub boss_phases_seen: Vec<String>,
    pub boss_telegraphs_dodged: u32,
    pub boss_defeated: bool,
    // Timing
    pub total_time: f32,
    pub boss_fight_time: f32,
}

impl TelemetrySnapshot {
    /// Build from a live [`TelemetryCollector`].
    pub fn capture(tc: &TelemetryCollector) -> Self {
        Self {
            damage_dealt: tc.damage_dealt,
            damage_taken: tc.damage_taken,
            enemies_defeated: tc.enemies_defeated,
            deaths: tc.deaths,
            combos_landed: tc.combos_landed,
            zones_visited: tc.zones_visited.clone(),
            secrets_found: tc.secrets_found,
            anchors_repaired: tc.anchors_repaired,
            echoes_collected: tc.echoes_collected,
            echoes_spent: tc.echoes_spent,
            companion_support_actions: tc.companion_support_actions,
            final_affinity_rank: tc.final_affinity_rank.map(|r| format!("{}", r)),
            storm_choice: tc.storm_choice,
            dialogues_completed: tc.dialogues_completed,
            boss_phases_seen: tc
                .boss_phases_seen
                .iter()
                .map(|p| format!("{}", p))
                .collect(),
            boss_telegraphs_dodged: tc.boss_telegraphs_dodged,
            boss_defeated: tc.boss_defeated,
            total_time: tc.total_time,
            boss_fight_time: tc.boss_fight_time,
        }
    }
}

// ── VFX/audio proxy ────────────────────────────────────────────────────

/// Serialisable representation of [`VfxAudioDispatch`] state.
///
/// `VfxAudioDispatch` already derives `Serialize`, so we wrap its JSON
/// value directly to avoid duplicating every sub-field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VfxSnapshot {
    /// Current zone index (0–4).
    pub current_zone: usize,
    /// Whether the boss encounter is active.
    pub in_boss_encounter: bool,
    /// Elapsed time.
    pub elapsed: f32,
    /// Full JSON blob of the VfxAudioDispatch (leverages existing Serialize).
    pub full_state: serde_json::Value,
}

impl VfxSnapshot {
    /// Build from a live [`VfxAudioDispatch`].
    ///
    /// Falls back to a null blob if serialisation fails (should not happen).
    /// The `full_state` is normalised through a JSON-string round-trip so
    /// that `PartialEq` comparisons are stable across save/load cycles
    /// (avoids f32→f64 representation drift in `serde_json::Value`).
    pub fn capture(vfx: &VfxAudioDispatch) -> Self {
        let raw_value = serde_json::to_value(vfx).unwrap_or(serde_json::Value::Null);
        // Normalise: Value → String → Value (stabilises float precision).
        let full_state = serde_json::to_string(&raw_value)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or(raw_value);

        Self {
            current_zone: vfx.current_zone(),
            in_boss_encounter: vfx.in_boss_encounter(),
            elapsed: vfx.elapsed(),
            full_state,
        }
    }
}

// ── Player proxy ───────────────────────────────────────────────────

/// Serialisable representation of [`PlayerState`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PlayerSnapshot {
    pub hp: f32,
    pub max_hp: f32,
    pub zone_name: String,
    pub echoes: u32,
    pub anchors_repaired: u32,
    pub tutorial_complete: bool,
    pub alive: bool,
    pub thread_stability: f32,
}

impl PlayerSnapshot {
    /// Build from a live [`PlayerState`].
    pub fn capture(ps: &PlayerState) -> Self {
        Self {
            hp: ps.hp,
            max_hp: ps.max_hp,
            zone_name: ps.zone_name.clone(),
            echoes: ps.echoes,
            anchors_repaired: ps.anchors_repaired,
            tutorial_complete: ps.tutorial_complete,
            alive: ps.alive,
            thread_stability: ps.thread_stability,
        }
    }
}

// ── Combat proxy ───────────────────────────────────────────────────

/// Serialisable representation of [`CombatEncounter`] state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CombatSnapshot {
    pub encounter_id: String,
    pub phase: String,
    pub current_wave: usize,
    pub total_waves: usize,
    pub total_kills: u32,
}

impl CombatSnapshot {
    /// Build from a live [`CombatEncounter`].
    pub fn capture(ce: &CombatEncounter) -> Self {
        let phase = match ce.phase {
            EncounterPhase::Idle => "Idle",
            EncounterPhase::Active => "Active",
            EncounterPhase::Cleared => "Cleared",
        }
        .to_string();

        Self {
            encounter_id: ce.encounter_id.clone(),
            phase,
            current_wave: ce.current_wave,
            total_waves: ce.waves.len(),
            total_kills: ce.total_kills,
        }
    }
}

// ── Boss HUD proxy ─────────────────────────────────────────────────

/// Serialisable representation of [`BossHealthBar`] state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BossHudSnapshot {
    pub boss_name: String,
    pub current_hp: f32,
    pub max_hp: f32,
    pub hp_fraction: f32,
    pub phase: String,
    pub is_defeated: bool,
    pub active_telegraphs: usize,
}

impl BossHudSnapshot {
    /// Build from a live [`BossHealthBar`].
    pub fn capture(bhb: &BossHealthBar) -> Self {
        Self {
            boss_name: bhb.boss_name.clone(),
            current_hp: bhb.current_hp,
            max_hp: bhb.max_hp,
            hp_fraction: bhb.hp_fraction(),
            phase: bhb.phase_label().to_string(),
            is_defeated: bhb.is_defeated(),
            active_telegraphs: bhb.telegraphs.len(),
        }
    }
}

// ── Companion HUD proxy ────────────────────────────────────────────

/// Serialisable representation of [`CompanionAffinityMeter`] state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompanionHudSnapshot {
    pub companion_name: String,
    pub affinity: f32,
    pub rank: String,
}

impl CompanionHudSnapshot {
    /// Build from a live [`CompanionAffinityMeter`].
    pub fn capture(cam: &CompanionAffinityMeter) -> Self {
        Self {
            companion_name: cam.companion_name.clone(),
            affinity: cam.affinity(),
            rank: format!("{}", cam.rank()),
        }
    }
}

// ── Decision radial proxy ──────────────────────────────────────────

/// Serialisable representation of [`DecisionRadial`] state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionSnapshot {
    pub prompt: String,
    pub phase: String,
    pub option_count: usize,
    pub selected_index: Option<usize>,
}

impl DecisionSnapshot {
    /// Build from a live [`DecisionRadial`].
    pub fn capture(dr: &DecisionRadial) -> Self {
        let phase = match dr.phase() {
            DecisionPhase::Hidden => "Hidden",
            DecisionPhase::Opening => "Opening",
            DecisionPhase::Active => "Active",
            DecisionPhase::Closing => "Closing",
            DecisionPhase::Done => "Done",
        }
        .to_string();

        Self {
            prompt: dr.prompt.clone(),
            phase,
            option_count: dr.options.len(),
            selected_index: dr.selected_index,
        }
    }
}

// ── Recap proxy ────────────────────────────────────────────────────

/// Serialisable representation of [`RecapPanel`] state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecapSnapshot {
    pub is_finalized: bool,
    pub metric_count: usize,
    pub overall_rating: Option<String>,
}

impl RecapSnapshot {
    /// Build from a live [`RecapPanel`].
    pub fn capture(rp: &RecapPanel) -> Self {
        Self {
            is_finalized: rp.is_finalized(),
            metric_count: rp.metric_count(),
            overall_rating: rp.overall_rating().map(|r| format!("{}", r)),
        }
    }
}

// ── Root checkpoint ────────────────────────────────────────────────────

/// A complete, serialisable snapshot of the Veilweaver vertical-slice state.
///
/// This is the unit of persistence — save it to a file, reload it later,
/// or diff it against another run for determinism validation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SliceCheckpoint {
    /// Metadata.
    pub header: CheckpointHeader,
    /// Game-loop subsystem state.
    pub game_loop: GameLoopSnapshot,
    /// Thread HUD state.
    pub hud: HudSnapshot,
    /// Telemetry counters.
    pub telemetry: TelemetrySnapshot,
    /// VFX / audio dispatch state.
    pub vfx: VfxSnapshot,
    /// Player state (HP, zone, echoes, etc.).
    #[serde(default)]
    pub player: Option<PlayerSnapshot>,
    /// Combat encounter state.
    #[serde(default)]
    pub combat: Option<CombatSnapshot>,
    /// Current walkthrough beat.
    #[serde(default)]
    pub walkthrough_beat: Option<WalkthroughBeat>,
    /// Boss HUD state.
    #[serde(default)]
    pub boss_hud: Option<BossHudSnapshot>,
    /// Companion affinity meter state.
    #[serde(default)]
    pub companion_hud: Option<CompanionHudSnapshot>,
    /// Decision radial state.
    #[serde(default)]
    pub decision: Option<DecisionSnapshot>,
    /// Recap panel state.
    #[serde(default)]
    pub recap: Option<RecapSnapshot>,
}

impl SliceCheckpoint {
    /// Capture a full checkpoint from the live subsystem references.
    ///
    /// This is the legacy API — new callers should prefer
    /// [`capture_from_orchestrator`] which captures the full state.
    pub fn capture(
        tick: u64,
        label: impl Into<String>,
        game_loop: &GameLoop,
        hud: &ThreadHud,
        telemetry: &TelemetryCollector,
        vfx: &VfxAudioDispatch,
    ) -> Self {
        Self {
            header: CheckpointHeader::new(tick, label),
            game_loop: GameLoopSnapshot::capture(game_loop),
            hud: HudSnapshot::capture(hud),
            telemetry: TelemetrySnapshot::capture(telemetry),
            vfx: VfxSnapshot::capture(vfx),
            player: None,
            combat: None,
            walkthrough_beat: None,
            boss_hud: None,
            companion_hud: None,
            decision: None,
            recap: None,
        }
    }

    /// Capture a complete checkpoint from a [`SliceOrchestrator`].
    ///
    /// This captures the full orchestrator state including player, combat,
    /// walkthrough beat, all HUD subsystems, and VFX/audio.
    pub fn capture_from_orchestrator(orch: &SliceOrchestrator, label: impl Into<String>) -> Self {
        Self {
            header: CheckpointHeader::new(orch.tick_count, label),
            game_loop: GameLoopSnapshot::capture(&orch.game_loop),
            hud: HudSnapshot::capture(&orch.thread_hud),
            telemetry: TelemetrySnapshot::capture(&orch.telemetry),
            vfx: VfxSnapshot::capture(&orch.vfx_audio),
            player: Some(PlayerSnapshot::capture(&orch.player)),
            combat: Some(CombatSnapshot::capture(&orch.combat)),
            walkthrough_beat: Some(orch.beat()),
            boss_hud: Some(BossHudSnapshot::capture(&orch.boss_hud)),
            companion_hud: Some(CompanionHudSnapshot::capture(&orch.companion_hud)),
            decision: Some(DecisionSnapshot::capture(&orch.decision_radial)),
            recap: Some(RecapSnapshot::capture(&orch.recap)),
        }
    }

    /// Serialise to pretty-printed JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if serialisation fails (e.g., non-finite floats).
    pub fn save_to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self).context("Failed to serialise checkpoint to JSON")
    }

    /// Deserialise from JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is malformed or missing required fields.
    pub fn load_from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).context("Failed to deserialise checkpoint from JSON")
    }

    /// Compare two checkpoints, returning a human-readable diff summary.
    ///
    /// The comparison ignores the `header.timestamp` field (which is
    /// expected to differ between runs).
    pub fn diff(&self, other: &Self) -> CheckpointDiff {
        let mut mismatches = Vec::new();

        if self.header.tick != other.header.tick {
            mismatches.push(format!(
                "header.tick: {} vs {}",
                self.header.tick, other.header.tick
            ));
        }

        // Game loop
        if self.game_loop.active_zone != other.game_loop.active_zone {
            mismatches.push(format!(
                "game_loop.active_zone: {:?} vs {:?}",
                self.game_loop.active_zone, other.game_loop.active_zone
            ));
        }
        if self.game_loop.storm_phase != other.game_loop.storm_phase {
            mismatches.push(format!(
                "game_loop.storm_phase: {} vs {}",
                self.game_loop.storm_phase, other.game_loop.storm_phase
            ));
        }
        if self.game_loop.storm_choice != other.game_loop.storm_choice {
            mismatches.push(format!(
                "game_loop.storm_choice: {:?} vs {:?}",
                self.game_loop.storm_choice, other.game_loop.storm_choice
            ));
        }

        // HUD
        if self.hud.echo_balance != other.hud.echo_balance {
            mismatches.push(format!(
                "hud.echo_balance: {} vs {}",
                self.hud.echo_balance, other.hud.echo_balance
            ));
        }
        if self.hud.anchors.len() != other.hud.anchors.len() {
            mismatches.push(format!(
                "hud.anchors.len: {} vs {}",
                self.hud.anchors.len(),
                other.hud.anchors.len()
            ));
        }

        // Telemetry — compare all scalar counters
        diff_f32(
            &mut mismatches,
            "telemetry.damage_dealt",
            self.telemetry.damage_dealt,
            other.telemetry.damage_dealt,
        );
        diff_f32(
            &mut mismatches,
            "telemetry.damage_taken",
            self.telemetry.damage_taken,
            other.telemetry.damage_taken,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.enemies_defeated",
            self.telemetry.enemies_defeated,
            other.telemetry.enemies_defeated,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.deaths",
            self.telemetry.deaths,
            other.telemetry.deaths,
        );
        diff_f32(
            &mut mismatches,
            "telemetry.total_time",
            self.telemetry.total_time,
            other.telemetry.total_time,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.combos_landed",
            self.telemetry.combos_landed,
            other.telemetry.combos_landed,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.secrets_found",
            self.telemetry.secrets_found,
            other.telemetry.secrets_found,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.anchors_repaired",
            self.telemetry.anchors_repaired,
            other.telemetry.anchors_repaired,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.echoes_collected",
            self.telemetry.echoes_collected,
            other.telemetry.echoes_collected,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.echoes_spent",
            self.telemetry.echoes_spent,
            other.telemetry.echoes_spent,
        );
        diff_u32(
            &mut mismatches,
            "telemetry.companion_support_actions",
            self.telemetry.companion_support_actions,
            other.telemetry.companion_support_actions,
        );
        if self.telemetry.final_affinity_rank != other.telemetry.final_affinity_rank {
            mismatches.push(format!(
                "telemetry.final_affinity_rank: {:?} vs {:?}",
                self.telemetry.final_affinity_rank, other.telemetry.final_affinity_rank
            ));
        }
        diff_u32(
            &mut mismatches,
            "telemetry.dialogues_completed",
            self.telemetry.dialogues_completed,
            other.telemetry.dialogues_completed,
        );
        if self.telemetry.boss_phases_seen != other.telemetry.boss_phases_seen {
            mismatches.push(format!(
                "telemetry.boss_phases_seen: {:?} vs {:?}",
                self.telemetry.boss_phases_seen, other.telemetry.boss_phases_seen
            ));
        }
        diff_u32(
            &mut mismatches,
            "telemetry.boss_telegraphs_dodged",
            self.telemetry.boss_telegraphs_dodged,
            other.telemetry.boss_telegraphs_dodged,
        );
        if self.telemetry.boss_defeated != other.telemetry.boss_defeated {
            mismatches.push(format!(
                "telemetry.boss_defeated: {} vs {}",
                self.telemetry.boss_defeated, other.telemetry.boss_defeated
            ));
        }
        diff_f32(
            &mut mismatches,
            "telemetry.boss_fight_time",
            self.telemetry.boss_fight_time,
            other.telemetry.boss_fight_time,
        );
        if self.telemetry.zones_visited != other.telemetry.zones_visited {
            mismatches.push(format!(
                "telemetry.zones_visited: {:?} vs {:?}",
                self.telemetry.zones_visited, other.telemetry.zones_visited
            ));
        }
        if self.telemetry.storm_choice != other.telemetry.storm_choice {
            mismatches.push(format!(
                "telemetry.storm_choice: {:?} vs {:?}",
                self.telemetry.storm_choice, other.telemetry.storm_choice
            ));
        }

        // VFX
        if self.vfx.current_zone != other.vfx.current_zone {
            mismatches.push(format!(
                "vfx.current_zone: {} vs {}",
                self.vfx.current_zone, other.vfx.current_zone
            ));
        }

        // Extended state (Phase 3+4 orchestrator fields)
        if self.walkthrough_beat != other.walkthrough_beat {
            mismatches.push(format!(
                "walkthrough_beat: {:?} vs {:?}",
                self.walkthrough_beat, other.walkthrough_beat
            ));
        }
        if self.player != other.player {
            mismatches.push("player: differs".to_string());
        }
        if self.combat != other.combat {
            mismatches.push("combat: differs".to_string());
        }
        if self.boss_hud != other.boss_hud {
            mismatches.push("boss_hud: differs".to_string());
        }
        if self.companion_hud != other.companion_hud {
            mismatches.push("companion_hud: differs".to_string());
        }
        if self.decision != other.decision {
            mismatches.push("decision: differs".to_string());
        }
        if self.recap != other.recap {
            mismatches.push("recap: differs".to_string());
        }

        CheckpointDiff {
            is_identical: mismatches.is_empty(),
            mismatch_count: mismatches.len(),
            mismatches,
        }
    }
}

// ── Diff result ────────────────────────────────────────────────────────

/// Result of comparing two [`SliceCheckpoint`]s.
#[derive(Debug, Clone, Default, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct CheckpointDiff {
    /// `true` if no differences were found.
    pub is_identical: bool,
    /// Number of differing fields.
    pub mismatch_count: usize,
    /// Human-readable descriptions of each mismatch.
    pub mismatches: Vec<String>,
}

impl std::fmt::Display for CheckpointDiff {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_identical {
            write!(f, "Checkpoints are identical")
        } else {
            writeln!(f, "{} mismatch(es):", self.mismatch_count)?;
            for m in &self.mismatches {
                writeln!(f, "  - {}", m)?;
            }
            Ok(())
        }
    }
}

// ── Checkpoint store ───────────────────────────────────────────────────

/// In-memory store that holds multiple checkpoints keyed by tick.
///
/// Useful for:
/// - Periodic autosave  (push every N ticks)
/// - Determinism replay (compare runs tick-by-tick)
/// - Debug inspection   (dump entire history)
#[derive(Debug, Clone, Default)]
pub struct CheckpointStore {
    checkpoints: BTreeMap<u64, SliceCheckpoint>,
    /// Maximum number of checkpoints to retain (0 = unlimited).
    max_entries: usize,
}

impl CheckpointStore {
    /// Create a store with no limit on retained checkpoints.
    #[must_use]
    pub fn new() -> Self {
        Self {
            checkpoints: BTreeMap::new(),
            max_entries: 0,
        }
    }

    /// Create a store that retains at most `max` checkpoints (FIFO eviction).
    pub fn with_capacity(max: usize) -> Self {
        Self {
            checkpoints: BTreeMap::new(),
            max_entries: max,
        }
    }

    /// Insert a checkpoint. If the store is at capacity, the oldest entry
    /// (lowest tick) is evicted.
    pub fn push(&mut self, cp: SliceCheckpoint) {
        let tick = cp.header.tick;
        self.checkpoints.insert(tick, cp);

        if self.max_entries > 0 && self.checkpoints.len() > self.max_entries {
            if let Some(&oldest) = self.checkpoints.keys().next() {
                self.checkpoints.remove(&oldest);
            }
        }
    }

    /// Retrieve a checkpoint by tick.
    pub fn get(&self, tick: u64) -> Option<&SliceCheckpoint> {
        self.checkpoints.get(&tick)
    }

    /// The most recent checkpoint.
    pub fn latest(&self) -> Option<&SliceCheckpoint> {
        self.checkpoints.values().next_back()
    }

    /// Number of stored checkpoints.
    pub fn len(&self) -> usize {
        self.checkpoints.len()
    }

    /// Whether the store is empty.
    pub fn is_empty(&self) -> bool {
        self.checkpoints.is_empty()
    }

    /// All stored ticks, ascending.
    pub fn ticks(&self) -> Vec<u64> {
        self.checkpoints.keys().copied().collect()
    }

    /// Serialise the entire store to JSON.
    ///
    /// # Errors
    ///
    /// Returns an error if serialisation fails.
    pub fn save_all_to_json(&self) -> Result<String> {
        let entries: Vec<&SliceCheckpoint> = self.checkpoints.values().collect();
        serde_json::to_string_pretty(&entries).context("Failed to serialise checkpoint store")
    }

    /// Deserialise an entire store from JSON (produced by [`save_all_to_json`]).
    ///
    /// # Errors
    ///
    /// Returns an error if the JSON is malformed or contains invalid entries.
    pub fn load_all_from_json(json: &str) -> Result<Self> {
        let entries: Vec<SliceCheckpoint> =
            serde_json::from_str(json).context("Failed to deserialise checkpoint store")?;
        let mut store = Self::new();
        for cp in entries {
            store.checkpoints.insert(cp.header.tick, cp);
        }
        Ok(store)
    }

    /// Remove all checkpoints.
    pub fn clear(&mut self) {
        self.checkpoints.clear();
    }
}

// ── Helpers ────────────────────────────────────────────────────────────

fn diff_f32(out: &mut Vec<String>, field: &str, a: f32, b: f32) {
    if (a - b).abs() > f32::EPSILON {
        out.push(format!("{}: {} vs {}", field, a, b));
    }
}

fn diff_u32(out: &mut Vec<String>, field: &str, a: u32, b: u32) {
    if a != b {
        out.push(format!("{}: {} vs {}", field, a, b));
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game_loop::GameLoop;
    use crate::hud_state::ThreadHud;
    use crate::telemetry::TelemetryCollector;
    use crate::vfx_dispatch::VfxAudioDispatch;

    fn make_checkpoint(tick: u64) -> SliceCheckpoint {
        let gl = GameLoop::new();
        let hud = ThreadHud::new(10);
        let tc = TelemetryCollector::new();
        let vfx = VfxAudioDispatch::new();
        SliceCheckpoint::capture(tick, "test", &gl, &hud, &tc, &vfx)
    }

    // ── Header ─────────────────────────────────────────────────────

    #[test]
    fn header_format_version() {
        let h = CheckpointHeader::new(42, "autosave");
        assert_eq!(h.format_version, "1.0.0");
        assert_eq!(h.tick, 42);
        assert_eq!(h.label, "autosave");
    }

    // ── Capture ────────────────────────────────────────────────────

    #[test]
    fn capture_default_game_loop() {
        let gl = GameLoop::new();
        let snap = GameLoopSnapshot::capture(&gl);
        assert!(snap.active_zone.is_none());
        assert_eq!(snap.storm_phase, "PreCrossroads");
        assert!(snap.storm_choice.is_none());
    }

    #[test]
    fn capture_hud_with_anchors() {
        let mut hud = ThreadHud::new(25);
        hud.add_anchor("Z0_core", 0.8);
        hud.add_anchor("Z1_left", 0.5);
        let snap = HudSnapshot::capture(&hud);
        assert_eq!(snap.echo_balance, 25);
        assert_eq!(snap.anchors.len(), 2);
    }

    #[test]
    fn capture_telemetry() {
        let mut tc = TelemetryCollector::new();
        tc.record_damage_dealt(500.0);
        tc.record_enemy_defeated();
        tc.record_enemy_defeated();
        tc.deaths = 1;
        let snap = TelemetrySnapshot::capture(&tc);
        assert_eq!(snap.damage_dealt, 500.0);
        assert_eq!(snap.enemies_defeated, 2);
        assert_eq!(snap.deaths, 1);
    }

    #[test]
    fn capture_vfx_dispatch() {
        let vfx = VfxAudioDispatch::new();
        let snap = VfxSnapshot::capture(&vfx);
        assert_eq!(snap.current_zone, 0);
        assert!(!snap.in_boss_encounter);
        assert!(snap.full_state.is_object());
    }

    // ── Serialisation round-trip ───────────────────────────────────

    #[test]
    fn checkpoint_json_round_trip() {
        let cp = make_checkpoint(100);
        let json = cp.save_to_json().unwrap();
        let loaded = SliceCheckpoint::load_from_json(&json).unwrap();
        assert_eq!(cp, loaded);
    }

    #[test]
    fn checkpoint_json_is_human_readable() {
        let cp = make_checkpoint(1);
        let json = cp.save_to_json().unwrap();
        assert!(json.contains("format_version"));
        assert!(json.contains("PreCrossroads"));
        assert!(json.contains("echo_balance"));
    }

    #[test]
    fn load_from_invalid_json_returns_error() {
        let result = SliceCheckpoint::load_from_json("{not valid}");
        assert!(result.is_err());
    }

    // ── Diff ───────────────────────────────────────────────────────

    #[test]
    fn identical_checkpoints_diff_clean() {
        let a = make_checkpoint(10);
        let b = make_checkpoint(10);
        let diff = a.diff(&b);
        assert!(diff.is_identical);
        assert_eq!(diff.mismatch_count, 0);
    }

    #[test]
    fn different_checkpoints_produce_mismatches() {
        let a = make_checkpoint(10);
        let mut b = make_checkpoint(10);
        b.telemetry.damage_dealt = 999.0;
        b.hud.echo_balance = 42;
        let diff = a.diff(&b);
        assert!(!diff.is_identical);
        assert!(diff.mismatch_count >= 2);
    }

    #[test]
    fn diff_display_format() {
        let a = make_checkpoint(1);
        let mut b = make_checkpoint(2);
        b.telemetry.deaths = 5;
        let diff = a.diff(&b);
        let text = format!("{}", diff);
        assert!(text.contains("mismatch"));
    }

    #[test]
    fn diff_catches_new_telemetry_fields() {
        let a = make_checkpoint(1);
        let mut b = make_checkpoint(1);

        // Diverge every telemetry field added in the recent completeness audit.
        b.telemetry.combos_landed = 3;
        b.telemetry.secrets_found = 1;
        b.telemetry.anchors_repaired = 2;
        b.telemetry.echoes_collected = 50;
        b.telemetry.echoes_spent = 10;
        b.telemetry.companion_support_actions = 4;
        b.telemetry.dialogues_completed = 2;
        b.telemetry.boss_phases_seen = vec![
            "Assessment".into(),
            "FulcrumShift".into(),
            "DirectiveOverride".into(),
        ];
        b.telemetry.boss_telegraphs_dodged = 7;
        b.telemetry.boss_fight_time = 42.0;
        b.telemetry.zones_visited = vec![
            "zone_0".into(),
            "zone_1".into(),
            "zone_2".into(),
            "zone_3".into(),
        ];

        let diff = a.diff(&b);
        assert!(!diff.is_identical, "diverged telemetry must be detected");
        // Each diverged field should contribute at least one mismatch.
        assert!(
            diff.mismatch_count >= 11,
            "expected >= 11 mismatches from diverged telemetry fields, got {}",
            diff.mismatch_count,
        );
    }

    #[test]
    fn diff_catches_storm_choice_divergence() {
        let a = make_checkpoint(1);
        let mut b = make_checkpoint(1);
        b.telemetry.storm_choice = Some(crate::storm_choice::StormChoice::Redirect);
        let diff = a.diff(&b);
        assert!(!diff.is_identical);
        assert!(
            diff.mismatches.iter().any(|m| m.contains("storm_choice")),
            "storm_choice mismatch should be reported"
        );
    }

    #[test]
    fn diff_catches_affinity_rank_divergence() {
        let a = make_checkpoint(1);
        let mut b = make_checkpoint(1);
        b.telemetry.final_affinity_rank = Some("Bonded".to_string());
        let diff = a.diff(&b);
        assert!(!diff.is_identical);
        assert!(
            diff.mismatches
                .iter()
                .any(|m| m.contains("final_affinity_rank")),
            "final_affinity_rank mismatch should be reported"
        );
    }

    // ── CheckpointStore ────────────────────────────────────────────

    #[test]
    fn store_push_and_get() {
        let mut store = CheckpointStore::new();
        store.push(make_checkpoint(1));
        store.push(make_checkpoint(2));
        store.push(make_checkpoint(3));
        assert_eq!(store.len(), 3);
        assert!(store.get(2).is_some());
        assert!(store.get(99).is_none());
    }

    #[test]
    fn store_latest() {
        let mut store = CheckpointStore::new();
        store.push(make_checkpoint(5));
        store.push(make_checkpoint(10));
        assert_eq!(store.latest().unwrap().header.tick, 10);
    }

    #[test]
    fn store_capacity_eviction() {
        let mut store = CheckpointStore::with_capacity(3);
        store.push(make_checkpoint(1));
        store.push(make_checkpoint(2));
        store.push(make_checkpoint(3));
        store.push(make_checkpoint(4)); // evicts tick=1
        assert_eq!(store.len(), 3);
        assert!(store.get(1).is_none());
        assert!(store.get(4).is_some());
    }

    #[test]
    fn store_ticks_ascending() {
        let mut store = CheckpointStore::new();
        store.push(make_checkpoint(30));
        store.push(make_checkpoint(10));
        store.push(make_checkpoint(20));
        assert_eq!(store.ticks(), vec![10, 20, 30]);
    }

    #[test]
    fn store_json_round_trip() {
        let mut store = CheckpointStore::new();
        store.push(make_checkpoint(1));
        store.push(make_checkpoint(2));
        let json = store.save_all_to_json().unwrap();
        let loaded = CheckpointStore::load_all_from_json(&json).unwrap();
        assert_eq!(loaded.len(), 2);
        assert_eq!(loaded.get(1).unwrap().header.tick, 1);
    }

    #[test]
    fn store_clear() {
        let mut store = CheckpointStore::new();
        store.push(make_checkpoint(1));
        store.clear();
        assert!(store.is_empty());
    }

    #[test]
    fn store_empty_by_default() {
        let store = CheckpointStore::new();
        assert!(store.is_empty());
        assert!(store.latest().is_none());
        assert_eq!(store.ticks(), Vec::<u64>::new());
    }

    // ── Edge cases ─────────────────────────────────────────────────

    #[test]
    fn checkpoint_with_storm_choice() {
        let mut gl = GameLoop::new();
        gl.storm_state.enter_crossroads();
        gl.storm_state.make_choice(StormChoice::Redirect);
        let hud = ThreadHud::new(0);
        let tc = TelemetryCollector::new();
        let vfx = VfxAudioDispatch::new();
        let cp = SliceCheckpoint::capture(50, "storm", &gl, &hud, &tc, &vfx);
        assert_eq!(cp.game_loop.storm_phase, "ChoiceMade");
        assert_eq!(cp.game_loop.storm_choice, Some(StormChoice::Redirect));

        // Round-trip preserves choice
        let json = cp.save_to_json().unwrap();
        let loaded = SliceCheckpoint::load_from_json(&json).unwrap();
        assert_eq!(loaded.game_loop.storm_choice, Some(StormChoice::Redirect));
    }

    #[test]
    fn checkpoint_hud_repaired_anchors() {
        let mut hud = ThreadHud::new(100);
        hud.add_anchor("A1", 0.3);
        hud.set_anchor_repairing("A1", true);
        hud.set_anchor_repairing("A1", false); // marks was_repaired
        let snap = HudSnapshot::capture(&hud);
        assert!(snap.anchors[0].was_repaired);
    }

    #[test]
    fn telemetry_snapshot_affinity_rank_string() {
        let mut tc = TelemetryCollector::new();
        tc.set_final_affinity_rank(crate::companion_hud::AffinityRank::Bonded);
        let snap = TelemetrySnapshot::capture(&tc);
        assert_eq!(snap.final_affinity_rank.as_deref(), Some("Bonded"));
    }

    #[test]
    fn telemetry_snapshot_boss_phases_as_strings() {
        let mut tc = TelemetryCollector::new();
        tc.record_boss_phase(crate::boss_hud::BossPhase::Assessment);
        tc.record_boss_phase(crate::boss_hud::BossPhase::FulcrumShift);
        let snap = TelemetrySnapshot::capture(&tc);
        assert_eq!(snap.boss_phases_seen, vec!["Assessment", "Fulcrum Shift"]);
    }
}
