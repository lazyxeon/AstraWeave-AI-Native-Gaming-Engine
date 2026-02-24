//! Integration tests for Display impls, serde round-trips, Default impls,
//! and `capture_from_orchestrator`.

// ── Display tests ──────────────────────────────────────────────────────────

#[test]
fn display_playback_state() {
    use veilweaver_slice_runtime::cinematic_player::PlaybackState;
    assert_eq!(PlaybackState::Idle.to_string(), "Idle");
    assert_eq!(PlaybackState::Playing.to_string(), "Playing");
    assert_eq!(PlaybackState::Paused.to_string(), "Paused");
    assert_eq!(PlaybackState::Finished.to_string(), "Finished");
}

#[test]
fn display_storm_choice() {
    use veilweaver_slice_runtime::storm_choice::StormChoice;
    assert_eq!(StormChoice::Stabilize.to_string(), "Stabilize the Storm");
    assert_eq!(StormChoice::Redirect.to_string(), "Redirect the Surge");
}

#[test]
fn display_storm_phase() {
    use veilweaver_slice_runtime::storm_choice::StormPhase;
    assert_eq!(StormPhase::PreCrossroads.to_string(), "Pre-Crossroads");
    assert_eq!(StormPhase::DecisionPending.to_string(), "Decision Pending");
    assert_eq!(StormPhase::ChoiceMade.to_string(), "Choice Made");
    assert_eq!(StormPhase::Resolved.to_string(), "Resolved");
}

#[test]
fn display_vfx_color() {
    use veilweaver_slice_runtime::vfx_specs::VfxColor;
    let c = VfxColor::new(0.1, 0.2, 0.3, 1.0);
    assert_eq!(c.to_string(), "rgba(0.10, 0.20, 0.30, 1.00)");
}

#[test]
fn display_thread_state() {
    use veilweaver_slice_runtime::vfx_specs::ThreadState;
    assert_eq!(ThreadState::Stable.to_string(), "Stable");
    assert_eq!(ThreadState::Weaving.to_string(), "Weaving");
    assert_eq!(ThreadState::Stressed.to_string(), "Stressed");
    assert_eq!(ThreadState::Severed.to_string(), "Severed");
}

#[test]
fn display_anchor_vfx_state() {
    use veilweaver_slice_runtime::vfx_specs::AnchorVfxState;
    assert_eq!(AnchorVfxState::Perfect.to_string(), "Perfect");
    assert_eq!(AnchorVfxState::Stable.to_string(), "Stable");
    assert_eq!(AnchorVfxState::Unstable.to_string(), "Unstable");
    assert_eq!(AnchorVfxState::Critical.to_string(), "Critical");
    assert_eq!(AnchorVfxState::Broken.to_string(), "Broken");
}

#[test]
fn display_burst_shape() {
    use veilweaver_slice_runtime::vfx_specs::BurstShape;
    assert_eq!(BurstShape::Spherical.to_string(), "Spherical");
    assert_eq!(BurstShape::Spiral.to_string(), "Spiral");
    assert_eq!(BurstShape::Directional.to_string(), "Directional");
}

#[test]
fn display_telegraph_shape() {
    use veilweaver_slice_runtime::vfx_specs::TelegraphShape;
    assert_eq!(TelegraphShape::Circle.to_string(), "Circle");
    assert_eq!(TelegraphShape::Cone.to_string(), "Cone");
    assert_eq!(TelegraphShape::Line.to_string(), "Line");
    assert_eq!(TelegraphShape::Chain.to_string(), "Chain");
}

#[test]
fn display_storm_variant() {
    use veilweaver_slice_runtime::vfx_specs::StormVariant;
    assert_eq!(StormVariant::Stabilized.to_string(), "Stabilized");
    assert_eq!(StormVariant::Redirected.to_string(), "Redirected");
}

