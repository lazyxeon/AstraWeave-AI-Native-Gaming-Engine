//! Determinism tests for astraweave-weaving
//! 
//! These tests validate that weaving operations produce identical results
//! across multiple runs with the same RNG seed.

mod common;

use astraweave_weaving::*;
use common::*;

#[test]
fn test_fixed_seed_replay_3_runs() {
    // Run the same sequence 3 times with seed 11111
    assert_deterministic_behavior(11111, |rng| {
        let mut adjudicator = create_test_adjudicator();
        adjudicator.begin_tick();

        // Perform 10 weaving operations with RNG-influenced priorities
        let mut total_approved = 0;
        for i in 0..10 {
            let priority = (rng.next() % 100) as f32 / 100.0;
            let cost = (rng.next() % 15) as u32 + 1;
            
            let intents = vec![create_test_intent(&format!("intent_{}", i), priority, cost)];
            let approved = adjudicator.adjudicate(intents);
            total_approved += approved.len();
            
            adjudicator.begin_tick(); // Reset for next iteration
        }
        
        total_approved
    });
}

#[test]
fn test_fixed_seed_replay_100_operations() {
    // Test longer sequence for drift detection
    assert_deterministic_behavior(22222, |rng| {
        let mut adjudicator = create_test_adjudicator();
        let mut state_hashes = Vec::new();

        for _ in 0..100 {
            adjudicator.begin_tick();
            
            let priority = (rng.next() % 100) as f32 / 100.0;
            let cost = (rng.next() % 10) as u32 + 1;
            
            let intents = vec![create_test_intent("weave", priority, cost)];
            adjudicator.adjudicate(intents);
            
            state_hashes.push(hash_adjudicator_state(&adjudicator));
        }
        
        state_hashes
    });
}

#[test]
fn test_event_ordering_guarantees() {
    // Verify that intents are always processed in deterministic order
    assert_deterministic_behavior(33333, |rng| {
        let mut adjudicator = create_test_adjudicator();
        adjudicator.begin_tick();

        // Create multiple intents with RNG-based priorities
        let mut intents = Vec::new();
        for i in 0..5 {
            let priority = (rng.next() % 100) as f32 / 100.0;
            intents.push(create_test_intent(&format!("intent_{}", i), priority, 3));
        }

        let approved = adjudicator.adjudicate(intents);
        
        // Return the kinds in approval order
        approved.iter().map(|intent| intent.kind.clone()).collect::<Vec<_>>()
    });
}

#[test]
fn test_storm_choice_branch_consistency() {
    // Simulate storm choice branching with deterministic RNG
    assert_deterministic_behavior(44444, |rng| {
        let mut adjudicator = create_test_adjudicator();
        
        // Simulate player choice based on RNG (e.g., 0 = Stabilize, 1 = Redirect)
        let choice = (rng.next() % 2) as u32;
        
        // Apply different weaving intents based on choice
        let mut results = Vec::new();
        for tick in 0..10 {
            adjudicator.begin_tick();
            
            let intent_kind = if choice == 0 {
                "stabilize_anchor"
            } else {
                "redirect_storm"
            };
            
            let priority = 0.8 + (tick as f32 * 0.01);
            let intents = vec![create_test_intent(intent_kind, priority, 5)];
            let approved = adjudicator.adjudicate(intents);
            
            results.push((choice, approved.len(), tick));
        }
        
        results
    });
}

#[test]
fn test_boss_adaptive_ability_determinism() {
    // Boss should always select same adaptive ability given same player tactics
    assert_deterministic_behavior(55555, |rng| {
        let mut adjudicator = create_test_adjudicator();
        
        // Simulate player damage breakdown (RNG-based but deterministic)
        let player_melee_damage = rng.next() % 100;
        let player_ranged_damage = rng.next() % 100;
        let total_damage = player_melee_damage + player_ranged_damage;
        
        let melee_ratio = if total_damage > 0 {
            player_melee_damage as f32 / total_damage as f32
        } else {
            0.0
        };
        
        // Boss adaptive decision (deterministic based on ratio)
        let selected_ability = if melee_ratio >= 0.6 {
            "CounterShockAura" // Reflects melee damage
        } else {
            "AntiRangedField" // Reduces ranged damage
        };
        
        // Apply the selected ability
        adjudicator.begin_tick();
        let intents = vec![create_test_intent(selected_ability, 1.0, 10)];
        let approved = adjudicator.adjudicate(intents);
        
        (selected_ability.to_string(), approved.len(), melee_ratio)
    });
}

