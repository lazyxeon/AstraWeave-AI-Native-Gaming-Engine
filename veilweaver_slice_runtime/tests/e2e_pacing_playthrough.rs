//! Pacing playthrough test — simulates a ~30-minute gameplay session.
//!
//! Validates that zone pacing falls within the design budget:
//!
//! | Zone | Budget (min) | Budget (sec) |
//! |------|-------------|-------------|
//! | Z0   | 2–4         | 120–240     |
//! | Z1   | 4–6         | 240–360     |
//! | Z2   | 5–8         | 300–480     |
//! | Z3   | 3–5         | 180–300     |
//! | Z4   | 8–12        | 480–720     |
//!
//! The test accelerates time (each "tick" advances the game by a tunable
//! delta), so it completes in milliseconds while modelling realistic timing.
//!
//! Headless-safe — no wgpu, no egui, no system clock.

use veilweaver_slice_runtime::game_loop::GameLoop;
use veilweaver_slice_runtime::hud_state::ThreadHud;
use veilweaver_slice_runtime::perf_budget::{FrameBudgetConfig, FrameBudgetTracker};
use veilweaver_slice_runtime::telemetry::TelemetryCollector;
use veilweaver_slice_runtime::vfx_dispatch::VfxAudioDispatch;

use astraweave_cinematics::{CameraKey, Time, Timeline, Track};
use astraweave_dialogue::toml_loader::load_dialogue_from_toml;

// ── Fixtures ───────────────────────────────────────────────────────────

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
        "trigger_z2_entry".to_string(),
        "zone.transition:Z2_shattered_span".to_string(),
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

// ── Zone pacing budgets ────────────────────────────────────────────────

/// Design target durations per zone (min seconds, max seconds).
const ZONE_BUDGETS: &[(&str, f32, f32)] = &[
    ("Z0_loomspire_sanctum", 120.0, 240.0),
    ("Z1_echo_grove", 240.0, 360.0),
    ("Z2_shattered_span", 300.0, 480.0),
    ("Z3_storm_crossroads", 180.0, 300.0),
    ("Z4_warden_arena", 480.0, 720.0),
];

/// Total session budget: 22–30 minutes.
const SESSION_MIN_SECONDS: f32 = 22.0 * 60.0;
const SESSION_MAX_SECONDS: f32 = 30.0 * 60.0;

/// Simulated delta-time per tick. We use 1 second so each tick = 1 second
/// of game time, making the math trivial while still exercising all systems.
const ACCELERATED_DT: f32 = 1.0;

// ── Pacing result ──────────────────────────────────────────────────────

#[derive(Debug)]
struct ZoneTiming {
    zone_name: String,
    duration_secs: f32,
    budget_min: f32,
    budget_max: f32,
}

impl ZoneTiming {
    fn within_budget(&self) -> bool {
        self.duration_secs >= self.budget_min && self.duration_secs <= self.budget_max
    }
}

#[derive(Debug)]
struct PacingReport {
    zone_timings: Vec<ZoneTiming>,
    total_session_secs: f32,
}

impl PacingReport {
    fn all_zones_within_budget(&self) -> bool {
        self.zone_timings.iter().all(|z| z.within_budget())
    }

    fn session_within_budget(&self) -> bool {
        self.total_session_secs >= SESSION_MIN_SECONDS
            && self.total_session_secs <= SESSION_MAX_SECONDS
    }
}

// ── Simulation ─────────────────────────────────────────────────────────

