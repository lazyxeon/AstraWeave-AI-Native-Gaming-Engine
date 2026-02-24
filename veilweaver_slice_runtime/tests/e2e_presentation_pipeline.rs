//! Phase 3+4 Integration Test — full presentation pipeline end-to-end.
//!
//! Validates that the SliceOrchestrator correctly wires:
//! - ThreadHud (echo counter + anchor stability)
//! - BossHealthBar (boss HUD)
//! - CompanionAffinityMeter (companion bond)
//! - DecisionRadial (storm choice UI)
//! - VfxAudioDispatch (VFX/audio events)
//! - RecapPanel (post-run debrief)
//!
//! All through a single orchestrated Z0→Z4 walkthrough.

use veilweaver_slice_runtime::boss_hud::BossPhase;
use veilweaver_slice_runtime::companion_hud::AffinityRank;
use veilweaver_slice_runtime::decision_ui::DecisionPhase;
use veilweaver_slice_runtime::storm_choice::StormChoice;
use veilweaver_slice_runtime::walkthrough::{SliceOrchestrator, WalkthroughBeat, WalkthroughEvent};

const DT: f32 = 1.0 / 60.0;

// ── Helper ─────────────────────────────────────────────────────────────

/// Sets up a standard orchestrator with all zone transition triggers.
fn setup_orchestrator() -> SliceOrchestrator {
    let mut orch = SliceOrchestrator::new();
    orch.game_loop
        .register_trigger_action("z1", "zone.transition:Z1_echo_grove");
    orch.game_loop
        .register_trigger_action("z2", "zone.transition:Z2_fractured_cliffs");
    orch.game_loop
        .register_trigger_action("z3", "zone.transition:Z3_loom_crossroads");
    orch.game_loop
        .register_trigger_action("storm", "decision.open:storm_routing");
    orch.game_loop
        .register_trigger_action("z4", "zone.transition:Z4_boss_courtyard");
    orch
}

// ── Tests ──────────────────────────────────────────────────────────────

