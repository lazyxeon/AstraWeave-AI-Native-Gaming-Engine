// Week 3 Day 6: AI Performance Benchmarks
// Validates AI planning performance against Phase 7 targets using up-to-date APIs

use astraweave_ai::{
    orchestrator::{GoapOrchestrator, Orchestrator, RuleOrchestrator, UtilityOrchestrator},
    tool_sandbox::{validate_tool_action, ToolVerb, ValidationContext},
};
use astraweave_core::{CompanionState, EnemyState, IVec2, PlayerState, Poi, WorldSnapshot};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::{collections::BTreeMap, hint::black_box};

// =============================================================================
// Helper: Create Minimal Snapshot
// =============================================================================

fn create_minimal_snapshot(num_enemies: usize, num_pois: usize) -> WorldSnapshot {
    let enemies = (0..num_enemies)
        .map(|i| EnemyState {
            id: i as u32,
            pos: IVec2 {
                x: (i as i32) * 10,
                y: 0,
            physics_context: None,
            },
            hp: 50,
            cover: "none".to_string(),
            last_seen: 0.0,
        })
        .collect();

    let pois = (0..num_pois)
        .map(|i| Poi {
            k: format!("poi_{i}"),
            pos: IVec2 {
                x: (i as i32) * 15,
                y: 5,
            },
        })
        .collect();

    WorldSnapshot {
        t: 1.0,
        player: PlayerState {,
            physics_context: None,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            stance: "stand".to_string(),
            orders: vec![],
        },
        me: CompanionState {
            pos: IVec2 { x: 1, y: 1 },
            ammo: 30,
            morale: 0.8,
            cooldowns: BTreeMap::new(),
        },
        enemies,
        pois,
        obstacles: vec![],
        objective: None,
    }
}

fn create_complex_snapshot() -> WorldSnapshot {
    let mut cooldowns = BTreeMap::new();
    cooldowns.insert("throw:smoke".to_string(), 2.5);
    cooldowns.insert("attack".to_string(), 0.5);

    WorldSnapshot {
        t: 5.0,
        player: PlayerState {,
            physics_context: None,
            pos: IVec2 { x: 10, y: 10 },
            hp: 75,
            stance: "crouch".to_string(),
            orders: vec!["hold_position".to_string()],
        },
        me: CompanionState {
            pos: IVec2 { x: 12, y: 12 },
            ammo: 15,
            morale: 0.6,
            cooldowns,
        },
        enemies: vec![
            EnemyState {
                id: 1,
                pos: IVec2 { x: 20, y: 20 },
                hp: 60,
                cover: "low".to_string(),
                last_seen: 0.5,
            },
            EnemyState {
                id: 2,
                pos: IVec2 { x: 25, y: 18 },
                hp: 40,
                cover: "none".to_string(),
                last_seen: 0.7,
            },
            EnemyState {
                id: 3,
                pos: IVec2 { x: 30, y: 22 },
                hp: 80,
                cover: "high".to_string(),
                last_seen: 0.1,
            },
        ],
        pois: vec![
            Poi {
                k: "poi_alpha".to_string(),
                pos: IVec2 { x: 15, y: 15 },
            },
            Poi {
                k: "poi_beta".to_string(),
                pos: IVec2 { x: 18, y: 20 },
            },
        ],
        obstacles: vec![
            IVec2 { x: 16, y: 16 },
            IVec2 { x: 17, y: 16 },
            IVec2 { x: 18, y: 16 },
        ],
        objective: Some("Defend the checkpoint".to_string()),
    }
}

// =============================================================================
// Benchmark 1: GOAP Planning Latency
// =============================================================================

