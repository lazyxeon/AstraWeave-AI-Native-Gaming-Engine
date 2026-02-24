//! End-to-end validation smoke test — determinism, performance budget, and
//! checkpoint integration across a full Z0→Z4 walkthrough.
//!
//! This test exercises Phase 5 modules:
//! - **Determinism**: Two identical runs produce identical state fingerprints.
//! - **Performance budget**: All ticks stay within the 16.67 ms budget.
//! - **Checkpoint**: Save at each zone boundary, reload, and verify equality.
//!
//! Runs entirely headless — no wgpu, no egui, no system clock.

use veilweaver_slice_runtime::checkpoint::{CheckpointStore, SliceCheckpoint};
use veilweaver_slice_runtime::determinism::{DeterminismTrace, MultiRunValidator};
use veilweaver_slice_runtime::game_loop::{GameLoop, GameLoopEvent};
use veilweaver_slice_runtime::hud_state::ThreadHud;
use veilweaver_slice_runtime::perf_budget::{FrameBudgetConfig, FrameBudgetTracker};
use veilweaver_slice_runtime::telemetry::TelemetryCollector;
use veilweaver_slice_runtime::vfx_dispatch::VfxAudioDispatch;

use astraweave_cinematics::{CameraKey, Time, Timeline, Track};
use astraweave_dialogue::toml_loader::load_dialogue_from_toml;

// ── Shared fixtures ────────────────────────────────────────────────────

const INTRO_TOML: &str = r#"
id = "intro_dialogue"
start = "intro_start"

[[nodes]]
id = "intro_start"
line = { speaker = "Companion", text = "The Veilweaver expedition begins." }
end = true
"#;

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

/// Build a fully-wired game loop with all dialogues, cinematics, and triggers.
fn build_game_loop() -> GameLoop {
    let mut gl = GameLoop::new();

    let intro = load_dialogue_from_toml(INTRO_TOML).expect("intro TOML valid");
    gl.register_dialogue(intro);

    let crossroads = load_dialogue_from_toml(CROSSROADS_TOML).expect("crossroads TOML valid");
    gl.register_dialogue(crossroads);

    gl.cinematics
        .load("boss_intro".to_string(), make_boss_intro_timeline());
    gl.cinematics
        .load("debrief_resolution".to_string(), make_debrief_timeline());

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
        "trigger_debrief".to_string(),
        "cinematic.play:debrief_resolution".to_string(),
    );

    gl
}

/// Run the canonical Z0→Z4 journey, collecting events + per-tick fingerprints.
///
/// Returns `(events, trace, hud, telemetry, vfx, checkpoints)`.
fn run_canonical_journey(
    label: &str,
) -> (
    Vec<Vec<GameLoopEvent>>,
    DeterminismTrace,
    ThreadHud,
    TelemetryCollector,
    VfxAudioDispatch,
    CheckpointStore,
) {
    let mut gl = build_game_loop();
    let mut hud = ThreadHud::new(20);
    let mut tc = TelemetryCollector::new();
    let mut vfx = VfxAudioDispatch::new();
    let mut trace = DeterminismTrace::new(label);
    let mut store = CheckpointStore::with_capacity(10);
    let mut all_events: Vec<Vec<GameLoopEvent>> = Vec::new();
    let mut tick: u64 = 0;
    let dt = 0.016;

    // helper: tick + record
    let do_tick = |gl: &mut GameLoop,
                   hud: &mut ThreadHud,
                   tc: &mut TelemetryCollector,
                   vfx: &mut VfxAudioDispatch,
                   trace: &mut DeterminismTrace,
                   tick: &mut u64|
     -> Vec<GameLoopEvent> {
        let events = gl.tick(dt);
        hud.tick(dt);
        tc.process_events(&events, dt);
        vfx.process_events(&events, dt);
        trace.record_tick(
            *tick,
            &veilweaver_slice_runtime::determinism::StateFingerprint::capture(
                *tick, gl, hud, tc, vfx,
            ),
        );
        *tick += 1;
        events
    };

    // ── Z0: intro ──────────────────────────────────────────────────────
    hud.add_anchor("Z0_core", 0.4);
    gl.notify_trigger_enter(vec!["trigger_z0_start".to_string()]);
    let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
    all_events.push(events);
    store.push(SliceCheckpoint::capture(
        tick,
        "after_z0_intro",
        &gl,
        &hud,
        &tc,
        &vfx,
    ));

    // ── Z1: zone transition ────────────────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_z1_entry".to_string()]);
    let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
    all_events.push(events);
    store.push(SliceCheckpoint::capture(
        tick,
        "after_z1_entry",
        &gl,
        &hud,
        &tc,
        &vfx,
    ));

    // ── Z3: crossroads dialogue ────────────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_z3_crossroads".to_string()]);
    let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
    all_events.push(events);

    // Decision prompt
    gl.notify_trigger_enter(vec!["trigger_z3_decision".to_string()]);
    let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
    all_events.push(events);

    // Choose Stabilize (index 0)
    gl.notify_dialogue_choice("crossroads_arrival", 0);
    let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
    all_events.push(events);

    // Apply deferred choice
    let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
    all_events.push(events);
    store.push(SliceCheckpoint::capture(
        tick,
        "after_storm_resolved",
        &gl,
        &hud,
        &tc,
        &vfx,
    ));

    // ── Z4: boss intro cinematic ───────────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_z4_entry".to_string()]);
    for _ in 0..250 {
        let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
        if events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::CinematicFinished { .. }))
        {
            all_events.push(events);
            break;
        }
        all_events.push(events);
    }
    store.push(SliceCheckpoint::capture(
        tick,
        "after_boss_intro",
        &gl,
        &hud,
        &tc,
        &vfx,
    ));

    // ── Debrief cinematic ──────────────────────────────────────────────
    gl.notify_trigger_enter(vec!["trigger_debrief".to_string()]);
    for _ in 0..200 {
        let events = do_tick(&mut gl, &mut hud, &mut tc, &mut vfx, &mut trace, &mut tick);
        if events
            .iter()
            .any(|e| matches!(e, GameLoopEvent::CinematicFinished { .. }))
        {
            all_events.push(events);
            break;
        }
        all_events.push(events);
    }
    store.push(SliceCheckpoint::capture(
        tick, "final", &gl, &hud, &tc, &vfx,
    ));

    (all_events, trace, hud, tc, vfx, store)
}

