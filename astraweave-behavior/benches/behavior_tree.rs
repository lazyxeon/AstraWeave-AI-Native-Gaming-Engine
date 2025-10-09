use astraweave_behavior::*;
use criterion::{criterion_group, criterion_main, Criterion};
use std::hint::black_box;

/// Benchmark simple behavior tree execution (3 nodes)
fn bench_behavior_tree_simple(c: &mut Criterion) {
    c.bench_function("behavior_tree_simple_3_nodes", |b| {
        // Simple selector: try attack, else move to cover
        let tree = BehaviorGraph::new(BehaviorNode::Selector(vec![
            BehaviorNode::Action("attack_enemy".to_string()),
            BehaviorNode::Action("move_to_cover".to_string()),
        ]));

        let mut context = BehaviorContext::new();
        context.register_action("attack_enemy", || BehaviorStatus::Failure); // Will fail first
        context.register_action("move_to_cover", || BehaviorStatus::Success); // Then succeed

        b.iter(|| {
            let result = tree.tick(black_box(&context));
            black_box(result)
        });
    });
}

/// Benchmark moderate behavior tree (10 nodes)
fn bench_behavior_tree_moderate(c: &mut Criterion) {
    c.bench_function("behavior_tree_10_nodes", |b| {
        // More complex combat AI tree
        let tree = BehaviorGraph::new(BehaviorNode::Selector(vec![
            // First priority: engage if possible
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("enemy_visible".to_string()),
                BehaviorNode::Condition("has_weapon".to_string()),
                BehaviorNode::Action("aim_at_enemy".to_string()),
                BehaviorNode::Action("fire_weapon".to_string()),
            ]),
            // Second priority: prepare for combat
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("low_health".to_string()),
                BehaviorNode::Action("use_medkit".to_string()),
                BehaviorNode::Action("move_to_cover".to_string()),
            ]),
            // Fallback: patrol
            BehaviorNode::Action("patrol".to_string()),
        ]));

        let mut context = BehaviorContext::new();
        context.register_condition("enemy_visible", || true);
        context.register_condition("has_weapon", || true);
        context.register_condition("low_health", || false);
        context.register_action("aim_at_enemy", || BehaviorStatus::Success);
        context.register_action("fire_weapon", || BehaviorStatus::Success);
        context.register_action("use_medkit", || BehaviorStatus::Success);
        context.register_action("move_to_cover", || BehaviorStatus::Success);
        context.register_action("patrol", || BehaviorStatus::Success);

        b.iter(|| {
            let result = tree.tick(black_box(&context));
            black_box(result)
        });
    });
}