#[test]
fn display_walkthrough_beat_all_variants() {
    use veilweaver_slice_runtime::walkthrough::WalkthroughBeat;
    let cases = [
        (WalkthroughBeat::TutorialBegin, "Tutorial Begin"),
        (WalkthroughBeat::TutorialComplete, "Tutorial Complete"),
        (WalkthroughBeat::EchoGroveEntry, "Echo Grove Entry"),
        (
            WalkthroughBeat::FracturedCliffEntry,
            "Fractured Cliff Entry",
        ),
        (WalkthroughBeat::SkirmishActive, "Skirmish Active"),
        (WalkthroughBeat::SkirmishCleared, "Skirmish Cleared"),
        (WalkthroughBeat::SideAlcoveVisited, "Side Alcove Visited"),
        (
            WalkthroughBeat::LoomCrossroadsEntry,
            "Loom Crossroads Entry",
        ),
        (WalkthroughBeat::StormDecisionMade, "Storm Decision Made"),
        (WalkthroughBeat::BossArenaEntry, "Boss Arena Entry"),
        (WalkthroughBeat::BossActive, "Boss Active"),
        (WalkthroughBeat::BossDefeated, "Boss Defeated"),
        (WalkthroughBeat::Debrief, "Debrief"),
        (WalkthroughBeat::Complete, "Complete"),
    ];
    for (beat, expected) in &cases {
        assert_eq!(beat.to_string(), *expected, "failed for {:?}", beat);
    }
}

#[test]
fn display_walkthrough_event_variants() {
    use veilweaver_slice_runtime::walkthrough::{WalkthroughBeat, WalkthroughEvent};

    let adv = WalkthroughEvent::BeatAdvanced {
        from: WalkthroughBeat::TutorialBegin,
        to: WalkthroughBeat::TutorialComplete,
    };
    assert!(adv.to_string().contains("Tutorial Begin"));
    assert!(adv.to_string().contains("Tutorial Complete"));

    assert_eq!(
        WalkthroughEvent::TutorialComplete.to_string(),
        "Tutorial complete"
    );
    assert!(WalkthroughEvent::PlayerDamaged {
        amount: 10.0,
        new_hp: 90.0,
    }
    .to_string()
    .contains("Damaged"));
    assert!(WalkthroughEvent::EchoCollected {
        amount: 3,
        total: 15,
    }
    .to_string()
    .contains("Echo"));
}

#[test]
fn display_checkpoint_diff_identical() {
    use veilweaver_slice_runtime::checkpoint::CheckpointDiff;
    let diff = CheckpointDiff {
        is_identical: true,
        mismatch_count: 0,
        mismatches: vec![],
    };
    assert_eq!(diff.to_string(), "Checkpoints are identical");
}

#[test]
fn display_checkpoint_diff_mismatches() {
    use veilweaver_slice_runtime::checkpoint::CheckpointDiff;
    let diff = CheckpointDiff {
        is_identical: false,
        mismatch_count: 2,
        mismatches: vec!["field_a differs".into(), "field_b differs".into()],
    };
    let s = diff.to_string();
    assert!(s.contains("2 mismatch(es)"));
    assert!(s.contains("field_a differs"));
    assert!(s.contains("field_b differs"));
}

#[test]
fn display_metric_category() {
    use veilweaver_slice_runtime::recap_panel::MetricCategory;
    assert_eq!(MetricCategory::Combat.to_string(), "Combat");
    assert_eq!(MetricCategory::Exploration.to_string(), "Exploration");
    assert_eq!(MetricCategory::Weaving.to_string(), "Weaving");
    assert_eq!(MetricCategory::Companion.to_string(), "Companion");
    assert_eq!(MetricCategory::Narrative.to_string(), "Narrative");
    assert_eq!(MetricCategory::Timing.to_string(), "Timing");
}

#[test]
fn display_metric_value() {
    use veilweaver_slice_runtime::recap_panel::MetricValue;
    assert_eq!(MetricValue::Integer(42).to_string(), "42");
    assert_eq!(MetricValue::Float(3.14).to_string(), "3.1");
    assert_eq!(MetricValue::Text("hello".into()).to_string(), "hello");
}

#[test]
fn display_rating() {
    use veilweaver_slice_runtime::recap_panel::Rating;
    assert_eq!(Rating::S.to_string(), "S");
    assert_eq!(Rating::A.to_string(), "A");
    assert_eq!(Rating::B.to_string(), "B");
    assert_eq!(Rating::C.to_string(), "C");
    assert_eq!(Rating::D.to_string(), "D");
}