/// Full Z0→Z4 walkthrough verifying every HUD + VFX subsystem is wired.
#[test]
fn full_presentation_pipeline_z0_to_z4() {
    let mut orch = setup_orchestrator();

    // ── Z0: Tutorial ───────────────────────────────────────────────
    assert_eq!(orch.beat(), WalkthroughBeat::TutorialBegin);
    assert_eq!(orch.thread_hud().echoes.balance, 0);
    assert!(orch.thread_hud().anchors.is_empty());
    assert!(!orch.vfx_audio().in_boss_encounter());

    orch.complete_tutorial();
    orch.collect_echoes(20);
    assert_eq!(orch.thread_hud().echoes.balance, 20);

    // Repair an anchor.
    orch.repair_anchor("sanctum_core");
    assert!(orch.thread_hud().anchors.contains_key("sanctum_core"));
    assert!(orch.thread_hud().anchors["sanctum_core"].was_repaired);

    // ── Z0 → Z1: Echo Grove ───────────────────────────────────────
    orch.game_loop.notify_trigger_enter(vec!["z1".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::EchoGroveEntry);
    assert_eq!(orch.vfx_audio().current_zone(), 1);
    assert!(orch.vfx_audio().audio_scene().ambience.is_some());

    // More echoes.
    orch.collect_echoes(15);
    assert_eq!(orch.thread_hud().echoes.balance, 35);

    // Companion interactions.
    orch.apply_companion_affinity("Protected during ambush", 0.15, 5.0);
    orch.apply_companion_affinity("Followed tactical advice", 0.10, 8.0);
    assert!((orch.companion_hud().affinity() - 0.25).abs() < 0.01);
    assert_eq!(orch.companion_hud().rank(), AffinityRank::Cautious);
    assert_eq!(orch.telemetry().companion_support_actions, 2);

    // ── Z1 → Z2: Fractured Cliffs ─────────────────────────────────
    orch.game_loop.notify_trigger_enter(vec!["z2".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::FracturedCliffEntry);
    assert_eq!(orch.vfx_audio().current_zone(), 2);

    // Start combat.
    orch.start_combat();
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::SkirmishActive);

    // Clear combat.
    orch.clear_combat_wave();
    orch.clear_combat_wave();
    let result = orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::SkirmishCleared);
    assert!(result.combat_events.iter().any(|e| matches!(
        e,
        veilweaver_slice_runtime::combat::CombatEvent::EncounterCleared { .. }
    )));

    // ── Z2 → Z3: Loom Crossroads (Storm Choice) ───────────────────
    orch.game_loop.notify_trigger_enter(vec!["z3".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::LoomCrossroadsEntry);
    assert_eq!(orch.vfx_audio().current_zone(), 3);

    // Open storm decision.
    orch.game_loop
        .notify_trigger_enter(vec!["storm".to_string()]);
    let result = orch.tick(DT);
    assert!(result.game_events.iter().any(|e| matches!(
        e,
        veilweaver_slice_runtime::game_loop::GameLoopEvent::StormDecisionPrompt
    )));

    // Decision radial should be visible (Opening phase).
    assert!(orch.decision_radial().is_visible());

    // Animate to Active.
    for _ in 0..30 {
        orch.tick(DT);
    }
    assert_eq!(orch.decision_radial().phase(), DecisionPhase::Active);

    // Make the choice.
    orch.make_storm_choice(StormChoice::Redirect);
    let result = orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::StormDecisionMade);
    assert_eq!(orch.decision_radial().selected_key(), Some("redirect"));
    assert!(result.game_events.iter().any(|e| matches!(
        e,
        veilweaver_slice_runtime::game_loop::GameLoopEvent::StormDecisionMade { choice }
            if *choice == StormChoice::Redirect
    )));

    // VFX dispatch should have storm VFX after storm resolves.
    // (StormResolved may need another tick depending on game_loop internals.)

    // ── Z3 → Z4: Boss Courtyard ───────────────────────────────────
    orch.game_loop.notify_trigger_enter(vec!["z4".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::BossArenaEntry);

    // VFX dispatch should now be in boss encounter mode.
    assert!(orch.vfx_audio().in_boss_encounter());

    // ── Boss Fight ─────────────────────────────────────────────────
    // Boss HUD starts at full HP.
    assert_eq!(orch.boss_hud().current_hp, 10_000.0);
    assert_eq!(orch.boss_hud().phase, BossPhase::Assessment);

    // Phase 1: Assessment (100% → 60%).
    orch.damage_boss(2000.0);
    assert_eq!(orch.boss_hud().current_hp, 8_000.0);
    assert_eq!(orch.boss_hud().phase, BossPhase::Assessment);

    // Add a telegraph.
    orch.add_boss_telegraph("Oathbound Cleave", "Dodge left!", 2.0);
    assert_eq!(orch.boss_hud().telegraphs.len(), 1);
    assert_eq!(orch.vfx_audio().vfx_scene().telegraphs.len(), 1);

    // Phase 2: Fulcrum Shift (60% → 30%).
    orch.damage_boss(3000.0); // 8000 → 5000 (50% = FulcrumShift zone)
    assert_eq!(orch.boss_hud().phase, BossPhase::FulcrumShift);

    // Companion rank-up during boss fight.
    orch.apply_companion_affinity("Boss combo assist", 0.25, 30.0);
    assert_eq!(orch.companion_hud().rank(), AffinityRank::Cooperative);

    // Phase 3: Directive Override (<30%).
    orch.damage_boss(2500.0); // 5000 → 2500 (25%)
    assert_eq!(orch.boss_hud().phase, BossPhase::DirectiveOverride);

    // Finish boss.
    orch.damage_boss(2500.0);
    assert!(orch.boss_hud().is_defeated());

    // ── Debrief ────────────────────────────────────────────────────
    orch.force_beat(WalkthroughBeat::BossDefeated);
    orch.tick(DT); // BossDefeated → Debrief

    assert_eq!(orch.beat(), WalkthroughBeat::Debrief);

    // Recap should be finalized.
    assert!(orch.recap().is_finalized());
    assert!(orch.recap().metric_count() > 0);
    assert!(orch.recap().overall_rating().is_some());

    // Verify specific recap metrics.
    let damage_dealt = orch.recap().metric("damage_dealt");
    assert!(damage_dealt.is_some());
    assert!(damage_dealt.unwrap().value.as_f64() > 0.0);

    let echoes = orch.recap().metric("echoes_collected");
    assert!(echoes.is_some());
    assert!(echoes.unwrap().value.as_f64() >= 35.0);

    // Complete the run.
    orch.advance_to_debrief();
    assert!(orch.is_complete());
}

/// Verify echo and anchor HUD syncs happen immediately (not deferred to tick).
#[test]
fn hud_syncs_immediately_on_external_commands() {
    let mut orch = SliceOrchestrator::new();

    // Echo collection: HUD updates immediately.
    orch.collect_echoes(42);
    assert_eq!(orch.thread_hud().echoes.balance, 42);

    // Anchor repair: HUD updates immediately.
    orch.repair_anchor("alpha");
    assert!(orch.thread_hud().anchors.contains_key("alpha"));
    assert!(orch.thread_hud().anchors["alpha"].was_repaired);
    assert!((orch.thread_hud().anchors["alpha"].stability - 1.0).abs() < 0.01);

    // After tick, HUD state remains consistent.
    orch.tick(DT);
    assert_eq!(orch.thread_hud().echoes.balance, 42);
    assert!(orch.thread_hud().anchors.contains_key("alpha"));
}

/// Decision radial opens on storm prompt and closes on decision.
#[test]
fn decision_radial_lifecycle_through_orchestrator() {
    let mut orch = setup_orchestrator();
    orch.force_beat(WalkthroughBeat::LoomCrossroadsEntry);

    // Initially hidden.
    assert_eq!(orch.decision_radial().phase(), DecisionPhase::Hidden);

    // Trigger storm prompt.
    orch.game_loop
        .notify_trigger_enter(vec!["storm".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.decision_radial().phase(), DecisionPhase::Opening);

    // Animate through opening.
    for _ in 0..30 {
        orch.tick(DT);
    }
    assert_eq!(orch.decision_radial().phase(), DecisionPhase::Active);

    // Make choice.
    orch.make_storm_choice(StormChoice::Stabilize);
    orch.tick(DT);
    assert_eq!(orch.decision_radial().selected_key(), Some("stabilize"));

    // Closing → Done.
    for _ in 0..30 {
        orch.tick(DT);
    }
    assert!(orch.decision_radial().is_done());
}

/// Companion affinity progression triggers VFX stingers on rank-up.
#[test]
fn companion_affinity_rank_ups_trigger_stingers() {
    let mut orch = SliceOrchestrator::new();

    // Start at Wary (0.0).
    assert_eq!(orch.companion_hud().rank(), AffinityRank::Wary);

    // Big boost → Cautious.
    orch.apply_companion_affinity("Saved Aria", 0.25, 1.0);
    assert_eq!(orch.companion_hud().rank(), AffinityRank::Cautious);

    // Check that a rank-up stinger was queued.
    let stingers = orch.vfx_audio.drain_stingers();
    assert!(
        stingers
            .iter()
            .any(|s| s.kind == veilweaver_slice_runtime::audio_specs::StingerKind::AffinityRankUp),
        "Expected AffinityRankUp stinger"
    );
}

/// Boss HUD tracks phase transitions accurately.
#[test]
fn boss_hud_phase_progression() {
    let mut orch = SliceOrchestrator::new();

    // Full HP → Assessment.
    assert_eq!(orch.boss_hud().phase, BossPhase::Assessment);
    assert_eq!(orch.boss_hud().current_hp, 10_000.0);

    // Drain to 50% → FulcrumShift.
    orch.damage_boss(5000.0);
    assert_eq!(orch.boss_hud().phase, BossPhase::FulcrumShift);

    // Drain to 20% → DirectiveOverride.
    orch.damage_boss(3000.0);
    assert_eq!(orch.boss_hud().phase, BossPhase::DirectiveOverride);

    // Defeat.
    orch.damage_boss(2000.0);
    assert!(orch.boss_hud().is_defeated());

    // Telemetry should track all damage.
    assert_eq!(orch.telemetry().damage_dealt, 10_000.0);
}

/// VFX dispatch enters boss mode on BossArenaEntry beat.
#[test]
fn vfx_enters_boss_mode_on_arena_entry() {
    let mut orch = setup_orchestrator();

    // Advance to boss arena.
    orch.complete_tutorial();
    orch.game_loop.notify_trigger_enter(vec!["z1".to_string()]);
    orch.tick(DT);

    orch.game_loop.notify_trigger_enter(vec!["z2".to_string()]);
    orch.tick(DT);
    orch.start_combat();
    orch.tick(DT);
    orch.clear_combat_wave();
    orch.clear_combat_wave();
    orch.tick(DT);

    orch.game_loop.notify_trigger_enter(vec!["z3".to_string()]);
    orch.tick(DT);
    orch.game_loop
        .notify_trigger_enter(vec!["storm".to_string()]);
    orch.tick(DT);
    orch.make_storm_choice(StormChoice::Stabilize);
    orch.tick(DT);

    orch.game_loop.notify_trigger_enter(vec!["z4".to_string()]);
    orch.tick(DT);

    assert_eq!(orch.beat(), WalkthroughBeat::BossArenaEntry);
    assert!(orch.vfx_audio().in_boss_encounter());
}

/// Recap finalization happens exactly once on debrief.
#[test]
fn recap_finalized_once_at_debrief() {
    let mut orch = SliceOrchestrator::new();
    orch.collect_echoes(10);
    orch.repair_anchor("a1");

    // Not yet finalized.
    assert!(!orch.recap().is_finalized());

    // Force to debrief.
    orch.force_beat(WalkthroughBeat::BossDefeated);
    orch.tick(DT);
    assert!(orch.recap().is_finalized());

    let count = orch.recap().metric_count();
    assert!(count > 0);

    // Forcing again should not double-finalize.
    orch.force_beat(WalkthroughBeat::Complete);
    assert_eq!(orch.recap().metric_count(), count);
}

/// Thread HUD animations converge after multiple ticks.
#[test]
fn thread_hud_animations_converge() {
    let mut orch = SliceOrchestrator::new();

    // Add anchor with low stability, then raise it.
    orch.thread_hud.add_anchor("test", 0.0);
    orch.thread_hud.update_anchor_stability("test", 1.0);

    // Run 120 frames (~2 seconds).
    for _ in 0..120 {
        orch.tick(DT);
    }

    let entry = &orch.thread_hud().anchors["test"];
    assert!(
        (entry.display_value - 1.0).abs() < 0.05,
        "Display value should converge to 1.0, got {}",
        entry.display_value
    );
}