fn bench_goap_planning_latency(c: &mut Criterion) {
    let orchestrator = GoapOrchestrator;
    let mut group = c.benchmark_group("GOAP Planning");

    let simple_snap = create_minimal_snapshot(1, 0);
    group.bench_function("1 enemy (simple)", |b| {
        b.iter(|| {
            let snap = black_box(&simple_snap);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    let moderate_snap = create_minimal_snapshot(3, 2);
    group.bench_function("3 enemies + 2 POIs (moderate)", |b| {
        b.iter(|| {
            let snap = black_box(&moderate_snap);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    let complex_snap = create_minimal_snapshot(10, 5);
    group.bench_function("10 enemies + 5 POIs (complex)", |b| {
        b.iter(|| {
            let snap = black_box(&complex_snap);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 2: Rule-Based Planning
// =============================================================================

fn bench_rule_based_planning(c: &mut Criterion) {
    let orchestrator = RuleOrchestrator;
    let mut group = c.benchmark_group("Rule-Based Planning");

    let simple_snap = create_minimal_snapshot(1, 0);
    group.bench_function("1 enemy", |b| {
        b.iter(|| {
            let snap = black_box(&simple_snap);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    let moderate_snap = create_minimal_snapshot(3, 2);
    group.bench_function("3 enemies", |b| {
        b.iter(|| {
            let snap = black_box(&moderate_snap);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 3: Utility AI Planning
// =============================================================================

fn bench_utility_planning(c: &mut Criterion) {
    let orchestrator = UtilityOrchestrator;
    let mut group = c.benchmark_group("Utility AI Planning");

    let simple_snap = create_minimal_snapshot(1, 0);
    group.bench_function("1 enemy", |b| {
        b.iter(|| {
            let snap = black_box(&simple_snap);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    let complex_snap = create_complex_snapshot();
    group.bench_function("complex scenario", |b| {
        b.iter(|| {
            let snap = black_box(&complex_snap);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 4: Tool Validation
// =============================================================================

fn bench_tool_validation(c: &mut Criterion) {
    let mut group = c.benchmark_group("Tool Validation");
    let snap = create_complex_snapshot();
    let context: ValidationContext<'static> = ValidationContext::new();
    let target = IVec2 { x: 20, y: 20 };

    group.bench_function("validate MoveTo", |b| {
        b.iter(|| {
            let _ = validate_tool_action(0, ToolVerb::MoveTo, &snap, &context, Some(target));
        })
    });

    group.bench_function("validate CoverFire", |b| {
        b.iter(|| {
            let _ = validate_tool_action(0, ToolVerb::CoverFire, &snap, &context, Some(target));
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 5: Multi-Agent Throughput
// =============================================================================

fn bench_multi_agent_throughput(c: &mut Criterion) {
    let orchestrator = GoapOrchestrator;
    let mut group = c.benchmark_group("Multi-Agent Throughput");

    for agent_count in [10_u32, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(agent_count),
            agent_count,
            |b, &count| {
                let snap = create_minimal_snapshot(1, 0);
                b.iter(|| {
                    for _ in 0..count {
                        let s = black_box(&snap);
                        black_box(orchestrator.propose_plan(s));
                    }
                })
            },
        );
    }

    group.finish();
}

// =============================================================================
// Benchmark 6: Snapshot Cloning (Memory)
// =============================================================================

fn bench_snapshot_cloning(c: &mut Criterion) {
    let mut group = c.benchmark_group("WorldSnapshot Operations");

    let simple_snap = create_minimal_snapshot(1, 0);
    group.bench_function("clone simple snapshot", |b| {
        b.iter(|| {
            let snap = black_box(&simple_snap);
            snap.clone()
        })
    });

    let complex_snap = create_complex_snapshot();
    group.bench_function("clone complex snapshot", |b| {
        b.iter(|| {
            let snap = black_box(&complex_snap);
            snap.clone()
        })
    });

    let large_snap = create_minimal_snapshot(100, 50);
    group.bench_function("clone large snapshot (100 enemies)", |b| {
        b.iter(|| {
            let snap = black_box(&large_snap);
            snap.clone()
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 7: Planning Under Different Conditions
// =============================================================================

fn bench_planning_conditions(c: &mut Criterion) {
    let orchestrator = GoapOrchestrator;
    let mut group = c.benchmark_group("Planning Conditions");

    let no_enemies = create_minimal_snapshot(0, 2);
    group.bench_function("no enemies (idle)", |b| {
        b.iter(|| {
            let snap = black_box(&no_enemies);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    let mut low_ammo = create_minimal_snapshot(3, 0);
    low_ammo.me.ammo = 2;
    group.bench_function("low ammo (3 enemies)", |b| {
        b.iter(|| {
            let snap = black_box(&low_ammo);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    let mut low_morale = create_minimal_snapshot(5, 0);
    low_morale.me.morale = 0.2;
    group.bench_function("low morale (5 enemies)", |b| {
        b.iter(|| {
            let snap = black_box(&low_morale);
            black_box(orchestrator.propose_plan(snap))
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 8: Orchestrator Comparison
// =============================================================================

fn bench_orchestrator_comparison(c: &mut Criterion) {
    let goap = GoapOrchestrator;
    let rule = RuleOrchestrator;
    let utility = UtilityOrchestrator;
    let mut group = c.benchmark_group("Orchestrator Comparison");
    let snap = create_complex_snapshot();

    group.bench_function("Rule-based", |b| {
        b.iter(|| {
            let s = black_box(&snap);
            black_box(rule.propose_plan(s))
        })
    });

    group.bench_function("GOAP", |b| {
        b.iter(|| {
            let s = black_box(&snap);
            black_box(goap.propose_plan(s))
        })
    });

    group.bench_function("Utility", |b| {
        b.iter(|| {
            let s = black_box(&snap);
            black_box(utility.propose_plan(s))
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark Groups
// =============================================================================

criterion_group!(
    benches,
    bench_goap_planning_latency,
    bench_rule_based_planning,
    bench_utility_planning,
    bench_tool_validation,
    bench_multi_agent_throughput,
    bench_snapshot_cloning,
    bench_planning_conditions,
    bench_orchestrator_comparison,
);

criterion_main!(benches);