#[test]
fn display_stability_color() {
    use veilweaver_slice_runtime::hud_state::StabilityColor;
    assert_eq!(StabilityColor::Green.to_string(), "Green");
    assert_eq!(StabilityColor::Yellow.to_string(), "Yellow");
    assert_eq!(StabilityColor::Orange.to_string(), "Orange");
    assert_eq!(StabilityColor::Red.to_string(), "Red");
}

#[test]
fn display_decision_phase() {
    use veilweaver_slice_runtime::decision_ui::DecisionPhase;
    assert_eq!(DecisionPhase::Hidden.to_string(), "Hidden");
    assert_eq!(DecisionPhase::Opening.to_string(), "Opening");
    assert_eq!(DecisionPhase::Active.to_string(), "Active");
    assert_eq!(DecisionPhase::Closing.to_string(), "Closing");
    assert_eq!(DecisionPhase::Done.to_string(), "Done");
}

#[test]
fn display_affinity_rank() {
    use veilweaver_slice_runtime::companion_hud::AffinityRank;
    assert_eq!(AffinityRank::Wary.to_string(), "Wary");
    assert_eq!(AffinityRank::Cautious.to_string(), "Cautious");
    assert_eq!(AffinityRank::Cooperative.to_string(), "Cooperative");
    assert_eq!(AffinityRank::Bonded.to_string(), "Bonded");
    assert_eq!(AffinityRank::Synced.to_string(), "Synced");
}

#[test]
fn display_combat_event_variants() {
    use veilweaver_slice_runtime::combat::CombatEvent;

    let started = CombatEvent::EncounterStarted {
        encounter_id: "z2".into(),
        enemy_count: 5,
    };
    assert!(started.to_string().contains("z2"));
    assert!(started.to_string().contains("5 enemies"));

    let defeated = CombatEvent::EnemyDefeated {
        encounter_id: "z2".into(),
        remaining: 3,
    };
    assert!(defeated.to_string().contains("3 remaining"));

    let cleared = CombatEvent::EncounterCleared {
        encounter_id: "z2".into(),
    };
    assert!(cleared.to_string().contains("cleared"));
}

#[test]
fn display_encounter_phase() {
    use veilweaver_slice_runtime::combat::EncounterPhase;
    assert_eq!(EncounterPhase::Idle.to_string(), "Idle");
    assert_eq!(EncounterPhase::Active.to_string(), "Active");
    assert_eq!(EncounterPhase::Cleared.to_string(), "Cleared");
}

#[test]
fn display_boss_phase() {
    use veilweaver_slice_runtime::boss_hud::BossPhase;
    assert_eq!(BossPhase::Assessment.to_string(), "Assessment");
    assert_eq!(BossPhase::FulcrumShift.to_string(), "Fulcrum Shift");
    assert_eq!(
        BossPhase::DirectiveOverride.to_string(),
        "Directive Override"
    );
}

// ── Serde round-trip tests ─────────────────────────────────────────────────

#[test]
fn serde_roundtrip_playback_state() {
    use veilweaver_slice_runtime::cinematic_player::PlaybackState;
    for variant in [
        PlaybackState::Idle,
        PlaybackState::Playing,
        PlaybackState::Paused,
        PlaybackState::Finished,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: PlaybackState = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, back);
    }
}

#[test]
fn serde_roundtrip_storm_choice() {
    use veilweaver_slice_runtime::storm_choice::StormChoice;
    for variant in [StormChoice::Stabilize, StormChoice::Redirect] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: StormChoice = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, back);
    }
}

#[test]
fn serde_roundtrip_storm_phase() {
    use veilweaver_slice_runtime::storm_choice::StormPhase;
    for variant in [
        StormPhase::PreCrossroads,
        StormPhase::DecisionPending,
        StormPhase::ChoiceMade,
        StormPhase::Resolved,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: StormPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, back);
    }
}

#[test]
fn serde_roundtrip_encounter_phase() {
    use veilweaver_slice_runtime::combat::EncounterPhase;
    for variant in [
        EncounterPhase::Idle,
        EncounterPhase::Active,
        EncounterPhase::Cleared,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: EncounterPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, back);
    }
}

