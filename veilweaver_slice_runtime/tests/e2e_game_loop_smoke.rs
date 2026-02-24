//! End-to-end smoke test — exercises the full Z0→Z4 game-loop sequence.
//!
//! This test walks through the intended player journey:
//! 1. Start in Z0, dialogue plays
//! 2. Transition through Z1, Z2
//! 3. Arrive at Z3 crossroads — storm decision
//! 4. Storm choice made (Stabilize) → arena modifiers resolved
//! 5. Transition to Z4 — boss intro cinematic
//! 6. Boss encounter active → tick directive
//! 7. Boss defeated → debrief cinematic
//!
//! Validates that all subsystems cooperate correctly via the GameLoop
//! orchestrator without panics, deadlocks, or lost events.

use astraweave_cinematics::{CameraKey, Time, Timeline, Track};
use astraweave_dialogue::toml_loader::load_dialogue_from_toml;
use veilweaver_slice_runtime::game_loop::{GameLoop, GameLoopEvent};

/// Minimal dialogue TOML for the crossroads scene.
const CROSSROADS_TOML: &str = r#"
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

/// Simple intro dialogue for Z0.
const INTRO_TOML: &str = r#"
id = "intro_dialogue"
start = "intro_start"

[[nodes]]
id = "intro_start"
line = { speaker = "Companion", text = "The Veilweaver expedition begins." }
end = true
"#;

fn make_boss_intro_timeline() -> Timeline {
    Timeline {
        name: "boss_intro".to_string(),
        duration: Time(3.0),
        tracks: vec![Track::camera(vec![
            CameraKey::new(Time(0.0), (0.0, 5.0, -10.0), (0.0, 0.0, 0.0), 60.0),
            CameraKey::new(Time(3.0), (0.0, 3.0, -5.0), (0.0, 1.0, 0.0), 50.0),
        ])],
    }
}

fn make_debrief_timeline() -> Timeline {
    Timeline {
        name: "debrief_resolution".to_string(),
        duration: Time(2.0),
        tracks: vec![Track::camera(vec![
            CameraKey::new(Time(0.0), (0.0, 2.0, -8.0), (0.0, 0.0, 0.0), 55.0),
            CameraKey::new(Time(2.0), (0.0, 4.0, -6.0), (0.0, 1.0, 0.0), 50.0),
        ])],
    }
}

fn has_event(kind: &str, events: &[GameLoopEvent]) -> bool {
    events.iter().any(|e| {
        let debug = format!("{:?}", e);
        debug.starts_with(kind)
    })
}

