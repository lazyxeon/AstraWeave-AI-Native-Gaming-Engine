//! End-to-end walkthrough integration test — exercises the full Z0→Z4 journey
//! through the [`SliceOrchestrator`].
//!
//! This test validates that all Phase 2 subsystems cooperate correctly:
//! - PlayerState tracks zone, HP, echoes, anchors, tutorial
//! - CombatEncounter runs a multi-wave skirmish
//! - GameLoop handles triggers, dialogues, cinematics, storm choice
//! - TelemetryCollector accumulates all metrics
//! - WalkthroughBeat progression advances through the intended path
//!
//! No feature-gated subsystems (CompanionAI, BossEncounter) are tested here;
//! they require the `ai-companion` / `boss-director` features.

use astraweave_cinematics::{CameraKey, Time, Timeline};
use astraweave_dialogue::toml_loader::load_dialogue_from_toml;
use veilweaver_slice_runtime::combat::{CombatEvent, EncounterPhase};
use veilweaver_slice_runtime::game_loop::GameLoopEvent;
use veilweaver_slice_runtime::storm_choice::StormChoice;
use veilweaver_slice_runtime::walkthrough::{SliceOrchestrator, WalkthroughBeat, WalkthroughEvent};

const DT: f32 = 1.0 / 60.0;

// ── Helpers ────────────────────────────────────────────────────────────────

fn setup_orchestrator() -> SliceOrchestrator {
    let mut orch = SliceOrchestrator::new();

    // Register zone transition triggers.
    orch.game_loop
        .register_trigger_action("exit_z0", "zone.transition:Z1_echo_grove");
    orch.game_loop
        .register_trigger_action("exit_z1", "zone.transition:Z2_fractured_cliffs");
    orch.game_loop
        .register_trigger_action("exit_z2", "zone.transition:Z3_loom_crossroads");
    orch.game_loop
        .register_trigger_action("exit_z3", "zone.transition:Z4_boss_courtyard");
    orch.game_loop
        .register_trigger_action("enter_z2a", "zone.transition:Z2a_side_alcove");

    // Register storm decision trigger.
    orch.game_loop
        .register_trigger_action("storm_trigger", "decision.open:storm_routing");

    // Register a simple cinematic for boss intro.
    let mut boss_tl = Timeline::new("boss_intro", 1.5);
    boss_tl.add_camera_track(vec![CameraKey::new(
        Time::from_secs(0.5),
        (0.0, 5.0, 10.0),
        (0.0, 0.0, 0.0),
        60.0,
    )]);
    orch.game_loop.cinematics.load("boss_intro", boss_tl);

    // Register a cinematic trigger for Z4 entry.
    orch.game_loop
        .register_trigger_action("cinematic_z4", "cinematic.play:boss_intro");

    // Register a crossroads arrival dialogue.
    let crossroads_toml = r#"
id = "crossroads_arrival"
start = "crossroads_intro"

[[nodes]]
id = "crossroads_intro"
line = { speaker = "Companion", text = "The storm crossroads lies ahead. Which path?" }
choices = [
    { text = "Stabilize the storm", go_to = "storm_stabilize" },
    { text = "Redirect it toward the Warden", go_to = "storm_redirect" },
]

[[nodes]]
id = "storm_stabilize"
line = { speaker = "Companion", text = "We'll calm the storm. Visibility restored." }
end = true

[[nodes]]
id = "storm_redirect"
line = { speaker = "Companion", text = "The storm bears down on the Warden's courtyard!" }
end = true
"#;
    let loaded = load_dialogue_from_toml(crossroads_toml).unwrap();
    orch.game_loop.register_dialogue(loaded);
    orch.game_loop
        .register_trigger_action("dialogue_crossroads", "dialogue.play:crossroads_arrival");

    orch
}

// ── Phase tests ────────────────────────────────────────────────────────────

#[test]
fn z0_tutorial_phase() {
    let mut orch = setup_orchestrator();

    // Start at TutorialBegin.
    assert_eq!(orch.beat(), WalkthroughBeat::TutorialBegin);
    assert!(!orch.player().tutorial_complete);

    // Simulate tutorial actions: repair 2 anchors, collect echoes.
    orch.repair_anchor("anchor_z0_01");
    orch.repair_anchor("anchor_z0_02");
    orch.collect_echoes(3);
    assert_eq!(orch.player().anchors_repaired, 2);
    assert_eq!(orch.player().echoes, 3);

    // Complete the tutorial.
    orch.complete_tutorial();
    assert!(orch.player().tutorial_complete);

    // Tick advances beat to TutorialComplete.
    let result = orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::TutorialComplete);
    // The TutorialComplete walkthrough event is pushed by complete_tutorial()
    // before the tick, so it's cleared by the tick's event flush. The beat
    // advancement event is what the tick produces.
    assert!(result.walkthrough_events.iter().any(|e| matches!(
        e,
        WalkthroughEvent::BeatAdvanced {
            to: WalkthroughBeat::TutorialComplete,
            ..
        }
    )));
}