#[test]
fn serde_roundtrip_walkthrough_beat() {
    use veilweaver_slice_runtime::walkthrough::WalkthroughBeat;
    for variant in [
        WalkthroughBeat::TutorialBegin,
        WalkthroughBeat::TutorialComplete,
        WalkthroughBeat::EchoGroveEntry,
        WalkthroughBeat::FracturedCliffEntry,
        WalkthroughBeat::SkirmishActive,
        WalkthroughBeat::SkirmishCleared,
        WalkthroughBeat::SideAlcoveVisited,
        WalkthroughBeat::LoomCrossroadsEntry,
        WalkthroughBeat::StormDecisionMade,
        WalkthroughBeat::BossArenaEntry,
        WalkthroughBeat::BossActive,
        WalkthroughBeat::BossDefeated,
        WalkthroughBeat::Debrief,
        WalkthroughBeat::Complete,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: WalkthroughBeat = serde_json::from_str(&json).unwrap();
        assert_eq!(variant, back);
    }
}

// ── Default impl tests ────────────────────────────────────────────────────

#[test]
fn default_frame_budget_report() {
    use veilweaver_slice_runtime::perf_budget::FrameBudgetReport;
    let r = FrameBudgetReport::default();
    assert_eq!(r.total_ticks, 0);
    assert_eq!(r.sample_count, 0);
    assert_eq!(r.violation_count, 0);
    assert!(!r.within_budget);
}

#[test]
fn default_checkpoint_diff() {
    use veilweaver_slice_runtime::checkpoint::CheckpointDiff;
    let d = CheckpointDiff::default();
    assert!(!d.is_identical);
    assert_eq!(d.mismatch_count, 0);
    assert!(d.mismatches.is_empty());
}

#[test]
fn default_echo_counter() {
    use veilweaver_slice_runtime::hud_state::EchoCounter;
    let c = EchoCounter::default();
    assert_eq!(c.balance, 0);
    assert!(c.feedbacks.is_empty());
}

#[test]
fn default_vfx_color() {
    use veilweaver_slice_runtime::vfx_specs::VfxColor;
    let c = VfxColor::default();
    assert_eq!(c.to_string(), "rgba(0.00, 0.00, 0.00, 1.00)");
}

// ── capture_from_orchestrator integration test ─────────────────────────────

#[test]
fn capture_from_orchestrator_roundtrip() {
    use veilweaver_slice_runtime::checkpoint::SliceCheckpoint;
    use veilweaver_slice_runtime::walkthrough::SliceOrchestrator;

    let orch = SliceOrchestrator::default();
    let checkpoint = SliceCheckpoint::capture_from_orchestrator(&orch, "integration-test");

    // Verify header
    assert_eq!(checkpoint.header.label, "integration-test");
    assert_eq!(checkpoint.header.tick, 0);

    // Verify JSON round-trip
    let json = checkpoint.save_to_json().expect("checkpoint serialisation");
    assert!(!json.is_empty());
    let restored = SliceCheckpoint::load_from_json(&json).expect("checkpoint deserialisation");
    let diff = checkpoint.diff(&restored);
    assert!(diff.is_identical, "round-trip diff: {}", diff);
}

// ── PartialEq smoke tests ─────────────────────────────────────────────────

#[test]
fn partial_eq_checkpoint_diff() {
    use veilweaver_slice_runtime::checkpoint::CheckpointDiff;
    let a = CheckpointDiff::default();
    let b = CheckpointDiff::default();
    assert_eq!(a, b);
}

#[test]
fn partial_eq_decision_option() {
    use veilweaver_slice_runtime::decision_ui::DecisionOption;
    let a = DecisionOption::new("key", "label", "desc", "icon", (1.0, 0.0, 0.0));
    let b = DecisionOption::new("key", "label", "desc", "icon", (1.0, 0.0, 0.0));
    assert_eq!(a, b);
}

#[test]
fn partial_eq_affinity_event() {
    use veilweaver_slice_runtime::companion_hud::AffinityEvent;
    let a = AffinityEvent {
        reason: "heal".into(),
        delta: 0.1,
        time: 5.0,
    };
    let b = AffinityEvent {
        reason: "heal".into(),
        delta: 0.1,
        time: 5.0,
    };
    assert_eq!(a, b);
}