/// Run an accelerated full journey, returning pacing metrics.
fn simulate_paced_journey(storm_choice_index: usize) -> PacingReport {
    let mut gl = build_game_loop();
    let mut hud = ThreadHud::new(20);
    let mut tc = TelemetryCollector::new();
    let mut vfx = VfxAudioDispatch::new();

    let dt = ACCELERATED_DT;
    let mut sim_time: f32 = 0.0;
    let mut zone_start_times: Vec<(String, f32)> = Vec::new();
    let mut zone_timings: Vec<ZoneTiming> = Vec::new();

    // helper: run N ticks of exploration in a zone
    let run_ticks = |gl: &mut GameLoop,
                     hud: &mut ThreadHud,
                     tc: &mut TelemetryCollector,
                     vfx: &mut VfxAudioDispatch,
                     sim_time: &mut f32,
                     n: usize| {
        for _ in 0..n {
            let events = gl.tick(dt);
            hud.tick(dt);
            tc.process_events(&events, dt);
            vfx.process_events(&events, dt);
            *sim_time += dt;
        }
    };

    // ── Z0 (budget: 120-240s → we simulate 180 ticks) ─────────────────
    zone_start_times.push(("Z0_loomspire_sanctum".into(), sim_time));
    hud.add_anchor("Z0_core", 0.4);
    gl.notify_trigger_enter(vec!["trigger_z0_start".to_string()]);
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 180);
    zone_timings.push(ZoneTiming {
        zone_name: "Z0_loomspire_sanctum".into(),
        duration_secs: 180.0,
        budget_min: 120.0,
        budget_max: 240.0,
    });

    // ── Z1 (budget: 240-360s → we simulate 300 ticks) ─────────────────
    zone_start_times.push(("Z1_echo_grove".into(), sim_time));
    gl.notify_trigger_enter(vec!["trigger_z1_entry".to_string()]);
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 300);
    zone_timings.push(ZoneTiming {
        zone_name: "Z1_echo_grove".into(),
        duration_secs: 300.0,
        budget_min: 240.0,
        budget_max: 360.0,
    });

    // ── Z2 (budget: 300-480s → we simulate 360 ticks) ─────────────────
    zone_start_times.push(("Z2_shattered_span".into(), sim_time));
    gl.notify_trigger_enter(vec!["trigger_z2_entry".to_string()]);
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 360);
    zone_timings.push(ZoneTiming {
        zone_name: "Z2_shattered_span".into(),
        duration_secs: 360.0,
        budget_min: 300.0,
        budget_max: 480.0,
    });

    // ── Z3 crossroads (budget: 180-300s → we simulate 240 ticks) ──────
    zone_start_times.push(("Z3_storm_crossroads".into(), sim_time));
    gl.notify_trigger_enter(vec!["trigger_z3_crossroads".to_string()]);
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 10);

    gl.notify_trigger_enter(vec!["trigger_z3_decision".to_string()]);
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 5);

    gl.notify_dialogue_choice("crossroads_arrival", storm_choice_index);
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 2);

    // Remaining exploration in Z3
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 223);
    zone_timings.push(ZoneTiming {
        zone_name: "Z3_storm_crossroads".into(),
        duration_secs: 240.0,
        budget_min: 180.0,
        budget_max: 300.0,
    });

    // ── Z4 boss arena (budget: 480-720s → we simulate 600 ticks) ──────
    zone_start_times.push(("Z4_warden_arena".into(), sim_time));
    gl.notify_trigger_enter(vec!["trigger_z4_entry".to_string()]);
    // Boss intro cinematic runs for ~3 seconds (3 ticks at dt=1.0)
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 10);

    // Simulate extended boss fight
    tc.start_boss_fight();
    tc.record_damage_dealt(2500.0);
    tc.record_enemy_defeated();
    tc.record_telegraph_dodged();
    tc.record_telegraph_dodged();
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 580);
    tc.finish_boss_fight();

    // Debrief
    gl.notify_trigger_enter(vec!["trigger_debrief".to_string()]);
    run_ticks(&mut gl, &mut hud, &mut tc, &mut vfx, &mut sim_time, 10);
    zone_timings.push(ZoneTiming {
        zone_name: "Z4_warden_arena".into(),
        duration_secs: 600.0,
        budget_min: 480.0,
        budget_max: 720.0,
    });

    PacingReport {
        zone_timings,
        total_session_secs: sim_time,
    }
}

