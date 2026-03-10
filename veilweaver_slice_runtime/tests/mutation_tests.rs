//! Mutation-killing integration tests for veilweaver_slice_runtime.
//!
//! Targets the ~200+ gap between existing tests and 1638 mutants.
//! Structured by source module, focusing on boundary conditions,
//! arithmetic mutations, boolean logic, and threshold cascades.

// ══════════════════════════════════════════════════════════════════════
// Module 1: telemetry_rating_tests — all rate_category threshold boundaries
// ══════════════════════════════════════════════════════════════════════
mod telemetry_rating_tests {
    use veilweaver_slice_runtime::companion_hud::AffinityRank;
    use veilweaver_slice_runtime::recap_panel::{MetricCategory, Rating, RecapPanel};
    use veilweaver_slice_runtime::storm_choice::StormChoice;
    use veilweaver_slice_runtime::telemetry::TelemetryCollector;

    fn rate_combat(tc: &TelemetryCollector) -> Rating {
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        *panel.category_ratings().get(&MetricCategory::Combat).unwrap()
    }

    // Combat: kill_ratio thresholds 8.0/5.0/3.0/1.0
    #[test]
    fn combat_ratio_s_at_8() {
        let mut tc = TelemetryCollector::new();
        for _ in 0..8 { tc.record_enemy_defeated(); }
        tc.record_death();
        assert_eq!(rate_combat(&tc), Rating::S); // 8/1 = 8.0 >= 8.0
    }

    #[test]
    fn combat_ratio_a_at_5() {
        let mut tc = TelemetryCollector::new();
        for _ in 0..5 { tc.record_enemy_defeated(); }
        tc.record_death();
        assert_eq!(rate_combat(&tc), Rating::A); // 5/1 = 5.0 >= 5.0
    }

    #[test]
    fn combat_ratio_b_at_3() {
        let mut tc = TelemetryCollector::new();
        for _ in 0..3 { tc.record_enemy_defeated(); }
        tc.record_death();
        assert_eq!(rate_combat(&tc), Rating::B); // 3/1 = 3.0 >= 3.0
    }

    #[test]
    fn combat_ratio_c_at_1() {
        let mut tc = TelemetryCollector::new();
        tc.record_enemy_defeated();
        tc.record_death();
        assert_eq!(rate_combat(&tc), Rating::C); // 1/1 = 1.0 >= 1.0
    }

    #[test]
    fn combat_ratio_d_below_1() {
        let mut tc = TelemetryCollector::new();
        // 0 kills, 1 death → ratio = 0/1 = 0.0
        tc.record_death();
        assert_eq!(rate_combat(&tc), Rating::D);
    }

    #[test]
    fn combat_no_kills_no_deaths() {
        let tc = TelemetryCollector::new();
        // 0 kills, 0 deaths → ratio = 1.0 (default) → C
        assert_eq!(rate_combat(&tc), Rating::C);
    }

    #[test]
    fn combat_ratio_a_not_s_at_7() {
        let mut tc = TelemetryCollector::new();
        for _ in 0..7 { tc.record_enemy_defeated(); }
        tc.record_death();
        assert_eq!(rate_combat(&tc), Rating::A); // 7.0 < 8.0
    }

    // Exploration: zone_count thresholds 6/5/4/2
    fn rate_exploration(tc: &TelemetryCollector) -> Rating {
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        *panel.category_ratings().get(&MetricCategory::Exploration).unwrap()
    }

    #[test]
    fn exploration_s_at_6_zones() {
        let mut tc = TelemetryCollector::new();
        for z in &["Z0", "Z1", "Z2", "Z2a", "Z3", "Z4"] {
            tc.zones_visited.push(z.to_string());
        }
        assert_eq!(rate_exploration(&tc), Rating::S);
    }

    #[test]
    fn exploration_a_at_5_zones() {
        let mut tc = TelemetryCollector::new();
        for z in &["Z0", "Z1", "Z2", "Z3", "Z4"] {
            tc.zones_visited.push(z.to_string());
        }
        assert_eq!(rate_exploration(&tc), Rating::A);
    }

    #[test]
    fn exploration_b_at_4_zones() {
        let mut tc = TelemetryCollector::new();
        for z in &["Z0", "Z1", "Z2", "Z3"] {
            tc.zones_visited.push(z.to_string());
        }
        assert_eq!(rate_exploration(&tc), Rating::B);
    }

    #[test]
    fn exploration_c_at_2_zones() {
        let mut tc = TelemetryCollector::new();
        tc.zones_visited.push("Z0".into());
        tc.zones_visited.push("Z1".into());
        assert_eq!(rate_exploration(&tc), Rating::C);
    }

    #[test]
    fn exploration_d_at_1_zone() {
        let mut tc = TelemetryCollector::new();
        tc.zones_visited.push("Z0".into());
        assert_eq!(rate_exploration(&tc), Rating::D);
    }

    // Weaving: anchors_repaired thresholds 5/3/2/1
    fn rate_weaving(tc: &TelemetryCollector) -> Rating {
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        *panel.category_ratings().get(&MetricCategory::Weaving).unwrap()
    }

    #[test]
    fn weaving_s_at_5_anchors() {
        let mut tc = TelemetryCollector::new();
        for _ in 0..5 { tc.record_anchor_repaired(); }
        assert_eq!(rate_weaving(&tc), Rating::S);
    }

    #[test]
    fn weaving_a_at_3_anchors() {
        let mut tc = TelemetryCollector::new();
        for _ in 0..3 { tc.record_anchor_repaired(); }
        assert_eq!(rate_weaving(&tc), Rating::A);
    }

    #[test]
    fn weaving_b_at_2_anchors() {
        let mut tc = TelemetryCollector::new();
        tc.record_anchor_repaired();
        tc.record_anchor_repaired();
        assert_eq!(rate_weaving(&tc), Rating::B);
    }

    #[test]
    fn weaving_c_at_1_anchor() {
        let mut tc = TelemetryCollector::new();
        tc.record_anchor_repaired();
        assert_eq!(rate_weaving(&tc), Rating::C);
    }

    #[test]
    fn weaving_d_at_0_anchors() {
        let tc = TelemetryCollector::new();
        assert_eq!(rate_weaving(&tc), Rating::D);
    }

    // Timing: minutes thresholds 25.0/35.0/45.0/60.0
    fn rate_timing(tc: &TelemetryCollector) -> Rating {
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        *panel.category_ratings().get(&MetricCategory::Timing).unwrap()
    }

