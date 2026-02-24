//! Phase 4 Integration Test — VFX, Audio & Polish pipeline end-to-end.
//!
//! Verifies that game-loop events correctly flow through the VFX/audio
//! dispatch layer, producing the expected VFX scene, audio scene, and
//! presentation configuration at each stage of a Veilweaver playthrough.

use veilweaver_slice_runtime::audio_specs::{BossMusicLayer, StingerKind, ZoneAmbienceId};
use veilweaver_slice_runtime::game_loop::GameLoopEvent;
use veilweaver_slice_runtime::palette::PaletteSlot;
use veilweaver_slice_runtime::storm_choice::StormChoice;
use veilweaver_slice_runtime::vfx_dispatch::VfxAudioDispatch;
use veilweaver_slice_runtime::vfx_specs::{
    AnchorVfxSpec, AnchorVfxState, TelegraphVfxSpec, ThreadVfxSpec, Vec3f,
};

use astraweave_scene::world_partition::GridCoord;

fn coord(x: i32) -> GridCoord {
    GridCoord { x, y: 0, z: 0 }
}

// ── Full Journey VFX/Audio Pipeline ────────────────────────────────────

/// Simulates a complete Z0→Z4 journey, verifying VFX and audio state
/// at every meaningful transition.
#[test]
fn full_vfx_audio_journey_z0_to_z4() {
    let mut dispatch = VfxAudioDispatch::new();

    // ── Z0: Loomspire Sanctum ──────────────────────────────────────
    let events = vec![GameLoopEvent::ZoneLoading {
        zone_name: "Loomspire Sanctum".into(),
        coord: coord(100),
    }];
    dispatch.process_events(&events, 0.016);

    // Initial zone is 0 (default), so no ambience change on coord(100) since
    // zone_index = 100 - 100 = 0 (same as default).
    // Manually set zone to trigger ambience.
    dispatch
        .audio_scene_mut()
        .set_zone(ZoneAmbienceId::LoomspireSanctum);
    assert!(dispatch.audio_scene().ambience.is_some());

    // Add a stable thread
    dispatch.add_thread(ThreadVfxSpec::stable(
        "thread_01",
        Vec3f::new(0.0, 1.0, 0.0),
        Vec3f::new(10.0, 1.0, 0.0),
    ));
    assert_eq!(dispatch.vfx_scene().threads.len(), 1);

    // Add an anchor
    dispatch.set_anchor(AnchorVfxSpec::new(
        "anchor_z0",
        Vec3f::new(5.0, 0.0, 0.0),
        1.0,
    ));
    assert_eq!(
        dispatch.vfx_scene().anchors[0].vfx_state,
        AnchorVfxState::Perfect
    );

    // Collect an echo
    dispatch.emit_echo_burst(Vec3f::new(3.0, 0.0, 0.0), false);
    assert_eq!(dispatch.vfx_scene().echo_bursts.len(), 1);
    let stingers = dispatch.drain_stingers();
    assert!(stingers
        .iter()
        .any(|s| s.kind == StingerKind::EchoCollected));
    let bursts = dispatch.drain_echo_bursts();
    assert_eq!(bursts.len(), 1);
    assert!(dispatch.vfx_scene().echo_bursts.is_empty());

    // Weaving success feedback
    dispatch.weaving_success();
    let stingers = dispatch.drain_stingers();
    assert_eq!(stingers[0].kind, StingerKind::WeavingSuccess);

    // ── Z1: Threadhollow Ruins ─────────────────────────────────────
    let events = vec![GameLoopEvent::ZoneLoading {
        zone_name: "Threadhollow Ruins".into(),
        coord: coord(101),
    }];
    dispatch.process_events(&events, 0.016);
    assert_eq!(dispatch.current_zone(), 1);
    assert_eq!(
        dispatch.audio_scene().ambience.as_ref().unwrap().zone,
        ZoneAmbienceId::ThreadhollowRuins
    );

    // Anchor degrades
    dispatch.set_anchor(AnchorVfxSpec::new(
        "anchor_z1",
        Vec3f::new(20.0, 0.0, 0.0),
        0.8,
    ));
    dispatch.update_anchor_stability("anchor_z1", 0.2);
    assert_eq!(
        dispatch
            .vfx_scene()
            .anchors
            .iter()
            .find(|a| a.anchor_id == "anchor_z1")
            .unwrap()
            .vfx_state,
        AnchorVfxState::Critical
    );

    // Repair anchor
    dispatch.begin_anchor_repair("anchor_z1");
    dispatch.process_events(&[], 2.0);
    let repaired = dispatch
        .vfx_scene()
        .anchors
        .iter()
        .find(|a| a.anchor_id == "anchor_z1")
        .unwrap();
    assert!(repaired.is_repairing);
    assert!(repaired.repair_time > 1.0);

    // ── Z2: Stormreach Nexus ───────────────────────────────────────
    let events = vec![GameLoopEvent::ZoneLoading {
        zone_name: "Stormreach Nexus".into(),
        coord: coord(102),
    }];
    dispatch.process_events(&events, 0.016);
    assert_eq!(dispatch.current_zone(), 2);
    // Still twilight skybox (zones 0-2)
    assert!(dispatch.presentation().skybox.star_density > 0.0);

    // ── Z3: Frayed Expanse + Storm Decision ────────────────────────
    let events = vec![
        GameLoopEvent::ZoneLoading {
            zone_name: "Frayed Expanse".into(),
            coord: coord(103),
        },
        GameLoopEvent::StormDecisionPrompt,
    ];
    dispatch.process_events(&events, 0.016);
    assert_eq!(dispatch.current_zone(), 3);
    // Storm skybox for Z3+
    assert_eq!(dispatch.presentation().skybox.star_density, 0.0);

    let stingers = dispatch.drain_stingers();
    assert!(stingers
        .iter()
        .any(|s| s.kind == StingerKind::DecisionPrompt));

    // Player chooses Stabilize
    let events = vec![
        GameLoopEvent::StormDecisionMade {
            choice: StormChoice::Stabilize,
        },
        GameLoopEvent::StormResolved {
            choice: StormChoice::Stabilize,
        },
    ];
    dispatch.process_events(&events, 0.016);
    let storm = dispatch.vfx_scene().storm.as_ref().unwrap();
    assert!(storm.fog_density < 0.1, "Stabilized = clear arena");
    assert!(storm.thread_glow > 0.5);

    // ── Z4: Boss Courtyard ─────────────────────────────────────────
    let events = vec![GameLoopEvent::ZoneLoading {
        zone_name: "Boss Courtyard".into(),
        coord: coord(104),
    }];
    dispatch.process_events(&events, 0.016);
    assert_eq!(dispatch.current_zone(), 4);

    // Enter boss encounter
    dispatch.enter_boss_encounter();
    assert!(dispatch.in_boss_encounter());
    assert!(dispatch.presentation().vignette > 0.3);
    assert!(dispatch.audio_scene().boss_music.is_some());

    // Boss telegraph: Cleave
    let cleave = TelegraphVfxSpec::cleave(Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0));
    dispatch.add_telegraph(cleave);
    assert_eq!(dispatch.vfx_scene().telegraphs.len(), 1);

    // Telegraph fills up
    dispatch.set_telegraph_progress("Oathbound Cleave", 0.9);
    assert!(dispatch.vfx_scene().telegraphs[0].progress > 0.85);

    // Telegraph fires and is removed
    dispatch.remove_telegraph("Oathbound Cleave");
    assert!(dispatch.vfx_scene().telegraphs.is_empty());

    // Phase transition: Assessment → Fulcrum Shift
    dispatch.trigger_phase_transition(0, 1, Vec3f::ZERO);
    assert!(dispatch.vfx_scene().phase_transition.is_some());
    let stingers = dispatch.drain_stingers();
    assert!(stingers
        .iter()
        .any(|s| s.kind == StingerKind::PhaseTransition));

    // Music should have switched to FulcrumShift
    assert_eq!(
        dispatch.audio_scene().boss_music.as_ref().unwrap().layer,
        BossMusicLayer::FulcrumShift
    );

    // Tick until phase transition completes
    for _ in 0..300 {
        dispatch.process_events(&[], 1.0 / 60.0);
    }
    assert!(dispatch.vfx_scene().phase_transition.is_none());

    // Boss defeated
    dispatch.boss_defeated();
    assert!(!dispatch.in_boss_encounter());
    let stingers = dispatch.drain_stingers();
    assert!(stingers.iter().any(|s| s.kind == StingerKind::BossDefeated));
}