#[test]
fn full_z0_to_z4_journey() {
    let mut gl = GameLoop::new();

    // ── Register dialogues ──────────────────────────────────────────────
    let intro = load_dialogue_from_toml(INTRO_TOML).expect("intro TOML valid");
    gl.register_dialogue(intro);

    let crossroads = load_dialogue_from_toml(CROSSROADS_TOML).expect("crossroads TOML valid");
    gl.register_dialogue(crossroads);

    // ── Register cinematics ─────────────────────────────────────────────
    gl.cinematics
        .load("boss_intro".to_string(), make_boss_intro_timeline());
    gl.cinematics
        .load("debrief_resolution".to_string(), make_debrief_timeline());

    // ── Register trigger actions ────────────────────────────────────────
    gl.register_trigger_action(
        "trigger_z0_start".to_string(),
        "dialogue.play:intro_dialogue".to_string(),
    );
    gl.register_trigger_action(
        "trigger_z1_entry".to_string(),
        "zone.transition:Z1_echo_grove".to_string(),
    );
    gl.register_trigger_action(
        "trigger_z3_crossroads".to_string(),
        "dialogue.play:crossroads_arrival".to_string(),
    );
    gl.register_trigger_action(
        "trigger_z3_decision".to_string(),
        "decision.open:storm_choice".to_string(),
    );
    gl.register_trigger_action(
        "trigger_z4_entry".to_string(),
        "cinematic.play:boss_intro".to_string(),
    );
    gl.register_trigger_action(
        "trigger_z4_boss".to_string(),
        "boss.start:oathbound_warden".to_string(),
    );
    gl.register_trigger_action(
        "trigger_debrief".to_string(),
        "cinematic.play:debrief_resolution".to_string(),
    );

    // ── Phase 1: Z0 intro ──────────────────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_z0_start".to_string()]);
    let events = gl.tick(0.016);
    // Terminal dialogue: NodeEntered + Ended both fire in start(), so both
    // DialogueDisplay AND DialogueEnded appear in the same tick.
    assert!(
        has_event("DialogueDisplay", &events),
        "Z0 dialogue should fire"
    );
    assert!(
        has_event("DialogueEnded", &events),
        "Intro dialogue should end (terminal node, same tick)"
    );

    // ── Phase 2: Transition to Z1 ─────────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_z1_entry".to_string()]);
    let events = gl.tick(0.016);
    assert!(
        has_event("ZoneLoading", &events),
        "Z1 zone transition should fire"
    );

    // ── Phase 3: Z3 crossroads dialogue ────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_z3_crossroads".to_string()]);
    let events = gl.tick(0.016);
    assert!(
        has_event("DialogueDisplay", &events),
        "Crossroads dialogue should display"
    );

    // Verify choices are present.
    let has_choices = events.iter().any(|e| match e {
        GameLoopEvent::DialogueDisplay { choices, .. } => choices.len() == 2,
        _ => false,
    });
    assert!(has_choices, "Crossroads should present 2 storm choices");

    // ── Phase 4: Storm decision ────────────────────────────────────────
    // Enter the decision trigger zone → storm prompt.
    gl.notify_trigger_enter(vec!["trigger_z3_decision".to_string()]);
    let events = gl.tick(0.016);
    assert!(
        has_event("StormDecisionPrompt", &events),
        "Storm decision prompt should fire"
    );

    // Player makes a dialogue choice (index 0 = Stabilize).
    gl.notify_dialogue_choice("crossroads_arrival", 0);
    let _events = gl.tick(0.016);
    // The deferred storm choice was SET during this tick's process_dialogues,
    // but it is APPLIED at the start of the next tick.

    let events = gl.tick(0.016);
    // Now the deferred choice is applied → StormDecisionMade + process_storm → StormResolved.
    assert!(
        has_event("StormDecisionMade", &events),
        "Storm decision should fire on tick after dialogue choice"
    );

    // Storm should be fully resolved at this point (auto-resolve in process_storm).
    assert!(
        gl.storm_state.is_resolved(),
        "Storm should be resolved after Stabilize"
    );

    // ── Phase 5: Z4 boss intro cinematic ───────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_z4_entry".to_string()]);
    let events = gl.tick(0.016);
    assert!(
        has_event("CinematicStarted", &events),
        "Boss intro cinematic should start"
    );

    // Advance cinematic to completion (3 second timeline at 60fps ≈ 188 ticks).
    let mut finished = false;
    for _ in 0..250 {
        let events = gl.tick(0.016);
        if events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::CinematicFinished { .. }))
        {
            finished = true;
            break;
        }
    }
    assert!(
        finished,
        "Boss intro cinematic should finish within 250 ticks"
    );

    // ── Phase 6: Debrief cinematic ─────────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_debrief".to_string()]);
    let events = gl.tick(0.016);
    assert!(
        has_event("CinematicStarted", &events),
        "Debrief cinematic should start"
    );

    let mut debrief_finished = false;
    for _ in 0..200 {
        let events = gl.tick(0.016);
        if events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::CinematicFinished { .. }))
        {
            debrief_finished = true;
            break;
        }
    }
    assert!(
        debrief_finished,
        "Debrief cinematic should finish within 200 ticks"
    );
}

#[test]
fn storm_redirect_path_also_works() {
    let mut gl = GameLoop::new();

    let crossroads = load_dialogue_from_toml(CROSSROADS_TOML).expect("crossroads TOML valid");
    gl.register_dialogue(crossroads);

    gl.register_trigger_action(
        "trigger_z3_crossroads".to_string(),
        "dialogue.play:crossroads_arrival".to_string(),
    );
    gl.register_trigger_action(
        "trigger_z3_decision".to_string(),
        "decision.open:storm_choice".to_string(),
    );

    // Trigger crossroads dialogue.
    gl.notify_trigger_enter(vec!["trigger_z3_crossroads".to_string()]);
    gl.tick(0.016);

    // Enter decision zone.
    gl.notify_trigger_enter(vec!["trigger_z3_decision".to_string()]);
    gl.tick(0.016);

    // Choose index 1 = Redirect.
    gl.notify_dialogue_choice("crossroads_arrival", 1);
    let _events = gl.tick(0.016); // Sets deferred storm choice

    // Next tick: deferred choice applied → StormDecisionMade + auto-resolve.
    let events = gl.tick(0.016);
    assert!(
        has_event("StormDecisionMade", &events),
        "Redirect choice should fire decision event"
    );

    // Verify modifiers — storm is already auto-resolved by process_storm.
    let mods = gl.storm_state.modifiers();
    assert!(mods.is_some(), "Modifiers should exist after redirect");
    let m = mods.unwrap();
    assert!(m.visibility < 0.5, "Redirect reduces visibility");
    assert!(
        !m.fog_reinforcements_enabled,
        "Redirect disables fog reinforcements"
    );
}

#[test]
fn multiple_ticks_without_events_are_no_ops() {
    let mut gl = GameLoop::new();

    // 100 ticks with no triggers — should produce no events and not panic.
    for _ in 0..100 {
        let events = gl.tick(0.016);
        assert!(events.is_empty(), "No events expected on idle ticks");
    }
}
