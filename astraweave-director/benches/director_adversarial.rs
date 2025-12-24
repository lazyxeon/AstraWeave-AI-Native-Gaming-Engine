//! Adversarial Director Benchmarks
//!
//! Stress testing for boss AI, phase systems, and LLM-driven directors.

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::hint::black_box as std_black_box;

// ============================================================================
// LOCAL TYPES (Mirror astraweave-director API)
// ============================================================================

#[derive(Clone, Debug)]
struct DirectorPlan {
    id: u64,
    operations: Vec<DirectorOp>,
    priority: u32,
    duration: f32,
}

#[derive(Clone, Debug)]
enum DirectorOp {
    Fortify { position: [f32; 3], strength: f32 },
    SpawnWave { enemy_types: Vec<String>, count: u32 },
    Collapse { area_radius: f32 },
    BuffBoss { stat: String, amount: f32 },
    SpawnMinion { minion_type: String, position: [f32; 3] },
    EnvironmentalHazard { hazard_type: String, duration: f32 },
    PhaseTransition { target_phase: u32 },
}

#[derive(Clone, Debug)]
struct BossState {
    id: u64,
    health: f32,
    max_health: f32,
    phase: u32,
    position: [f32; 3],
    enrage_timer: f32,
    active_buffs: Vec<(String, f32)>, // (buff_name, remaining_duration)
    minions: Vec<u64>,
}

#[derive(Clone, Debug)]
struct PhaseConfig {
    phase_number: u32,
    health_threshold: f32, // Boss enters phase when health falls below this %
    available_ops: Vec<String>,
    attack_speed_multiplier: f32,
    damage_multiplier: f32,
}

#[derive(Clone, Debug)]
struct EncounterMetrics {
    time_elapsed: f32,
    total_damage_dealt: f32,
    total_damage_taken: f32,
    phases_completed: u32,
    minions_spawned: u32,
    player_deaths: u32,
}

impl BossState {
    fn new(max_health: f32) -> Self {
        Self {
            id: 1,
            health: max_health,
            max_health,
            phase: 1,
            position: [50.0, 50.0, 0.0],
            enrage_timer: 600.0, // 10 minutes
            active_buffs: Vec::new(),
            minions: Vec::new(),
        }
    }

    fn health_percent(&self) -> f32 {
        self.health / self.max_health
    }
}

fn generate_phase_configs() -> Vec<PhaseConfig> {
    vec![
        PhaseConfig {
            phase_number: 1,
            health_threshold: 1.0,
            available_ops: vec!["SpawnWave".to_string(), "Fortify".to_string()],
            attack_speed_multiplier: 1.0,
            damage_multiplier: 1.0,
        },
        PhaseConfig {
            phase_number: 2,
            health_threshold: 0.7,
            available_ops: vec!["SpawnWave".to_string(), "BuffBoss".to_string(), "SpawnMinion".to_string()],
            attack_speed_multiplier: 1.2,
            damage_multiplier: 1.1,
        },
        PhaseConfig {
            phase_number: 3,
            health_threshold: 0.4,
            available_ops: vec!["SpawnWave".to_string(), "Collapse".to_string(), "EnvironmentalHazard".to_string()],
            attack_speed_multiplier: 1.5,
            damage_multiplier: 1.3,
        },
        PhaseConfig {
            phase_number: 4,
            health_threshold: 0.15,
            available_ops: vec!["SpawnWave".to_string(), "Collapse".to_string(), "BuffBoss".to_string()],
            attack_speed_multiplier: 2.0,
            damage_multiplier: 1.5,
        },
    ]
}

fn generate_director_op(op_type: &str, seed: u64) -> DirectorOp {
    match op_type {
        "Fortify" => DirectorOp::Fortify {
            position: [(seed % 100) as f32, ((seed / 100) % 100) as f32, 0.0],
            strength: 1.0 + (seed % 10) as f32 * 0.1,
        },
        "SpawnWave" => DirectorOp::SpawnWave {
            enemy_types: vec!["minion".to_string(), "elite".to_string()],
            count: 3 + (seed % 5) as u32,
        },
        "Collapse" => DirectorOp::Collapse {
            area_radius: 10.0 + (seed % 20) as f32,
        },
        "BuffBoss" => DirectorOp::BuffBoss {
            stat: "damage".to_string(),
            amount: 0.1 + (seed % 5) as f32 * 0.05,
        },
        "SpawnMinion" => DirectorOp::SpawnMinion {
            minion_type: "guardian".to_string(),
            position: [(seed % 100) as f32, ((seed / 100) % 100) as f32, 0.0],
        },
        "EnvironmentalHazard" => DirectorOp::EnvironmentalHazard {
            hazard_type: "fire".to_string(),
            duration: 5.0 + (seed % 10) as f32,
        },
        _ => DirectorOp::PhaseTransition {
            target_phase: 2,
        },
    }
}