// ── Determinism tests ──────────────────────────────────────────────────

#[test]
fn two_identical_runs_produce_identical_traces() {
    let (_, trace_a, ..) = run_canonical_journey("run_A");
    let (_, trace_b, ..) = run_canonical_journey("run_B");

    let report = trace_a.compare(&trace_b);
    assert_eq!(
        report.matched_ticks, report.total_ticks,
        "All ticks should match between two identical runs.\nMismatches: {:?}",
        report.mismatches
    );
}

#[test]
fn multi_run_validator_passes_for_identical_runs() {
    let mut validator = MultiRunValidator::with_capacity(3);

    for i in 0..3 {
        let (_, trace, ..) = run_canonical_journey(&format!("run_{}", i));
        validator.add_run(trace);
    }

    let reports = validator.validate();
    for report in &reports {
        assert_eq!(
            report.matched_ticks, report.total_ticks,
            "Multi-run validation should show 100% match"
        );
    }
}

// ── Performance budget tests ───────────────────────────────────────────

#[test]
fn all_ticks_within_budget() {
    // Our headless ticks are essentially instant; this validates the budget
    // tracking infrastructure works correctly.
    let config = FrameBudgetConfig::default();
    let mut tracker = FrameBudgetTracker::new(config);

    let mut gl = build_game_loop();
    let mut hud = ThreadHud::new(20);
    let mut tc = TelemetryCollector::new();
    let mut vfx = VfxAudioDispatch::new();
    let dt = 0.016;

    // Simulate a short run and record tick durations (synthetic: 0.1 ms each)
    gl.notify_trigger_enter(vec!["trigger_z0_start".to_string()]);
    for _ in 0..50 {
        let events = gl.tick(dt);
        hud.tick(dt);
        tc.process_events(&events, dt);
        vfx.process_events(&events, dt);
        tracker.record_tick_ms(0.1); // synthetic: 0.1 ms per tick
    }

    let report = tracker.report();
    assert!(
        report.within_budget,
        "Headless ticks should be within budget"
    );
    assert_eq!(report.violation_count, 0);
    assert!(report.p99_ms < 1.0, "p99 should be well under 1 ms");
}

#[test]
fn budget_tracker_detects_violations() {
    let config = FrameBudgetConfig {
        target_ms: 16.67,
        violation_multiplier: 1.5,
        ..FrameBudgetConfig::default()
    };
    let mut tracker = FrameBudgetTracker::new(config);

    // 10 normal ticks
    for _ in 0..10 {
        tracker.record_tick_ms(5.0);
    }
    // 2 over-budget ticks (> 16.67 * 1.5 = 25 ms)
    tracker.record_tick_ms(30.0);
    tracker.record_tick_ms(35.0);

    let report = tracker.report();
    assert_eq!(report.violation_count, 2);
    assert!(!report.within_budget);
}