#[test]
fn z1_echo_grove_exploration() {
    let mut orch = setup_orchestrator();
    orch.complete_tutorial();

    // Transition to Z1.
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z0".to_string()]);
    let result = orch.tick(DT);

    assert_eq!(orch.beat(), WalkthroughBeat::EchoGroveEntry);
    assert_eq!(orch.player().zone_name, "Z1_echo_grove");

    assert!(result
        .game_events
        .iter()
        .any(|e| matches!(e, GameLoopEvent::ZoneLoading { zone_name, .. } if zone_name == "Z1_echo_grove")));

    // Explore: collect echoes, take a hit from a trap.
    orch.collect_echoes(5);
    orch.damage_player(15.0);
    assert_eq!(orch.player().echoes, 5);
    assert_eq!(orch.player().hp, 85.0);
}

#[test]
fn z2_fractured_cliffs_combat() {
    let mut orch = setup_orchestrator();
    orch.complete_tutorial();

    // Skip to Z2.
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z0".to_string()]);
    orch.tick(DT);
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z1".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::FracturedCliffEntry);
    assert_eq!(orch.player().zone_name, "Z2_fractured_cliffs");

    // Start the skirmish encounter.
    orch.start_combat();
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::SkirmishActive);
    assert_eq!(orch.combat.phase(), EncounterPhase::Active);

    // Kill wave 1 enemies one by one.
    orch.damage_enemy(1, 50.0); // Scout 1 (40hp) → killed
    orch.damage_enemy(2, 50.0); // Scout 2 (40hp) → killed
    orch.damage_enemy(3, 70.0); // Stalker (60hp) → killed
                                // Player takes some hits.
    orch.damage_player(20.0);

    // Drain combat events from this wave.
    let result = orch.tick(DT);
    assert!(result
        .combat_events
        .iter()
        .any(|e| matches!(e, CombatEvent::WaveCleared { wave: 0, .. })));

    // Wave 2 should now be active.
    assert_eq!(orch.combat.current_wave, 1);
    orch.damage_enemy(4, 90.0); // Brute 1 (80hp)
    orch.damage_enemy(5, 90.0); // Brute 2 (80hp)

    let result = orch.tick(DT);
    assert!(orch.combat.is_cleared());
    assert_eq!(orch.beat(), WalkthroughBeat::SkirmishCleared);
    assert!(result
        .combat_events
        .iter()
        .any(|e| matches!(e, CombatEvent::EncounterCleared { .. })));

    // Telemetry should reflect all kills.
    assert_eq!(orch.telemetry().enemies_defeated, 5);
}

#[test]
fn z2a_optional_side_alcove() {
    let mut orch = setup_orchestrator();

    // Skip to SkirmishCleared.
    orch.force_beat(WalkthroughBeat::SkirmishCleared);

    // Visit optional side alcove.
    orch.game_loop
        .notify_trigger_enter(vec!["enter_z2a".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::SideAlcoveVisited);

    // Collect a secret echo from the alcove.
    orch.collect_echoes(10);
    assert_eq!(orch.player().echoes, 10);

    // Then continue to Z3.
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z2".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::LoomCrossroadsEntry);
}

#[test]
fn z3_storm_choice_stabilize() {
    let mut orch = setup_orchestrator();
    orch.force_beat(WalkthroughBeat::LoomCrossroadsEntry);

    // Trigger the storm decision.
    orch.game_loop
        .notify_trigger_enter(vec!["storm_trigger".to_string()]);
    let result = orch.tick(DT);
    assert!(result
        .game_events
        .iter()
        .any(|e| matches!(e, GameLoopEvent::StormDecisionPrompt)));

    // Choose Stabilize.
    orch.make_storm_choice(StormChoice::Stabilize);
    let result = orch.tick(DT);
    assert!(result.game_events.iter().any(|e| matches!(
        e,
        GameLoopEvent::StormDecisionMade { choice } if *choice == StormChoice::Stabilize
    )));
    assert!(result.game_events.iter().any(|e| matches!(
        e,
        GameLoopEvent::StormResolved { choice } if *choice == StormChoice::Stabilize
    )));

    assert_eq!(orch.beat(), WalkthroughBeat::StormDecisionMade);
    assert_eq!(orch.telemetry().storm_choice, Some(StormChoice::Stabilize));
}

#[test]
fn z3_storm_choice_redirect() {
    let mut orch = setup_orchestrator();
    orch.force_beat(WalkthroughBeat::LoomCrossroadsEntry);

    orch.game_loop
        .notify_trigger_enter(vec!["storm_trigger".to_string()]);
    orch.tick(DT);

    orch.make_storm_choice(StormChoice::Redirect);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::StormDecisionMade);
    assert_eq!(orch.telemetry().storm_choice, Some(StormChoice::Redirect));
}