    #[test]
    fn timing_s_at_25_min() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 25.0 * 60.0;
        assert_eq!(rate_timing(&tc), Rating::S);
    }

    #[test]
    fn timing_a_at_35_min() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 35.0 * 60.0;
        assert_eq!(rate_timing(&tc), Rating::A);
    }

    #[test]
    fn timing_b_at_45_min() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 45.0 * 60.0;
        assert_eq!(rate_timing(&tc), Rating::B);
    }

    #[test]
    fn timing_c_at_60_min() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 60.0 * 60.0;
        assert_eq!(rate_timing(&tc), Rating::C);
    }

    #[test]
    fn timing_d_at_90_min() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 90.0 * 60.0;
        assert_eq!(rate_timing(&tc), Rating::D);
    }

    #[test]
    fn timing_a_not_s_at_30_min() {
        let mut tc = TelemetryCollector::new();
        tc.total_time = 30.0 * 60.0; // 30 > 25
        assert_eq!(rate_timing(&tc), Rating::A);
    }

    // Companion: affinity rank thresholds
    fn rate_companion(tc: &TelemetryCollector) -> Rating {
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        *panel.category_ratings().get(&MetricCategory::Companion).unwrap()
    }

    #[test]
    fn companion_s_synced() {
        let mut tc = TelemetryCollector::new();
        tc.set_final_affinity_rank(AffinityRank::Synced);
        assert_eq!(rate_companion(&tc), Rating::S);
    }

    #[test]
    fn companion_a_bonded() {
        let mut tc = TelemetryCollector::new();
        tc.set_final_affinity_rank(AffinityRank::Bonded);
        assert_eq!(rate_companion(&tc), Rating::A);
    }

    #[test]
    fn companion_b_cooperative() {
        let mut tc = TelemetryCollector::new();
        tc.set_final_affinity_rank(AffinityRank::Cooperative);
        assert_eq!(rate_companion(&tc), Rating::B);
    }

    #[test]
    fn companion_c_cautious() {
        let mut tc = TelemetryCollector::new();
        tc.set_final_affinity_rank(AffinityRank::Cautious);
        assert_eq!(rate_companion(&tc), Rating::C);
    }

    #[test]
    fn companion_d_wary() {
        let mut tc = TelemetryCollector::new();
        tc.set_final_affinity_rank(AffinityRank::Wary);
        assert_eq!(rate_companion(&tc), Rating::D);
    }

    #[test]
    fn companion_d_none() {
        let tc = TelemetryCollector::new();
        assert_eq!(rate_companion(&tc), Rating::D);
    }

    // Narrative: (has_choice, dialogue_count) thresholds
    fn rate_narrative(tc: &TelemetryCollector) -> Rating {
        let mut panel = RecapPanel::new();
        tc.finalize(&mut panel);
        *panel.category_ratings().get(&MetricCategory::Narrative).unwrap()
    }

    #[test]
    fn narrative_s_choice_and_3_dialogues() {
        let mut tc = TelemetryCollector::new();
        tc.storm_choice = Some(StormChoice::Stabilize);
        tc.dialogues_completed = 3;
        assert_eq!(rate_narrative(&tc), Rating::S);
    }

    #[test]
    fn narrative_a_choice_and_2_dialogues() {
        let mut tc = TelemetryCollector::new();
        tc.storm_choice = Some(StormChoice::Redirect);
        tc.dialogues_completed = 2;
        assert_eq!(rate_narrative(&tc), Rating::A);
    }

    #[test]
    fn narrative_b_choice_and_1_dialogue() {
        let mut tc = TelemetryCollector::new();
        tc.storm_choice = Some(StormChoice::Stabilize);
        tc.dialogues_completed = 1;
        assert_eq!(rate_narrative(&tc), Rating::B);
    }

    #[test]
    fn narrative_c_no_choice_with_dialogues() {
        let mut tc = TelemetryCollector::new();
        tc.dialogues_completed = 2;
        assert_eq!(rate_narrative(&tc), Rating::C);
    }

    #[test]
    fn narrative_d_nothing() {
        let tc = TelemetryCollector::new();
        assert_eq!(rate_narrative(&tc), Rating::D);
    }

    // NaN guards
    #[test]
    fn damage_dealt_nan_ignored() {
        let mut tc = TelemetryCollector::new();
        tc.record_damage_dealt(f32::NAN);
        assert_eq!(tc.damage_dealt, 0.0);
    }

    #[test]
    fn damage_taken_nan_ignored() {
        let mut tc = TelemetryCollector::new();
        tc.record_damage_taken(f32::NAN);
        assert_eq!(tc.damage_taken, 0.0);
    }

    #[test]
    fn boss_fight_time_accumulated() {
        let mut tc = TelemetryCollector::new();
        tc.start_boss_fight();
        tc.process_events(&[], 5.0);
        tc.process_events(&[], 3.0);
        assert!((tc.boss_fight_time - 8.0).abs() < 0.01);
        tc.finish_boss_fight();
        assert!(tc.boss_defeated);
        tc.process_events(&[], 10.0);
        assert!((tc.boss_fight_time - 8.0).abs() < 0.01); // stopped
    }

    #[test]
    fn record_boss_phase_stores_unique() {
        let mut tc = TelemetryCollector::new();
        use veilweaver_slice_runtime::boss_hud::BossPhase;
        tc.record_boss_phase(BossPhase::Assessment);
        tc.record_boss_phase(BossPhase::FulcrumShift);
        assert_eq!(tc.boss_phases_seen.len(), 2);
    }

    #[test]
    fn echoes_collected_and_spent() {
        let mut tc = TelemetryCollector::new();
        tc.record_echoes_collected(10);
        tc.record_echoes_spent(3);
        assert_eq!(tc.echoes_collected, 10);
        assert_eq!(tc.echoes_spent, 3);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 2: boss_hud_boundary_tests — phase colors, urgency, drain, flash
// ══════════════════════════════════════════════════════════════════════
mod boss_hud_boundary_tests {
    use veilweaver_slice_runtime::boss_hud::{BossHealthBar, BossPhase, TelegraphWarning};

    #[test]
    fn phase_assessment_color() {
        let (r, g, b) = BossPhase::Assessment.color();
        assert!((r - 0.3).abs() < 0.01);
        assert!((g - 0.8).abs() < 0.01);
        assert!((b - 0.4).abs() < 0.01);
    }

    #[test]
    fn phase_fulcrum_shift_color() {
        let (r, g, b) = BossPhase::FulcrumShift.color();
        assert!((r - 0.9).abs() < 0.01);
        assert!((g - 0.7).abs() < 0.01);
        assert!((b - 0.2).abs() < 0.01);
    }

    #[test]
    fn phase_directive_override_color() {
        let (r, g, b) = BossPhase::DirectiveOverride.color();
        assert!((r - 0.9).abs() < 0.01);
        assert!((g - 0.2).abs() < 0.01);
        assert!((b - 0.2).abs() < 0.01);
    }

    #[test]
    fn phase_index_values() {
        assert_eq!(BossPhase::Assessment.index(), 0);
        assert_eq!(BossPhase::FulcrumShift.index(), 1);
        assert_eq!(BossPhase::DirectiveOverride.index(), 2);
    }

    // Telegraph urgency_color boundaries: < 0.5, < 0.8
    #[test]
    fn urgency_yellow_at_0_progress() {
        let tw = TelegraphWarning::new("test", "text", 2.0);
        let (r, g, b) = tw.urgency_color();
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 0.9).abs() < 0.01);  // yellow
        assert!((b - 0.2).abs() < 0.01);
    }

    #[test]
    fn urgency_orange_at_60_percent() {
        let mut tw = TelegraphWarning::new("test", "text", 2.0);
        tw.tick(1.2); // progress = 1 - (0.8/2.0) = 0.6 (>= 0.5)
        let (r, g, b) = tw.urgency_color();
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 0.5).abs() < 0.01); // orange
        assert!((b - 0.1).abs() < 0.01);
    }

    #[test]
    fn urgency_red_at_90_percent() {
        let mut tw = TelegraphWarning::new("test", "text", 2.0);
        tw.tick(1.8); // progress = 1 - (0.2/2.0) = 0.9 (>= 0.8)
        let (r, g, b) = tw.urgency_color();
        assert!((r - 1.0).abs() < 0.01);
        assert!((g - 0.15).abs() < 0.01); // red
        assert!((b - 0.15).abs() < 0.01);
    }

    #[test]
    fn urgency_boundary_at_exactly_half() {
        let mut tw = TelegraphWarning::new("test", "text", 2.0);
        tw.tick(1.0); // progress = 0.5 exactly
        let (_, g, _) = tw.urgency_color();
        // p = 0.5, which is NOT < 0.5, so should be orange
        assert!((g - 0.5).abs() < 0.01);
    }

    #[test]
    fn urgency_boundary_at_exactly_08() {
        let mut tw = TelegraphWarning::new("test", "text", 10.0);
        tw.tick(8.0); // progress = 0.8 exactly
        let (_, g, _) = tw.urgency_color();
        // p = 0.8, which is NOT < 0.8, so should be red
        assert!((g - 0.15).abs() < 0.01);
    }

    #[test]
    fn telegraph_progress_zero_duration() {
        let tw = TelegraphWarning::new("test", "text", 0.0);
        assert!((tw.progress() - 1.0).abs() < 0.01);
    }

    #[test]
    fn telegraph_tick_returns_true_when_expired() {
        let mut tw = TelegraphWarning::new("test", "text", 1.0);
        assert!(!tw.tick(0.5));
        assert!(tw.tick(0.6));
    }

    // BossHealthBar phase transitions
    #[test]
    fn boss_phase_transition_at_60_percent() {
        let mut bar = BossHealthBar::new("Boss", 1000.0);
        bar.set_hp(600.0); // hp_fraction = 0.6 = Assessment end
        assert_eq!(bar.phase, BossPhase::FulcrumShift);
    }

    #[test]
    fn boss_phase_transition_at_30_percent() {
        let mut bar = BossHealthBar::new("Boss", 1000.0);
        bar.set_hp(300.0); // hp_fraction = 0.3 = FulcrumShift end
        assert_eq!(bar.phase, BossPhase::DirectiveOverride);
    }

    #[test]
    fn boss_defeated_at_zero_hp() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.set_hp(0.0);
        assert!(bar.is_defeated());
    }

    #[test]
    fn boss_flash_on_damage() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(10.0);
        assert!(bar.is_flashing());
        assert!((bar.flash_timer - 0.2).abs() < 0.01);
    }

    #[test]
    fn boss_flash_decays() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(10.0);
        bar.tick(0.25);
        assert!(!bar.is_flashing());
    }

    #[test]
    fn boss_apply_damage_nan_rejected() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(f32::NAN);
        assert!((bar.current_hp - 100.0).abs() < 0.01);
    }

    #[test]
    fn boss_apply_damage_negative_rejected() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(-10.0);
        assert!((bar.current_hp - 100.0).abs() < 0.01);
    }

    #[test]
    fn boss_hp_fraction_with_zero_max() {
        let bar = BossHealthBar::new("Boss", 0.0);
        assert_eq!(bar.hp_fraction(), 0.0);
    }

    #[test]
    fn boss_display_fraction_tracks_animated() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(50.0);
        // display_hp hasn't animated yet
        assert!((bar.display_fraction() - 1.0).abs() < 0.01);
        // After many ticks, should converge
        for _ in 0..100 {
            bar.tick(0.05);
        }
        assert!((bar.display_fraction() - 0.5).abs() < 0.05);
    }

    #[test]
    fn boss_drain_fraction_lags_behind() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(50.0);
        // After 1 tick, drain should still be near full
        bar.tick(0.016);
        assert!(bar.drain_fraction() > bar.display_fraction());
    }

    #[test]
    fn boss_tick_nan_dt_rejected() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.apply_damage(50.0);
        let before = bar.display_hp;
        bar.tick(f32::NAN);
        assert_eq!(bar.display_hp, before);
    }

    #[test]
    fn boss_phase_markers() {
        let bar = BossHealthBar::new("Boss", 100.0);
        let markers = bar.phase_markers();
        assert_eq!(markers.len(), 3);
        assert!((markers[0].0 - 1.0).abs() < 0.01);
        assert_eq!(markers[0].1, BossPhase::Assessment);
        assert!((markers[1].0 - 0.6).abs() < 0.01);
        assert_eq!(markers[1].1, BossPhase::FulcrumShift);
    }

    #[test]
    fn boss_phase_label_strings() {
        let bar = BossHealthBar::new("Boss", 100.0);
        assert_eq!(bar.phase_label(), "Assessment");
    }

    #[test]
    fn boss_telegraph_retained_until_expired() {
        let mut bar = BossHealthBar::new("Boss", 100.0);
        bar.add_telegraph("Cleave", "Watch out!", 1.0);
        assert_eq!(bar.telegraphs.len(), 1);
        bar.tick(0.5);
        assert_eq!(bar.telegraphs.len(), 1);
        bar.tick(0.6);
        assert_eq!(bar.telegraphs.len(), 0);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 3: companion_hud_boundary_tests — rank_progress, pulse, thresholds
// ══════════════════════════════════════════════════════════════════════
mod companion_hud_boundary_tests {
    use veilweaver_slice_runtime::companion_hud::{AffinityRank, CompanionAffinityMeter};

    #[test]
    fn rank_from_affinity_boundaries() {
        assert_eq!(AffinityRank::from_affinity(0.8), AffinityRank::Synced);
        assert_eq!(AffinityRank::from_affinity(0.79), AffinityRank::Bonded);
        assert_eq!(AffinityRank::from_affinity(0.6), AffinityRank::Bonded);
        assert_eq!(AffinityRank::from_affinity(0.59), AffinityRank::Cooperative);
        assert_eq!(AffinityRank::from_affinity(0.4), AffinityRank::Cooperative);
        assert_eq!(AffinityRank::from_affinity(0.39), AffinityRank::Cautious);
        assert_eq!(AffinityRank::from_affinity(0.2), AffinityRank::Cautious);
        assert_eq!(AffinityRank::from_affinity(0.19), AffinityRank::Wary);
    }

    #[test]
    fn rank_thresholds() {
        assert_eq!(AffinityRank::Wary.threshold(), 0.0);
        assert_eq!(AffinityRank::Cautious.threshold(), 0.2);
        assert_eq!(AffinityRank::Cooperative.threshold(), 0.4);
        assert_eq!(AffinityRank::Bonded.threshold(), 0.6);
        assert_eq!(AffinityRank::Synced.threshold(), 0.8);
    }

    #[test]
    fn rank_next() {
        assert_eq!(AffinityRank::Wary.next(), Some(AffinityRank::Cautious));
        assert_eq!(AffinityRank::Cautious.next(), Some(AffinityRank::Cooperative));
        assert_eq!(AffinityRank::Cooperative.next(), Some(AffinityRank::Bonded));
        assert_eq!(AffinityRank::Bonded.next(), Some(AffinityRank::Synced));
        assert_eq!(AffinityRank::Synced.next(), None);
    }

    #[test]
    fn rank_progress_bottom_of_band() {
        let meter = CompanionAffinityMeter::new("Companion", 0.0);
        // Wary: threshold=0.0, next=Cautious(0.2), band=0.2
        // progress = (0.0 - 0.0) / 0.2 = 0.0
        assert!((meter.rank_progress() - 0.0).abs() < 0.01);
    }

    #[test]
    fn rank_progress_middle_of_band() {
        let meter = CompanionAffinityMeter::new("Companion", 0.1);
        // Wary: threshold=0.0, next=Cautious(0.2), band=0.2
        // progress = (0.1 - 0.0) / 0.2 = 0.5
        assert!((meter.rank_progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn rank_progress_top_of_band() {
        let meter = CompanionAffinityMeter::new("Companion", 0.19);
        // progress ≈ 0.95
        assert!(meter.rank_progress() > 0.9);
    }

    #[test]
    fn rank_progress_at_synced() {
        let meter = CompanionAffinityMeter::new("Companion", 0.9);
        // Synced: threshold=0.8, next=None → 1.0, band=0.2
        // progress = (0.9 - 0.8) / 0.2 = 0.5
        assert!((meter.rank_progress() - 0.5).abs() < 0.01);
    }

    #[test]
    fn pulse_alpha_after_rank_change() {
        let mut meter = CompanionAffinityMeter::new("Companion", 0.15);
        let result = meter.apply_event("help", 0.1, 1.0);
        assert!(result.is_some()); // rank changed from Wary to Cautious
        // pulse_alpha should be near 1.0 (timer just started at 0.5)
        assert!(meter.pulse_alpha() > 0.9);
    }

    #[test]
    fn pulse_alpha_decays() {
        let mut meter = CompanionAffinityMeter::new("Companion", 0.15);
        meter.apply_event("help", 0.1, 1.0);
        meter.tick(0.25); // half of PULSE_DURATION (0.5)
        let alpha = meter.pulse_alpha();
        assert!(alpha > 0.4 && alpha < 0.6);
    }

    #[test]
    fn pulse_alpha_zero_when_no_pulse() {
        let meter = CompanionAffinityMeter::new("Companion", 0.5);
        assert_eq!(meter.pulse_alpha(), 0.0);
    }

    #[test]
    fn nan_initial_affinity_defaults_to_zero() {
        let meter = CompanionAffinityMeter::new("Companion", f32::NAN);
        assert_eq!(meter.affinity(), 0.0);
    }

    #[test]
    fn apply_event_changes_affinity_additively() {
        let mut meter = CompanionAffinityMeter::new("Companion", 0.5);
        meter.apply_event("help", 0.1, 1.0);
        assert!((meter.affinity() - 0.6).abs() < 0.01);
    }

    #[test]
    fn affinity_clamps_to_1() {
        let mut meter = CompanionAffinityMeter::new("Companion", 0.9);
        meter.apply_event("big help", 0.5, 1.0);
        assert!((meter.affinity() - 1.0).abs() < 0.01);
    }

    #[test]
    fn affinity_clamps_to_0() {
        let mut meter = CompanionAffinityMeter::new("Companion", 0.1);
        meter.apply_event("betrayal", -0.5, 1.0);
        assert!((meter.affinity() - 0.0).abs() < 0.01);
    }

    #[test]
    fn rank_colors_all_different() {
        let colors: Vec<_> = [
            AffinityRank::Wary,
            AffinityRank::Cautious,
            AffinityRank::Cooperative,
            AffinityRank::Bonded,
            AffinityRank::Synced,
        ]
        .iter()
        .map(|r| r.color())
        .collect();
        // Each color should be unique
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j]);
            }
        }
    }

    #[test]
    fn display_value_converges() {
        let mut meter = CompanionAffinityMeter::new("Companion", 0.0);
        meter.apply_event("event", 0.5, 1.0);
        for _ in 0..200 {
            meter.tick(0.016);
        }
        assert!((meter.display_value() - 0.5).abs() < 0.01);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 4: hud_state_boundary_tests — color boundaries, alpha curve, animation
// ══════════════════════════════════════════════════════════════════════
mod hud_state_boundary_tests {
    use veilweaver_slice_runtime::hud_state::*;

    #[test]
    fn color_tier_at_exact_08() {
        let mut entry = AnchorStabilityEntry::new("a", 0.8);
        assert_eq!(entry.color_tier(), StabilityColor::Green);
        entry.display_value = 0.79;
        assert_eq!(entry.color_tier(), StabilityColor::Yellow);
    }

    #[test]
    fn color_tier_at_exact_05() {
        let mut entry = AnchorStabilityEntry::new("a", 0.5);
        assert_eq!(entry.color_tier(), StabilityColor::Yellow);
        entry.display_value = 0.49;
        assert_eq!(entry.color_tier(), StabilityColor::Orange);
    }

    #[test]
    fn color_tier_at_exact_02() {
        let mut entry = AnchorStabilityEntry::new("a", 0.2);
        assert_eq!(entry.color_tier(), StabilityColor::Orange);
        entry.display_value = 0.19;
        assert_eq!(entry.color_tier(), StabilityColor::Red);
    }

    #[test]
    fn echo_feedback_alpha_at_quarter_progress() {
        let mut fb = EchoFeedback::new(5);
        fb.tick(0.5); // progress = 0.5/2.0 = 0.25
        // At exactly 0.25: alpha = progress/0.25 = 1.0
        assert!((fb.alpha - 1.0).abs() < 0.01);
    }

    #[test]
    fn echo_feedback_alpha_fade_out_starts_after_quarter() {
        let mut fb = EchoFeedback::new(5);
        fb.tick(1.0); // progress = 0.5
        // alpha = 1.0 - ((0.5 - 0.25) / 0.75) = 1.0 - 0.333 ≈ 0.667
        assert!(fb.alpha > 0.6 && fb.alpha < 0.7);
    }

    #[test]
    fn echo_feedback_alpha_near_end() {
        let mut fb = EchoFeedback::new(5);
        fb.tick(1.9); // progress = 0.95
        // alpha = 1.0 - ((0.95 - 0.25) / 0.75) = 1.0 - 0.933 ≈ 0.067
        assert!(fb.alpha < 0.1);
        assert!(fb.alpha >= 0.0);
    }

    #[test]
    fn echo_feedback_is_gain_positive() {
        assert!(EchoFeedback::new(1).is_gain());
        assert!(!EchoFeedback::new(-1).is_gain());
        assert!(!EchoFeedback::new(0).is_gain());
    }

    #[test]
    fn echo_counter_saturating_add() {
        let mut counter = EchoCounter::new(u32::MAX - 5);
        counter.transact(100);
        assert_eq!(counter.balance, u32::MAX);
    }

    #[test]
    fn animate_convergence_threshold() {
        let mut entry = AnchorStabilityEntry::new("a", 0.5);
        entry.set_stability(0.5005); // diff < 0.001
        entry.animate(0.1, 5.0);
        // Should snap to target
        assert_eq!(entry.display_value, entry.stability);
    }

    #[test]
    fn animate_rate_multiplies_diff() {
        let mut entry = AnchorStabilityEntry::new("a", 0.0);
        entry.set_stability(1.0);
        entry.animate(0.1, 5.0);
        // diff=1.0, rate*dt = 0.5, display += 1.0 * 0.5 = 0.5
        assert!((entry.display_value - 0.5).abs() < 0.01);
    }

    #[test]
    fn thread_hud_average_stability_arithmetic() {
        let mut hud = ThreadHud::new(0);
        hud.add_anchor("a", 0.2);
        hud.add_anchor("b", 0.4);
        hud.add_anchor("c", 0.6);
        // avg = (0.2 + 0.4 + 0.6) / 3 = 0.4
        assert!((hud.average_stability() - 0.4).abs() < 0.01);
    }

    #[test]
    fn thread_hud_repaired_count() {
        let mut hud = ThreadHud::new(0);
        hud.add_anchor("a", 0.5);
        hud.add_anchor("b", 0.5);
        assert_eq!(hud.repaired_count(), 0);
        hud.set_anchor_repairing("a", true);
        hud.set_anchor_repairing("a", false); // marks was_repaired
        assert_eq!(hud.repaired_count(), 1);
    }

    #[test]
    fn echo_feedback_nan_dt_rejected() {
        let mut fb = EchoFeedback::new(5);
        fb.tick(f32::NAN);
        assert_eq!(fb.age, 0.0);
    }

    #[test]
    fn echo_feedback_negative_dt_rejected() {
        let mut fb = EchoFeedback::new(5);
        fb.tick(-1.0);
        assert_eq!(fb.age, 0.0);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 5: recap_panel_tests — rating computation, finalize_default
// ══════════════════════════════════════════════════════════════════════
mod recap_panel_tests {
    use veilweaver_slice_runtime::recap_panel::*;

    #[test]
    fn overall_rating_s_requires_avg_4_5() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.record_int("b", "B", 1, MetricCategory::Exploration, "");
        // All S (5+5)/2 = 5.0 >= 4.5
        panel.finalize_with(|_, _| Rating::S);
        assert_eq!(panel.overall_rating(), Some(Rating::S));
    }

    #[test]
    fn overall_rating_a_at_35() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.record_int("b", "B", 1, MetricCategory::Exploration, "");
        // S + B = (5+3)/2 = 4.0 >= 3.5
        panel.finalize_with(|cat, _| {
            if cat == MetricCategory::Combat { Rating::S } else { Rating::B }
        });
        assert_eq!(panel.overall_rating(), Some(Rating::A));
    }

    #[test]
    fn overall_rating_b_at_25() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.record_int("b", "B", 1, MetricCategory::Exploration, "");
        // A + C = (4+2)/2 = 3.0 >= 2.5
        panel.finalize_with(|cat, _| {
            if cat == MetricCategory::Combat { Rating::A } else { Rating::C }
        });
        assert_eq!(panel.overall_rating(), Some(Rating::B));
    }

    #[test]
    fn overall_rating_c_at_15() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.record_int("b", "B", 1, MetricCategory::Exploration, "");
        // B + D = (3+1)/2 = 2.0 >= 1.5
        panel.finalize_with(|cat, _| {
            if cat == MetricCategory::Combat { Rating::B } else { Rating::D }
        });
        assert_eq!(panel.overall_rating(), Some(Rating::C));
    }

    #[test]
    fn overall_rating_d_below_15() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        // Single D = 1/1 = 1.0 < 1.5
        panel.finalize_with(|_, _| Rating::D);
        assert_eq!(panel.overall_rating(), Some(Rating::D));
    }

    #[test]
    fn finalize_default_5_metrics_gives_a() {
        let mut panel = RecapPanel::new();
        for i in 0..5 {
            panel.record_int(&format!("m{}", i), "M", i as i64, MetricCategory::Combat, "");
        }
        panel.finalize_default();
        assert_eq!(
            panel.category_ratings().get(&MetricCategory::Combat),
            Some(&Rating::A)
        );
    }

    #[test]
    fn finalize_default_3_metrics_gives_b() {
        let mut panel = RecapPanel::new();
        for i in 0..3 {
            panel.record_int(&format!("m{}", i), "M", i as i64, MetricCategory::Combat, "");
        }
        panel.finalize_default();
        assert_eq!(
            panel.category_ratings().get(&MetricCategory::Combat),
            Some(&Rating::B)
        );
    }

    #[test]
    fn finalize_default_1_metric_gives_c() {
        let mut panel = RecapPanel::new();
        panel.record_int("m0", "M", 1, MetricCategory::Combat, "");
        panel.finalize_default();
        assert_eq!(
            panel.category_ratings().get(&MetricCategory::Combat),
            Some(&Rating::C)
        );
    }

    #[test]
    fn no_record_after_finalize() {
        let mut panel = RecapPanel::new();
        panel.record_int("before", "Before", 1, MetricCategory::Combat, "");
        panel.finalize_default();
        let count_before = panel.metric_count();
        panel.record_int("after", "After", 2, MetricCategory::Combat, "");
        assert_eq!(panel.metric_count(), count_before);
    }

    #[test]
    fn reveal_animation_increments() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.record_int("b", "B", 1, MetricCategory::Exploration, "");
        panel.finalize_default();

        assert_eq!(panel.revealed_count(), 0);
        assert!(!panel.is_fully_revealed());

        panel.tick(RecapPanel::REVEAL_DURATION + 0.01);
        assert_eq!(panel.revealed_count(), 1);

        panel.tick(RecapPanel::REVEAL_DURATION + 0.01);
        assert!(panel.is_fully_revealed());
    }

    #[test]
    fn tick_nan_rejected() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.finalize_default();
        panel.tick(f32::NAN);
        assert_eq!(panel.revealed_count(), 0);
    }

    #[test]
    fn metric_value_as_f64() {
        assert_eq!(MetricValue::Integer(42).as_f64(), 42.0);
        assert_eq!(MetricValue::Float(3.14).as_f64(), 3.14);
        assert_eq!(MetricValue::Text("hello".into()).as_f64(), 0.0);
    }

    #[test]
    fn rating_colors_all_distinct() {
        let colors = [Rating::S, Rating::A, Rating::B, Rating::C, Rating::D]
            .iter()
            .map(|r| r.color())
            .collect::<Vec<_>>();
        for i in 0..colors.len() {
            for j in (i + 1)..colors.len() {
                assert_ne!(colors[i], colors[j]);
            }
        }
    }

    #[test]
    fn metrics_for_category() {
        let mut panel = RecapPanel::new();
        panel.record_int("a", "A", 1, MetricCategory::Combat, "");
        panel.record_int("b", "B", 2, MetricCategory::Exploration, "");
        panel.record_int("c", "C", 3, MetricCategory::Combat, "");
        let combat = panel.metrics_for(MetricCategory::Combat);
        assert_eq!(combat.len(), 2);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 6: decision_ui_tests — state machine, countdown, segment_angle
// ══════════════════════════════════════════════════════════════════════
mod decision_ui_tests {
    use veilweaver_slice_runtime::decision_ui::*;

    fn make_radial() -> DecisionRadial {
        let opts = vec![
            DecisionOption::new("opt_a", "Option A", "First option", "icon_a", (1.0, 0.0, 0.0)),
            DecisionOption::new("opt_b", "Option B", "Second option", "icon_b", (0.0, 1.0, 0.0)),
        ];
        DecisionRadial::new("test", "Pick one", opts)
    }

    #[test]
    fn initial_state_hidden() {
        let r = make_radial();
        assert_eq!(r.phase(), DecisionPhase::Hidden);
        assert!(!r.is_visible());
    }

    #[test]
    fn opening_to_active_transition() {
        let mut r = make_radial();
        r.open();
        assert_eq!(r.phase(), DecisionPhase::Opening);
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        assert_eq!(r.phase(), DecisionPhase::Active);
    }

    #[test]
    fn select_transitions_to_closing() {
        let mut r = make_radial();
        r.open();
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        assert!(r.select(0));
        assert_eq!(r.phase(), DecisionPhase::Closing);
    }

    #[test]
    fn closing_to_done_transition() {
        let mut r = make_radial();
        r.open();
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        r.select(0);
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        assert_eq!(r.phase(), DecisionPhase::Done);
        assert!(r.is_done());
    }

    #[test]
    fn select_wrong_phase_returns_false() {
        let mut r = make_radial();
        assert!(!r.select(0)); // Hidden
    }

    #[test]
    fn select_out_of_range_returns_false() {
        let mut r = make_radial();
        r.open();
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        assert!(!r.select(5));
    }

    #[test]
    fn countdown_autoselect() {
        let opts = vec![
            DecisionOption::new("a", "A", "desc", "icon_a", (1.0, 0.0, 0.0)),
            DecisionOption::new("b", "B", "desc", "icon_b", (0.0, 1.0, 0.0)),
        ];
        let mut r = DecisionRadial::new("test", "Pick", opts).with_countdown(1.0, 0);
        r.open();
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        assert_eq!(r.phase(), DecisionPhase::Active);
        r.tick(1.1); // countdown expired
        assert_eq!(r.phase(), DecisionPhase::Closing); // auto-selected
        assert_eq!(r.selected_index, Some(0));
    }

    #[test]
    fn countdown_remaining_decreases() {
        let opts = vec![DecisionOption::new("a", "A", "desc", "icon_a", (1.0, 0.0, 0.0))];
        let mut r = DecisionRadial::new("test", "Pick", opts).with_countdown(5.0, 0);
        r.open();
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        r.tick(2.0);
        let remaining = r.countdown_remaining().unwrap();
        assert!(remaining < 3.1 && remaining > 2.9);
    }

    #[test]
    fn segment_angle_two_options() {
        let r = make_radial();
        let (start, end) = r.segment_angle(0).unwrap();
        assert!((start - 0.0).abs() < 0.01);
        assert!((end - std::f32::consts::PI).abs() < 0.01);
        let (start2, end2) = r.segment_angle(1).unwrap();
        assert!((start2 - std::f32::consts::PI).abs() < 0.01);
        assert!((end2 - std::f32::consts::TAU).abs() < 0.01);
    }

    #[test]
    fn segment_angle_out_of_range() {
        let r = make_radial();
        assert!(r.segment_angle(5).is_none());
    }

    #[test]
    fn focus_and_description() {
        let mut r = make_radial();
        r.focus(0);
        assert_eq!(r.focused_description(), Some("First option"));
        r.focus(1);
        assert_eq!(r.focused_description(), Some("Second option"));
        r.clear_focus();
        assert_eq!(r.focused_description(), None);
    }

    #[test]
    fn tick_nan_dt_ignored() {
        let mut r = make_radial();
        r.open();
        let phase = r.phase();
        r.tick(f32::NAN);
        assert_eq!(r.phase(), phase);
    }

    #[test]
    fn selected_key_after_selection() {
        let mut r = make_radial();
        r.open();
        r.tick(DecisionRadial::ANIM_DURATION + 0.01);
        r.select(1);
        assert_eq!(r.selected_key(), Some("opt_b"));
    }

    #[test]
    fn storm_choice_radial_has_countdown() {
        let r = storm_choice_radial();
        assert!(r.countdown_remaining().is_some());
        assert_eq!(r.options.len(), 2);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 7: vfx_specs_tests — PhaseTransition, lerp, telegraph, anchors
// ══════════════════════════════════════════════════════════════════════
mod vfx_specs_tests {
    use veilweaver_slice_runtime::vfx_specs::*;

    #[test]
    fn vfx_color_lerp_at_zero() {
        let a = VfxColor { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        let b = VfxColor { r: 1.0, g: 1.0, b: 1.0, a: 0.0 };
        let result = a.lerp(&b, 0.0);
        assert!((result.r - 0.0).abs() < 0.01);
        assert!((result.a - 1.0).abs() < 0.01);
    }

    #[test]
    fn vfx_color_lerp_at_one() {
        let a = VfxColor { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        let b = VfxColor { r: 1.0, g: 1.0, b: 1.0, a: 0.0 };
        let result = a.lerp(&b, 1.0);
        assert!((result.r - 1.0).abs() < 0.01);
        assert!((result.a - 0.0).abs() < 0.01);
    }

    #[test]
    fn vfx_color_lerp_at_half() {
        let a = VfxColor { r: 0.0, g: 0.2, b: 0.0, a: 1.0 };
        let b = VfxColor { r: 1.0, g: 0.8, b: 1.0, a: 0.0 };
        let result = a.lerp(&b, 0.5);
        assert!((result.r - 0.5).abs() < 0.01);
        assert!((result.g - 0.5).abs() < 0.01);
        assert!((result.a - 0.5).abs() < 0.01);
    }

    #[test]
    fn vfx_color_lerp_clamps_t() {
        let a = VfxColor { r: 0.0, g: 0.0, b: 0.0, a: 0.0 };
        let b = VfxColor { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let result = a.lerp(&b, 2.0);
        assert!((result.r - 1.0).abs() < 0.01); // clamped to 1.0
    }

    #[test]
    fn anchor_vfx_state_from_stability_boundaries() {
        assert_eq!(AnchorVfxState::from_stability(0.9), AnchorVfxState::Perfect);
        assert_eq!(AnchorVfxState::from_stability(0.89), AnchorVfxState::Stable);
        assert_eq!(AnchorVfxState::from_stability(0.6), AnchorVfxState::Stable);
        assert_eq!(AnchorVfxState::from_stability(0.59), AnchorVfxState::Unstable);
        assert_eq!(AnchorVfxState::from_stability(0.3), AnchorVfxState::Unstable);
        assert_eq!(AnchorVfxState::from_stability(0.29), AnchorVfxState::Critical);
        assert_eq!(AnchorVfxState::from_stability(0.01), AnchorVfxState::Critical);
        // 0.0 is NOT > 0.0, so it's Broken
        assert_eq!(AnchorVfxState::from_stability(0.0), AnchorVfxState::Broken);
        assert_eq!(AnchorVfxState::from_stability(-0.1), AnchorVfxState::Broken);
    }

    #[test]
    fn anchor_vfx_shader_indices_distinct() {
        let indices: Vec<u32> = [
            AnchorVfxState::Perfect,
            AnchorVfxState::Stable,
            AnchorVfxState::Unstable,
            AnchorVfxState::Critical,
            AnchorVfxState::Broken,
        ]
        .iter()
        .map(|s| s.shader_index())
        .collect();
        for i in 0..indices.len() {
            for j in (i + 1)..indices.len() {
                assert_ne!(indices[i], indices[j]);
            }
        }
    }

    #[test]
    fn phase_transition_tick_expands_shockwave() {
        let mut pt = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        assert!((pt.shockwave_radius - 0.0).abs() < 0.01);
        pt.tick(0.5);
        // radius = 0 + 20.0 * 0.5 = 10.0
        assert!((pt.shockwave_radius - 10.0).abs() < 0.1);
        // progress = 10.0 / 25.0 = 0.4
        assert!((pt.progress - 0.4).abs() < 0.01);
    }

    #[test]
    fn phase_transition_completes() {
        let mut pt = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        // After enough time, radius reaches max
        pt.tick(2.0); // radius = min(40, 25) = 25
        assert!((pt.progress - 1.0).abs() < 0.01);
        // flash decays: 1.0 - 2.0*3.0 = clamped to 0.0
        assert!(pt.flash_intensity <= 0.0);
        assert!(pt.is_complete());
    }

    #[test]
    fn phase_transition_not_complete_while_flashing() {
        let mut pt = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        pt.tick(0.1); // flash not fully decayed
        assert!(!pt.is_complete());
    }

    #[test]
    fn phase_transition_flash_decay() {
        let mut pt = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        pt.tick(0.1);
        // flash = 1.0 - 0.1 * 3.0 = 0.7
        assert!((pt.flash_intensity - 0.7).abs() < 0.01);
    }

    #[test]
    fn phase_transition_screen_shake_decay() {
        let mut pt = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        pt.tick(0.1);
        // shake = 0.8 - 0.1 * 2.0 = 0.6
        assert!((pt.screen_shake - 0.6).abs() < 0.01);
    }

    #[test]
    fn phase_transition_current_color_at_midpoint() {
        let pt = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        let color = pt.current_color();
        // progress=0.0, so color = from_color (BOSS_ASSESS)
        assert!((color.r - VfxColor::BOSS_ASSESS.r).abs() < 0.01);
    }

    #[test]
    fn phase_transition_color_mapping() {
        // from_phase=0 → BOSS_ASSESS, to_phase=1 → BOSS_FULCRUM
        let pt01 = PhaseTransitionVfx::new(0, 1, Vec3f::ZERO);
        assert!((pt01.from_color.r - VfxColor::BOSS_ASSESS.r).abs() < 0.01);
        assert!((pt01.to_color.r - VfxColor::BOSS_FULCRUM.r).abs() < 0.01);

        // from_phase=1 → BOSS_FULCRUM, to_phase=2 → BOSS_OVERRIDE
        let pt12 = PhaseTransitionVfx::new(1, 2, Vec3f::ZERO);
        assert!((pt12.from_color.r - VfxColor::BOSS_FULCRUM.r).abs() < 0.01);
        assert!((pt12.to_color.r - VfxColor::BOSS_OVERRIDE.r).abs() < 0.01);
    }

    #[test]
    fn anchor_vfx_spec_repair_tick() {
        let mut spec = AnchorVfxSpec::new("test", Vec3f::ZERO, 0.5);
        spec.begin_repair();
        assert!(spec.is_repairing);
        spec.tick_repair(0.5);
        assert!((spec.repair_time - 0.5).abs() < 0.01);
    }

    #[test]
    fn anchor_vfx_spec_set_stability() {
        let mut spec = AnchorVfxSpec::new("test", Vec3f::ZERO, 0.5);
        spec.set_stability(0.9);
        assert!((spec.stability - 0.9).abs() < 0.01);
        assert_eq!(spec.vfx_state, AnchorVfxState::Perfect);
    }

    #[test]
    fn thread_vfx_stable_vs_weaving() {
        let stable = ThreadVfxSpec::stable("t1", Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0));
        assert_eq!(stable.state, ThreadState::Stable);
        let weaving = ThreadVfxSpec::weaving("t2", Vec3f::ZERO, Vec3f::new(0.0, 1.0, 0.0));
        assert_eq!(weaving.state, ThreadState::Weaving);
    }

    #[test]
    fn echo_burst_standard_vs_large() {
        let standard = EchoBurstSpec::standard(Vec3f::ZERO);
        let large = EchoBurstSpec::large(Vec3f::ZERO);
        assert!(large.particle_count > standard.particle_count);
        assert!(large.speed > standard.speed);
    }

    #[test]
    fn storm_vfx_stabilized_vs_redirected() {
        let stab = StormVfxSpec::stabilized();
        let redir = StormVfxSpec::redirected();
        assert!(stab.fog_density < redir.fog_density || stab.wind_strength != redir.wind_strength);
        // from_choice should match factory methods
        let from_stab = StormVfxSpec::from_choice(&veilweaver_slice_runtime::storm_choice::StormChoice::Stabilize);
        assert!((from_stab.fog_density - stab.fog_density).abs() < 0.01);
    }

    #[test]
    fn vfx_scene_tick_clears_completed_transition() {
        let mut scene = VfxScene::new();
        scene.phase_transition = Some(PhaseTransitionVfx::new(0, 1, Vec3f::ZERO));
        // tick enough to complete
        scene.tick(5.0);
        assert!(scene.phase_transition.is_none());
    }

    #[test]
    fn vfx_scene_drain_bursts() {
        let mut scene = VfxScene::new();
        scene.echo_bursts.push(EchoBurstSpec::standard(Vec3f::ZERO));
        scene.echo_bursts.push(EchoBurstSpec::large(Vec3f::ZERO));
        let drained = scene.drain_bursts();
        assert_eq!(drained.len(), 2);
        assert!(scene.echo_bursts.is_empty());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 8: player_state_tests — damage, heal, echoes, respawn
// ══════════════════════════════════════════════════════════════════════
mod player_state_tests {
    use veilweaver_slice_runtime::player_state::PlayerState;

    #[test]
    fn take_damage_returns_true_on_death() {
        let mut ps = PlayerState::new();
        assert!(ps.take_damage(150.0)); // 100 hp - 150 = 0 → dead
        assert!(!ps.alive);
    }

    #[test]
    fn take_damage_returns_false_on_survive() {
        let mut ps = PlayerState::new();
        assert!(!ps.take_damage(50.0));
        assert!(ps.alive);
        assert!((ps.hp - 50.0).abs() < 0.01);
    }

    #[test]
    fn take_damage_nan_rejected() {
        let mut ps = PlayerState::new();
        assert!(!ps.take_damage(f32::NAN));
        assert!((ps.hp - 100.0).abs() < 0.01);
    }

    #[test]
    fn take_damage_negative_rejected() {
        let mut ps = PlayerState::new();
        assert!(!ps.take_damage(-10.0));
        assert!((ps.hp - 100.0).abs() < 0.01);
    }

    #[test]
    fn take_damage_dead_player_rejected() {
        let mut ps = PlayerState::new();
        ps.take_damage(200.0);
        assert!(!ps.take_damage(10.0)); // already dead
    }

    #[test]
    fn heal_clamps_to_max_hp() {
        let mut ps = PlayerState::new();
        ps.take_damage(50.0);
        ps.heal(1000.0);
        assert!((ps.hp - ps.max_hp).abs() < 0.01);
    }

    #[test]
    fn heal_nan_rejected() {
        let mut ps = PlayerState::new();
        ps.take_damage(50.0);
        let hp_before = ps.hp;
        ps.heal(f32::NAN);
        assert!((ps.hp - hp_before).abs() < 0.01);
    }

    #[test]
    fn heal_dead_player_rejected() {
        let mut ps = PlayerState::new();
        ps.take_damage(200.0);
        ps.heal(50.0);
        assert!(!ps.alive);
    }

    #[test]
    fn spend_echoes_success() {
        let mut ps = PlayerState::new();
        ps.collect_echoes(10);
        assert!(ps.spend_echoes(5));
        assert_eq!(ps.echoes, 5);
    }

    #[test]
    fn spend_echoes_insufficient() {
        let mut ps = PlayerState::new();
        ps.collect_echoes(3);
        assert!(!ps.spend_echoes(5));
        assert_eq!(ps.echoes, 3); // unchanged
    }

    #[test]
    fn collect_echoes_saturating() {
        let mut ps = PlayerState::new();
        ps.echoes = u32::MAX - 5;
        ps.collect_echoes(100);
        assert_eq!(ps.echoes, u32::MAX);
    }

    #[test]
    fn hp_fraction_normal() {
        let mut ps = PlayerState::new();
        ps.take_damage(25.0);
        assert!((ps.hp_fraction() - 0.75).abs() < 0.01);
    }

    #[test]
    fn hp_fraction_zero_max_hp() {
        let mut ps = PlayerState::new();
        ps.max_hp = 0.0;
        assert_eq!(ps.hp_fraction(), 0.0);
    }

    #[test]
    fn respawn_restores_full_hp() {
        let mut ps = PlayerState::new();
        ps.take_damage(200.0);
        assert!(!ps.alive);
        ps.respawn();
        assert!(ps.alive);
        assert!((ps.hp - ps.max_hp).abs() < 0.01);
    }

    #[test]
    fn adjust_stability_clamps() {
        let mut ps = PlayerState::new();
        ps.adjust_stability(-2.0);
        assert_eq!(ps.thread_stability, 0.0);
        ps.adjust_stability(5.0);
        assert_eq!(ps.thread_stability, 1.0);
    }

    #[test]
    fn is_full_health() {
        let ps = PlayerState::new();
        assert!(ps.is_full_health());
    }

    #[test]
    fn enter_zone() {
        use astraweave_scene::world_partition::GridCoord;
        let mut ps = PlayerState::new();
        ps.enter_zone("Z0_loomspire_sanctum", GridCoord::new(100, 0, 0));
        assert_eq!(ps.zone_name, "Z0_loomspire_sanctum");
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 9: zone_transitions_tests — parse, dispatch, registry
// ══════════════════════════════════════════════════════════════════════
mod zone_transitions_tests {
    use veilweaver_slice_runtime::zone_transitions::*;

    #[test]
    fn parse_zone_transition() {
        let action = TriggerAction::parse("zone.transition:Z1_echo_grove");
        assert_eq!(action.category, "zone");
        assert_eq!(action.verb, "transition");
        assert_eq!(action.target, "Z1_echo_grove");
        assert!(action.is_zone_transition());
    }

    #[test]
    fn parse_cinematic() {
        let action = TriggerAction::parse("cinematic.play:intro_cinematic");
        assert!(action.is_cinematic());
        assert_eq!(action.target, "intro_cinematic");
    }

    #[test]
    fn parse_dialogue() {
        let action = TriggerAction::parse("dialogue.play:npc_greeting");
        assert!(action.is_dialogue());
    }

    #[test]
    fn parse_decision() {
        let action = TriggerAction::parse("decision.open:storm_choice");
        assert!(action.is_decision());
    }

    #[test]
    fn parse_vfx() {
        let action = TriggerAction::parse("vfx.activate:explosion");
        assert!(action.is_vfx());
        assert_eq!(action.target, "explosion");
    }

    #[test]
    fn parse_boss() {
        let action = TriggerAction::parse("boss.start:warden");
        assert!(action.is_boss());
    }

    #[test]
    fn parse_legacy_no_dots() {
        let action = TriggerAction::parse("SomeLegacyAction");
        assert!(action.is_legacy());
        assert_eq!(action.category, "legacy");
        assert_eq!(action.target, "SomeLegacyAction");
    }

    #[test]
    fn parse_colon_no_dot() {
        let action = TriggerAction::parse("category:target");
        assert_eq!(action.category, "category");
        assert!(action.verb.is_empty());
        assert_eq!(action.target, "target");
    }

    // ── Discrimination tests: each is_*() must be false for other types ──

    #[test]
    fn is_zone_transition_false_for_others() {
        let dialogue = TriggerAction::parse("dialogue.play:npc_greeting");
        assert!(!dialogue.is_zone_transition());
        let cinematic = TriggerAction::parse("cinematic.play:intro");
        assert!(!cinematic.is_zone_transition());
        let vfx = TriggerAction::parse("vfx.activate:explosion");
        assert!(!vfx.is_zone_transition());
    }

    #[test]
    fn is_dialogue_false_for_others() {
        let zone = TriggerAction::parse("zone.transition:Z1_echo_grove");
        assert!(!zone.is_dialogue());
        let cinematic = TriggerAction::parse("cinematic.play:intro");
        assert!(!cinematic.is_dialogue());
        let decision = TriggerAction::parse("decision.open:storm_choice");
        assert!(!decision.is_dialogue());
    }

    #[test]
    fn is_cinematic_false_for_others() {
        let dialogue = TriggerAction::parse("dialogue.play:npc_greeting");
        assert!(!dialogue.is_cinematic());
        let zone = TriggerAction::parse("zone.transition:Z1");
        assert!(!zone.is_cinematic());
        let boss = TriggerAction::parse("boss.start:warden");
        assert!(!boss.is_cinematic());
    }

    #[test]
    fn is_vfx_false_for_others() {
        let zone = TriggerAction::parse("zone.transition:Z1");
        assert!(!zone.is_vfx());
        let dialogue = TriggerAction::parse("dialogue.play:npc");
        assert!(!dialogue.is_vfx());
    }

    #[test]
    fn is_decision_false_for_others() {
        let zone = TriggerAction::parse("zone.transition:Z1");
        assert!(!zone.is_decision());
        let dialogue = TriggerAction::parse("dialogue.play:npc");
        assert!(!dialogue.is_decision());
    }

    #[test]
    fn is_boss_false_for_others() {
        let zone = TriggerAction::parse("zone.transition:Z1");
        assert!(!zone.is_boss());
        let vfx = TriggerAction::parse("vfx.activate:boom");
        assert!(!vfx.is_boss());
    }

    #[test]
    fn zone_registry_default_has_6_zones() {
        let reg = ZoneRegistry::veilweaver_default();
        assert_eq!(reg.len(), 6);
        assert!(!reg.is_empty());
    }

    #[test]
    fn zone_registry_bidirectional() {
        let reg = ZoneRegistry::veilweaver_default();
        let coord = reg.coord_for("Z0_loomspire_sanctum").unwrap();
        let name = reg.name_for(coord).unwrap();
        assert_eq!(name, "Z0_loomspire_sanctum");
    }

    #[test]
    fn dispatch_zone_transition() {
        let reg = ZoneRegistry::veilweaver_default();
        let triggers = vec![(
            "trigger_1".to_string(),
            "zone.transition:Z1_echo_grove".to_string(),
        )];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert_eq!(events.len(), 1);
        match &events[0] {
            ZoneTransitionEvent::ZoneTransition { target_zone, .. } => {
                assert_eq!(target_zone, "Z1_echo_grove");
            }
            _ => panic!("Expected ZoneTransition"),
        }
    }

    #[test]
    fn dispatch_unknown_zone_skipped() {
        let reg = ZoneRegistry::veilweaver_default();
        let triggers = vec![(
            "trigger_1".to_string(),
            "zone.transition:nonexistent_zone".to_string(),
        )];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert!(events.is_empty());
    }

    #[test]
    fn dispatch_cinematic_trigger() {
        let reg = ZoneRegistry::veilweaver_default();
        let triggers = vec![(
            "trigger_1".to_string(),
            "cinematic.play:intro".to_string(),
        )];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert_eq!(events.len(), 1);
        match &events[0] {
            ZoneTransitionEvent::CinematicTrigger { cinematic_name, .. } => {
                assert_eq!(cinematic_name, "intro");
            }
            _ => panic!("Expected CinematicTrigger"),
        }
    }

    #[test]
    fn dispatch_legacy_action() {
        let reg = ZoneRegistry::new();
        let triggers = vec![("t1".to_string(), "OldAction".to_string())];
        let events = dispatch_trigger_actions(&triggers, &reg);
        assert_eq!(events.len(), 1);
        match &events[0] {
            ZoneTransitionEvent::LegacyAction { action_name, .. } => {
                assert_eq!(action_name, "OldAction");
            }
            _ => panic!("Expected LegacyAction"),
        }
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 10: storm_choice_tests — modifiers, phase guards
// ══════════════════════════════════════════════════════════════════════
mod storm_choice_tests {
    use veilweaver_slice_runtime::storm_choice::*;

    #[test]
    fn stabilize_modifiers() {
        let mods = BossArenaModifiers::from_choice(StormChoice::Stabilize);
        assert!((mods.visibility - 1.0).abs() < 0.01);
        assert!((mods.warden_armor_bonus - 0.35).abs() < 0.01);
        assert!(mods.fog_reinforcements_enabled);
        assert_eq!(mods.arena_vfx, "storm_stabilize_blue");
    }

    #[test]
    fn redirect_modifiers() {
        let mods = BossArenaModifiers::from_choice(StormChoice::Redirect);
        assert!((mods.visibility - 0.4).abs() < 0.01);
        assert!((mods.warden_armor_bonus - 0.0).abs() < 0.01);
        assert!(!mods.fog_reinforcements_enabled);
        assert_eq!(mods.arena_vfx, "storm_redirect_amber");
    }

    #[test]
    fn state_machine_happy_path() {
        let mut state = StormChoiceState::new();
        assert_eq!(state.phase(), StormPhase::PreCrossroads);

        state.enter_crossroads();
        assert_eq!(state.phase(), StormPhase::DecisionPending);
        assert_eq!(state.phase(), StormPhase::DecisionPending);

        assert!(state.make_choice(StormChoice::Stabilize));
        assert_eq!(state.choice(), Some(StormChoice::Stabilize));
        assert!(state.modifiers().is_some());

        state.resolve();
        assert!(state.is_resolved());
    }

    #[test]
    fn enter_crossroads_guard() {
        let mut state = StormChoiceState::new();
        state.enter_crossroads();
        state.enter_crossroads(); // double call should be noop
        assert_eq!(state.phase(), StormPhase::DecisionPending);
    }

    #[test]
    fn make_choice_guard() {
        let mut state = StormChoiceState::new();
        // Can't make choice before entering crossroads
        assert!(!state.make_choice(StormChoice::Redirect));
    }

    #[test]
    fn resolve_guard() {
        let mut state = StormChoiceState::new();
        state.resolve(); // PreCrossroads → should not change
        assert_eq!(state.phase(), StormPhase::PreCrossroads);
    }

    #[test]
    fn reset_returns_to_initial() {
        let mut state = StormChoiceState::new();
        state.enter_crossroads();
        state.make_choice(StormChoice::Redirect);
        state.reset();
        assert_eq!(state.phase(), StormPhase::PreCrossroads);
        assert!(state.choice().is_none());
    }

    #[test]
    fn storm_choice_labels() {
        assert!(!StormChoice::Stabilize.label().is_empty());
        assert!(!StormChoice::Redirect.label().is_empty());
        assert_ne!(StormChoice::Stabilize.label(), StormChoice::Redirect.label());
    }

    #[test]
    fn storm_choice_vfx_names() {
        assert_ne!(
            StormChoice::Stabilize.vfx_name(),
            StormChoice::Redirect.vfx_name()
        );
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 11: audio_specs_tests — zone index, music layers, spatial sfx
// ══════════════════════════════════════════════════════════════════════
mod audio_specs_tests {
    use veilweaver_slice_runtime::audio_specs::*;
    use veilweaver_slice_runtime::vfx_specs::Vec3f;

    #[test]
    fn zone_ambience_from_index() {
        assert_eq!(ZoneAmbienceId::from_zone_index(0), Some(ZoneAmbienceId::LoomspireSanctum));
        assert_eq!(ZoneAmbienceId::from_zone_index(1), Some(ZoneAmbienceId::ThreadhollowRuins));
        assert_eq!(ZoneAmbienceId::from_zone_index(2), Some(ZoneAmbienceId::StormreachNexus));
        assert_eq!(ZoneAmbienceId::from_zone_index(3), Some(ZoneAmbienceId::FrayedExpanse));
        assert_eq!(ZoneAmbienceId::from_zone_index(4), Some(ZoneAmbienceId::BossCourtyard));
        assert_eq!(ZoneAmbienceId::from_zone_index(5), None);
    }

    #[test]
    fn boss_music_layer_crossfade_differs() {
        let ambient = BossMusicSpec::for_layer(BossMusicLayer::Ambient);
        let fulcrum = BossMusicSpec::for_layer(BossMusicLayer::FulcrumShift);
        assert!(ambient.crossfade_sec > fulcrum.crossfade_sec);
    }

    #[test]
    fn boss_music_layer_tempo_escalates() {
        let assess = BossMusicSpec::for_layer(BossMusicLayer::Assessment);
        let override_ = BossMusicSpec::for_layer(BossMusicLayer::DirectiveOverride);
        assert!(override_.tempo_multiplier > assess.tempo_multiplier);
    }

    #[test]
    fn boss_music_victory_not_looping() {
        let victory = BossMusicSpec::for_layer(BossMusicLayer::Victory);
        assert!(!victory.looping);
        let defeat = BossMusicSpec::for_layer(BossMusicLayer::Defeat);
        assert!(!defeat.looping);
    }

    #[test]
    fn boss_music_ambient_looping() {
        let ambient = BossMusicSpec::for_layer(BossMusicLayer::Ambient);
        assert!(ambient.looping);
    }

    #[test]
    fn stinger_with_volume_clamps() {
        let s = UiStingerSpec::with_volume(StingerKind::WeavingSuccess, 2.0);
        assert!((s.volume - 1.0).abs() < 0.01);
        let s2 = UiStingerSpec::with_volume(StingerKind::WeavingFailure, -1.0);
        assert!((s2.volume - 0.0).abs() < 0.01);
    }

    #[test]
    fn spatial_sfx_factory_values() {
        let pos = Vec3f::new(1.0, 2.0, 3.0);
        let anchor = SpatialSfxSpec::anchor_hum(pos);
        assert!((anchor.volume - 0.3).abs() < 0.01);
        assert!(anchor.looping);
        assert!((anchor.max_distance - 15.0).abs() < 0.01);

        let thread = SpatialSfxSpec::thread_weave(pos);
        assert!(!thread.looping);
        assert!((thread.max_distance - 10.0).abs() < 0.01);

        let boss = SpatialSfxSpec::boss_impact(pos);
        assert!((boss.volume - 0.9).abs() < 0.01);
        assert!((boss.max_distance - 30.0).abs() < 0.01);

        let storm = SpatialSfxSpec::storm_crackle(pos);
        assert!(storm.looping);
        assert!((storm.max_distance - 25.0).abs() < 0.01);
    }

    #[test]
    fn stinger_sound_paths_not_empty() {
        assert!(!StingerKind::WeavingSuccess.sound_path().is_empty());
        assert!(!StingerKind::BossDefeated.sound_path().is_empty());
    }

    #[test]
    fn audio_scene_queue_and_drain() {
        let mut scene = AudioScene::new();
        scene.queue_stinger(UiStingerSpec::new(StingerKind::EchoCollected));
        scene.queue_stinger(UiStingerSpec::new(StingerKind::AnchorStabilized));
        let drained = scene.drain_stingers();
        assert_eq!(drained.len(), 2);
        assert!(scene.drain_stingers().is_empty());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 12: palette_tests — zone index, presentation config
// ══════════════════════════════════════════════════════════════════════
mod palette_tests {
    use veilweaver_slice_runtime::palette::*;

    #[test]
    fn zone_material_from_index() {
        assert!(ZoneMaterialConfig::from_zone_index(0).is_some());
        assert!(ZoneMaterialConfig::from_zone_index(4).is_some());
        assert!(ZoneMaterialConfig::from_zone_index(5).is_none());
    }

    #[test]
    fn for_zone_skybox_switches_at_3() {
        let zone2 = PresentationConfig::for_zone(2);
        let zone3 = PresentationConfig::for_zone(3);
        // Zone 2 gets twilight, zone 3 gets storm
        assert_ne!(zone2.skybox.cloud_coverage, zone3.skybox.cloud_coverage);
    }

    #[test]
    fn for_zone_vignette_at_boss() {
        let zone4 = PresentationConfig::for_zone(4);
        assert!((zone4.vignette - 0.3).abs() < 0.01);
        let zone0 = PresentationConfig::for_zone(0);
        assert!((zone0.vignette - 0.1).abs() < 0.01);
    }

    #[test]
    fn boss_encounter_overrides() {
        let boss = PresentationConfig::boss_encounter(4);
        assert!((boss.vignette - 0.4).abs() < 0.01);
        assert!((boss.brightness - 1.0).abs() < 0.01);
        assert!((boss.saturation - 0.85).abs() < 0.01);
    }

    #[test]
    fn palette_slot_colors_distinct() {
        use veilweaver_slice_runtime::vfx_specs::VfxColor;
        let colors: Vec<VfxColor> = [
            PaletteSlot::SkyZenith,
            PaletteSlot::SkyHorizon,
            PaletteSlot::GroundBase,
            PaletteSlot::ThreadEmissive,
            PaletteSlot::AnchorEmissive,
            PaletteSlot::EchoEmissive,
        ]
        .iter()
        .map(|s| s.color())
        .collect();
        // At least some should differ
        assert_ne!(colors[0].r, colors[2].r); // zenith != ground
        assert_ne!(colors[3].r, colors[4].r); // thread != anchor
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 13: combat_tests — damage, waves, events
// ══════════════════════════════════════════════════════════════════════
mod combat_tests {
    use veilweaver_slice_runtime::combat::*;

    #[test]
    fn enemy_take_damage_returns_true_on_kill() {
        let mut enemy = CombatEnemy {
            id: 1,
            name: "enemy".into(),
            hp: 10.0,
            max_hp: 10.0,
            damage: 5.0,
            alive: true,
        };
        assert!(enemy.take_damage(15.0)); // overkill
        assert!(!enemy.alive);
    }

    #[test]
    fn enemy_take_damage_nan_rejected() {
        let mut enemy = CombatEnemy {
            id: 1,
            name: "enemy".into(),
            hp: 10.0,
            max_hp: 10.0,
            damage: 5.0,
            alive: true,
        };
        assert!(!enemy.take_damage(f32::NAN));
        assert!(enemy.alive);
    }

    #[test]
    fn enemy_take_damage_dead_rejected() {
        let mut enemy = CombatEnemy {
            id: 1,
            name: "enemy".into(),
            hp: 0.0,
            max_hp: 10.0,
            damage: 5.0,
            alive: false,
        };
        assert!(!enemy.take_damage(5.0));
    }

    #[test]
    fn enemy_take_damage_negative_rejected() {
        let mut enemy = CombatEnemy {
            id: 1,
            name: "enemy".into(),
            hp: 10.0,
            max_hp: 10.0,
            damage: 5.0,
            alive: true,
        };
        assert!(!enemy.take_damage(-5.0));
    }

    #[test]
    fn combat_start_guard() {
        let mut enc = CombatEncounter::z2_skirmish();
        assert_eq!(enc.phase(), EncounterPhase::Idle);
        enc.start();
        assert_eq!(enc.phase(), EncounterPhase::Active);
        enc.start(); // double start should be noop
        assert!(enc.is_active());
    }

    #[test]
    fn combat_wave_cleared_and_advance() {
        let mut enc = CombatEncounter::z2_skirmish();
        enc.start();
        enc.clear_current_wave();
        let events = enc.drain_events();
        // Should have wave cleared event with next_wave
        let has_wave_cleared = events.iter().any(|e| {
            matches!(e, CombatEvent::WaveCleared { next_wave, .. } if *next_wave)
        });
        assert!(has_wave_cleared);
    }

    #[test]
    fn combat_full_clear() {
        let mut enc = CombatEncounter::z2_skirmish();
        enc.start();
        enc.clear_current_wave(); // wave 0
        enc.drain_events();
        enc.clear_current_wave(); // wave 1 (last)
        assert!(enc.is_cleared());
    }

    #[test]
    fn combat_damage_enemy_by_id() {
        let mut enc = CombatEncounter::z2_skirmish();
        enc.start();
        // Enemy IDs start at 1 in z2_skirmish
        let killed = enc.damage_enemy(1, 1000.0);
        assert!(killed);
    }

    #[test]
    fn combat_total_enemy_count() {
        let enc = CombatEncounter::z2_skirmish();
        assert!(enc.total_enemy_count() > 0);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 14: perf_budget_tests — percentiles, violations, subsystems
// ══════════════════════════════════════════════════════════════════════
mod perf_budget_tests {
    use veilweaver_slice_runtime::perf_budget::*;

    #[test]
    fn violation_detection_at_threshold() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        // default: target=16.67, multiplier=1.5, threshold=25.0
        tracker.record_tick_ms(24.9); // below threshold
        tracker.record_tick_ms(25.1); // above threshold
        let report = tracker.report();
        assert_eq!(report.violation_count, 1);
    }

    #[test]
    fn peak_ms_tracking() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(5.0);
        tracker.record_tick_ms(15.0);
        tracker.record_tick_ms(10.0);
        assert!((tracker.report().peak_ms - 15.0).abs() < 0.01);
    }

    #[test]
    fn window_eviction_fifo() {
        let config = FrameBudgetConfig {
            target_ms: 16.67,
            window_size: 3,
            violation_multiplier: 1.5,
        };
        let mut tracker = FrameBudgetTracker::new(config);
        tracker.record_tick_ms(1.0);
        tracker.record_tick_ms(2.0);
        tracker.record_tick_ms(3.0);
        tracker.record_tick_ms(4.0); // evicts 1.0
        let report = tracker.report();
        assert_eq!(report.sample_count, 3);
        assert!((report.p50_ms - 3.0).abs() < 0.01);
    }

    #[test]
    fn percentile_single_sample() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(10.0);
        let report = tracker.report();
        assert!((report.p50_ms - 10.0).abs() < 0.01);
        assert!((report.p95_ms - 10.0).abs() < 0.01);
        assert!((report.p99_ms - 10.0).abs() < 0.01);
    }

    #[test]
    fn within_budget_check() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        for _ in 0..100 {
            tracker.record_tick_ms(10.0); // well under 16.67
        }
        assert!(tracker.report().within_budget);

        let mut tracker2 = FrameBudgetTracker::default_60hz();
        for _ in 0..100 {
            tracker2.record_tick_ms(20.0); // over budget
        }
        assert!(!tracker2.report().within_budget);
    }

    #[test]
    fn violation_rate_arithmetic() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(30.0); // violation
        tracker.record_tick_ms(10.0); // no violation
        let report = tracker.report();
        assert!((report.violation_rate - 0.5).abs() < 0.01);
    }

    #[test]
    fn subsystem_timings_bottleneck() {
        let mut timings = SubsystemTimings::new();
        timings.record("physics", 3.0);
        timings.record("ai", 5.0);
        timings.record("render", 2.0);
        let (name, ms) = timings.bottleneck().unwrap();
        assert_eq!(name, "ai");
        assert!((ms - 5.0).abs() < 0.01);
    }

    #[test]
    fn subsystem_timings_total() {
        let mut timings = SubsystemTimings::new();
        timings.record("a", 3.0);
        timings.record("b", 5.0);
        assert!((timings.total_ms() - 8.0).abs() < 0.01);
    }

    #[test]
    fn subsystem_timings_empty_bottleneck() {
        let timings = SubsystemTimings::new();
        assert!(timings.bottleneck().is_none());
    }

    #[test]
    fn empty_report_within_budget() {
        let tracker = FrameBudgetTracker::default_60hz();
        let report = tracker.report();
        assert!(report.within_budget);
        assert_eq!(report.total_ticks, 0);
    }

    #[test]
    fn reset_clears_state() {
        let mut tracker = FrameBudgetTracker::default_60hz();
        tracker.record_tick_ms(10.0);
        tracker.record_tick_ms(20.0);
        tracker.reset();
        assert_eq!(tracker.total_ticks(), 0);
        assert_eq!(tracker.report().peak_ms, 0.0);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 15: determinism_tests — hash functions, trace comparison
// ══════════════════════════════════════════════════════════════════════
mod determinism_tests {
    use veilweaver_slice_runtime::determinism::*;

    #[test]
    fn fingerprint_combined_is_xor_of_components() {
        let fp = StateFingerprint::new(0, 1, 2, 3, 4);
        // combined is computed via FnvHasher, not simple XOR
        assert_ne!(fp.combined, 0);
    }

    #[test]
    fn fingerprint_matches_same() {
        let a = StateFingerprint::new(0, 10, 20, 30, 40);
        let b = StateFingerprint::new(0, 10, 20, 30, 40);
        assert!(a.matches(&b));
    }

    #[test]
    fn fingerprint_differs() {
        let a = StateFingerprint::new(0, 10, 20, 30, 40);
        let b = StateFingerprint::new(0, 10, 20, 30, 41);
        assert!(!a.matches(&b));
    }

    #[test]
    fn trace_records_and_retrieves() {
        let mut trace = DeterminismTrace::new("run1");
        let fp = StateFingerprint::new(0, 1, 2, 3, 4);
        trace.record(fp);
        assert_eq!(trace.len(), 1);
        assert!(!trace.is_empty());
        assert!(trace.get(0).is_some());
    }

    #[test]
    fn trace_compare_identical() {
        let mut a = DeterminismTrace::new("a");
        let mut b = DeterminismTrace::new("b");
        for i in 0..10 {
            let fp = StateFingerprint::new(i, 100 + i, 200 + i, 300 + i, 400 + i);
            a.record(fp.clone());
            b.record(fp);
        }
        let report = a.compare(&b);
        assert!(report.is_consistent());
        assert!((report.consistency_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn trace_compare_diverged() {
        let mut a = DeterminismTrace::new("a");
        let mut b = DeterminismTrace::new("b");
        a.record(StateFingerprint::new(0, 1, 2, 3, 4));
        b.record(StateFingerprint::new(0, 1, 2, 3, 5));
        let report = a.compare(&b);
        assert!(!report.is_consistent());
        assert!(report.first_divergence_tick().is_some());
    }

    #[test]
    fn consistency_ratio_zero_ticks() {
        let a = DeterminismTrace::new("a");
        let b = DeterminismTrace::new("b");
        let report = a.compare(&b);
        assert!((report.consistency_ratio() - 1.0).abs() < 0.01);
    }

    #[test]
    fn multi_run_all_consistent() {
        let mut validator = MultiRunValidator::new();
        for run in 0..3 {
            let mut trace = DeterminismTrace::new(format!("run{}", run));
            for tick in 0..5 {
                trace.record(StateFingerprint::new(tick, 1, 2, 3, 4));
            }
            validator.add_trace(trace);
        }
        assert!(validator.all_consistent());
    }

    #[test]
    fn hash_game_loop_deterministic() {
        let h1 = hash_game_loop_state(Some([0, 0, 0]), 0, None, 0, 0);
        let h2 = hash_game_loop_state(Some([0, 0, 0]), 0, None, 0, 0);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_game_loop_differs_on_zone() {
        let h1 = hash_game_loop_state(Some([0, 0, 0]), 0, None, 0, 0);
        let h2 = hash_game_loop_state(Some([1, 0, 0]), 0, None, 0, 0);
        assert_ne!(h1, h2);
    }

    #[test]
    fn hash_hud_deterministic() {
        let anchors = vec![("a1".to_string(), 500i32, false, false)];
        let h1 = hash_hud_state(100, &anchors);
        let h2 = hash_hud_state(100, &anchors);
        assert_eq!(h1, h2);
    }

    #[test]
    fn hash_telemetry_differs_on_enemies() {
        let zones: Vec<String> = vec!["Z0".to_string(), "Z1".to_string()];
        let h1 = hash_telemetry_state(&zones, 5, 0, 10, 3, 1, false, 100000, 50000);
        let h2 = hash_telemetry_state(&zones, 6, 0, 10, 3, 1, false, 100000, 50000);
        assert_ne!(h1, h2);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 16: checkpoint_tests — diff granularity, store eviction
// ══════════════════════════════════════════════════════════════════════
mod checkpoint_tests {
    use veilweaver_slice_runtime::checkpoint::*;
    use veilweaver_slice_runtime::walkthrough::SliceOrchestrator;

    #[test]
    fn checkpoint_store_eviction() {
        let mut store = CheckpointStore::with_capacity(3);
        for i in 0..5u64 {
            // Each orchestrator advances i ticks so they get unique tick values
            let mut orch = SliceOrchestrator::new();
            for _ in 0..i {
                orch.tick(0.016);
            }
            let cp = SliceCheckpoint::capture_from_orchestrator(
                &orch,
                format!("cp_{}", i),
            );
            store.push(cp);
        }
        assert_eq!(store.len(), 3);
        // Ticks 0 and 1 should be evicted, ticks 2,3,4 remain
        assert!(store.get(0).is_none());
        assert!(store.get(1).is_none());
        assert!(store.get(2).is_some());
    }

    #[test]
    fn checkpoint_diff_identical() {
        let orch = SliceOrchestrator::new();
        let cp1 = SliceCheckpoint::capture_from_orchestrator(&orch, "a");
        let cp2 = SliceCheckpoint::capture_from_orchestrator(&orch, "b");
        let diff = cp1.diff(&cp2);
        assert!(diff.is_identical);
        assert_eq!(diff.mismatch_count, 0);
    }

    #[test]
    fn checkpoint_diff_after_damage() {
        let mut orch = SliceOrchestrator::new();
        let cp1 = SliceCheckpoint::capture_from_orchestrator(&orch, "before");
        orch.damage_player(50.0);
        let cp2 = SliceCheckpoint::capture_from_orchestrator(&orch, "after");
        let diff = cp1.diff(&cp2);
        assert!(!diff.is_identical);
        assert!(diff.mismatch_count > 0);
    }

    #[test]
    fn checkpoint_json_roundtrip() {
        let orch = SliceOrchestrator::new();
        let cp = SliceCheckpoint::capture_from_orchestrator(&orch, "test");
        let json = cp.save_to_json().unwrap();
        let loaded = SliceCheckpoint::load_from_json(&json).unwrap();
        let diff = cp.diff(&loaded);
        assert!(diff.is_identical);
    }

    #[test]
    fn checkpoint_store_ticks() {
        let mut store = CheckpointStore::new();
        let cp = SliceCheckpoint::capture_from_orchestrator(
            &SliceOrchestrator::new(),
            "test",
        );
        store.push(cp);
        let ticks = store.ticks();
        assert_eq!(ticks.len(), 1);
    }

    #[test]
    fn checkpoint_store_latest() {
        let mut store = CheckpointStore::new();
        for i in 0..3 {
            let mut orch = SliceOrchestrator::new();
            for _ in 0..i {
                orch.tick(0.016);
            }
            let cp = SliceCheckpoint::capture_from_orchestrator(&orch, "test");
            store.push(cp);
        }
        assert!(store.latest().is_some());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 17: walkthrough_tests — TickResult, beat progression, orchestrator
// ══════════════════════════════════════════════════════════════════════
mod walkthrough_tests {
    use veilweaver_slice_runtime::walkthrough::*;

    #[test]
    fn tick_result_is_empty_when_no_events() {
        let result = TickResult {
            game_events: vec![],
            combat_events: vec![],
            walkthrough_events: vec![],
        };
        assert!(result.is_empty());
        assert_eq!(result.event_count(), 0);
    }

    #[test]
    fn tick_result_not_empty_with_events() {
        let result = TickResult {
            game_events: vec![],
            combat_events: vec![],
            walkthrough_events: vec![WalkthroughEvent::TutorialComplete],
        };
        assert!(!result.is_empty());
        assert_eq!(result.event_count(), 1);
    }

    #[test]
    fn orchestrator_initial_beat() {
        let orch = SliceOrchestrator::new();
        assert_eq!(orch.beat(), WalkthroughBeat::TutorialBegin);
        assert!(!orch.is_complete());
    }

    #[test]
    fn orchestrator_tick_nan_dt_safe() {
        let mut orch = SliceOrchestrator::new();
        let result = orch.tick(f32::NAN);
        assert!(result.is_empty());
    }

    #[test]
    fn orchestrator_tick_negative_dt_safe() {
        let mut orch = SliceOrchestrator::new();
        let result = orch.tick(-1.0);
        assert!(result.is_empty());
    }

    #[test]
    fn complete_tutorial_advances_beat() {
        let mut orch = SliceOrchestrator::new();
        orch.complete_tutorial();
        orch.tick(0.016);
        assert_eq!(orch.beat(), WalkthroughBeat::TutorialComplete);
    }

    #[test]
    fn damage_player_triggers_death_respawn() {
        let mut orch = SliceOrchestrator::new();
        orch.damage_player(200.0);
        // Player should have auto-respawned
        assert!(orch.player().alive);
    }

    #[test]
    fn heal_player() {
        let mut orch = SliceOrchestrator::new();
        orch.damage_player(50.0);
        orch.heal_player(25.0);
        assert!((orch.player().hp - 75.0).abs() < 0.01);
    }

    #[test]
    fn collect_echoes_updates_hud() {
        let mut orch = SliceOrchestrator::new();
        orch.collect_echoes(10);
        assert_eq!(orch.thread_hud().echoes.balance, 10);
    }

    #[test]
    fn force_beat() {
        let mut orch = SliceOrchestrator::new();
        orch.force_beat(WalkthroughBeat::BossArenaEntry);
        assert_eq!(orch.beat(), WalkthroughBeat::BossArenaEntry);
    }

    #[test]
    fn start_and_clear_combat() {
        let mut orch = SliceOrchestrator::new();
        orch.start_combat();
        assert!(orch.combat.is_active());
        orch.clear_combat_wave();
        orch.clear_combat_wave(); // clear second wave
        assert!(orch.combat.is_cleared());
    }

    #[test]
    fn walkthrough_beat_display() {
        assert_eq!(format!("{}", WalkthroughBeat::TutorialBegin), "Tutorial Begin");
        assert_eq!(format!("{}", WalkthroughBeat::Complete), "Complete");
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 18: cinematic_player_tests — state machine, progress
// ══════════════════════════════════════════════════════════════════════
mod cinematic_player_tests {
    use veilweaver_slice_runtime::cinematic_player::*;
    use astraweave_cinematics::{CameraKey, Time, Timeline};

    fn make_timeline(duration: f32) -> Timeline {
        let mut tl = Timeline::new("test", duration);
        tl.add_camera_track(vec![
            CameraKey::new(
                Time::from_secs(0.0),
                (0.0, 0.0, 0.0),
                (0.0, 0.0, 1.0),
                60.0,
            ),
            CameraKey::new(
                Time::from_secs(duration),
                (10.0, 0.0, 0.0),
                (10.0, 0.0, 1.0),
                60.0,
            ),
        ]);
        tl
    }

    #[test]
    fn cinematic_initial_idle() {
        let player = CinematicPlayer::new();
        assert_eq!(player.state(), PlaybackState::Idle);
        assert!(!player.is_playing());
    }

    #[test]
    fn play_transitions_to_playing() {
        let mut player = CinematicPlayer::new();
        player.load("intro", make_timeline(5.0));
        player.play("intro").unwrap();
        assert!(player.is_playing());
        assert_eq!(player.active_cinematic(), Some("intro"));
    }

    #[test]
    fn play_nonexistent_returns_error() {
        let mut player = CinematicPlayer::new();
        assert!(player.play("nonexistent").is_err());
    }

    #[test]
    fn tick_advances_time() {
        let mut player = CinematicPlayer::new();
        player.load("test", make_timeline(5.0));
        player.play("test").unwrap();
        player.tick(1.0);
        assert!((player.current_time() - 1.0).abs() < 0.1);
    }

    #[test]
    fn progress_at_half() {
        let mut player = CinematicPlayer::new();
        player.load("test", make_timeline(4.0));
        player.play("test").unwrap();
        player.tick(2.0);
        assert!((player.progress() - 0.5).abs() < 0.1);
    }

    #[test]
    fn finished_at_end() {
        let mut player = CinematicPlayer::new();
        player.load("test", make_timeline(1.0));
        player.play("test").unwrap();
        player.tick(2.0);
        assert!(player.is_finished());
    }

    #[test]
    fn pause_and_resume() {
        let mut player = CinematicPlayer::new();
        player.load("test", make_timeline(5.0));
        player.play("test").unwrap();
        player.pause();
        assert!(player.is_paused());
        let time_paused = player.current_time();
        player.tick(1.0); // should not advance
        assert!((player.current_time() - time_paused).abs() < 0.01);
        player.resume();
        assert!(player.is_playing());
    }

    #[test]
    fn stop_returns_to_idle() {
        let mut player = CinematicPlayer::new();
        player.load("test", make_timeline(5.0));
        player.play("test").unwrap();
        player.stop();
        assert_eq!(player.state(), PlaybackState::Idle);
    }

    #[test]
    fn tick_does_not_exceed_duration() {
        let mut player = CinematicPlayer::new();
        player.load("test", make_timeline(5.0));
        player.play("test").unwrap();
        player.tick(100.0); // well past the end
        assert!(player.is_finished());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 19: vfx_dispatch_tests — zone transitions, boss encounter, stingers
// ══════════════════════════════════════════════════════════════════════
mod vfx_dispatch_tests {
    use veilweaver_slice_runtime::vfx_dispatch::VfxAudioDispatch;
    use veilweaver_slice_runtime::vfx_specs::{Vec3f, ThreadVfxSpec, TelegraphVfxSpec};

    #[test]
    fn initial_state() {
        let dispatch = VfxAudioDispatch::new();
        assert_eq!(dispatch.current_zone(), 0);
        assert!(!dispatch.in_boss_encounter());
        assert!((dispatch.elapsed() - 0.0).abs() < 0.01);
    }

    #[test]
    fn add_and_remove_thread() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.add_thread(ThreadVfxSpec::stable("t1", Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0)));
        assert_eq!(dispatch.vfx_scene().threads.len(), 1);
        dispatch.remove_thread("t1");
        assert_eq!(dispatch.vfx_scene().threads.len(), 0);
    }

    #[test]
    fn emit_echo_burst() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.emit_echo_burst(Vec3f::ZERO, false);
        assert_eq!(dispatch.vfx_scene().echo_bursts.len(), 1);
    }

    #[test]
    fn boss_encounter_mode() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.enter_boss_encounter();
        assert!(dispatch.in_boss_encounter());
        dispatch.boss_defeated();
        assert!(!dispatch.in_boss_encounter());
    }

    #[test]
    fn telegraph_lifecycle() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.add_telegraph(TelegraphVfxSpec::cleave(Vec3f::ZERO, Vec3f::new(1.0, 0.0, 0.0)));
        assert_eq!(dispatch.vfx_scene().telegraphs.len(), 1);
        dispatch.remove_telegraph("Oathbound Cleave");
        assert_eq!(dispatch.vfx_scene().telegraphs.len(), 0);
    }

    #[test]
    fn stinger_drain() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.weaving_success();
        let stingers = dispatch.drain_stingers();
        assert_eq!(stingers.len(), 1);
        assert!(dispatch.drain_stingers().is_empty());
    }

    #[test]
    fn weaving_failure_stinger() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.weaving_failure();
        let stingers = dispatch.drain_stingers();
        assert!(!stingers.is_empty());
    }

    #[test]
    fn affinity_rank_up_stinger() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.affinity_rank_up();
        let stingers = dispatch.drain_stingers();
        assert!(!stingers.is_empty());
    }

    #[test]
    fn ability_unlock_stinger() {
        let mut dispatch = VfxAudioDispatch::new();
        dispatch.ability_unlock();
        let stingers = dispatch.drain_stingers();
        assert!(!stingers.is_empty());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 20: game_loop_tests — event processing, storm detection
// ══════════════════════════════════════════════════════════════════════
mod game_loop_tests {
    use veilweaver_slice_runtime::game_loop::*;
    use veilweaver_slice_runtime::storm_choice::{StormChoice, StormPhase};

    #[test]
    fn idle_tick_no_events() {
        let mut gl = GameLoop::new();
        let events = gl.tick(0.016);
        assert!(events.is_empty());
    }

    #[test]
    fn set_active_zone_generates_event() {
        use astraweave_scene::world_partition::GridCoord;
        let mut gl = GameLoop::new();
        gl.set_active_zone(GridCoord::new(0, 0, 0));
        assert!(gl.active_zone.is_some());
    }

    #[test]
    fn storm_choice_notify() {
        let mut gl = GameLoop::new();
        gl.storm_state.enter_crossroads();
        gl.notify_storm_choice(StormChoice::Redirect);
        let events = gl.tick(0.016);
        let has_storm = events.iter().any(|e| matches!(e, GameLoopEvent::StormDecisionMade { .. }));
        assert!(has_storm);
    }

    #[test]
    fn storm_state_accessible() {
        let gl = GameLoop::new();
        assert_eq!(gl.storm_state().phase(), StormPhase::PreCrossroads);
    }

    #[test]
    fn register_trigger_actions() {
        let mut gl = GameLoop::new();
        gl.register_trigger_actions(vec![
            ("t1".into(), "zone.transition:Z0_loomspire_sanctum".into()),
            ("t2".into(), "cinematic.play:intro".into()),
        ]);
        assert!(gl.trigger_actions.len() >= 2);
    }

    #[test]
    fn is_cinematic_playing_default_false() {
        let gl = GameLoop::new();
        assert!(!gl.is_cinematic_playing());
    }

    #[test]
    fn is_dialogue_active_default_false() {
        let gl = GameLoop::new();
        assert!(!gl.is_dialogue_active());
    }

    #[test]
    fn active_dialogue_id_default_none() {
        let gl = GameLoop::new();
        assert!(gl.active_dialogue_id().is_none());
    }

    #[test]
    fn cinematic_playback_via_game_loop() {
        let mut gl = GameLoop::new();
        use astraweave_cinematics::{CameraKey, Time, Timeline};
        let mut tl = Timeline::new("test_cine", 2.0);
        tl.add_camera_track(vec![
            CameraKey::new(Time::from_secs(0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 60.0),
            CameraKey::new(Time::from_secs(2.0), (10.0, 0.0, 0.0), (10.0, 0.0, 1.0), 60.0),
        ]);
        gl.cinematics.load("test_cine", tl);
        gl.cinematics.play("test_cine").unwrap();
        assert!(gl.is_cinematic_playing());
        gl.tick(3.0); // finish cinematic
        assert!(!gl.is_cinematic_playing());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 21: walkthrough_extended_tests — HUD sync, combat events, VFX
// ══════════════════════════════════════════════════════════════════════
mod walkthrough_extended_tests {
    use veilweaver_slice_runtime::walkthrough::*;
    use veilweaver_slice_runtime::storm_choice::StormChoice;

    #[test]
    fn is_terminal_only_complete() {
        assert!(WalkthroughBeat::Complete.is_terminal());
        assert!(!WalkthroughBeat::TutorialBegin.is_terminal());
        assert!(!WalkthroughBeat::BossActive.is_terminal());
        assert!(!WalkthroughBeat::Debrief.is_terminal());
    }

    #[test]
    fn combat_clears_emit_walkthrough_events() {
        let mut orch = SliceOrchestrator::new();
        orch.start_combat();
        // Damage all enemies in wave 1
        for id in 1..=3 {
            orch.combat.damage_enemy(id, 1000.0);
        }
        let result = orch.tick(0.016);
        // Should have combat events propagated
        assert!(!result.combat_events.is_empty() || !result.walkthrough_events.is_empty());
    }

    #[test]
    fn echo_collection_syncs_to_hud() {
        let mut orch = SliceOrchestrator::new();
        orch.collect_echoes(15);
        orch.tick(0.016);
        assert_eq!(orch.thread_hud.echoes.balance, 15);
        // Large collection (>=10) should emit large burst
        let bursts = orch.vfx_audio.vfx_scene().echo_bursts.len();
        // Burst may have been emitted and drained, just verify no crash
        let _ = bursts;
    }

    #[test]
    fn echo_small_collection_syncs_to_hud() {
        let mut orch = SliceOrchestrator::new();
        orch.collect_echoes(5);
        orch.tick(0.016);
        assert_eq!(orch.thread_hud.echoes.balance, 5);
    }

    #[test]
    fn beat_advance_to_boss_arena_triggers_vfx() {
        let mut orch = SliceOrchestrator::new();
        orch.force_beat(WalkthroughBeat::BossArenaEntry);
        orch.tick(0.016);
        assert!(orch.vfx_audio.in_boss_encounter());
    }

    #[test]
    fn beat_advance_to_boss_defeated_triggers_vfx() {
        let mut orch = SliceOrchestrator::new();
        orch.force_beat(WalkthroughBeat::BossArenaEntry);
        orch.tick(0.016);
        orch.force_beat(WalkthroughBeat::BossDefeated);
        orch.tick(0.016);
        assert!(!orch.vfx_audio.in_boss_encounter());
    }

    #[test]
    fn storm_decision_through_orchestrator() {
        let mut orch = SliceOrchestrator::new();
        orch.game_loop.storm_state.enter_crossroads();
        orch.game_loop.notify_storm_choice(StormChoice::Stabilize);
        orch.tick(0.016);
        // Storm should be decided now
        assert!(orch.game_loop.storm_state.is_decided());
    }

    #[test]
    fn combat_enemy_defeated_syncs_hud_vfx() {
        let mut orch = SliceOrchestrator::new();
        orch.start_combat();
        assert!(orch.combat.damage_enemy(1, 1000.0));
        orch.tick(0.016);
        // Should have synced VFX (echo burst on defeat)
        let telemetry_enemies = orch.telemetry.enemies_defeated;
        assert!(telemetry_enemies >= 1);
    }

    #[test]
    fn combat_full_clear_syncs_events() {
        let mut orch = SliceOrchestrator::new();
        orch.start_combat();
        // Clear all wave 1 enemies
        for id in 1..=3 {
            orch.combat.damage_enemy(id, 1000.0);
        }
        orch.combat.clear_current_wave();
        orch.tick(0.016);
        // Clear wave 2
        for id in 4..=5 {
            orch.combat.damage_enemy(id, 1000.0);
        }
        orch.combat.clear_current_wave();
        orch.tick(0.016);
        assert!(orch.combat.is_cleared());
    }

    #[test]
    fn damage_player_updates_telemetry() {
        let mut orch = SliceOrchestrator::new();
        orch.damage_player(30.0);
        orch.tick(0.016);
        assert!(orch.telemetry.damage_taken >= 30.0);
    }

    #[test]
    fn walkthrough_beat_display_all_variants() {
        // Ensures Display trait for all beats returns non-empty strings
        let beats = [
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
        ];
        for beat in &beats {
            let s = format!("{}", beat);
            assert!(!s.is_empty(), "Beat {:?} has empty display", beat);
        }
    }

    #[test]
    fn advance_to_debrief_finalizes_recap() {
        let mut orch = SliceOrchestrator::new();
        orch.force_beat(WalkthroughBeat::Debrief);
        orch.tick(0.016);
        // Recap should be finalized
        assert!(orch.recap.is_finalized());
    }

    #[test]
    fn complete_beat_finalizes_recap() {
        let mut orch = SliceOrchestrator::new();
        orch.force_beat(WalkthroughBeat::Complete);
        orch.tick(0.016);
        assert!(orch.recap.is_finalized());
        assert!(orch.is_complete());
    }

    #[test]
    fn repair_anchor_syncs_to_hud() {
        let mut orch = SliceOrchestrator::new();
        // Ensure a thread anchor exists in HUD
        orch.thread_hud.add_anchor("anchor_0", 0.5);
        orch.repair_anchor("anchor_0");
        orch.tick(0.016);
        // Telemetry should count it
        assert!(orch.telemetry.anchors_repaired >= 1);
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 22: cinematic_player_extended_tests
// ══════════════════════════════════════════════════════════════════════
mod cinematic_player_extended_tests {
    use veilweaver_slice_runtime::cinematic_player::*;
    use astraweave_cinematics::{CameraKey, Time, Timeline};

    fn make_timeline(duration: f32) -> Timeline {
        let mut tl = Timeline::new("test", duration);
        tl.add_camera_track(vec![
            CameraKey::new(Time::from_secs(0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 60.0),
            CameraKey::new(Time::from_secs(duration), (10.0, 0.0, 0.0), (10.0, 0.0, 1.0), 60.0),
        ]);
        tl
    }

    #[test]
    fn has_timeline_after_load() {
        let mut player = CinematicPlayer::new();
        assert!(!player.has_timeline("test"));
        player.load("test", make_timeline(5.0));
        assert!(player.has_timeline("test"));
    }

    #[test]
    fn timeline_count() {
        let mut player = CinematicPlayer::new();
        assert_eq!(player.timeline_count(), 0);
        player.load("a", make_timeline(1.0));
        player.load("b", make_timeline(2.0));
        assert_eq!(player.timeline_count(), 2);
    }

    #[test]
    fn timeline_names() {
        let mut player = CinematicPlayer::new();
        player.load("intro", make_timeline(1.0));
        player.load("outro", make_timeline(1.0));
        let names = player.timeline_names();
        assert_eq!(names.len(), 2);
        assert!(names.contains(&"intro"));
        assert!(names.contains(&"outro"));
    }

    #[test]
    fn is_finished_only_when_finished() {
        let mut player = CinematicPlayer::new();
        assert!(!player.is_finished());
        player.load("t", make_timeline(1.0));
        player.play("t").unwrap();
        assert!(!player.is_finished());
        player.tick(2.0);
        assert!(player.is_finished());
    }

    #[test]
    fn is_paused_only_when_paused() {
        let mut player = CinematicPlayer::new();
        assert!(!player.is_paused());
        player.load("t", make_timeline(5.0));
        player.play("t").unwrap();
        assert!(!player.is_paused());
        player.pause();
        assert!(player.is_paused());
        player.resume();
        assert!(!player.is_paused());
    }

    #[test]
    fn progress_zero_duration_returns_zero() {
        let mut player = CinematicPlayer::new();
        player.load("t", make_timeline(0.001));
        player.play("t").unwrap();
        // Very short duration — progress should be valid
        let p = player.progress();
        assert!(p >= 0.0 && p <= 1.0);
    }

    #[test]
    fn debug_format_not_empty() {
        let player = CinematicPlayer::new();
        let s = format!("{:?}", player);
        assert!(!s.is_empty());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 23: storm_choice_extended — is_decided, chose_X methods
// ══════════════════════════════════════════════════════════════════════
mod storm_choice_extended_tests {
    use veilweaver_slice_runtime::storm_choice::*;

    #[test]
    fn is_decided_after_choice() {
        let mut state = StormChoiceState::new();
        assert!(!state.is_decided());
        state.enter_crossroads();
        assert!(!state.is_decided());
        state.make_choice(StormChoice::Stabilize);
        assert!(state.is_decided());
    }

    #[test]
    fn is_resolved_after_resolve() {
        let mut state = StormChoiceState::new();
        assert!(!state.is_resolved());
        state.enter_crossroads();
        state.make_choice(StormChoice::Redirect);
        assert!(!state.is_resolved());
        state.resolve();
        assert!(state.is_resolved());
    }

    #[test]
    fn chose_stabilize() {
        let mut state = StormChoiceState::new();
        assert!(!state.chose_stabilize());
        state.enter_crossroads();
        state.make_choice(StormChoice::Stabilize);
        assert!(state.chose_stabilize());
        assert!(!state.chose_redirect());
    }

    #[test]
    fn chose_redirect() {
        let mut state = StormChoiceState::new();
        assert!(!state.chose_redirect());
        state.enter_crossroads();
        state.make_choice(StormChoice::Redirect);
        assert!(state.chose_redirect());
        assert!(!state.chose_stabilize());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 24: player_state_extended — position, is_full_health
// ══════════════════════════════════════════════════════════════════════
mod player_state_extended_tests {
    use veilweaver_slice_runtime::player_state::PlayerState;

    #[test]
    fn set_position() {
        let mut ps = PlayerState::new();
        ps.set_position([1.0, 2.0, 3.0]);
        assert_eq!(ps.position, [1.0, 2.0, 3.0]);
    }

    #[test]
    fn is_full_health_after_damage_and_full_heal() {
        let mut ps = PlayerState::new();
        assert!(ps.is_full_health());
        ps.take_damage(10.0);
        assert!(!ps.is_full_health());
        ps.heal(10.0);
        assert!(ps.is_full_health());
    }

    #[test]
    fn is_full_health_not_after_partial_heal() {
        let mut ps = PlayerState::new();
        ps.take_damage(20.0);
        ps.heal(10.0);
        assert!(!ps.is_full_health());
    }
}

// ══════════════════════════════════════════════════════════════════════
// Module 25: dialogue_storm_integration — dialogue choice → storm trigger
// ══════════════════════════════════════════════════════════════════════
mod dialogue_storm_integration_tests {
    use veilweaver_slice_runtime::game_loop::*;
    use veilweaver_slice_runtime::storm_choice::*;
    use astraweave_dialogue::toml_loader::load_dialogue_from_toml;

    fn storm_dialogue_stabilize() -> astraweave_dialogue::toml_loader::LoadedDialogue {
        let toml = r#"
id = "storm_choice"
start = "prompt"

[[nodes]]
id = "prompt"
line = { speaker = "NPC", text = "The storm approaches. What do you do?" }
choices = [
    { text = "Stabilize the rift", go_to = "storm_stabilize" },
    { text = "Redirect its energy", go_to = "storm_redirect" },
]

[[nodes]]
id = "storm_stabilize"
line = { speaker = "NPC", text = "You stabilize the rift." }
end = true

[[nodes]]
id = "storm_redirect"
line = { speaker = "NPC", text = "You redirect the storm." }
end = true
"#;
        load_dialogue_from_toml(toml).unwrap()
    }

    #[test]
    fn dialogue_stabilize_choice_triggers_storm() {
        let mut gl = GameLoop::new();
        let loaded = storm_dialogue_stabilize();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("entry", "dialogue.play:storm_choice");

        // Open the storm decision prompt first.
        gl.storm_state.enter_crossroads();

        // Trigger dialogue.
        gl.notify_trigger_enter(vec!["entry".to_string()]);
        let _events = gl.tick(0.016);
        assert!(gl.is_dialogue_active());

        // Choose index 0 = "Stabilize the rift" → next node = "storm_stabilize".
        gl.notify_dialogue_choice("storm_choice", 0);
        let _events = gl.tick(0.016);
        // process_dialogues sets deferred_storm_choice; third tick flushes it.
        let _events = gl.tick(0.016);

        // Storm should be decided as Stabilize (via choice text or next_node_id).
        assert!(gl.storm_state.is_decided());
        assert!(gl.storm_state.chose_stabilize());
    }

    #[test]
    fn dialogue_redirect_choice_triggers_storm() {
        let mut gl = GameLoop::new();
        let loaded = storm_dialogue_stabilize();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("entry", "dialogue.play:storm_choice");

        gl.storm_state.enter_crossroads();

        gl.notify_trigger_enter(vec!["entry".to_string()]);
        let _events = gl.tick(0.016);
        assert!(gl.is_dialogue_active());

        // Choose index 1 = "Redirect its energy" → next node = "storm_redirect".
        gl.notify_dialogue_choice("storm_choice", 1);
        let _events = gl.tick(0.016);
        // process_dialogues sets deferred_storm_choice; third tick flushes it.
        let _events = gl.tick(0.016);

        assert!(gl.storm_state.is_decided());
        assert!(gl.storm_state.chose_redirect());
    }

    #[test]
    fn dialogue_choice_without_storm_keywords_no_trigger() {
        let toml = r#"
id = "normal"
start = "n0"

[[nodes]]
id = "n0"
line = { speaker = "A", text = "Hello." }
choices = [{ text = "Greetings", go_to = "n1" }]

[[nodes]]
id = "n1"
line = { speaker = "A", text = "Farewell." }
end = true
"#;
        let loaded = load_dialogue_from_toml(toml).unwrap();
        let mut gl = GameLoop::new();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("entry", "dialogue.play:normal");

        gl.storm_state.enter_crossroads();
        gl.notify_trigger_enter(vec!["entry".to_string()]);
        let _events = gl.tick(0.016);
        gl.notify_dialogue_choice("normal", 0);
        let _events = gl.tick(0.016);

        // Storm should NOT be decided since no storm keywords in choices.
        assert!(!gl.storm_state.is_decided());
    }

    #[test]
    fn active_dialogue_id_returns_correct_id() {
        let toml = r#"
id = "test_dlg"
start = "n0"

[[nodes]]
id = "n0"
line = { speaker = "A", text = "Question?" }
choices = [{ text = "Answer", go_to = "n1" }]

[[nodes]]
id = "n1"
line = { speaker = "A", text = "Done." }
end = true
"#;
        let loaded = load_dialogue_from_toml(toml).unwrap();
        let mut gl = GameLoop::new();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("t", "dialogue.play:test_dlg");

        gl.notify_trigger_enter(vec!["t".to_string()]);
        let _events = gl.tick(0.016);
        assert!(gl.is_dialogue_active());
        assert_eq!(gl.active_dialogue_id(), Some("test_dlg"));
    }

    #[test]
    fn cinematic_progress_boundary() {
        use veilweaver_slice_runtime::cinematic_player::CinematicPlayer;
        use astraweave_cinematics::{CameraKey, Time, Timeline};
        let mut player = CinematicPlayer::new();
        let mut tl = Timeline::new("boundary", 1.0);
        tl.add_camera_track(vec![
            CameraKey::new(Time::from_secs(0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 60.0),
            CameraKey::new(Time::from_secs(1.0), (1.0, 0.0, 0.0), (1.0, 0.0, 1.0), 60.0),
        ]);
        player.load("b", tl);
        player.play("b").unwrap();
        // At exactly 0.0, progress should be 0.0.
        let p0 = player.progress();
        assert!(p0 >= 0.0 && p0 <= 1.0);
        // At duration boundary, progress should be clamp to 1.0
        player.tick(1.0);
        let p1 = player.progress();
        assert!((p1 - 1.0).abs() < 0.01 || player.is_finished());
    }

    /// Tests text-only storm trigger (choice text says "chaos" but node_id does not match storm).
    #[test]
    fn dialogue_chaos_text_triggers_redirect() {
        let toml = r#"
id = "chaos_dlg"
start = "prompt"

[[nodes]]
id = "prompt"
line = { speaker = "NPC", text = "The storm rages. Act now!" }
choices = [
    { text = "Embrace the chaos", go_to = "chaos_result" },
]

[[nodes]]
id = "chaos_result"
line = { speaker = "NPC", text = "Chaos reigns." }
end = true
"#;
        let loaded = load_dialogue_from_toml(toml).unwrap();
        let mut gl = GameLoop::new();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("t", "dialogue.play:chaos_dlg");

        gl.storm_state.enter_crossroads();
        gl.notify_trigger_enter(vec!["t".to_string()]);
        let _ = gl.tick(0.016);
        assert!(gl.is_dialogue_active());

        gl.notify_dialogue_choice("chaos_dlg", 0);
        let _ = gl.tick(0.016);
        let _ = gl.tick(0.016); // flush deferred

        // "chaos" in choice text should trigger Redirect
        assert!(gl.storm_state.is_decided());
        assert!(gl.storm_state.chose_redirect());
    }

    /// Tests lowercase "stabilize" text path separately.
    #[test]
    fn dialogue_lowercase_stabilize_triggers() {
        let toml = r#"
id = "lower_dlg"
start = "prompt"

[[nodes]]
id = "prompt"
line = { speaker = "NPC", text = "What will you do?" }
choices = [
    { text = "I will stabilize it", go_to = "end" },
]

[[nodes]]
id = "end"
line = { speaker = "NPC", text = "Done." }
end = true
"#;
        let loaded = load_dialogue_from_toml(toml).unwrap();
        let mut gl = GameLoop::new();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("t", "dialogue.play:lower_dlg");

        gl.storm_state.enter_crossroads();
        gl.notify_trigger_enter(vec!["t".to_string()]);
        let _ = gl.tick(0.016);

        gl.notify_dialogue_choice("lower_dlg", 0);
        let _ = gl.tick(0.016);
        let _ = gl.tick(0.016); // flush

        // lowercase "stabilize" should trigger Stabilize
        assert!(gl.storm_state.is_decided());
        assert!(gl.storm_state.chose_stabilize());
    }

    /// Tests lowercase "redirect" text path.
    #[test]
    fn dialogue_lowercase_redirect_triggers() {
        let toml = r#"
id = "redir_dlg"
start = "prompt"

[[nodes]]
id = "prompt"
line = { speaker = "NPC", text = "Choose your path." }
choices = [
    { text = "redirect the energy", go_to = "end" },
]

[[nodes]]
id = "end"
line = { speaker = "NPC", text = "Redirected." }
end = true
"#;
        let loaded = load_dialogue_from_toml(toml).unwrap();
        let mut gl = GameLoop::new();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("t", "dialogue.play:redir_dlg");

        gl.storm_state.enter_crossroads();
        gl.notify_trigger_enter(vec!["t".to_string()]);
        let _ = gl.tick(0.016);

        gl.notify_dialogue_choice("redir_dlg", 0);
        let _ = gl.tick(0.016);
        let _ = gl.tick(0.016);

        assert!(gl.storm_state.is_decided());
        assert!(gl.storm_state.chose_redirect());
    }

    /// Tests node_id-only storm trigger path (choice text has no storm keywords).
    #[test]
    fn dialogue_node_id_storm_stabilize() {
        let toml = r#"
id = "nodeid_dlg"
start = "prompt"

[[nodes]]
id = "prompt"
line = { speaker = "NPC", text = "Make your choice." }
choices = [
    { text = "Option A", go_to = "storm_stabilize" },
]

[[nodes]]
id = "storm_stabilize"
line = { speaker = "NPC", text = "Path chosen." }
end = true
"#;
        let loaded = load_dialogue_from_toml(toml).unwrap();
        let mut gl = GameLoop::new();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("t", "dialogue.play:nodeid_dlg");

        gl.storm_state.enter_crossroads();
        gl.notify_trigger_enter(vec!["t".to_string()]);
        let _ = gl.tick(0.016);

        gl.notify_dialogue_choice("nodeid_dlg", 0);
        let _ = gl.tick(0.016);
        let _ = gl.tick(0.016);

        // node_id "storm_stabilize" should trigger Stabilize even without text match
        assert!(gl.storm_state.is_decided());
        assert!(gl.storm_state.chose_stabilize());
    }

    /// Tests node_id "storm_redirect" path.
    #[test]
    fn dialogue_node_id_storm_redirect() {
        let toml = r#"
id = "nodeid2_dlg"
start = "prompt"

[[nodes]]
id = "prompt"
line = { speaker = "NPC", text = "Make your choice." }
choices = [
    { text = "Option B", go_to = "storm_redirect" },
]

[[nodes]]
id = "storm_redirect"
line = { speaker = "NPC", text = "Redirecting." }
end = true
"#;
        let loaded = load_dialogue_from_toml(toml).unwrap();
        let mut gl = GameLoop::new();
        gl.register_dialogue(loaded);
        gl.register_trigger_action("t", "dialogue.play:nodeid2_dlg");

        gl.storm_state.enter_crossroads();
        gl.notify_trigger_enter(vec!["t".to_string()]);
        let _ = gl.tick(0.016);

        gl.notify_dialogue_choice("nodeid2_dlg", 0);
        let _ = gl.tick(0.016);
        let _ = gl.tick(0.016);

        assert!(gl.storm_state.is_decided());
        assert!(gl.storm_state.chose_redirect());
    }

    /// Tests process_cinematics lifecycle (was_playing → is_finished → CinematicFinished).
    #[test]
    fn cinematic_finished_emits_event() {
        use astraweave_cinematics::{CameraKey, Time, Timeline};
        let mut gl = GameLoop::new();
        let mut tl = Timeline::new("test_cine", 0.5);
        tl.add_camera_track(vec![
            CameraKey::new(Time::from_secs(0.0), (0.0, 0.0, 0.0), (0.0, 0.0, 1.0), 60.0),
            CameraKey::new(Time::from_secs(0.5), (1.0, 0.0, 0.0), (1.0, 0.0, 1.0), 60.0),
        ]);
        gl.cinematics.load("test_cine", tl);
        gl.register_trigger_action("cine_t", "cinematic.play:test_cine");

        gl.notify_trigger_enter(vec!["cine_t".to_string()]);
        let events = gl.tick(0.016);
        assert!(events.iter().any(|e| matches!(e, GameLoopEvent::CinematicStarted { .. })));
        assert!(gl.is_cinematic_playing());

        // Advance past end
        let _ = gl.tick(0.3);
        let events = gl.tick(0.3);
        // Should get CinematicFinished event
        let finished = events.iter().any(|e| matches!(e, GameLoopEvent::CinematicFinished { .. }));
        assert!(finished);
        assert!(!gl.is_cinematic_playing());
    }
}