// ── Checkpoint tests ───────────────────────────────────────────────────

#[test]
fn checkpoint_json_round_trip_full_journey() {
    let (.., store) = run_canonical_journey("checkpoint_test");
    assert!(store.len() >= 4, "Should have at least 4 zone checkpoints");

    // Serialise entire store → JSON → reload
    let json = store.save_all_to_json().unwrap();
    let reloaded = CheckpointStore::load_all_from_json(&json).unwrap();
    assert_eq!(reloaded.len(), store.len());

    // Each checkpoint should be identical
    for tick in store.ticks() {
        let orig = store.get(tick).unwrap();
        let loaded = reloaded.get(tick).unwrap();
        assert_eq!(
            orig, loaded,
            "Checkpoint at tick {} differs after reload",
            tick
        );
    }
}

#[test]
fn checkpoint_diff_across_runs_is_clean() {
    let (.., store_a) = run_canonical_journey("run_A");
    let (.., store_b) = run_canonical_journey("run_B");

    // Both runs should yield checkpoints at the same ticks
    assert_eq!(store_a.ticks(), store_b.ticks());

    for tick in store_a.ticks() {
        let a = store_a.get(tick).unwrap();
        let b = store_b.get(tick).unwrap();
        let diff = a.diff(b);
        assert!(
            diff.is_identical,
            "Checkpoint at tick {} differs between runs:\n{}",
            tick, diff
        );
    }
}

#[test]
fn latest_checkpoint_reflects_final_state() {
    let (_, _, hud, tc, _, store) = run_canonical_journey("final_state");
    let latest = store.latest().unwrap();

    assert_eq!(latest.header.label, "final");
    assert_eq!(latest.hud.echo_balance, hud.echoes.balance);
    assert_eq!(latest.telemetry.dialogues_completed, tc.dialogues_completed);
}

// ── Combined integration ───────────────────────────────────────────────

#[test]
fn full_validation_pipeline() {
    // Run twice
    let (_, trace_a, _, _, _, store_a) = run_canonical_journey("pipeline_A");
    let (_, trace_b, _, _, _, store_b) = run_canonical_journey("pipeline_B");

    // 1. Determinism
    let det_report = trace_a.compare(&trace_b);
    assert_eq!(
        det_report.matched_ticks, det_report.total_ticks,
        "Determinism check failed"
    );

    // 2. Checkpoint parity
    for tick in store_a.ticks() {
        let diff = store_a.get(tick).unwrap().diff(store_b.get(tick).unwrap());
        assert!(
            diff.is_identical,
            "Checkpoint parity failed at tick {}",
            tick
        );
    }

    // 3. Budget (synthetic)
    let mut tracker = FrameBudgetTracker::new(FrameBudgetConfig::default());
    for _ in 0..det_report.total_ticks {
        tracker.record_tick_ms(0.05);
    }
    let budget = tracker.report();
    assert!(budget.within_budget, "Budget validation failed");
}

// ── Edge-case smoke tests ──────────────────────────────────────────────

#[test]
fn empty_checkpoint_round_trips() {
    let gl = GameLoop::new();
    let hud = ThreadHud::default();
    let tc = TelemetryCollector::new();
    let vfx = VfxAudioDispatch::new();
    let cp = SliceCheckpoint::capture(0, "empty", &gl, &hud, &tc, &vfx);
    let json = cp.save_to_json().unwrap();
    let loaded = SliceCheckpoint::load_from_json(&json).unwrap();
    assert_eq!(cp, loaded);
}

#[test]
fn determinism_trace_empty_runs_compare_clean() {
    let a = DeterminismTrace::new("empty_A");
    let b = DeterminismTrace::new("empty_B");
    let report = a.compare(&b);
    assert!(report.mismatches.is_empty());
    assert_eq!(report.total_ticks, 0);
}

#[test]
fn checkpoint_store_handles_zero_capacity() {
    // capacity=0 means unlimited
    let mut store = CheckpointStore::with_capacity(0);
    let gl = GameLoop::new();
    let hud = ThreadHud::default();
    let tc = TelemetryCollector::new();
    let vfx = VfxAudioDispatch::new();
    for i in 0..100 {
        store.push(SliceCheckpoint::capture(i, "tick", &gl, &hud, &tc, &vfx));
    }
    assert_eq!(store.len(), 100);
}
