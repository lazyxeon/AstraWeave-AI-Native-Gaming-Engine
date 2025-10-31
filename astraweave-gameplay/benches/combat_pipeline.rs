//! Combat Pipeline Integration Benchmark
//!
//! Benchmarks the full combat execution pipeline:
//! **AI Perception → Combat Decision → Attack Sweep → Physics Collision → Damage Application → Stats Update**
//!
//! **Targets**:
//! - <100µs per attack (10,000 attacks/frame capacity @ 60 FPS)
//! - <1ms for 10 simultaneous attacks (multi-combatant scenario)
//! - <10ms for 100 simultaneous attacks (large battle)
//!
//! **Scenarios**:
//! 1. Single attack (baseline overhead)
//! 2. Multi-target attack (1 attacker, 5 targets)
//! 3. Multi-attacker scenario (10 attackers, 5 targets each)
//! 4. Large battle (100 attacks simultaneously)

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use astraweave_gameplay::{
    combat_physics::perform_attack_sweep,
    Combatant, IFrame, Parry, DamageType,
};
use astraweave_physics::{PhysicsWorld, Layers};
use glam::Vec3;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a physics world with N combatants
fn create_combat_world(combatant_count: usize) -> (PhysicsWorld, Vec<Combatant>) {
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let mut combatants = vec![];
    
    for i in 0..combatant_count {
        // Create dynamic box for combatant (using add_dynamic_box)
        let position = Vec3::new((i as f32) * 2.0, 1.0, 0.0);
        let half_extents = Vec3::new(0.5, 1.0, 0.5); // Character-sized box
        let body_id = physics.add_dynamic_box(
            position,
            half_extents,
            75.0, // mass (average human weight in kg)
            Layers::DEFAULT,
        );
        
        combatants.push(Combatant {
            body: body_id,
            stats: astraweave_gameplay::Stats {
                hp: 100,
                max_hp: 100,
                armor: 10,
                attack_power: 20,
            },
            iframes: None,
            parry: None,
        });
    }
    
    (physics, combatants)
}

/// Create combatants with parry windows active
fn create_parrying_combatants(
    physics: &mut PhysicsWorld,
    count: usize,
    parry_window: f32,
) -> Vec<Combatant> {
    let mut combatants = vec![];
    
    for i in 0..count {
        let position = Vec3::new((i as f32) * 2.0, 1.0, 0.0);
        let half_extents = Vec3::new(0.5, 1.0, 0.5);
        let body_id = physics.add_dynamic_box(
            position,
            half_extents,
            75.0,
            Layers::DEFAULT,
        );
        
        combatants.push(Combatant {
            body: body_id,
            stats: astraweave_gameplay::Stats {
                hp: 100,
                max_hp: 100,
                armor: 10,
                attack_power: 20,
            },
            iframes: None,
            parry: Some(Parry {
                window: parry_window,
                active: true,
            }),
        });
    }
    
    combatants
}

/// Create combatants with invincibility frames
fn create_iframe_combatants(physics: &mut PhysicsWorld, count: usize) -> Vec<Combatant> {
    let mut combatants = vec![];
    
    for i in 0..count {
        let position = Vec3::new((i as f32) * 2.0, 1.0, 0.0);
        let half_extents = Vec3::new(0.5, 1.0, 0.5);
        let body_id = physics.add_dynamic_box(
            position,
            half_extents,
            75.0,
            Layers::DEFAULT,
        );
        
        combatants.push(Combatant {
            body: body_id,
            stats: astraweave_gameplay::Stats {
                hp: 100,
                max_hp: 100,
                armor: 10,
                attack_power: 20,
            },
            iframes: Some(IFrame { time_left: 0.5 }),
            parry: None,
        });
    }
    
    combatants
}

/// Get position from PhysicsWorld using body_transform
fn get_position(physics: &PhysicsWorld, body_id: u64) -> Vec3 {
    physics
        .body_transform(body_id)
        .map(|mat| mat.to_scale_rotation_translation().2)
        .unwrap_or(Vec3::ZERO)
}

/// Single attack execution
fn execute_single_attack(
    physics: &mut PhysicsWorld,
    attacker: &Combatant,
    targets: &mut [Combatant],
) -> Option<astraweave_gameplay::combat_physics::HitResult> {
    let attacker_pos = get_position(physics, attacker.body);
    let target_pos = get_position(physics, targets[0].body);
    let attack_dir = (target_pos - attacker_pos).normalize_or_zero();
    
    perform_attack_sweep(
        physics,
        attacker.body,
        attacker_pos,
        attacker_pos + attack_dir * 5.0, // 5m attack range
        0.5, // attack radius
        20, // base damage
        DamageType::Physical,
        targets,
    )
}

/// Multi-attack execution (N attackers)
fn execute_multi_attack(
    physics: &mut PhysicsWorld,
    attackers: &[Combatant],
    targets: &mut [Combatant],
) -> usize {
    let mut total_hits = 0;
    
    for attacker in attackers {
        let attacker_pos = get_position(physics, attacker.body);
        // Attack first target
        if !targets.is_empty() {
            let target_pos = get_position(physics, targets[0].body);
            let attack_dir = (target_pos - attacker_pos).normalize_or_zero();
            
            if let Some(_hit) = perform_attack_sweep(
                physics,
                attacker.body,
                attacker_pos,
                attacker_pos + attack_dir * 5.0,
                0.5,
                20,
                DamageType::Physical,
                targets,
            ) {
                total_hits += 1;
            }
        }
    }
    
    total_hits
}

