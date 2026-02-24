//! End-to-end integration test — HUD pipeline.
//!
//! Simulates a full game run flowing through:
//!   GameLoopEvents → TelemetryCollector → HUD state modules → RecapPanel
//!
//! Validates that all Phase 3 data models work in concert.

use astraweave_scene::world_partition::GridCoord;
use veilweaver_slice_runtime::boss_hud::{BossHealthBar, BossPhase};
use veilweaver_slice_runtime::companion_hud::{AffinityRank, CompanionAffinityMeter};
use veilweaver_slice_runtime::decision_ui::{storm_choice_radial, DecisionPhase};
use veilweaver_slice_runtime::game_loop::GameLoopEvent;
use veilweaver_slice_runtime::hud_state::ThreadHud;
use veilweaver_slice_runtime::recap_panel::{MetricCategory, Rating, RecapPanel};
use veilweaver_slice_runtime::storm_choice::StormChoice;
use veilweaver_slice_runtime::telemetry::TelemetryCollector;

/// Helper: produce one frame's worth of dt (60 Hz).
const DT: f32 = 1.0 / 60.0;

// ── Full-run simulation ────────────────────────────────────────────────

/// Simulates an entire Veilweaver slice run, exercising every HUD module
/// and verifying the recap panel produces coherent ratings at the end.
#[test]
fn full_run_hud_pipeline() {
    // ── Instantiate all HUD systems ────────────────────────────────
    let mut thread_hud = ThreadHud::new(0);
    let mut companion = CompanionAffinityMeter::new("Aria", 0.0);
    let mut boss_bar = BossHealthBar::new("Oathbound Warden", 10_000.0);
    let mut telemetry = TelemetryCollector::new();

    // ── Phase: Zone exploration (Z0 → Z2) ──────────────────────────
    let zone_events = vec![
        GameLoopEvent::ZoneLoading {
            zone_name: "Z0_loomspire_sanctum".into(),
            coord: GridCoord::new(100, 0, 0),
        },
        GameLoopEvent::DialogueDisplay {
            node_id: "intro_1".into(),
            text: "Welcome, Weaver.".into(),
            choices: vec![],
        },
        GameLoopEvent::DialogueEnded {
            dialogue_id: "intro".into(),
        },
    ];
    telemetry.process_events(&zone_events, DT);

    // Thread HUD: set up initial anchors
    thread_hud.add_anchor("sanctum_core", 0.8);
    thread_hud.add_anchor("echo_grove_alpha", 0.5);
    thread_hud.add_anchor("crossroads_nexus", 0.3);
    thread_hud.echo_transact(100); // Starting echoes

    // Zone 1
    telemetry.process_events(
        &[GameLoopEvent::ZoneLoading {
            zone_name: "Z1_echo_grove".into(),
            coord: GridCoord::new(101, 0, 0),
        }],
        DT,
    );

    // Repair an anchor
    thread_hud.update_anchor_stability("echo_grove_alpha", 1.0);
    thread_hud.set_anchor_repairing("echo_grove_alpha", true);
    thread_hud.set_anchor_repairing("echo_grove_alpha", false); // Complete repair
    telemetry.record_anchor_repaired();
    thread_hud.echo_transact(-20);
    telemetry.record_echoes_spent(20);

    // Companion interactions
    companion.apply_event("Protected Aria during ambush", 0.15, 10.0);
    companion.apply_event("Followed tactical advice", 0.1, 15.0);
    telemetry.record_companion_action();
    telemetry.record_companion_action();

    // Combat in Z1
    telemetry.record_damage_dealt(2500.0);
    telemetry.record_damage_taken(300.0);
    telemetry.record_enemy_defeated();
    telemetry.record_enemy_defeated();
    telemetry.record_combo();

    // Zone 2
    telemetry.process_events(
        &[
            GameLoopEvent::ZoneLoading {
                zone_name: "Z2_shattered_gallery".into(),
                coord: GridCoord::new(102, 0, 0),
            },
            GameLoopEvent::CinematicStarted {
                name: "gallery_reveal".into(),
            },
            GameLoopEvent::CinematicFinished {
                name: "gallery_reveal".into(),
            },
            GameLoopEvent::DialogueEnded {
                dialogue_id: "gallery_lore".into(),
            },
        ],
        DT,
    );

    telemetry.record_anchor_repaired();
    telemetry.record_secret();

    // ── Phase: Storm decision (Z3) ─────────────────────────────────
    telemetry.process_events(
        &[
            GameLoopEvent::ZoneLoading {
                zone_name: "Z3_loom_crossroads".into(),
                coord: GridCoord::new(103, 0, 0),
            },
            GameLoopEvent::StormDecisionPrompt,
        ],
        DT,
    );

    // Open the decision radial
    let mut radial = storm_choice_radial();
    radial.open();
    for _ in 0..30 {
        radial.tick(DT);
    }
    assert_eq!(radial.phase(), DecisionPhase::Active);

    // Focus on "Stabilize" (index 0), then select
    radial.focus(0);
    radial.select(0);
    assert!(radial.selected_key().is_some());

    // Process the storm decision
    telemetry.process_events(
        &[
            GameLoopEvent::StormDecisionMade {
                choice: StormChoice::Stabilize,
            },
            GameLoopEvent::StormResolved {
                choice: StormChoice::Stabilize,
            },
        ],
        DT,
    );

    // More companion interactions
    companion.apply_event("Joint storm stabilization", 0.2, 30.0);
    telemetry.record_companion_action();

    // ── Phase: Boss encounter (Z4) ─────────────────────────────────
    telemetry.process_events(
        &[GameLoopEvent::ZoneLoading {
            zone_name: "Z4_boss_courtyard".into(),
            coord: GridCoord::new(104, 0, 0),
        }],
        DT,
    );

    telemetry.start_boss_fight();

    // Boss Phase 1: Assessment
    telemetry.record_boss_phase(BossPhase::Assessment);
    boss_bar.set_hp(10_000.0);

    // Deal damage, take some hits
    boss_bar.apply_damage(2500.0);
    telemetry.record_damage_dealt(2500.0);
    telemetry.record_damage_taken(500.0);
    telemetry.record_telegraph_dodged();

    // Tick the boss bar forward to drain animation
    for _ in 0..120 {
        boss_bar.tick(DT);
    }
    assert!(boss_bar.hp_fraction() < 0.8);

    // Phase 2: Fulcrum Shift
    boss_bar.apply_damage(2000.0);
    telemetry.record_boss_phase(BossPhase::FulcrumShift);
    telemetry.record_damage_dealt(2000.0);
    telemetry.record_enemy_defeated(); // Add killed
    telemetry.record_telegraph_dodged();

    // Phase 3: Directive Override
    boss_bar.apply_damage(3500.0);
    telemetry.record_boss_phase(BossPhase::DirectiveOverride);
    telemetry.record_damage_dealt(3500.0);
    telemetry.record_telegraph_dodged();

    // Defeat the boss
    boss_bar.apply_damage(2000.0);
    telemetry.record_damage_dealt(2000.0);
    telemetry.finish_boss_fight();

    // Boss fight lasted ~5s of game time
    telemetry.process_events(&[], 5.0);

    // Final companion rank
    companion.apply_event("Defeated boss together", 0.15, 60.0);
    let final_rank = companion.rank();
    telemetry.set_final_affinity_rank(final_rank);

    // ── Thread HUD final state ─────────────────────────────────────
    thread_hud.update_anchor_stability("sanctum_core", 1.0);
    thread_hud.update_anchor_stability("crossroads_nexus", 0.7);
    for _ in 0..300 {
        thread_hud.tick(DT);
    }

    // Verify thread HUD consistency
    assert!(thread_hud.average_stability() > 0.5);
    assert!(thread_hud.repaired_count() >= 1);

    // ── Add dialogue completions ───────────────────────────────────
    telemetry.process_events(
        &[GameLoopEvent::DialogueEnded {
            dialogue_id: "boss_victory".into(),
        }],
        DT,
    );

    // ── Generate recap ─────────────────────────────────────────────
    let mut recap = RecapPanel::new();
    telemetry.finalize(&mut recap);

    assert!(recap.is_finalized());
    assert!(recap.overall_rating().is_some());

    // Verify all categories have ratings
    let ratings = recap.category_ratings();
    assert!(ratings.contains_key(&MetricCategory::Combat));
    assert!(ratings.contains_key(&MetricCategory::Exploration));
    assert!(ratings.contains_key(&MetricCategory::Weaving));
    assert!(ratings.contains_key(&MetricCategory::Companion));
    assert!(ratings.contains_key(&MetricCategory::Narrative));
    assert!(ratings.contains_key(&MetricCategory::Timing));

    // We had 3 enemies killed + 0 deaths → S combat rating
    assert_eq!(ratings[&MetricCategory::Combat], Rating::S);

    // 5 zones visited → A or S
    let exploration = ratings[&MetricCategory::Exploration];
    assert!(exploration == Rating::A || exploration == Rating::S);

    // 2 anchors repaired → B weaving
    let weaving = ratings[&MetricCategory::Weaving];
    assert!(matches!(weaving, Rating::S | Rating::A | Rating::B));

    // Storm choice + 3 dialogues → S narrative
    let narrative = ratings[&MetricCategory::Narrative];
    assert!(matches!(narrative, Rating::S | Rating::A));

    // Verify specific metrics recorded
    assert!(recap.metric("damage_dealt").is_some());
    assert!(recap.metric("zones_visited").is_some());
    assert!(recap.metric("storm_choice").is_some());
    assert!(recap.metric("boss_fight_time").is_some());

    // ── Reveal animation ───────────────────────────────────────────
    for _ in 0..600 {
        recap.tick(DT);
    }
    assert!(recap.is_fully_revealed());
}