/// Benchmark complex behavior tree (20+ nodes)
fn bench_behavior_tree_complex(c: &mut Criterion) {
    c.bench_function("behavior_tree_20_nodes", |b| {
        // Full tactical AI behavior tree
        let tree = BehaviorGraph::new(BehaviorNode::Selector(vec![
            // Emergency response
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("under_fire".to_string()),
                BehaviorNode::Selector(vec![
                    BehaviorNode::Sequence(vec![
                        BehaviorNode::Condition("has_smoke_grenade".to_string()),
                        BehaviorNode::Action("throw_smoke".to_string()),
                    ]),
                    BehaviorNode::Action("dive_to_cover".to_string()),
                ]),
            ]),
            // Combat engagement
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("enemy_visible".to_string()),
                BehaviorNode::Selector(vec![
                    // Close range combat
                    BehaviorNode::Sequence(vec![
                        BehaviorNode::Condition("enemy_close".to_string()),
                        BehaviorNode::Condition("has_melee_weapon".to_string()),
                        BehaviorNode::Action("melee_attack".to_string()),
                    ]),
                    // Ranged combat
                    BehaviorNode::Sequence(vec![
                        BehaviorNode::Condition("has_weapon".to_string()),
                        BehaviorNode::Condition("has_ammo".to_string()),
                        BehaviorNode::Parallel(
                            vec![
                                BehaviorNode::Action("aim_at_enemy".to_string()),
                                BehaviorNode::Action("track_target".to_string()),
                            ],
                            1, // Need at least 1 to succeed
                        ),
                        BehaviorNode::Action("fire_weapon".to_string()),
                    ]),
                ]),
            ]),
            // Support actions
            BehaviorNode::Sequence(vec![
                BehaviorNode::Condition("team_nearby".to_string()),
                BehaviorNode::Selector(vec![
                    BehaviorNode::Sequence(vec![
                        BehaviorNode::Condition("teammate_low_health".to_string()),
                        BehaviorNode::Action("provide_cover_fire".to_string()),
                    ]),
                    BehaviorNode::Action("maintain_formation".to_string()),
                ]),
            ]),
            // Default patrol behavior
            BehaviorNode::Action("patrol".to_string()),
        ]));

        let mut context = BehaviorContext::new();
        context.register_condition("under_fire", || false);
        context.register_condition("enemy_visible", || true);
        context.register_condition("enemy_close", || false);
        context.register_condition("has_weapon", || true);
        context.register_condition("has_ammo", || true);
        context.register_condition("has_smoke_grenade", || false);
        context.register_condition("has_melee_weapon", || false);
        context.register_condition("team_nearby", || false);
        context.register_condition("teammate_low_health", || false);
        context.register_action("throw_smoke", || BehaviorStatus::Success);
        context.register_action("dive_to_cover", || BehaviorStatus::Success);
        context.register_action("melee_attack", || BehaviorStatus::Success);
        context.register_action("aim_at_enemy", || BehaviorStatus::Success);
        context.register_action("track_target", || BehaviorStatus::Success);
        context.register_action("fire_weapon", || BehaviorStatus::Success);
        context.register_action("provide_cover_fire", || BehaviorStatus::Success);
        context.register_action("maintain_formation", || BehaviorStatus::Success);
        context.register_action("patrol", || BehaviorStatus::Success);

        b.iter(|| {
            let result = tree.tick(black_box(&context));
            black_box(result)
        });
    });
}

/// Benchmark composite node evaluation (sequence)
fn bench_sequence_node(c: &mut Criterion) {
    c.bench_function("behavior_tree_sequence_evaluation", |b| {
        let sequence = BehaviorNode::Sequence(vec![
            BehaviorNode::Condition("check1".to_string()),
            BehaviorNode::Condition("check2".to_string()),
            BehaviorNode::Condition("check3".to_string()),
            BehaviorNode::Action("execute".to_string()),
        ]);

        let mut context = BehaviorContext::new();
        context.register_condition("check1", || true);
        context.register_condition("check2", || true);
        context.register_condition("check3", || true);
        context.register_action("execute", || BehaviorStatus::Success);

        b.iter(|| {
            let result = sequence.tick(black_box(&context));
            black_box(result)
        });
    });
}

/// Benchmark decorator node (inverter)
fn bench_decorator_node(c: &mut Criterion) {
    c.bench_function("behavior_tree_decorator", |b| {
        let decorator = BehaviorNode::Decorator(
            DecoratorType::Inverter,
            Box::new(BehaviorNode::Condition("enemy_visible".to_string())),
        );

        let mut context = BehaviorContext::new();
        context.register_condition("enemy_visible", || true);

        b.iter(|| {
            let result = decorator.tick(black_box(&context));
            black_box(result)
        });
    });
}

/// Benchmark condition evaluation patterns
fn bench_blackboard_access(c: &mut Criterion) {
    c.bench_function("behavior_tree_condition_evaluation", |b| {
        // Test multiple condition evaluations (simulates blackboard access patterns)
        let tree = BehaviorNode::Sequence(vec![
            BehaviorNode::Condition("fact_0".to_string()),
            BehaviorNode::Condition("fact_1".to_string()),
            BehaviorNode::Condition("fact_2".to_string()),
            BehaviorNode::Condition("fact_3".to_string()),
            BehaviorNode::Condition("fact_4".to_string()),
        ]);
        
        let mut context = BehaviorContext::new();
        for i in 0..20 {
            let value = i % 2 == 0;
            context.register_condition(&format!("fact_{}", i), move || value);
        }

        b.iter(|| {
            let result = tree.tick(black_box(&context));
            black_box(result)
        });
    });
}

criterion_group!(
    benches,
    bench_behavior_tree_simple,
    bench_behavior_tree_moderate,
    bench_behavior_tree_complex,
    bench_sequence_node,
    bench_decorator_node,
    bench_blackboard_access
);
criterion_main!(benches);