// ============================================================================
// Benchmarks
// ============================================================================

fn bench_single_attack(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_attack");
    
    let (mut physics, combatants) = create_combat_world(10);
    let attacker = combatants[0].clone();
    let mut targets = combatants[1..6].to_vec(); // 5 targets
    
    group.bench_function("5_targets_no_defense", |b| {
        b.iter(|| {
            execute_single_attack(
                black_box(&mut physics),
                black_box(&attacker),
                black_box(&mut targets),
            )
        })
    });
    
    group.finish();
}

fn bench_single_attack_with_parry(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_attack_parry");
    
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
    let attacker_body = physics.add_dynamic_box(
        attacker_pos,
        Vec3::new(0.5, 1.0, 0.5),
        75.0,
        Layers::DEFAULT,
    );
    let attacker = Combatant {
        body: attacker_body,
        stats: astraweave_gameplay::Stats {
            hp: 100,
            max_hp: 100,
            armor: 10,
            attack_power: 20,
        },
        iframes: None,
        parry: None,
    };
    
    let mut targets = create_parrying_combatants(&mut physics, 5, 0.5);
    
    group.bench_function("5_targets_50%_parry", |b| {
        b.iter(|| {
            execute_single_attack(
                black_box(&mut physics),
                black_box(&attacker),
                black_box(&mut targets),
            )
        })
    });
    
    group.finish();
}

fn bench_single_attack_with_iframe(c: &mut Criterion) {
    let mut group = c.benchmark_group("single_attack_iframe");
    
    let mut physics = PhysicsWorld::new(Vec3::new(0.0, -9.81, 0.0));
    let attacker_pos = Vec3::new(0.0, 1.0, 0.0);
    let attacker_body = physics.add_dynamic_box(
        attacker_pos,
        Vec3::new(0.5, 1.0, 0.5),
        75.0,
        Layers::DEFAULT,
    );
    let attacker = Combatant {
        body: attacker_body,
        stats: astraweave_gameplay::Stats {
            hp: 100,
            max_hp: 100,
            armor: 10,
            attack_power: 20,
        },
        iframes: None,
        parry: None,
    };
    
    let mut targets = create_iframe_combatants(&mut physics, 5);
    
    group.bench_function("5_targets_50%_iframe", |b| {
        b.iter(|| {
            execute_single_attack(
                black_box(&mut physics),
                black_box(&attacker),
                black_box(&mut targets),
            )
        })
    });
    
    group.finish();
}

fn bench_multi_attacker_scenario(c: &mut Criterion) {
    let mut group = c.benchmark_group("multi_attacker_scenario");
    
    let scenarios = [
        (10, 5, "10_attackers_5_targets"),
        (50, 10, "50_attackers_10_targets"),
        (100, 20, "100_attackers_20_targets"),
    ];
    
    for (attacker_count, target_count, name) in scenarios {
        let total_combatants = attacker_count + target_count;
        let (mut physics, combatants) = create_combat_world(total_combatants);
        
        let attackers = combatants[0..attacker_count].to_vec();
        let mut targets = combatants[attacker_count..].to_vec();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &name,
            |b, _| {
                b.iter(|| {
                    execute_multi_attack(
                        black_box(&mut physics),
                        black_box(&attackers),
                        black_box(&mut targets),
                    )
                })
            }
        );
    }
    
    group.finish();
}

fn bench_large_battle(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_battle");
    
    // 200 combatants (100 vs 100)
    let (mut physics, combatants) = create_combat_world(200);
    let attackers = combatants[0..100].to_vec();
    let mut targets = combatants[100..].to_vec();
    
    group.bench_function("100v100_battle", |b| {
        b.iter(|| {
            execute_multi_attack(
                black_box(&mut physics),
                black_box(&attackers),
                black_box(&mut targets),
            )
        })
    });
    
    group.finish();
}

fn bench_attack_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("attack_scaling");
    
    let attacker_counts = [1, 5, 10, 25, 50, 100];
    
    for &attacker_count in &attacker_counts {
        let total_combatants = attacker_count + 20; // fixed 20 targets
        let (mut physics, combatants) = create_combat_world(total_combatants);
        
        let attackers = combatants[0..attacker_count].to_vec();
        let mut targets = combatants[attacker_count..].to_vec();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(attacker_count),
            &attacker_count,
            |b, _| {
                b.iter(|| {
                    execute_multi_attack(
                        black_box(&mut physics),
                        black_box(&attackers),
                        black_box(&mut targets),
                    )
                })
            }
        );
    }
    
    group.finish();
}

criterion_group!(
    benches,
    bench_single_attack,
    bench_single_attack_with_parry,
    bench_single_attack_with_iframe,
    bench_multi_attacker_scenario,
    bench_large_battle,
    bench_attack_scaling
);

criterion_main!(benches);