#[test]
fn partial_eq_echo_feedback() {
    use veilweaver_slice_runtime::hud_state::EchoFeedback;
    let a = EchoFeedback::new(5);
    let b = EchoFeedback::new(5);
    assert_eq!(a, b);
}

// ── New derive tests (release audit session) ───────────────────────────────

#[test]
fn serde_roundtrip_boss_phase() {
    use veilweaver_slice_runtime::boss_hud::BossPhase;
    for phase in [
        BossPhase::Assessment,
        BossPhase::FulcrumShift,
        BossPhase::DirectiveOverride,
    ] {
        let json = serde_json::to_string(&phase).unwrap();
        let deser: BossPhase = serde_json::from_str(&json).unwrap();
        assert_eq!(phase, deser);
    }
}

#[test]
fn serde_roundtrip_phase_threshold() {
    use veilweaver_slice_runtime::boss_hud::{BossPhase, PhaseThreshold};
    let pt = PhaseThreshold {
        phase: BossPhase::FulcrumShift,
        hp_start: 0.6,
        hp_end: 0.3,
    };
    let json = serde_json::to_string(&pt).unwrap();
    let deser: PhaseThreshold = serde_json::from_str(&json).unwrap();
    assert_eq!(pt, deser);
}

#[test]
fn serde_roundtrip_affinity_rank() {
    use veilweaver_slice_runtime::companion_hud::AffinityRank;
    for rank in [
        AffinityRank::Wary,
        AffinityRank::Cautious,
        AffinityRank::Cooperative,
        AffinityRank::Bonded,
        AffinityRank::Synced,
    ] {
        let json = serde_json::to_string(&rank).unwrap();
        let deser: AffinityRank = serde_json::from_str(&json).unwrap();
        assert_eq!(rank, deser);
    }
}

#[test]
fn serde_roundtrip_affinity_event() {
    use veilweaver_slice_runtime::companion_hud::AffinityEvent;
    let event = AffinityEvent {
        reason: "combo_assist".into(),
        delta: 0.15,
        time: 12.5,
    };
    let json = serde_json::to_string(&event).unwrap();
    let deser: AffinityEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, deser);
}

#[cfg(feature = "boss-director")]
#[test]
fn serde_roundtrip_encounter_state() {
    use veilweaver_slice_runtime::boss_encounter::EncounterState;
    for state in [
        EncounterState::Inactive,
        EncounterState::Active,
        EncounterState::Defeated,
    ] {
        let json = serde_json::to_string(&state).unwrap();
        let deser: EncounterState = serde_json::from_str(&json).unwrap();
        assert_eq!(state, deser);
    }
}

#[test]
fn serde_roundtrip_checkpoint_diff() {
    use veilweaver_slice_runtime::checkpoint::CheckpointDiff;
    let diff = CheckpointDiff {
        is_identical: false,
        mismatch_count: 2,
        mismatches: vec!["field A differs".into(), "field B differs".into()],
    };
    let json = serde_json::to_string(&diff).unwrap();
    let deser: CheckpointDiff = serde_json::from_str(&json).unwrap();
    assert_eq!(diff, deser);
}

#[test]
fn partial_eq_game_loop_event() {
    use veilweaver_slice_runtime::game_loop::GameLoopEvent;
    let a = GameLoopEvent::StormDecisionPrompt;
    let b = GameLoopEvent::StormDecisionPrompt;
    assert_eq!(a, b);
}

#[test]
fn partial_eq_combat_enemy() {
    use veilweaver_slice_runtime::combat::CombatEnemy;
    let a = CombatEnemy::new(1, "Wraith", 100.0, 10.0);
    let b = CombatEnemy::new(1, "Wraith", 100.0, 10.0);
    assert_eq!(a, b);
}

#[test]
fn partial_eq_player_state() {
    use veilweaver_slice_runtime::player_state::PlayerState;
    let a = PlayerState::new();
    let b = PlayerState::new();
    assert_eq!(a, b);
}

#[cfg(feature = "ai-companion")]
#[test]
fn debug_companion_ai() {
    use veilweaver_slice_runtime::companion_ai::CompanionAI;
    let ai = CompanionAI::new();
    let debug = format!("{:?}", ai);
    assert!(debug.contains("CompanionAI"));
    assert!(debug.contains("replan_interval"));
}