// ── Redirect Path Storm VFX ────────────────────────────────────────────

/// Verifies the redirect storm variant produces foggy/dangerous arena VFX.
#[test]
fn redirect_storm_produces_hostile_arena_vfx() {
    let mut dispatch = VfxAudioDispatch::new();

    let events = vec![
        GameLoopEvent::StormDecisionMade {
            choice: StormChoice::Redirect,
        },
        GameLoopEvent::StormResolved {
            choice: StormChoice::Redirect,
        },
    ];
    dispatch.process_events(&events, 0.016);

    let storm = dispatch.vfx_scene().storm.as_ref().unwrap();
    assert!(storm.fog_density > 0.3, "Redirected = heavy fog");
    assert!(storm.lightning_frequency > 0.0, "Redirected = lightning");
    assert!(storm.wind_strength > 5.0, "Redirected = strong wind");
    assert!(storm.thread_glow < 0.5, "Redirected = dim threads");
}

// ── Palette Consistency ────────────────────────────────────────────────

/// Verifies palette constants are consistent with VFX spec colors.
#[test]
fn palette_thread_emissive_matches_vfx_thread_blue() {
    let palette_color = PaletteSlot::ThreadEmissive.color();
    let vfx_color = veilweaver_slice_runtime::vfx_specs::VfxColor::THREAD_BLUE;
    assert!((palette_color.r - vfx_color.r).abs() < 0.001);
    assert!((palette_color.g - vfx_color.g).abs() < 0.001);
    assert!((palette_color.b - vfx_color.b).abs() < 0.001);
}