#[test]
fn z4_boss_arena_entry() {
    let mut orch = setup_orchestrator();
    orch.force_beat(WalkthroughBeat::StormDecisionMade);

    // Enter boss courtyard.
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z3".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::BossArenaEntry);
    assert_eq!(orch.player().zone_name, "Z4_boss_courtyard");

    // Trigger boss intro cinematic.
    orch.game_loop
        .notify_trigger_enter(vec!["cinematic_z4".to_string()]);
    let result = orch.tick(DT);
    assert!(result
        .game_events
        .iter()
        .any(|e| matches!(e, GameLoopEvent::CinematicStarted { .. })));
    assert!(orch.game_loop.is_cinematic_playing());
}

#[test]
fn boss_defeat_to_debrief() {
    let mut orch = setup_orchestrator();
    orch.force_beat(WalkthroughBeat::BossDefeated);

    // Tick should auto-advance to Debrief.
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::Debrief);

    // Complete the run.
    orch.advance_to_debrief();
    assert!(orch.is_complete());
}

// ── Full Z0→Z4 walkthrough ────────────────────────────────────────────────

#[test]
fn full_z0_to_z4_walkthrough() {
    let mut orch = setup_orchestrator();
    let beats: Vec<WalkthroughBeat> = Vec::new();

    // ── Z0: Tutorial ──────────────────────────────────────────────
    assert_eq!(orch.beat(), WalkthroughBeat::TutorialBegin);
    orch.repair_anchor("anchor_z0_01");
    orch.repair_anchor("anchor_z0_02");
    orch.collect_echoes(3);
    orch.complete_tutorial();

    // ── Z0 → Z1 ──────────────────────────────────────────────────
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z0".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::EchoGroveEntry);
    assert_eq!(orch.player().zone_name, "Z1_echo_grove");

    // Explore Echo Grove.
    orch.collect_echoes(5);
    orch.damage_player(10.0);
    orch.heal_player(10.0); // Found a heal item.

    // ── Z1 → Z2 ──────────────────────────────────────────────────
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z1".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::FracturedCliffEntry);

    // ── Z2: Combat skirmish ───────────────────────────────────────
    orch.start_combat();
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::SkirmishActive);

    // Wave 1.
    orch.damage_enemy(1, 50.0);
    orch.damage_enemy(2, 50.0);
    orch.damage_enemy(3, 70.0);
    orch.damage_player(25.0);
    // Wave 2.
    orch.damage_enemy(4, 90.0);
    orch.damage_enemy(5, 90.0);

    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::SkirmishCleared);

    // ── Z2 → Z3 ──────────────────────────────────────────────────
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z2".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::LoomCrossroadsEntry);

    // ── Z3: Storm choice ──────────────────────────────────────────
    orch.game_loop
        .notify_trigger_enter(vec!["storm_trigger".to_string()]);
    orch.tick(DT);

    orch.make_storm_choice(StormChoice::Redirect);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::StormDecisionMade);

    // ── Z3 → Z4 ──────────────────────────────────────────────────
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z3".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::BossArenaEntry);
    assert_eq!(orch.player().zone_name, "Z4_boss_courtyard");

    // ── Z4: Boss fight (simulated — no boss-director feature) ────
    orch.force_beat(WalkthroughBeat::BossDefeated);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::Debrief);

    orch.advance_to_debrief();
    assert!(orch.is_complete());

    // ── Telemetry validation ──────────────────────────────────────
    let tel = orch.telemetry();
    assert_eq!(tel.enemies_defeated, 5, "should have 5 kills from Z2");
    assert_eq!(tel.anchors_repaired, 2, "should have 2 anchors from Z0");
    assert_eq!(tel.echoes_collected, 8, "should have 3+5 echoes");
    assert_eq!(tel.storm_choice, Some(StormChoice::Redirect));
    assert!(tel.total_time > 0.0, "should have accumulated time");
    assert!(tel.damage_taken > 0.0, "player took damage");
    assert_eq!(tel.deaths, 0, "player should not have died");

    // ── Beat trail validation ─────────────────────────────────────
    // The walkthrough should have visited these zones.
    assert!(
        tel.zones_visited.contains(&"Z1_echo_grove".to_string()),
        "Z1 visited"
    );
    assert!(
        tel.zones_visited
            .contains(&"Z2_fractured_cliffs".to_string()),
        "Z2 visited"
    );
    assert!(
        tel.zones_visited
            .contains(&"Z3_loom_crossroads".to_string()),
        "Z3 visited"
    );
    assert!(
        tel.zones_visited.contains(&"Z4_boss_courtyard".to_string()),
        "Z4 visited"
    );
    drop(beats); // suppress unused warning
}