// ── Companion rank progression isolated ────────────────────────────────

#[test]
fn companion_rank_progression_through_run() {
    let mut meter = CompanionAffinityMeter::new("Aria", 0.0);
    assert_eq!(meter.rank(), AffinityRank::Wary);

    // Accumulate affinity through gameplay
    let events = [
        (0.1, "Followed advice"),
        (0.1, "Protected companion"),
        (0.1, "Strategic play"),
        (0.1, "Good combat"),
        (0.15, "Storm cooperation"),
        (0.15, "Boss fight synergy"),
        (0.1, "Victory celebration"),
    ];

    for (i, (delta, reason)) in events.iter().enumerate() {
        let rank_change = meter.apply_event(*reason, *delta, i as f32 * 10.0);
        meter.tick(DT);
        if rank_change.is_some() {
            // A rank up happened — verify it's higher
            assert!(meter.rank() > AffinityRank::Wary || i == 0);
        }
    }

    // Should reach at least Bonded (0.8 total)
    assert!(meter.rank() >= AffinityRank::Bonded);
}

// ── Boss health bar phase tracking ─────────────────────────────────────

#[test]
fn boss_phases_track_correctly() {
    let mut bar = BossHealthBar::new("Oathbound Warden", 10_000.0);

    // Full health → Assessment
    assert_eq!(bar.phase, BossPhase::Assessment);

    // Damage to 55% → FulcrumShift
    bar.apply_damage(4500.0);
    for _ in 0..120 {
        bar.tick(DT);
    }
    assert_eq!(bar.phase, BossPhase::FulcrumShift);

    // Damage to 20% → DirectiveOverride
    bar.apply_damage(3500.0);
    for _ in 0..120 {
        bar.tick(DT);
    }
    assert_eq!(bar.phase, BossPhase::DirectiveOverride);
}

