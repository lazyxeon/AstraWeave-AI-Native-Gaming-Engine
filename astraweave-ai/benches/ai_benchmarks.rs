// Week 3 Day 6: AI Performance Benchmarks
// Validates AI planning performance against Phase 7 targets

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use astraweave_ai::{
    orchestrator::{RuleOrchestrator, GOAPOrchestrator, UtilityOrchestrator, Orchestrator},
    tool_sandbox::{validate_action_plan, PerceptionConfig},
};
use astraweave_core::{
    schema::{WorldSnapshot, CompanionState, PlayerState, EnemyState, Poi},
    glam::IVec2,
};
use std::collections::BTreeMap;

// =============================================================================
// Helper: Create Minimal Snapshot
// =============================================================================

fn create_minimal_snapshot(num_enemies: usize, num_pois: usize) -> WorldSnapshot {
    let enemies = (0..num_enemies)
        .map(|i| EnemyState {
            pos: IVec2::new((i * 10) as i32, 0),
            hp: 50,
        })
        .collect();

    let pois = (0..num_pois)
        .map(|i| Poi {
            pos: IVec2::new((i * 15) as i32, 5),
        })
        .collect();

    WorldSnapshot {
        t: 1.0,
        player: PlayerState {
            pos: IVec2::new(0, 0),
            hp: 100,
        },
        me: CompanionState {
            pos: IVec2::new(1, 1),
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
        player: PlayerState {
            pos: IVec2::new(10, 10),
            hp: 75,
        },
        me: CompanionState {
            pos: IVec2::new(12, 12),
            ammo: 15,
            morale: 0.6,
            cooldowns,
        },
        enemies: vec![
            EnemyState { pos: IVec2::new(20, 20), hp: 60 },
            EnemyState { pos: IVec2::new(25, 18), hp: 40 },
            EnemyState { pos: IVec2::new(30, 22), hp: 80 },
        ],
        pois: vec![
            Poi { pos: IVec2::new(15, 15) },
            Poi { pos: IVec2::new(18, 20) },
        ],
        obstacles: vec![
            IVec2::new(16, 16),
            IVec2::new(17, 16),
            IVec2::new(18, 16),
        ],
        objective: Some("Defend the checkpoint".to_string()),
    }
}

// =============================================================================
// Benchmark 1: GOAP Planning Latency
// =============================================================================

fn bench_goap_planning_latency(c: &mut Criterion) {
    let mut group = c.benchmark_group("GOAP Planning");
    
    // Simple scenario (1 enemy)
    let simple_snap = create_minimal_snapshot(1, 0);
    group.bench_function("1 enemy (simple)", |b| {
        let orchestrator = GOAPOrchestrator;
        b.iter(|| {
            let snap = black_box(&simple_snap);
            orchestrator.next_action(snap)
        })
    });

    // Moderate scenario (3 enemies, 2 POIs)
    let moderate_snap = create_minimal_snapshot(3, 2);
    group.bench_function("3 enemies + 2 POIs (moderate)", |b| {
        let orchestrator = GOAPOrchestrator;
        b.iter(|| {
            let snap = black_box(&moderate_snap);
            orchestrator.next_action(snap)
        })
    });

    // Complex scenario (10 enemies, 5 POIs)
    let complex_snap = create_minimal_snapshot(10, 5);
    group.bench_function("10 enemies + 5 POIs (complex)", |b| {
        let orchestrator = GOAPOrchestrator;
        b.iter(|| {
            let snap = black_box(&complex_snap);
            orchestrator.next_action(snap)
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 2: Rule-Based Planning
// =============================================================================

fn bench_rule_based_planning(c: &mut Criterion) {
    let mut group = c.benchmark_group("Rule-Based Planning");

    let simple_snap = create_minimal_snapshot(1, 0);
    group.bench_function("1 enemy", |b| {
        let orchestrator = RuleOrchestrator;
        b.iter(|| {
            let snap = black_box(&simple_snap);
            orchestrator.next_action(snap)
        })
    });

    let moderate_snap = create_minimal_snapshot(3, 2);
    group.bench_function("3 enemies", |b| {
        let orchestrator = RuleOrchestrator;
        b.iter(|| {
            let snap = black_box(&moderate_snap);
            orchestrator.next_action(snap)
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 3: Utility AI Planning
// =============================================================================

fn bench_utility_planning(c: &mut Criterion) {
    let mut group = c.benchmark_group("Utility AI Planning");

    let simple_snap = create_minimal_snapshot(1, 0);
    group.bench_function("1 enemy", |b| {
        let orchestrator = UtilityOrchestrator;
        b.iter(|| {
            let snap = black_box(&simple_snap);
            orchestrator.next_action(snap)
        })
    });

    let complex_snap = create_complex_snapshot();
    group.bench_function("complex scenario", |b| {
        let orchestrator = UtilityOrchestrator;
        b.iter(|| {
            let snap = black_box(&complex_snap);
            orchestrator.next_action(snap)
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
    let config = PerceptionConfig { los_max: 50 };

    // Create a simple MoveTo plan
    let mut plan_args = BTreeMap::new();
    plan_args.insert("pos".to_string(), "20,20".to_string());
    
    let plan = astraweave_core::schema::PlanIntent {
        plan_id: "test_plan".to_string(),
        steps: vec![
            astraweave_core::schema::ActionStep {
                verb: "MoveTo".to_string(),
                args: plan_args,
            }
        ],
    };

    group.bench_function("validate MoveTo", |b| {
        b.iter(|| {
            let s = black_box(&snap);
            let p = black_box(&plan);
            validate_action_plan(s, p, &config)
        })
    });

    // Create a CoverFire plan
    let mut cover_args = BTreeMap::new();
    cover_args.insert("target".to_string(), "20,20".to_string());
    
    let cover_plan = astraweave_core::schema::PlanIntent {
        plan_id: "cover_plan".to_string(),
        steps: vec![
            astraweave_core::schema::ActionStep {
                verb: "CoverFire".to_string(),
                args: cover_args,
            }
        ],
    };

    group.bench_function("validate CoverFire", |b| {
        b.iter(|| {
            let s = black_box(&snap);
            let p = black_box(&cover_plan);
            validate_action_plan(s, p, &config)
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 5: Multi-Agent Throughput
// =============================================================================

fn bench_multi_agent_throughput(c: &mut Criterion) {
    let mut group = c.benchmark_group("Multi-Agent Throughput");

    for agent_count in [10, 50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::from_parameter(agent_count),
            agent_count,
            |b, &count| {
                let orchestrator = GOAPOrchestrator;
                let snap = create_minimal_snapshot(1, 0);
                
                b.iter(|| {
                    for _ in 0..count {
                        let s = black_box(&snap);
                        orchestrator.next_action(s);
                    }
                })
            }
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
    let mut group = c.benchmark_group("Planning Conditions");

    // No enemies (idle planning)
    let no_enemies = create_minimal_snapshot(0, 2);
    group.bench_function("no enemies (idle)", |b| {
        let orchestrator = GOAPOrchestrator;
        b.iter(|| {
            let snap = black_box(&no_enemies);
            orchestrator.next_action(snap)
        })
    });

    // Low ammo
    let mut low_ammo = create_minimal_snapshot(3, 0);
    low_ammo.me.ammo = 2;
    group.bench_function("low ammo (3 enemies)", |b| {
        let orchestrator = GOAPOrchestrator;
        b.iter(|| {
            let snap = black_box(&low_ammo);
            orchestrator.next_action(snap)
        })
    });

    // Low morale
    let mut low_morale = create_minimal_snapshot(5, 0);
    low_morale.me.morale = 0.2;
    group.bench_function("low morale (5 enemies)", |b| {
        let orchestrator = GOAPOrchestrator;
        b.iter(|| {
            let snap = black_box(&low_morale);
            orchestrator.next_action(snap)
        })
    });

    group.finish();
}

// =============================================================================
// Benchmark 8: Orchestrator Comparison
// =============================================================================

fn bench_orchestrator_comparison(c: &mut Criterion) {
    let mut group = c.benchmark_group("Orchestrator Comparison");
    let snap = create_complex_snapshot();

    group.bench_function("Rule-based", |b| {
        let orchestrator = RuleOrchestrator;
        b.iter(|| {
            let s = black_box(&snap);
            orchestrator.next_action(s)
        })
    });

    group.bench_function("GOAP", |b| {
        let orchestrator = GOAPOrchestrator;
        b.iter(|| {
            let s = black_box(&snap);
            orchestrator.next_action(s)
        })
    });

    group.bench_function("Utility", |b| {
        let orchestrator = UtilityOrchestrator;
        b.iter(|| {
            let s = black_box(&snap);
            orchestrator.next_action(s)
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