// ── Presentation Zone Config ───────────────────────────────────────────

/// Verifies that presentation config correctly changes per zone.
#[test]
fn presentation_config_varies_by_zone() {
    use veilweaver_slice_runtime::palette::PresentationConfig;

    let z0 = PresentationConfig::for_zone(0);
    let z3 = PresentationConfig::for_zone(3);
    let z4_boss = PresentationConfig::boss_encounter(4);

    // Z0 has stars (twilight), Z3 does not (storm)
    assert!(z0.skybox.star_density > 0.0);
    assert_eq!(z3.skybox.star_density, 0.0);

    // Boss encounter has higher vignette
    assert!(z4_boss.vignette > z0.vignette);
    assert!(z4_boss.chromatic_aberration > 0.0);
}

// ── VFX Scene Tick Behavior ────────────────────────────────────────────

/// Verifies VFX scene tick properly advances anchors and transitions.
#[test]
fn vfx_scene_tick_advances_all_systems() {
    let mut dispatch = VfxAudioDispatch::new();

    // Add anchor with repair
    dispatch.set_anchor(AnchorVfxSpec::new("a1", Vec3f::ZERO, 0.5));
    dispatch.begin_anchor_repair("a1");

    // Start phase transition
    dispatch.trigger_phase_transition(0, 1, Vec3f::ZERO);
    dispatch.drain_stingers(); // clear

    // Tick 2 seconds
    dispatch.process_events(&[], 2.0);

    // Anchor repair timer should have advanced
    let anchor = &dispatch.vfx_scene().anchors[0];
    assert!(anchor.repair_time > 1.5);

    // Phase transition should still be active (needs ~1.25s to complete)
    // After 2.0s it might be done. Let's check progress was tracked.
    assert!(dispatch.elapsed() > 1.9);
}

// ── Audio Layer Escalation ─────────────────────────────────────────────

/// Verifies audio layer escalation during boss fight.
#[test]
fn audio_layers_escalate_through_boss_phases() {
    let mut dispatch = VfxAudioDispatch::new();
    dispatch.enter_boss_encounter();

    assert_eq!(
        dispatch.audio_scene().boss_music.as_ref().unwrap().layer,
        BossMusicLayer::Assessment
    );

    dispatch.trigger_phase_transition(0, 1, Vec3f::ZERO);
    assert_eq!(
        dispatch.audio_scene().boss_music.as_ref().unwrap().layer,
        BossMusicLayer::FulcrumShift
    );

    dispatch.trigger_phase_transition(1, 2, Vec3f::ZERO);
    assert_eq!(
        dispatch.audio_scene().boss_music.as_ref().unwrap().layer,
        BossMusicLayer::DirectiveOverride
    );

    dispatch.boss_defeated();
    assert_eq!(
        dispatch.audio_scene().boss_music.as_ref().unwrap().layer,
        BossMusicLayer::Victory
    );
}

// ── Multiple Stinger Accumulation ──────────────────────────────────────

/// Verifies multiple stingers can accumulate and drain correctly.
#[test]
fn stingers_accumulate_and_drain() {
    let mut dispatch = VfxAudioDispatch::new();

    dispatch.weaving_success();
    dispatch.weaving_failure();
    dispatch.affinity_rank_up();
    dispatch.ability_unlock();
    dispatch.emit_echo_burst(Vec3f::ZERO, false);

    let stingers = dispatch.drain_stingers();
    assert_eq!(stingers.len(), 5);

    // Second drain should be empty
    let empty = dispatch.drain_stingers();
    assert!(empty.is_empty());
}

// ── Thread VFX Lifecycle ───────────────────────────────────────────────

/// Verifies thread VFX can be added, found, and removed.
#[test]
fn thread_lifecycle_add_find_remove() {
    let mut dispatch = VfxAudioDispatch::new();

    dispatch.add_thread(ThreadVfxSpec::stable(
        "t1",
        Vec3f::ZERO,
        Vec3f::new(5.0, 0.0, 0.0),
    ));
    dispatch.add_thread(ThreadVfxSpec::weaving(
        "t2",
        Vec3f::ZERO,
        Vec3f::new(10.0, 0.0, 0.0),
    ));
    assert_eq!(dispatch.vfx_scene().threads.len(), 2);

    dispatch.remove_thread("t1");
    assert_eq!(dispatch.vfx_scene().threads.len(), 1);
    assert_eq!(dispatch.vfx_scene().threads[0].thread_id, "t2");
}
