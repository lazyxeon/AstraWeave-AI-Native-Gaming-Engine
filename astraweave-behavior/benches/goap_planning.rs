use astraweave_behavior::goap::*;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

/// Benchmark GOAP planning with a simple scenario
fn bench_goap_planning_simple(c: &mut Criterion) {
    c.bench_function("goap_planning_simple", |b| {
        // Initial state: agent has no weapon, is not at cover
        let initial = WorldState::from_facts(&[
            ("has_weapon", false),
            ("at_cover", false),
            ("enemy_visible", true),
        ]);

        // Goal: be at cover and have weapon
        let goal = GoapGoal::new(
            "combat_ready",
            WorldState::from_facts(&[("has_weapon", true), ("at_cover", true)]),
        );

        // Available actions
        let actions = vec![
            GoapAction::new("pick_up_weapon")
                .with_cost(1.0)
                .with_precondition("has_weapon", false)
                .with_effect("has_weapon", true),
            GoapAction::new("move_to_cover")
                .with_cost(2.0)
                .with_precondition("at_cover", false)
                .with_effect("at_cover", true),
        ];

        let planner = GoapPlanner::new();
        
        b.iter(|| {
            let plan = planner.plan(
                black_box(&initial),
                black_box(&goal),
                black_box(&actions),
            );
            black_box(plan)
        });
    });
}

/// Benchmark GOAP planning with moderate complexity (10 actions)
fn bench_goap_planning_moderate(c: &mut Criterion) {
    c.bench_function("goap_planning_10_actions", |b| {
        // More complex state space
        let initial = WorldState::from_facts(&[
            ("has_weapon", false),
            ("has_ammo", false),
            ("at_cover", false),
            ("enemy_visible", true),
            ("low_health", true),
        ]);

        let goal = GoapGoal::new(
            "tactical_ready",
            WorldState::from_facts(&[
                ("has_weapon", true),
                ("has_ammo", true),
                ("at_cover", true),
                ("low_health", false),
            ]),
        );

        // 10 available actions with dependencies
        let actions = vec![
            GoapAction::new("pick_up_weapon")
                .with_cost(1.0)
                .with_precondition("has_weapon", false)
                .with_effect("has_weapon", true),
            GoapAction::new("pick_up_ammo")
                .with_cost(1.0)
                .with_precondition("has_ammo", false)
                .with_effect("has_ammo", true),
            GoapAction::new("move_to_cover")
                .with_cost(2.0)
                .with_precondition("at_cover", false)
                .with_effect("at_cover", true),
            GoapAction::new("use_medkit")
                .with_cost(3.0)
                .with_precondition("low_health", true)
                .with_effect("low_health", false),
            GoapAction::new("reload_weapon")
                .with_cost(1.5)
                .with_precondition("has_weapon", true)
                .with_precondition("has_ammo", true)
                .with_effect("weapon_loaded", true),
            GoapAction::new("aim_at_enemy")
                .with_cost(0.5)
                .with_precondition("has_weapon", true)
                .with_effect("aiming", true),
            GoapAction::new("sprint")
                .with_cost(1.0)
                .with_effect("moving_fast", true),
            GoapAction::new("crouch")
                .with_cost(0.5)
                .with_effect("crouched", true),
            GoapAction::new("scan_area")
                .with_cost(1.0)
                .with_effect("area_scanned", true),
            GoapAction::new("call_for_backup")
                .with_cost(2.0)
                .with_effect("backup_called", true),
        ];

        let planner = GoapPlanner::new();
        
        b.iter(|| {
            let plan = planner.plan(
                black_box(&initial),
                black_box(&goal),
                black_box(&actions),
            );
            black_box(plan)
        });
    });
}