#[test]
fn player_death_mid_walkthrough_recovers() {
    let mut orch = setup_orchestrator();
    orch.complete_tutorial();
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z0".to_string()]);
    orch.tick(DT);

    // Take lethal damage → auto-respawn.
    orch.damage_player(200.0);
    assert!(orch.player().alive, "should auto-respawn");
    assert_eq!(orch.player().hp, 100.0);
    assert_eq!(orch.telemetry().deaths, 1);

    // Walkthrough should still be functional.
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z1".to_string()]);
    orch.tick(DT);
    assert_eq!(orch.beat(), WalkthroughBeat::FracturedCliffEntry);
}

#[test]
fn dialogue_during_walkthrough() {
    let mut orch = setup_orchestrator();
    orch.force_beat(WalkthroughBeat::LoomCrossroadsEntry);

    // Trigger the crossroads dialogue.
    orch.game_loop
        .notify_trigger_enter(vec!["dialogue_crossroads".to_string()]);
    let result = orch.tick(DT);

    assert!(result
        .game_events
        .iter()
        .any(|e| matches!(e, GameLoopEvent::DialogueDisplay { .. })));
    assert!(orch.game_loop.is_dialogue_active());

    // Make a choice — "Stabilize the storm".
    orch.game_loop
        .notify_dialogue_choice("crossroads_arrival", 0);
    let result = orch.tick(DT);
    assert!(result
        .game_events
        .iter()
        .any(|e| matches!(e, GameLoopEvent::DialogueEnded { .. })));
    assert_eq!(orch.telemetry().dialogues_completed, 1);
}

#[test]
fn telemetry_complete_at_end() {
    let mut orch = setup_orchestrator();

    // Fast-track through the whole walkthrough.
    orch.complete_tutorial();
    orch.repair_anchor("a1");
    orch.collect_echoes(10);
    orch.damage_player(30.0);
    orch.heal_player(15.0);

    orch.game_loop
        .notify_trigger_enter(vec!["exit_z0".to_string()]);
    orch.tick(DT);
    orch.game_loop
        .notify_trigger_enter(vec!["exit_z1".to_string()]);
    orch.tick(DT);

    orch.start_combat();
    orch.tick(DT);
    orch.clear_combat_wave();
    orch.clear_combat_wave();
    orch.tick(DT);

    orch.game_loop
        .notify_trigger_enter(vec!["exit_z2".to_string()]);
    orch.tick(DT);
    orch.game_loop
        .notify_trigger_enter(vec!["storm_trigger".to_string()]);
    orch.tick(DT);
    orch.make_storm_choice(StormChoice::Stabilize);
    orch.tick(DT);

    orch.game_loop
        .notify_trigger_enter(vec!["exit_z3".to_string()]);
    orch.tick(DT);

    orch.force_beat(WalkthroughBeat::BossDefeated);
    orch.tick(DT);
    orch.advance_to_debrief();

    assert!(orch.is_complete());

    let tel = orch.telemetry();
    assert_eq!(tel.anchors_repaired, 1);
    assert_eq!(tel.echoes_collected, 10);
    assert!(tel.damage_taken > 0.0);
    assert_eq!(tel.enemies_defeated, 5);
    assert_eq!(tel.storm_choice, Some(StormChoice::Stabilize));
    assert!(tel.zones_visited.len() >= 4);
}