#[test]
fn test_companion_goap_plan_determinism() {
    // Companion AI should produce identical GOAP plans with same world state
    assert_deterministic_behavior(66666, |rng| {
        let mut adjudicator = create_test_adjudicator();
        
        // Simulate world state metrics (RNG-based but deterministic)
        let player_health = (rng.next() % 100) as u32;
        let anchor_stability = (rng.next() % 100) as f32 / 100.0;
        let enemy_distance = (rng.next() % 20) as f32;
        
        // Companion GOAP planning (simplified)
        let mut plan = Vec::new();
        
        // Goal 1: ProtectPlayer (highest priority)
        if player_health < 40 {
            plan.push(create_test_intent("HealPlayer", 1.0, 5));
        }
        
        // Goal 2: StabilizeThreads
        if anchor_stability < 0.5 {
            plan.push(create_test_intent("CastStabilityPulse", 0.8, 3));
        }
        
        // Goal 3: MaintainPositioning
        if enemy_distance < 3.0 {
            plan.push(create_test_intent("Reposition", 0.6, 2));
        }
        
        // Execute plan
        adjudicator.begin_tick();
        let approved = adjudicator.adjudicate(plan);
        
        (player_health, anchor_stability, approved.len())
    });
}

#[test]
fn test_weaving_pattern_detection_determinism() {
    // Pattern detection should produce identical patterns with same metrics
    assert_deterministic_behavior(77777, |rng| {
        let critical_health_count = (rng.next() % 10) as usize;
        let avg_health = (rng.next() % 100) as f32 / 100.0;
        let recent_damage_events = (rng.next() % 50) as usize;
        
        let metrics = create_test_metrics(critical_health_count, avg_health, recent_damage_events);
        
        // Run pattern detectors
        let low_health_detector = create_low_health_detector();
        let resource_detector = create_resource_scarcity_detector();
        let combat_detector = create_combat_intensity_detector();
        
        let patterns_low_health = low_health_detector.detect(&metrics);
        let patterns_resource = resource_detector.detect(&metrics);
        let patterns_combat = combat_detector.detect(&metrics);
        
        (
            patterns_low_health.len(),
            patterns_resource.len(),
            patterns_combat.len(),
            avg_health,
        )
    });
}

#[test]
fn test_tutorial_sequence_determinism() {
    // Tutorial progression should be deterministic (Z1 Frayed Causeway)
    assert_deterministic_behavior(88888, |rng| {
        let mut adjudicator = create_test_adjudicator();
        let mut anchor_stabilization_order = Vec::new();
        
        // Simulate 3 anchors being stabilized
        for anchor_idx in 0..3 {
            adjudicator.begin_tick();
            
            // Player attempts stabilization (RNG for success timing)
            let attempt_delay = (rng.next() % 5) as u32;
            
            if attempt_delay < 3 {
                // Attempt succeeds
                let intent = create_test_intent(
                    &format!("stabilize_anchor_{}", anchor_idx),
                    0.9,
                    5,
                );
                let approved = adjudicator.adjudicate(vec![intent]);
                
                if !approved.is_empty() {
                    anchor_stabilization_order.push(anchor_idx);
                }
            }
        }
        
        anchor_stabilization_order
    });
}

#[test]
fn test_echo_shard_collection_determinism() {
    // Echo Shard resource tracking should be deterministic
    assert_deterministic_behavior(99999, |rng| {
        let mut echo_shard_count = 0u32;
        let mut collection_order = Vec::new();
        
        // Simulate 5 Echo Shards in world
        for shard_id in 0..5 {
            // Player proximity check (RNG-based but deterministic)
            let distance = (rng.next() % 10) as f32;
            
            if distance < 2.0 {
                // Player is close enough to collect
                echo_shard_count += 1;
                collection_order.push(shard_id);
            }
        }
        
        (echo_shard_count, collection_order)
    });
}

#[test]
fn test_determinism_with_cooldowns() {
    // Verify determinism when cooldowns are involved
    assert_deterministic_behavior(111111, |rng| {
        let mut adjudicator = create_test_adjudicator();
        let mut approved_count = 0;
        
        for tick in 0..20 {
            adjudicator.begin_tick();
            
            // Try to trigger an intent with cooldown
            let intent = create_test_intent("aid_event_action", 0.9, 5)
                .with_cooldown("aid_event");
            
            let approved = adjudicator.adjudicate(vec![intent]);
            approved_count += approved.len();
            
            // Verify cooldown state is deterministic
            if tick == 0 {
                // First tick should approve
                assert_eq!(approved.len(), 1, "First tick should approve intent");
            } else if tick < 300 {
                // Should be on cooldown (300 ticks)
                assert_eq!(approved.len(), 0, "Should be on cooldown at tick {}", tick);
            }
        }
        
        approved_count
    });
}