/// Benchmark GOAP planning with complex scenario (20 actions)
fn bench_goap_planning_complex(c: &mut Criterion) {
    c.bench_function("goap_planning_20_actions", |b| {
        // Complex tactical scenario
        let initial = WorldState::from_facts(&[
            ("has_weapon", false),
            ("has_ammo", false),
            ("at_cover", false),
            ("enemy_visible", true),
            ("low_health", true),
            ("team_nearby", false),
        ]);

        let goal = GoapGoal::new(
            "combat_ready_complex",
            WorldState::from_facts(&[
                ("has_weapon", true),
                ("has_ammo", true),
                ("at_cover", true),
                ("low_health", false),
                ("ready_to_engage", true),
            ]),
        );

        // 20 tactical actions
        let actions = vec![
            GoapAction::new("pick_up_weapon").with_cost(1.0).with_effect("has_weapon", true),
            GoapAction::new("pick_up_ammo").with_cost(1.0).with_effect("has_ammo", true),
            GoapAction::new("move_to_cover").with_cost(2.0).with_effect("at_cover", true),
            GoapAction::new("use_medkit").with_cost(3.0).with_precondition("low_health", true).with_effect("low_health", false),
            GoapAction::new("reload_weapon").with_cost(1.5).with_precondition("has_weapon", true).with_effect("weapon_loaded", true),
            GoapAction::new("aim_at_enemy").with_cost(0.5).with_precondition("has_weapon", true).with_effect("aiming", true),
            GoapAction::new("sprint").with_cost(1.0).with_effect("moving_fast", true),
            GoapAction::new("crouch").with_cost(0.5).with_effect("crouched", true),
            GoapAction::new("scan_area").with_cost(1.0).with_effect("area_scanned", true),
            GoapAction::new("call_for_backup").with_cost(2.0).with_effect("backup_called", true),
            GoapAction::new("throw_grenade").with_cost(2.5).with_precondition("has_weapon", true).with_effect("grenade_thrown", true),
            GoapAction::new("take_tactical_position").with_cost(1.5).with_effect("tactical_position", true),
            GoapAction::new("suppress_fire").with_cost(2.0).with_precondition("has_ammo", true).with_effect("suppressing", true),
            GoapAction::new("flank_enemy").with_cost(3.0).with_effect("flanking", true),
            GoapAction::new("check_corners").with_cost(1.0).with_effect("corners_clear", true),
            GoapAction::new("mark_target").with_cost(0.5).with_effect("target_marked", true),
            GoapAction::new("switch_weapon").with_cost(1.0).with_precondition("has_weapon", true).with_effect("weapon_switched", true),
            GoapAction::new("deploy_shield").with_cost(2.0).with_effect("shield_deployed", true),
            GoapAction::new("prepare_ambush").with_cost(2.5).with_effect("ambush_ready", true),
            GoapAction::new("confirm_ready").with_cost(0.5)
                .with_precondition("has_weapon", true)
                .with_precondition("has_ammo", true)
                .with_precondition("at_cover", true)
                .with_effect("ready_to_engage", true),
        ];

        let planner = GoapPlanner::new();
        
        b.iter(|| {
            let plan = planner.plan(
                black_box(&initial),
                black_box(&goal),
                black_box(&actions),
            );
            black_box(plan)
        });
    });
}

/// Benchmark goal evaluation
fn bench_goal_evaluation(c: &mut Criterion) {
    c.bench_function("goap_goal_evaluation", |b| {
        let state = WorldState::from_facts(&[
            ("has_weapon", true),
            ("has_ammo", true),
            ("at_cover", false),
        ]);

        let goal = GoapGoal::new(
            "combat_ready",
            WorldState::from_facts(&[
                ("has_weapon", true),
                ("has_ammo", true),
                ("at_cover", true),
            ]),
        )
        .with_priority(10.0);

        b.iter(|| {
            let satisfied = goal.is_satisfied(black_box(&state));
            black_box(satisfied)
        });
    });
}

/// Benchmark action precondition checking
fn bench_action_preconditions(c: &mut Criterion) {
    c.bench_function("goap_action_preconditions", |b| {
        let state = WorldState::from_facts(&[
            ("has_weapon", true),
            ("has_ammo", true),
            ("weapon_loaded", false),
        ]);

        let action = GoapAction::new("reload_weapon")
            .with_cost(1.5)
            .with_precondition("has_weapon", true)
            .with_precondition("has_ammo", true)
            .with_precondition("weapon_loaded", false)
            .with_effect("weapon_loaded", true);

        b.iter(|| {
            let can_apply = action.can_apply(black_box(&state));
            black_box(can_apply)
        });
    });
}

criterion_group!(
    benches,
    bench_goap_planning_simple,
    bench_goap_planning_moderate,
    bench_goap_planning_complex,
    bench_goal_evaluation,
    bench_action_preconditions
);
criterion_main!(benches);