// ============================================================================
// CATEGORY 1: BOSS AI PLANNING
// ============================================================================

fn bench_boss_planning(c: &mut Criterion) {
    let mut group = c.benchmark_group("director_adversarial/boss_planning");

    // Test 1: Phase determination
    group.bench_function("phase_determination_1000", |bencher| {
        let phases = generate_phase_configs();
        let health_values: Vec<f32> = (0..1000).map(|i| i as f32 / 1000.0).collect();

        bencher.iter(|| {
            let determined_phases: Vec<u32> = health_values
                .iter()
                .map(|&health| {
                    phases
                        .iter()
                        .filter(|p| health <= p.health_threshold)
                        .max_by_key(|p| p.phase_number)
                        .map(|p| p.phase_number)
                        .unwrap_or(1)
                })
                .collect();

            std_black_box(determined_phases.iter().sum::<u32>())
        });
    });

    // Test 2: Operation selection
    group.bench_function("operation_selection_500", |bencher| {
        let phases = generate_phase_configs();

        bencher.iter(|| {
            let selected_ops: Vec<DirectorOp> = (0..500)
                .map(|i| {
                    let phase_idx = i % phases.len();
                    let phase = &phases[phase_idx];
                    let op_idx = i % phase.available_ops.len();
                    generate_director_op(&phase.available_ops[op_idx], i as u64)
                })
                .collect();

            std_black_box(selected_ops.len())
        });
    });

    // Test 3: Plan generation
    group.bench_function("plan_generation_100", |bencher| {
        let phases = generate_phase_configs();

        bencher.iter(|| {
            let plans: Vec<DirectorPlan> = (0..100)
                .map(|i| {
                    let phase_idx = (i / 25) % phases.len();
                    let phase = &phases[phase_idx];

                    let ops: Vec<DirectorOp> = (0..5)
                        .map(|j| {
                            let op_idx = (i + j) % phase.available_ops.len();
                            generate_director_op(&phase.available_ops[op_idx], (i * 5 + j) as u64)
                        })
                        .collect();

                    DirectorPlan {
                        id: i as u64,
                        operations: ops,
                        priority: (i % 3) as u32 + 1,
                        duration: 5.0 + (i % 10) as f32,
                    }
                })
                .collect();

            let total_ops: usize = plans.iter().map(|p| p.operations.len()).sum();
            std_black_box(total_ops)
        });
    });

    // Test 4: Threat assessment
    group.bench_function("threat_assessment_200", |bencher| {
        let player_positions: Vec<[f32; 3]> = (0..200)
            .map(|i| [(i % 100) as f32, ((i / 100) % 100) as f32, 0.0])
            .collect();

        let boss_pos = [50.0f32, 50.0, 0.0];

        bencher.iter(|| {
            let threats: Vec<(usize, f32)> = player_positions
                .iter()
                .enumerate()
                .map(|(i, pos)| {
                    let dist = ((pos[0] - boss_pos[0]).powi(2)
                        + (pos[1] - boss_pos[1]).powi(2))
                    .sqrt();
                    let threat = 100.0 / (1.0 + dist);
                    (i, threat)
                })
                .collect();

            let highest_threat = threats.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            std_black_box(highest_threat.map(|t| t.0))
        });
    });

    // Test 5: Enrage timer handling
    group.bench_function("enrage_calculation_1000", |bencher| {
        let mut bosses: Vec<BossState> = (0..1000)
            .map(|i| {
                let mut boss = BossState::new(1_000_000.0);
                boss.enrage_timer = 600.0 - (i as f32 * 0.5);
                boss
            })
            .collect();

        bencher.iter(|| {
            let delta_time = 0.016f32; // ~60 FPS

            for boss in &mut bosses {
                boss.enrage_timer -= delta_time;

                // Apply enrage buff at 0
                if boss.enrage_timer <= 0.0 && boss.active_buffs.iter().all(|(n, _)| n != "enrage") {
                    boss.active_buffs.push(("enrage".to_string(), f32::MAX));
                }
            }

            let enraged_count = bosses.iter().filter(|b| b.enrage_timer <= 0.0).count();
            std_black_box(enraged_count)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 2: PHASE TRANSITIONS
// ============================================================================

fn bench_phase_transitions(c: &mut Criterion) {
    let mut group = c.benchmark_group("director_adversarial/phase_transitions");

    // Test 1: Transition detection
    group.bench_function("transition_detection_1000", |bencher| {
        let phases = generate_phase_configs();
        let health_deltas: Vec<(f32, f32)> = (0..1000)
            .map(|i| {
                let before = (i + 50) as f32 / 1000.0;
                let after = i as f32 / 1000.0;
                (before, after)
            })
            .collect();

        bencher.iter(|| {
            let transitions: Vec<Option<(u32, u32)>> = health_deltas
                .iter()
                .map(|&(before, after)| {
                    let phase_before = phases
                        .iter()
                        .filter(|p| before <= p.health_threshold)
                        .max_by_key(|p| p.phase_number)
                        .map(|p| p.phase_number)
                        .unwrap_or(1);

                    let phase_after = phases
                        .iter()
                        .filter(|p| after <= p.health_threshold)
                        .max_by_key(|p| p.phase_number)
                        .map(|p| p.phase_number)
                        .unwrap_or(1);

                    if phase_after > phase_before {
                        Some((phase_before, phase_after))
                    } else {
                        None
                    }
                })
                .collect();

            let transition_count = transitions.iter().filter(|t| t.is_some()).count();
            std_black_box(transition_count)
        });
    });

    // Test 2: Phase ability unlocking
    group.bench_function("ability_unlock_50", |bencher| {
        let phases = generate_phase_configs();

        bencher.iter(|| {
            let unlocked_abilities: Vec<Vec<&String>> = (0..50)
                .map(|i| {
                    let current_phase = (i % 4) + 1;
                    phases
                        .iter()
                        .filter(|p| p.phase_number <= current_phase as u32)
                        .flat_map(|p| &p.available_ops)
                        .collect()
                })
                .collect();

            let total_abilities: usize = unlocked_abilities.iter().map(|a| a.len()).sum();
            std_black_box(total_abilities)
        });
    });

    // Test 3: Transition animation scheduling
    group.bench_function("transition_animation_scheduling", |bencher| {
        let transitions: Vec<(u32, u32, f32)> = (0..100)
            .map(|i| ((i % 3 + 1) as u32, (i % 3 + 2) as u32, i as f32 * 0.5))
            .collect();

        bencher.iter(|| {
            let animations: Vec<(String, f32, f32)> = transitions
                .iter()
                .map(|&(from, to, start_time)| {
                    let anim_name = format!("phase_{}_{}_transition", from, to);
                    let duration = match to - from {
                        1 => 2.0,
                        2 => 3.0,
                        _ => 4.0,
                    };
                    (anim_name, start_time, duration)
                })
                .collect();

            std_black_box(animations.len())
        });
    });

    // Test 4: Intermission phase handling
    group.bench_function("intermission_handling_50", |bencher| {
        let mut bosses: Vec<BossState> = (0..50)
            .map(|_| BossState::new(1_000_000.0))
            .collect();

        bencher.iter(|| {
            let mut intermission_results: Vec<(u64, bool, f32)> = Vec::new();

            for boss in &mut bosses {
                // Simulate intermission check
                let should_intermission = boss.phase == 2 && boss.health_percent() <= 0.5;

                if should_intermission {
                    // Boss becomes invulnerable, spawns adds
                    let heal_amount = boss.max_health * 0.1;
                    boss.health = (boss.health + heal_amount).min(boss.max_health);

                    // Spawn minions
                    for i in 0..3 {
                        boss.minions.push(1000 + boss.id * 10 + i);
                    }
                }

                intermission_results.push((boss.id, should_intermission, boss.health_percent()));
            }

            let intermission_count = intermission_results.iter().filter(|(_, i, _)| *i).count();
            std_black_box(intermission_count)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 3: MINION MANAGEMENT
// ============================================================================

fn bench_minion_management(c: &mut Criterion) {
    let mut group = c.benchmark_group("director_adversarial/minion_management");

    // Test 1: Minion spawning
    group.bench_function("minion_spawning_200", |bencher| {
        let boss_pos = [50.0f32, 50.0, 0.0];

        bencher.iter(|| {
            let minions: Vec<(u64, [f32; 3], String)> = (0..200)
                .map(|i| {
                    let angle = (i as f32 / 200.0) * std::f32::consts::TAU;
                    let distance = 10.0 + (i % 10) as f32;
                    let pos = [
                        boss_pos[0] + angle.cos() * distance,
                        boss_pos[1] + angle.sin() * distance,
                        boss_pos[2],
                    ];

                    let minion_type = match i % 3 {
                        0 => "melee",
                        1 => "ranged",
                        _ => "healer",
                    };

                    (i as u64, pos, minion_type.to_string())
                })
                .collect();

            std_black_box(minions.len())
        });
    });

    // Test 2: Minion AI coordination
    group.bench_function("minion_coordination_100", |bencher| {
        let minion_positions: Vec<(u64, [f32; 3])> = (0..100)
            .map(|i| {
                (
                    i as u64,
                    [(i % 10) as f32 * 10.0, (i / 10) as f32 * 10.0, 0.0],
                )
            })
            .collect();

        let target_pos = [50.0f32, 50.0, 0.0];

        bencher.iter(|| {
            let assignments: Vec<(u64, String)> = minion_positions
                .iter()
                .map(|(id, pos)| {
                    let dist_to_target = ((pos[0] - target_pos[0]).powi(2)
                        + (pos[1] - target_pos[1]).powi(2))
                    .sqrt();

                    let role = if dist_to_target < 20.0 {
                        "attack"
                    } else if dist_to_target < 40.0 {
                        "approach"
                    } else {
                        "support"
                    };

                    (*id, role.to_string())
                })
                .collect();

            let attackers = assignments.iter().filter(|(_, r)| r == "attack").count();
            std_black_box(attackers)
        });
    });

    // Test 3: Minion death tracking
    group.bench_function("minion_death_tracking_500", |bencher| {
        let mut boss = BossState::new(1_000_000.0);
        boss.minions = (0..500).collect();

        let deaths: Vec<u64> = (0..100).map(|i| i * 5).collect(); // Every 5th minion dies

        bencher.iter(|| {
            let mut current_minions = boss.minions.clone();

            for &dead_id in &deaths {
                if let Some(pos) = current_minions.iter().position(|&m| m == dead_id) {
                    current_minions.swap_remove(pos);
                }
            }

            std_black_box(current_minions.len())
        });
    });

    // Test 4: Minion reinforcement waves
    group.bench_function("reinforcement_waves_20", |bencher| {
        let wave_configs: Vec<(u32, Vec<(&str, u32)>)> = (0..20)
            .map(|i| {
                let minions = vec![
                    ("melee", 3 + i % 3),
                    ("ranged", 2 + i % 2),
                    ("elite", if i > 10 { 1 } else { 0 }),
                ];
                (i, minions)
            })
            .collect();

        bencher.iter(|| {
            let spawn_commands: Vec<(u32, String, u32)> = wave_configs
                .iter()
                .flat_map(|(wave, minions)| {
                    minions
                        .iter()
                        .filter(|(_, count)| *count > 0)
                        .map(|(minion_type, count)| (*wave, minion_type.to_string(), *count))
                        .collect::<Vec<_>>()
                })
                .collect();

            let total_spawns: u32 = spawn_commands.iter().map(|(_, _, c)| *c).sum();
            std_black_box(total_spawns)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 4: ENCOUNTER METRICS
// ============================================================================

fn bench_encounter_metrics(c: &mut Criterion) {
    let mut group = c.benchmark_group("director_adversarial/encounter_metrics");

    // Test 1: Metrics aggregation
    group.bench_function("metrics_aggregation_1000", |bencher| {
        let damage_events: Vec<(f64, f32, bool)> = (0..1000)
            .map(|i| (i as f64 * 0.016, (i % 100 + 10) as f32, i % 2 == 0))
            .collect();

        bencher.iter(|| {
            let mut metrics = EncounterMetrics {
                time_elapsed: 0.0,
                total_damage_dealt: 0.0,
                total_damage_taken: 0.0,
                phases_completed: 0,
                minions_spawned: 0,
                player_deaths: 0,
            };

            for (time, damage, is_player_damage) in &damage_events {
                metrics.time_elapsed = *time as f32;
                if *is_player_damage {
                    metrics.total_damage_dealt += damage;
                } else {
                    metrics.total_damage_taken += damage;
                }
            }

            std_black_box((metrics.total_damage_dealt, metrics.total_damage_taken))
        });
    });

    // Test 2: DPS calculation
    group.bench_function("dps_calculation_100", |bencher| {
        let damage_samples: Vec<(f32, f32)> = (0..100)
            .map(|i| (i as f32 * 0.5, (i % 50 + 100) as f32))
            .collect();

        bencher.iter(|| {
            let window_size = 5.0f32; // 5 second window
            let dps_values: Vec<f32> = damage_samples
                .iter()
                .enumerate()
                .map(|(i, (time, _))| {
                    let window_damage: f32 = damage_samples
                        .iter()
                        .filter(|(t, _)| t >= &(*time - window_size) && t <= time)
                        .map(|(_, d)| d)
                        .sum();

                    window_damage / window_size
                })
                .collect();

            let max_dps = dps_values.iter().cloned().fold(0.0f32, f32::max);
            std_black_box(max_dps)
        });
    });

    // Test 3: Performance rating calculation
    group.bench_function("performance_rating", |bencher| {
        let encounters: Vec<EncounterMetrics> = (0..50)
            .map(|i| EncounterMetrics {
                time_elapsed: 300.0 + (i as f32 * 10.0),
                total_damage_dealt: 1_000_000.0 + (i as f32 * 10000.0),
                total_damage_taken: 100_000.0 + (i as f32 * 5000.0),
                phases_completed: 4,
                minions_spawned: 50 + i,
                player_deaths: i % 3,
            })
            .collect();

        bencher.iter(|| {
            let ratings: Vec<f32> = encounters
                .iter()
                .map(|e| {
                    let time_factor = (600.0 / e.time_elapsed).min(1.5); // Faster is better, cap at 1.5x
                    let death_penalty = 1.0 - (e.player_deaths as f32 * 0.1);
                    let damage_ratio = e.total_damage_dealt / e.total_damage_taken.max(1.0);
                    let efficiency = damage_ratio.min(10.0) / 10.0;

                    time_factor * death_penalty * efficiency * 100.0
                })
                .collect();

            let avg_rating: f32 = ratings.iter().sum::<f32>() / ratings.len() as f32;
            std_black_box(avg_rating)
        });
    });

    // Test 4: Phase timing analysis
    group.bench_function("phase_timing_analysis", |bencher| {
        let phase_events: Vec<(u32, f32)> = vec![
            (1, 0.0),
            (2, 120.0),
            (3, 240.0),
            (4, 360.0),
        ];

        let expected_times = vec![90.0f32, 180.0, 270.0, 400.0];

        bencher.iter(|| {
            let analysis: Vec<(u32, f32, &str)> = phase_events
                .iter()
                .enumerate()
                .map(|(i, (phase, actual_time))| {
                    if i == 0 {
                        return (*phase, *actual_time, "start");
                    }

                    let expected = expected_times[i - 1];
                    let diff = actual_time - expected;

                    let status = if diff.abs() < 10.0 {
                        "on_time"
                    } else if diff < 0.0 {
                        "ahead"
                    } else {
                        "behind"
                    };

                    (*phase, diff, status)
                })
                .collect();

            let behind_count = analysis.iter().filter(|(_, _, s)| *s == "behind").count();
            std_black_box(behind_count)
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 5: LLM DIRECTOR
// ============================================================================

fn bench_llm_director(c: &mut Criterion) {
    let mut group = c.benchmark_group("director_adversarial/llm_director");

    // Test 1: Context building for LLM
    group.bench_function("llm_context_building", |bencher| {
        let boss = BossState::new(1_000_000.0);
        let metrics = EncounterMetrics {
            time_elapsed: 180.0,
            total_damage_dealt: 500_000.0,
            total_damage_taken: 50_000.0,
            phases_completed: 2,
            minions_spawned: 30,
            player_deaths: 1,
        };

        bencher.iter(|| {
            let context = format!(
                r#"{{
                    "boss_health_percent": {:.2},
                    "current_phase": {},
                    "time_elapsed": {:.1},
                    "player_dps": {:.0},
                    "damage_taken_rate": {:.0},
                    "minions_alive": {},
                    "enrage_remaining": {:.1}
                }}"#,
                boss.health_percent(),
                boss.phase,
                metrics.time_elapsed,
                metrics.total_damage_dealt / metrics.time_elapsed,
                metrics.total_damage_taken / metrics.time_elapsed,
                boss.minions.len(),
                boss.enrage_timer
            );

            std_black_box(context.len())
        });
    });

    // Test 2: Response parsing
    group.bench_function("llm_response_parsing_100", |bencher| {
        let responses: Vec<String> = (0..100)
            .map(|i| {
                format!(
                    r#"{{"action": "{}", "target": "player_{}", "intensity": {}}}"#,
                    match i % 4 {
                        0 => "spawn_wave",
                        1 => "buff_boss",
                        2 => "environmental",
                        _ => "collapse",
                    },
                    i % 10,
                    0.5 + (i % 5) as f32 * 0.1
                )
            })
            .collect();

        bencher.iter(|| {
            let parsed: Vec<(String, String, f32)> = responses
                .iter()
                .filter_map(|r| {
                    // Simple JSON-like parsing (not real JSON)
                    let action = r.split("\"action\": \"").nth(1)?.split('"').next()?;
                    let target = r.split("\"target\": \"").nth(1)?.split('"').next()?;
                    let intensity_str = r.split("\"intensity\": ").nth(1)?;
                    let intensity: f32 = intensity_str.split('}').next()?.parse().ok()?;

                    Some((action.to_string(), target.to_string(), intensity))
                })
                .collect();

            std_black_box(parsed.len())
        });
    });

    // Test 3: Action validation
    group.bench_function("action_validation_200", |bencher| {
        let proposed_actions: Vec<DirectorOp> = (0..200)
            .map(|i| generate_director_op(
                match i % 5 {
                    0 => "Fortify",
                    1 => "SpawnWave",
                    2 => "Collapse",
                    3 => "BuffBoss",
                    _ => "EnvironmentalHazard",
                },
                i as u64,
            ))
            .collect();

        let current_phase = 2;
        let phases = generate_phase_configs();
        let allowed_ops: Vec<&String> = phases
            .iter()
            .filter(|p| p.phase_number <= current_phase)
            .flat_map(|p| &p.available_ops)
            .collect();

        bencher.iter(|| {
            let validated: Vec<&DirectorOp> = proposed_actions
                .iter()
                .filter(|op| {
                    let op_name = match op {
                        DirectorOp::Fortify { .. } => "Fortify",
                        DirectorOp::SpawnWave { .. } => "SpawnWave",
                        DirectorOp::Collapse { .. } => "Collapse",
                        DirectorOp::BuffBoss { .. } => "BuffBoss",
                        DirectorOp::SpawnMinion { .. } => "SpawnMinion",
                        DirectorOp::EnvironmentalHazard { .. } => "EnvironmentalHazard",
                        DirectorOp::PhaseTransition { .. } => "PhaseTransition",
                    };

                    allowed_ops.iter().any(|a| a.as_str() == op_name)
                })
                .collect();

            std_black_box(validated.len())
        });
    });

    // Test 4: Fallback decision making
    group.bench_function("fallback_decision_100", |bencher| {
        let bosses: Vec<BossState> = (0..100)
            .map(|i| {
                let mut boss = BossState::new(1_000_000.0);
                boss.health = (100 - i) as f32 * 10000.0;
                boss.phase = (i / 25 + 1) as u32;
                boss
            })
            .collect();

        bencher.iter(|| {
            let decisions: Vec<DirectorOp> = bosses
                .iter()
                .map(|boss| {
                    // Heuristic fallback when LLM is unavailable
                    if boss.health_percent() < 0.2 {
                        DirectorOp::Collapse { area_radius: 30.0 }
                    } else if boss.minions.len() < 5 {
                        DirectorOp::SpawnWave {
                            enemy_types: vec!["minion".to_string()],
                            count: 5,
                        }
                    } else if boss.active_buffs.len() < 2 {
                        DirectorOp::BuffBoss {
                            stat: "damage".to_string(),
                            amount: 0.2,
                        }
                    } else {
                        DirectorOp::Fortify {
                            position: boss.position,
                            strength: 1.5,
                        }
                    }
                })
                .collect();

            std_black_box(decisions.len())
        });
    });

    group.finish();
}

// ============================================================================
// CATEGORY 6: DIFFICULTY SCALING
// ============================================================================

fn bench_difficulty_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("director_adversarial/difficulty_scaling");

    // Test 1: Player skill assessment
    group.bench_function("skill_assessment_50", |bencher| {
        let player_histories: Vec<Vec<(f32, u32, f32)>> = (0..50)
            .map(|i| {
                (0..10)
                    .map(|j| {
                        (
                            300.0 + (i + j) as f32 * 5.0,  // time
                            (i + j) as u32 % 4,            // deaths
                            100.0 - (i + j) as f32 * 2.0,  // rating
                        )
                    })
                    .collect()
            })
            .collect();

        bencher.iter(|| {
            let skill_ratings: Vec<f32> = player_histories
                .iter()
                .map(|history| {
                    let avg_time: f32 = history.iter().map(|(t, _, _)| t).sum::<f32>() / history.len() as f32;
                    let avg_deaths: f32 = history.iter().map(|(_, d, _)| *d as f32).sum::<f32>() / history.len() as f32;
                    let avg_rating: f32 = history.iter().map(|(_, _, r)| r).sum::<f32>() / history.len() as f32;

                    let time_factor = 600.0 / avg_time;
                    let death_factor = 1.0 / (1.0 + avg_deaths);
                    let rating_factor = avg_rating / 100.0;

                    (time_factor * death_factor * rating_factor * 100.0).min(100.0)
                })
                .collect();

            let avg_skill: f32 = skill_ratings.iter().sum::<f32>() / skill_ratings.len() as f32;
            std_black_box(avg_skill)
        });
    });

    // Test 2: Dynamic difficulty adjustment
    group.bench_function("dynamic_difficulty_adjustment", |bencher| {
        let mut difficulty_levels: Vec<f32> = (0..100).map(|i| 0.5 + (i as f32 * 0.005)).collect();

        let performance_samples: Vec<f32> = (0..100)
            .map(|i| 0.3 + (i as f32 * 0.01)) // Performance from 0.3 to 1.3
            .collect();

        bencher.iter(|| {
            for (i, &performance) in performance_samples.iter().enumerate() {
                let target = 0.7f32; // Target 70% success rate
                let diff = performance - target;

                // Adjust difficulty
                let adjustment = diff * 0.1; // 10% of difference
                difficulty_levels[i] = (difficulty_levels[i] + adjustment).clamp(0.1, 2.0);
            }

            let avg_difficulty: f32 = difficulty_levels.iter().sum::<f32>() / difficulty_levels.len() as f32;
            std_black_box(avg_difficulty)
        });
    });

    // Test 3: Stat scaling
    group.bench_function("stat_scaling_1000", |bencher| {
        let base_stats: Vec<(f32, f32, f32)> = (0..1000)
            .map(|i| (1000.0 + i as f32, 100.0 + i as f32 * 0.1, 50.0))
            .collect();

        let difficulty_multiplier = 1.5f32;

        bencher.iter(|| {
            let scaled_stats: Vec<(f32, f32, f32)> = base_stats
                .iter()
                .map(|(health, damage, speed)| {
                    (
                        health * difficulty_multiplier,
                        damage * difficulty_multiplier.sqrt(),
                        speed * (1.0 + (difficulty_multiplier - 1.0) * 0.2),
                    )
                })
                .collect();

            let total_health: f32 = scaled_stats.iter().map(|(h, _, _)| h).sum();
            std_black_box(total_health)
        });
    });

    // Test 4: Loot quality adjustment
    group.bench_function("loot_quality_adjustment_200", |bencher| {
        let base_loot: Vec<(String, u32, f32)> = (0..200)
            .map(|i| {
                (
                    format!("item_{}", i),
                    i as u32 * 10,           // base value
                    0.1 + (i % 10) as f32 * 0.05, // base drop chance
                )
            })
            .collect();

        let player_skill = 0.8f32;

        bencher.iter(|| {
            let adjusted_loot: Vec<(String, u32, f32)> = base_loot
                .iter()
                .map(|(name, value, chance)| {
                    let skill_bonus = player_skill * 0.5;
                    let adjusted_value = (*value as f32 * (1.0 + skill_bonus)) as u32;
                    let adjusted_chance = chance * (1.0 + skill_bonus * 0.2);

                    (name.clone(), adjusted_value, adjusted_chance.min(1.0))
                })
                .collect();

            let total_value: u32 = adjusted_loot.iter().map(|(_, v, _)| v).sum();
            std_black_box(total_value)
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_boss_planning,
    bench_phase_transitions,
    bench_minion_management,
    bench_encounter_metrics,
    bench_llm_director,
    bench_difficulty_scaling,
);

criterion_main!(benches);