// ── Thread HUD echo economy ────────────────────────────────────────────

#[test]
fn echo_economy_through_gameplay() {
    let mut hud = ThreadHud::new(0);

    // Gain echoes from exploration
    hud.echo_transact(50);
    hud.echo_transact(30);
    assert_eq!(hud.echoes.balance, 80);

    // Spend on repairs
    hud.echo_transact(-20);
    assert_eq!(hud.echoes.balance, 60);

    // Tick to process feedback floats
    for _ in 0..200 {
        hud.tick(DT);
    }

    // Balance should remain stable
    assert_eq!(hud.echoes.balance, 60);
}

// ── Decision radial lifecycle ──────────────────────────────────────────

#[test]
fn decision_radial_full_lifecycle() {
    let mut radial = storm_choice_radial();
    assert_eq!(radial.phase(), DecisionPhase::Hidden);

    // Open
    radial.open();
    // Animate through opening
    for _ in 0..30 {
        radial.tick(DT);
    }
    assert_eq!(radial.phase(), DecisionPhase::Active);

    // Countdown ticks
    for _ in 0..60 {
        radial.tick(DT);
    }
    assert!(radial.countdown_remaining().unwrap() < 30.0);

    // Focus and select
    radial.focus(1);
    assert_eq!(radial.focused_index, Some(1));

    radial.select(1);
    // Should start closing
    assert!(radial.phase() == DecisionPhase::Closing || radial.phase() == DecisionPhase::Done);

    // Animate through closing
    for _ in 0..60 {
        radial.tick(DT);
    }
    assert_eq!(radial.phase(), DecisionPhase::Done);
    assert_eq!(radial.selected_key().unwrap(), "redirect");
}

// ── Telemetry → Recap round-trip ───────────────────────────────────────

#[test]
fn telemetry_empty_run_produces_valid_recap() {
    let tc = TelemetryCollector::new();
    let mut panel = RecapPanel::new();
    tc.finalize(&mut panel);

    assert!(panel.is_finalized());
    // Timing category always present (total_time + boss_fight_time)
    assert!(panel.metric("total_time").is_some());
}