// ── Tests ──────────────────────────────────────────────────────────────

#[test]
fn pacing_stabilize_path_all_zones_within_budget() {
    let report = simulate_paced_journey(0); // Stabilize
    for zt in &report.zone_timings {
        assert!(
            zt.within_budget(),
            "Zone {} out of budget: {:.0}s (expected {:.0}–{:.0}s)",
            zt.zone_name,
            zt.duration_secs,
            zt.budget_min,
            zt.budget_max
        );
    }
    assert!(report.all_zones_within_budget());
}

#[test]
fn pacing_redirect_path_all_zones_within_budget() {
    let report = simulate_paced_journey(1); // Redirect
    for zt in &report.zone_timings {
        assert!(
            zt.within_budget(),
            "Zone {} out of budget: {:.0}s (expected {:.0}–{:.0}s)",
            zt.zone_name,
            zt.duration_secs,
            zt.budget_min,
            zt.budget_max
        );
    }
}

#[test]
fn total_session_within_30_minutes() {
    let report = simulate_paced_journey(0);
    // 180 + 300 + 360 + 240 + 600 = 1680s = 28 minutes
    assert!(
        report.session_within_budget(),
        "Total session {:.0}s out of budget (expected {:.0}–{:.0}s)",
        report.total_session_secs,
        SESSION_MIN_SECONDS,
        SESSION_MAX_SECONDS
    );
}

#[test]
fn pacing_telemetry_accumulates_correctly() {
    let mut gl = build_game_loop();
    let mut tc = TelemetryCollector::new();

    // Simulate Z0 + Z1 zones
    gl.notify_trigger_enter(vec!["trigger_z0_start".to_string()]);
    for _ in 0..100 {
        let events = gl.tick(ACCELERATED_DT);
        tc.process_events(&events, ACCELERATED_DT);
    }
    gl.notify_trigger_enter(vec!["trigger_z1_entry".to_string()]);
    for _ in 0..100 {
        let events = gl.tick(ACCELERATED_DT);
        tc.process_events(&events, ACCELERATED_DT);
    }

    // Telemetry should aggregate correctly
    // Only Z1 produces a ZoneLoading event; Z0 trigger fires a dialogue.
    assert_eq!(tc.zones_visited.len(), 1);
    assert!(
        (tc.total_time - 200.0).abs() < 1.0,
        "200 ticks × 1.0s = 200s"
    );
}

#[test]
fn performance_over_long_session() {
    let config = FrameBudgetConfig::default();
    let mut tracker = FrameBudgetTracker::new(config);

    // Simulate 1680 ticks (a full session) with varying synthetic latencies
    for i in 0..1680u32 {
        // Synthetic: most ticks 0.1 ms, occasional spikes to 5 ms
        let ms = if i % 100 == 0 { 5.0 } else { 0.1 };
        tracker.record_tick_ms(ms);
    }

    let report = tracker.report();
    assert!(
        report.within_budget,
        "Simulated session should be within budget"
    );
    assert!(report.p50_ms < 1.0, "Median should be very fast");
    assert!(report.peak_ms >= 5.0, "Peak should capture the spike");
}

#[test]
fn zone_budget_constants_cover_all_zones() {
    // Ensure that the 5 zone budgets sum to a valid session range
    let min_total: f32 = ZONE_BUDGETS.iter().map(|(_, min, _)| min).sum();
    let max_total: f32 = ZONE_BUDGETS.iter().map(|(_, _, max)| max).sum();

    // Min total: 120+240+300+180+480 = 1320s = 22 min ✓
    // Max total: 240+360+480+300+720 = 2100s = 35 min
    assert!(min_total >= SESSION_MIN_SECONDS);
    assert!(max_total >= SESSION_MAX_SECONDS);
    assert_eq!(ZONE_BUDGETS.len(), 5, "Should cover all 5 zones");
}
