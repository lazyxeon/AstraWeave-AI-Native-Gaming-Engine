//! Mutation-resistant tests for gameplay systems.
//!
//! These tests are designed to catch common mutations in:
//! - Stats and damage calculations
//! - Status effect processing
//! - Combat formulas

use crate::{DamageType, Stats, StatusEffect};

// ============================================================================
// Stats Creation Tests
// ============================================================================

mod stats_creation_tests {
    use super::*;

    #[test]
    fn test_stats_new_hp_value() {
        let stats = Stats::new(100);
        assert_eq!(stats.hp, 100, "HP should match constructor argument");
    }

    #[test]
    fn test_stats_new_default_stamina() {
        let stats = Stats::new(50);
        assert_eq!(stats.stamina, 100, "Default stamina should be 100");
    }

    #[test]
    fn test_stats_new_default_power() {
        let stats = Stats::new(50);
        assert_eq!(stats.power, 10, "Default power should be 10");
    }

    #[test]
    fn test_stats_new_default_defense() {
        let stats = Stats::new(50);
        assert_eq!(stats.defense, 5, "Default defense should be 5");
    }

    #[test]
    fn test_stats_new_default_echo_amp() {
        let stats = Stats::new(50);
        assert!((stats.echo_amp - 1.0).abs() < f32::EPSILON, "Default echo_amp should be 1.0");
    }

    #[test]
    fn test_stats_new_empty_effects() {
        let stats = Stats::new(50);
        assert!(stats.effects.is_empty(), "New stats should have no effects");
    }
}

// ============================================================================
// Damage Calculation Tests
// ============================================================================

mod damage_tests {
    use super::*;

    #[test]
    fn test_damage_mitigation_formula() {
        let mut stats = Stats::new(100);
        stats.defense = 10;
        // mitigated = max(20 - 10 * 0.5, 1) = max(15, 1) = 15
        let damage = stats.apply_damage(20, DamageType::Physical);
        assert_eq!(damage, 15, "Damage should be mitigated by defense * 0.5");
    }

    #[test]
    fn test_damage_reduces_hp() {
        let mut stats = Stats::new(100);
        stats.defense = 0;
        stats.apply_damage(30, DamageType::Physical);
        assert_eq!(stats.hp, 70, "HP should be reduced by damage");
    }

    #[test]
    fn test_damage_minimum_is_one() {
        let mut stats = Stats::new(100);
        stats.defense = 1000; // Very high defense
        let damage = stats.apply_damage(1, DamageType::Physical);
        assert_eq!(damage, 1, "Minimum damage should be 1");
        assert_eq!(stats.hp, 99, "HP should decrease by at least 1");
    }

    #[test]
    fn test_damage_zero_defense() {
        let mut stats = Stats::new(100);
        stats.defense = 0;
        let damage = stats.apply_damage(25, DamageType::Fire);
        assert_eq!(damage, 25, "With zero defense, full damage should apply");
    }

    #[test]
    fn test_damage_can_reduce_hp_below_zero() {
        let mut stats = Stats::new(10);
        stats.defense = 0;
        stats.apply_damage(50, DamageType::Physical);
        assert!(stats.hp < 0, "HP can go below zero");
    }

    #[test]
    fn test_damage_returns_mitigated_amount() {
        let mut stats = Stats::new(100);
        stats.defense = 20;
        // mitigated = max(30 - 10, 1) = 20
        let returned = stats.apply_damage(30, DamageType::Physical);
        let expected = (30.0_f32 - 20.0_f32 * 0.5_f32).max(1.0_f32) as i32;
        assert_eq!(returned, expected, "Should return the mitigated damage amount");
    }
}

// ============================================================================
// Status Effect Tests
// ============================================================================

mod status_effect_tests {
    use super::*;

    #[test]
    fn test_bleed_deals_damage_over_time() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 5.0 });
        let dot = stats.tick(1.0);
        assert_eq!(dot, 10, "Bleed should deal dps * dt damage");
        assert_eq!(stats.hp, 90, "HP should be reduced by bleed damage");
    }

    #[test]
    fn test_bleed_time_decreases() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 5.0, time: 3.0 });
        stats.tick(1.0);
        if let StatusEffect::Bleed { time, .. } = &stats.effects[0] {
            assert!((time - 2.0).abs() < f32::EPSILON, "Bleed time should decrease by dt");
        } else {
            panic!("Effect should be Bleed");
        }
    }

    #[test]
    fn test_bleed_expires_when_time_zero() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 5.0, time: 0.5 });
        stats.tick(1.0);
        assert!(stats.effects.is_empty(), "Bleed should expire when time <= 0");
    }

    #[test]
    fn test_stagger_does_not_deal_damage() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Stagger { time: 2.0 });
        let dot = stats.tick(1.0);
        assert_eq!(dot, 0, "Stagger should not deal damage");
        assert_eq!(stats.hp, 100, "HP should not change from stagger");
    }

    #[test]
    fn test_stagger_expires() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Stagger { time: 0.5 });
        stats.tick(1.0);
        assert!(stats.effects.is_empty(), "Stagger should expire");
    }

    #[test]
    fn test_chill_does_not_deal_damage() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Chill { slow: 0.5, time: 3.0 });
        let dot = stats.tick(1.0);
        assert_eq!(dot, 0, "Chill should not deal damage");
        assert_eq!(stats.hp, 100, "HP should not change from chill");
    }

    #[test]
    fn test_chill_expires() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Chill { slow: 0.5, time: 0.3 });
        stats.tick(1.0);
        assert!(stats.effects.is_empty(), "Chill should expire");
    }

    #[test]
    fn test_multiple_bleeds_stack() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 5.0, time: 2.0 });
        stats.effects.push(StatusEffect::Bleed { dps: 3.0, time: 2.0 });
        let dot = stats.tick(1.0);
        assert_eq!(dot, 8, "Multiple bleeds should stack: 5 + 3 = 8");
        assert_eq!(stats.hp, 92);
    }

    #[test]
    fn test_no_effects_no_damage() {
        let mut stats = Stats::new(100);
        let dot = stats.tick(1.0);
        assert_eq!(dot, 0, "No effects should deal no damage");
        assert_eq!(stats.hp, 100, "HP should not change");
    }

    #[test]
    fn test_partial_tick_bleed() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 2.0 });
        let dot = stats.tick(0.5); // Half second tick
        assert_eq!(dot, 5, "Bleed damage should scale with dt: 10 * 0.5 = 5");
        assert_eq!(stats.hp, 95);
    }

    #[test]
    fn test_mixed_effects_only_bleed_deals_damage() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 2.0 });
        stats.effects.push(StatusEffect::Stagger { time: 2.0 });
        stats.effects.push(StatusEffect::Chill { slow: 0.5, time: 2.0 });
        let dot = stats.tick(1.0);
        assert_eq!(dot, 10, "Only bleed should contribute to DoT");
        assert_eq!(stats.hp, 90);
        assert_eq!(stats.effects.len(), 3, "All effects should remain");
    }
}

// ============================================================================
// Tick Timing Tests
// ============================================================================

mod tick_timing_tests {
    use super::*;

    #[test]
    fn test_tick_zero_dt() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 100.0, time: 2.0 });
        let dot = stats.tick(0.0);
        assert_eq!(dot, 0, "Zero dt should deal no damage");
        assert_eq!(stats.hp, 100);
    }

    #[test]
    fn test_tick_large_dt_expires_effect() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 1.0 });
        let dot = stats.tick(2.0); // dt > time
        // Deals 10 * 2.0 = 20 damage even though effect expires
        assert_eq!(dot, 20);
        assert!(stats.effects.is_empty(), "Effect should expire");
    }

    #[test]
    fn test_effect_removal_order() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 1.0, time: 0.5 }); // Will expire
        stats.effects.push(StatusEffect::Bleed { dps: 2.0, time: 2.0 }); // Will remain
        stats.effects.push(StatusEffect::Bleed { dps: 3.0, time: 0.3 }); // Will expire
        
        stats.tick(1.0);
        
        // Only the 2.0 time bleed should remain
        assert_eq!(stats.effects.len(), 1);
        if let StatusEffect::Bleed { dps, .. } = stats.effects[0] {
            assert!((dps - 2.0).abs() < f32::EPSILON, "The 2.0 dps bleed should remain");
        }
    }
}

// ============================================================================
// Behavioral Correctness Tests
// ============================================================================

mod behavioral_tests {
    use super::*;

    #[test]
    fn test_damage_is_always_at_least_one() {
        for defense in 0..100 {
            let mut stats = Stats::new(100);
            stats.defense = defense;
            let damage = stats.apply_damage(1, DamageType::Physical);
            assert!(damage >= 1, "Damage should always be at least 1, got {} with defense {}", damage, defense);
        }
    }

    #[test]
    fn test_bleed_damage_proportional_to_time() {
        let mut stats1 = Stats::new(1000);
        stats1.effects.push(StatusEffect::Bleed { dps: 10.0, time: 10.0 });
        let dot1 = stats1.tick(1.0);

        let mut stats2 = Stats::new(1000);
        stats2.effects.push(StatusEffect::Bleed { dps: 10.0, time: 10.0 });
        let dot2 = stats2.tick(2.0);

        assert_eq!(dot2, dot1 * 2, "Double dt should deal double damage");
    }

    #[test]
    fn test_hp_change_equals_returned_damage() {
        let mut stats = Stats::new(100);
        stats.defense = 8;
        let initial_hp = stats.hp;
        let damage = stats.apply_damage(25, DamageType::Fire);
        let hp_lost = initial_hp - stats.hp;
        assert_eq!(damage, hp_lost, "Returned damage should equal HP lost");
    }

    #[test]
    fn test_tick_hp_change_equals_returned_dot() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 7.0, time: 5.0 });
        let initial_hp = stats.hp;
        let dot = stats.tick(1.0);
        let hp_lost = initial_hp - stats.hp;
        assert_eq!(dot, hp_lost, "Returned DoT should equal HP lost");
    }

    #[test]
    fn test_defense_reduces_damage() {
        let mut low_def = Stats::new(100);
        low_def.defense = 0;
        let damage_low = low_def.apply_damage(20, DamageType::Physical);

        let mut high_def = Stats::new(100);
        high_def.defense = 10;
        let damage_high = high_def.apply_damage(20, DamageType::Physical);

        assert!(damage_high < damage_low, "Higher defense should reduce damage");
    }

    #[test]
    fn test_stats_serialization_roundtrip() {
        let mut stats = Stats::new(75);
        stats.power = 15;
        stats.defense = 8;
        stats.effects.push(StatusEffect::Bleed { dps: 5.0, time: 2.0 });

        let json = serde_json::to_string(&stats).expect("serialize");
        let deserialized: Stats = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.hp, 75);
        assert_eq!(deserialized.power, 15);
        assert_eq!(deserialized.defense, 8);
        assert_eq!(deserialized.effects.len(), 1);
    }
}

// ============================================================================
// MUTATION-RESISTANT TEST MODULE 1: Boundary Condition Tests
// ============================================================================
// These tests catch mutations that change boundary conditions:
// - < changed to <=
// - > changed to >=
// - Off-by-one errors in loop bounds
// - Fence-post errors

mod boundary_condition_tests {
    use super::*;

    // --- Stats HP Boundaries ---

    #[test]
    fn test_stats_hp_zero_boundary() {
        let stats = Stats::new(0);
        assert_eq!(stats.hp, 0, "HP should be exactly 0");
    }

    #[test]
    fn test_stats_hp_one_boundary() {
        let stats = Stats::new(1);
        assert_eq!(stats.hp, 1, "HP should be exactly 1");
    }

    #[test]
    fn test_stats_hp_negative_boundary() {
        let stats = Stats::new(-1);
        assert_eq!(stats.hp, -1, "HP can be negative");
    }

    #[test]
    fn test_stats_hp_max_i32_boundary() {
        let stats = Stats::new(i32::MAX);
        assert_eq!(stats.hp, i32::MAX, "HP should handle max i32");
    }

    // --- Damage Mitigation Boundaries ---

    #[test]
    fn test_damage_mitigation_zero_defense_boundary() {
        let mut stats = Stats::new(100);
        stats.defense = 0;
        let damage = stats.apply_damage(10, DamageType::Physical);
        assert_eq!(damage, 10, "Zero defense should not mitigate damage");
    }

    #[test]
    fn test_damage_mitigation_one_defense_boundary() {
        let mut stats = Stats::new(100);
        stats.defense = 1;
        // mitigated = max(10 - 0.5, 1) = 9.5 -> 9
        let damage = stats.apply_damage(10, DamageType::Physical);
        let expected = (10.0_f32 - 1.0_f32 * 0.5_f32).max(1.0_f32) as i32;
        assert_eq!(damage, expected, "One defense should mitigate by 0.5");
    }

    #[test]
    fn test_damage_minimum_one_exactly() {
        let mut stats = Stats::new(100);
        stats.defense = 100;
        let damage = stats.apply_damage(1, DamageType::Physical);
        assert_eq!(damage, 1, "Damage should be at least 1, not 0");
    }

    #[test]
    fn test_damage_minimum_boundary_defense_exactly_needed() {
        let mut stats = Stats::new(100);
        // We want damage - defense*0.5 = 1 exactly
        // 10 - defense*0.5 = 1 => defense = 18
        stats.defense = 18;
        let damage = stats.apply_damage(10, DamageType::Physical);
        // max(10 - 9, 1) = max(1, 1) = 1
        assert_eq!(damage, 1, "Damage exactly at minimum boundary");
    }

    #[test]
    fn test_damage_minimum_boundary_just_above() {
        let mut stats = Stats::new(100);
        stats.defense = 16;
        let damage = stats.apply_damage(10, DamageType::Physical);
        // max(10 - 8, 1) = max(2, 1) = 2
        assert_eq!(damage, 2, "Damage just above minimum boundary");
    }

    // --- Effect Time Boundaries ---

    #[test]
    fn test_bleed_time_exactly_zero_expires() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 1.0 });
        stats.tick(1.0); // time becomes 0.0
        assert!(stats.effects.is_empty(), "Effect with time 0.0 should expire (> 0.0 check)");
    }

    #[test]
    fn test_bleed_time_just_above_zero_persists() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 1.001 });
        stats.tick(1.0); // time becomes 0.001
        assert_eq!(stats.effects.len(), 1, "Effect with time 0.001 should persist (> 0.0 check)");
    }

    #[test]
    fn test_bleed_time_just_below_zero_expires() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 0.5 });
        stats.tick(1.0); // time becomes -0.5
        assert!(stats.effects.is_empty(), "Effect with negative time should expire");
    }

    #[test]
    fn test_stagger_time_exactly_zero_expires() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Stagger { time: 1.0 });
        stats.tick(1.0);
        assert!(stats.effects.is_empty(), "Stagger with time 0.0 should expire");
    }

    #[test]
    fn test_chill_time_exactly_zero_expires() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Chill { slow: 0.5, time: 1.0 });
        stats.tick(1.0);
        assert!(stats.effects.is_empty(), "Chill with time 0.0 should expire");
    }

    // --- Tick dt Boundaries ---

    #[test]
    fn test_tick_dt_zero_boundary() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 100.0, time: 2.0 });
        let dot = stats.tick(0.0);
        assert_eq!(dot, 0, "Zero dt should deal zero damage");
        assert_eq!(stats.hp, 100, "HP should not change with zero dt");
    }

    #[test]
    fn test_tick_dt_epsilon_boundary() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 100.0, time: 2.0 });
        let dot = stats.tick(f32::EPSILON);
        // Very small damage, might be 0 due to integer conversion
        assert!(dot >= 0, "Epsilon dt should deal non-negative damage");
    }

    #[test]
    fn test_tick_dt_one_boundary() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 2.0 });
        let dot = stats.tick(1.0);
        assert_eq!(dot, 10, "dt=1.0 should deal exactly dps damage");
    }

    // --- Combo Window Boundaries ---

    #[test]
    fn test_combo_window_start_boundary() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.5, 1.0), // Window starts at 0.5
                damage: 10,
                reach: 10.0,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Tick to exactly window start
        state.tick(0.5, false, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        // Now at t_since_last = 0.5, should be in window
        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(hit, "Attack at exactly window start should hit");
    }

    #[test]
    fn test_combo_window_end_boundary() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.5, 1.0), // Window ends at 1.0
                damage: 10,
                reach: 10.0,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Tick to exactly window end
        state.tick(1.0, false, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        // Now at t_since_last = 1.0, should still be in window (<= check)
        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(hit, "Attack at exactly window end should hit");
    }

    #[test]
    fn test_combo_window_just_before_boundary() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.5, 1.0),
                damage: 10,
                reach: 10.0,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Tick to just before window
        state.tick(0.49, false, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        // Attack before window shouldn't hit
        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(!hit, "Attack just before window should not hit");
    }

    // --- Reach Boundaries ---

    #[test]
    fn test_attack_reach_exactly_at_boundary() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let reach = 5.0;
        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.0, 1.0),
                damage: 10,
                reach,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Exactly at reach distance
        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::new(reach, 0.0, 0.0), &Stats::new(100), None, &mut target);
        assert!(hit, "Attack at exactly reach should hit (<= check)");
    }

    #[test]
    fn test_attack_reach_just_beyond_boundary() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let reach = 5.0;
        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.0, 1.0),
                damage: 10,
                reach,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Just beyond reach
        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::new(reach + 0.01, 0.0, 0.0), &Stats::new(100), None, &mut target);
        assert!(!hit, "Attack just beyond reach should not hit");
    }

    // --- Inventory Resource Boundaries ---

    #[test]
    fn test_inventory_add_zero_resources() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();
        inv.add_resource(ResourceKind::Wood, 0);
        // Should still create entry with 0
        assert!(inv.resources.iter().any(|(k, c)| *k == ResourceKind::Wood && *c == 0));
    }

    #[test]
    fn test_inventory_remove_exactly_available() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();
        inv.add_resource(ResourceKind::Wood, 10);
        let result = inv.remove_resource(ResourceKind::Wood, 10);
        assert!(result, "Should succeed removing exactly available amount");
        assert!(inv.resources.iter().any(|(k, c)| *k == ResourceKind::Wood && *c == 0));
    }

    #[test]
    fn test_inventory_remove_one_more_than_available() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();
        inv.add_resource(ResourceKind::Wood, 10);
        let result = inv.remove_resource(ResourceKind::Wood, 11);
        assert!(!result, "Should fail removing more than available (>= check)");
    }

    // --- Quest Progress Boundaries ---

    #[test]
    fn test_quest_gather_exactly_enough() {
        use crate::quests::{Quest, QuestLog, Task, TaskKind};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![Task {
                id: "t1".into(),
                kind: TaskKind::Gather { kind: "wood".into(), count: 10 },
                done: false,
            }],
            reward_text: "".into(),
            completed: false,
        });

        log.progress_gather("q1", "wood", 10);
        let q = log.quests.get("q1").unwrap();
        assert!(q.tasks[0].done, "Task should be done with exactly enough");
        assert!(q.completed, "Quest should be completed");
    }

    #[test]
    fn test_quest_gather_one_short() {
        use crate::quests::{Quest, QuestLog, Task, TaskKind};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![Task {
                id: "t1".into(),
                kind: TaskKind::Gather { kind: "wood".into(), count: 10 },
                done: false,
            }],
            reward_text: "".into(),
            completed: false,
        });

        log.progress_gather("q1", "wood", 9);
        let q = log.quests.get("q1").unwrap();
        assert!(!q.tasks[0].done, "Task should not be done with one short");
    }

    #[test]
    fn test_quest_gather_more_than_needed() {
        use crate::quests::{Quest, QuestLog, Task, TaskKind};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![Task {
                id: "t1".into(),
                kind: TaskKind::Gather { kind: "wood".into(), count: 10 },
                done: false,
            }],
            reward_text: "".into(),
            completed: false,
        });

        log.progress_gather("q1", "wood", 100);
        let q = log.quests.get("q1").unwrap();
        assert!(q.tasks[0].done, "Task should be done with more than needed");
    }
}

// ============================================================================
// MUTATION-RESISTANT TEST MODULE 2: Comparison Operator Tests
// ============================================================================
// These tests catch mutations that change comparison operators:
// - == changed to !=
// - < changed to >
// - Enum variant comparison errors

mod comparison_operator_tests {
    use super::*;

    // --- DamageType Comparison ---

    #[test]
    fn test_damage_type_physical_vs_fire() {
        let physical = DamageType::Physical;
        let fire = DamageType::Fire;
        assert!(
            !matches!(physical, DamageType::Fire),
            "Physical should not match Fire"
        );
        assert!(
            !matches!(fire, DamageType::Physical),
            "Fire should not match Physical"
        );
    }

    #[test]
    fn test_damage_type_all_variants_distinct() {
        let types = [
            DamageType::Physical,
            DamageType::Echo,
            DamageType::Fire,
            DamageType::Frost,
            DamageType::Shock,
            DamageType::Poison,
        ];

        for i in 0..types.len() {
            for j in 0..types.len() {
                if i != j {
                    let same = std::mem::discriminant(&types[i]) == std::mem::discriminant(&types[j]);
                    assert!(!same, "Different DamageType variants should have different discriminants");
                }
            }
        }
    }

    // --- StatusEffect Comparison ---

    #[test]
    fn test_status_effect_bleed_vs_stagger() {
        let bleed = StatusEffect::Bleed { dps: 10.0, time: 2.0 };
        let stagger = StatusEffect::Stagger { time: 2.0 };

        assert!(matches!(bleed, StatusEffect::Bleed { .. }));
        assert!(!matches!(bleed, StatusEffect::Stagger { .. }));
        assert!(matches!(stagger, StatusEffect::Stagger { .. }));
        assert!(!matches!(stagger, StatusEffect::Bleed { .. }));
    }

    #[test]
    fn test_status_effect_bleed_vs_chill() {
        let bleed = StatusEffect::Bleed { dps: 10.0, time: 2.0 };
        let chill = StatusEffect::Chill { slow: 0.5, time: 2.0 };

        assert!(!matches!(bleed, StatusEffect::Chill { .. }));
        assert!(!matches!(chill, StatusEffect::Bleed { .. }));
    }

    #[test]
    fn test_status_effect_stagger_vs_chill() {
        let stagger = StatusEffect::Stagger { time: 2.0 };
        let chill = StatusEffect::Chill { slow: 0.5, time: 2.0 };

        assert!(!matches!(stagger, StatusEffect::Chill { .. }));
        assert!(!matches!(chill, StatusEffect::Stagger { .. }));
    }

    // --- AttackKind Comparison ---

    #[test]
    fn test_attack_kind_light_vs_heavy() {
        use crate::combat::AttackKind;

        let light = AttackKind::Light;
        let heavy = AttackKind::Heavy;

        assert!(matches!(light, AttackKind::Light));
        assert!(!matches!(light, AttackKind::Heavy));
        assert!(matches!(heavy, AttackKind::Heavy));
        assert!(!matches!(heavy, AttackKind::Light));
    }

    #[test]
    fn test_attack_kind_correct_button_light() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.0, 1.0),
                damage: 10,
                reach: 10.0,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Light attack with light button
        let (hit_light, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(hit_light, "Light attack should respond to light button");
    }

    #[test]
    fn test_attack_kind_wrong_button_light() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.0, 1.0),
                damage: 10,
                reach: 10.0,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Light attack with heavy button - should not hit
        let (hit, _) = state.tick(0.0, false, true, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(!hit, "Light attack should not respond to heavy button");
    }

    #[test]
    fn test_attack_kind_correct_button_heavy() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Heavy,
                window: (0.0, 1.0),
                damage: 20,
                reach: 10.0,
                stagger: 0.2,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Heavy attack with heavy button
        let (hit, _) = state.tick(0.0, false, true, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(hit, "Heavy attack should respond to heavy button");
    }

    // --- ResourceKind Comparison ---

    #[test]
    fn test_resource_kind_equality() {
        use crate::ResourceKind;

        assert_eq!(ResourceKind::Wood, ResourceKind::Wood);
        assert_ne!(ResourceKind::Wood, ResourceKind::Crystal);
        assert_ne!(ResourceKind::Wood, ResourceKind::Ore);
        assert_ne!(ResourceKind::Crystal, ResourceKind::Fiber);
        assert_ne!(ResourceKind::Ore, ResourceKind::Essence);
    }

    #[test]
    fn test_inventory_finds_correct_resource() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();
        inv.add_resource(ResourceKind::Wood, 10);
        inv.add_resource(ResourceKind::Crystal, 5);

        inv.add_resource(ResourceKind::Wood, 5);
        // Wood should have 15, Crystal should still have 5
        let wood = inv.resources.iter().find(|(k, _)| *k == ResourceKind::Wood).map(|(_, c)| *c);
        let crystal = inv.resources.iter().find(|(k, _)| *k == ResourceKind::Crystal).map(|(_, c)| *c);

        assert_eq!(wood, Some(15), "Wood should be 15 (== comparison)");
        assert_eq!(crystal, Some(5), "Crystal should be 5 (== comparison)");
    }

    #[test]
    fn test_inventory_remove_correct_resource() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();
        inv.add_resource(ResourceKind::Wood, 10);
        inv.add_resource(ResourceKind::Crystal, 10);

        inv.remove_resource(ResourceKind::Wood, 3);

        let wood = inv.resources.iter().find(|(k, _)| *k == ResourceKind::Wood).map(|(_, c)| *c);
        let crystal = inv.resources.iter().find(|(k, _)| *k == ResourceKind::Crystal).map(|(_, c)| *c);

        assert_eq!(wood, Some(7), "Only Wood should be affected");
        assert_eq!(crystal, Some(10), "Crystal should be unaffected");
    }

    // --- Rarity Comparison ---

    #[test]
    fn test_rarity_equality() {
        use crate::items::Rarity;

        assert_eq!(Rarity::Common, Rarity::Common);
        assert_ne!(Rarity::Common, Rarity::Uncommon);
        assert_ne!(Rarity::Uncommon, Rarity::Rare);
        assert_ne!(Rarity::Rare, Rarity::Epic);
        assert_ne!(Rarity::Epic, Rarity::Legendary);
    }

    // --- TaskKind Comparison ---

    #[test]
    fn test_task_kind_gather_pattern_match() {
        use crate::quests::TaskKind;

        let gather = TaskKind::Gather { kind: "wood".into(), count: 10 };
        let visit = TaskKind::Visit { marker: "town".into() };
        let defeat = TaskKind::Defeat { enemy: "goblin".into(), count: 5 };

        assert!(matches!(gather, TaskKind::Gather { .. }));
        assert!(!matches!(gather, TaskKind::Visit { .. }));
        assert!(!matches!(gather, TaskKind::Defeat { .. }));

        assert!(matches!(visit, TaskKind::Visit { .. }));
        assert!(matches!(defeat, TaskKind::Defeat { .. }));
    }

    // --- Stats Comparison ---

    #[test]
    fn test_stats_hp_comparison() {
        let stats1 = Stats::new(100);
        let stats2 = Stats::new(50);

        assert!(stats1.hp > stats2.hp, "> comparison");
        assert!(stats1.hp >= stats2.hp, ">= comparison");
        assert!(stats2.hp < stats1.hp, "< comparison");
        assert!(stats2.hp <= stats1.hp, "<= comparison");
        assert!(stats1.hp != stats2.hp, "!= comparison");
    }

    #[test]
    fn test_stats_hp_equal_comparison() {
        let stats1 = Stats::new(100);
        let stats2 = Stats::new(100);

        assert!(stats1.hp == stats2.hp, "== comparison");
        assert!(stats1.hp >= stats2.hp, ">= with equal values");
        assert!(stats1.hp <= stats2.hp, "<= with equal values");
        assert!(!(stats1.hp > stats2.hp), "not > with equal values");
        assert!(!(stats1.hp < stats2.hp), "not < with equal values");
    }

    // --- Defense Mitigation Comparison ---

    #[test]
    fn test_damage_mitigation_comparison_order() {
        let mut stats1 = Stats::new(100);
        stats1.defense = 10;
        let dmg1 = stats1.apply_damage(20, DamageType::Physical);

        let mut stats2 = Stats::new(100);
        stats2.defense = 5;
        let dmg2 = stats2.apply_damage(20, DamageType::Physical);

        assert!(dmg1 < dmg2, "Higher defense should result in less damage (< comparison)");
    }

    // --- Quest Completion Comparison ---

    #[test]
    fn test_quest_is_done_true_vs_false() {
        use crate::quests::{Quest, QuestLog};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "done".into(),
            title: "Done".into(),
            tasks: vec![],
            reward_text: "".into(),
            completed: true,
        });
        log.add(Quest {
            id: "notdone".into(),
            title: "NotDone".into(),
            tasks: vec![],
            reward_text: "".into(),
            completed: false,
        });

        assert!(log.is_done("done"), "Completed quest should return true");
        assert!(!log.is_done("notdone"), "Incomplete quest should return false");
        assert!(log.is_done("done") != log.is_done("notdone"), "Results should differ");
    }
}

// ============================================================================
// MUTATION-RESISTANT TEST MODULE 3: Boolean Return Path Tests
// ============================================================================
// These tests catch mutations that:
// - Change `return true` to `return false`
// - Change `return false` to `return true`
// - Invert boolean expressions
// - Remove early returns

mod boolean_return_path_tests {
    use super::*;

    // --- Effects Empty Check ---

    #[test]
    fn test_effects_is_empty_true_path() {
        let stats = Stats::new(100);
        assert!(stats.effects.is_empty(), "New stats should have empty effects");
    }

    #[test]
    fn test_effects_is_empty_false_path() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Stagger { time: 1.0 });
        assert!(!stats.effects.is_empty(), "Stats with effect should not be empty");
    }

    // --- Quest is_done Return Paths ---

    #[test]
    fn test_quest_is_done_returns_true() {
        use crate::quests::{Quest, QuestLog};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![],
            reward_text: "".into(),
            completed: true,
        });

        let result = log.is_done("q1");
        assert!(result, "is_done should return true for completed quest");
        assert_eq!(result, true, "is_done should equal true exactly");
    }

    #[test]
    fn test_quest_is_done_returns_false() {
        use crate::quests::{Quest, QuestLog};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![],
            reward_text: "".into(),
            completed: false,
        });

        let result = log.is_done("q1");
        assert!(!result, "is_done should return false for incomplete quest");
        assert_eq!(result, false, "is_done should equal false exactly");
    }

    #[test]
    fn test_quest_is_done_nonexistent_returns_false() {
        use crate::quests::QuestLog;

        let log = QuestLog::default();
        let result = log.is_done("nonexistent");
        assert!(!result, "is_done should return false for nonexistent quest");
        assert_eq!(result, false, "Result should be exactly false");
    }

    // --- Inventory remove_resource Return Paths ---

    #[test]
    fn test_remove_resource_returns_true() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();
        inv.add_resource(ResourceKind::Wood, 10);

        let result = inv.remove_resource(ResourceKind::Wood, 5);
        assert!(result, "Should return true when removal succeeds");
        assert_eq!(result, true, "Result should be exactly true");
    }

    #[test]
    fn test_remove_resource_returns_false_insufficient() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();
        inv.add_resource(ResourceKind::Wood, 5);

        let result = inv.remove_resource(ResourceKind::Wood, 10);
        assert!(!result, "Should return false when insufficient resources");
        assert_eq!(result, false, "Result should be exactly false");
    }

    #[test]
    fn test_remove_resource_returns_false_nonexistent() {
        use crate::items::Inventory;
        use crate::ResourceKind;

        let mut inv = Inventory::default();

        let result = inv.remove_resource(ResourceKind::Crystal, 1);
        assert!(!result, "Should return false for nonexistent resource");
        assert_eq!(result, false, "Result should be exactly false");
    }

    // --- AttackState active Boolean ---

    #[test]
    fn test_attack_state_active_initially_false() {
        use crate::combat::{AttackState, ComboChain};

        let chain = ComboChain { name: "test".into(), steps: vec![] };
        let state = AttackState::new(chain);

        assert!(!state.active, "New AttackState should not be active");
        assert_eq!(state.active, false, "active should be exactly false");
    }

    #[test]
    fn test_attack_state_active_after_start_true() {
        use crate::combat::{AttackState, ComboChain};

        let chain = ComboChain { name: "test".into(), steps: vec![] };
        let mut state = AttackState::new(chain);
        state.start();

        assert!(state.active, "AttackState should be active after start");
        assert_eq!(state.active, true, "active should be exactly true");
    }

    #[test]
    fn test_attack_state_active_after_combo_complete_false() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.0, 1.0),
                damage: 10,
                reach: 10.0,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Complete the combo
        state.tick(0.0, true, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);

        assert!(!state.active, "AttackState should not be active after combo completes");
        assert_eq!(state.active, false, "active should be exactly false");
    }

    // --- Tick Returns Correct Damage ---

    #[test]
    fn test_tick_returns_zero_with_no_effects() {
        let mut stats = Stats::new(100);
        let dot = stats.tick(1.0);
        assert_eq!(dot, 0, "Tick should return exactly 0 with no effects");
    }

    #[test]
    fn test_tick_returns_nonzero_with_bleed() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 2.0 });
        let dot = stats.tick(1.0);
        assert_ne!(dot, 0, "Tick should return non-zero with bleed effect");
        assert!(dot > 0, "DoT should be positive");
    }

    // --- Effect Retain Logic ---

    #[test]
    fn test_effect_retain_returns_true_when_time_positive() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 5.0 });
        stats.tick(1.0); // time becomes 4.0

        assert_eq!(stats.effects.len(), 1, "Effect should be retained (return true path)");
    }

    #[test]
    fn test_effect_retain_returns_false_when_time_zero_or_negative() {
        let mut stats = Stats::new(100);
        stats.effects.push(StatusEffect::Bleed { dps: 10.0, time: 0.5 });
        stats.tick(1.0); // time becomes -0.5

        assert_eq!(stats.effects.len(), 0, "Effect should be removed (return false path)");
    }

    // --- Task done Boolean ---

    #[test]
    fn test_task_done_initially_false() {
        use crate::quests::{Task, TaskKind};

        let task = Task {
            id: "t1".into(),
            kind: TaskKind::Gather { kind: "wood".into(), count: 10 },
            done: false,
        };

        assert!(!task.done, "New task should not be done");
        assert_eq!(task.done, false, "done should be exactly false");
    }

    #[test]
    fn test_task_done_becomes_true_on_completion() {
        use crate::quests::{Quest, QuestLog, Task, TaskKind};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![Task {
                id: "t1".into(),
                kind: TaskKind::Gather { kind: "wood".into(), count: 5 },
                done: false,
            }],
            reward_text: "".into(),
            completed: false,
        });

        log.progress_gather("q1", "wood", 10);
        let q = log.quests.get("q1").unwrap();

        assert!(q.tasks[0].done, "Task should be done after completion");
        assert_eq!(q.tasks[0].done, true, "done should be exactly true");
    }

    // --- Quest completed Boolean ---

    #[test]
    fn test_quest_completed_initially_false() {
        use crate::quests::Quest;

        let quest = Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![],
            reward_text: "".into(),
            completed: false,
        };

        assert!(!quest.completed, "New quest should not be completed");
        assert_eq!(quest.completed, false, "completed should be exactly false");
    }

    #[test]
    fn test_quest_completed_becomes_true_when_all_tasks_done() {
        use crate::quests::{Quest, QuestLog, Task, TaskKind};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![
                Task { id: "t1".into(), kind: TaskKind::Gather { kind: "wood".into(), count: 5 }, done: false },
                Task { id: "t2".into(), kind: TaskKind::Gather { kind: "wood".into(), count: 5 }, done: false },
            ],
            reward_text: "".into(),
            completed: false,
        });

        log.progress_gather("q1", "wood", 100); // Complete both tasks

        let q = log.quests.get("q1").unwrap();
        assert!(q.completed, "Quest should be completed when all tasks done");
        assert_eq!(q.completed, true, "completed should be exactly true");
    }

    #[test]
    fn test_quest_not_completed_when_some_tasks_incomplete() {
        use crate::quests::{Quest, QuestLog, Task, TaskKind};

        let mut log = QuestLog::default();
        log.add(Quest {
            id: "q1".into(),
            title: "Test".into(),
            tasks: vec![
                Task { id: "t1".into(), kind: TaskKind::Gather { kind: "wood".into(), count: 5 }, done: false },
                Task { id: "t2".into(), kind: TaskKind::Visit { marker: "town".into() }, done: false },
            ],
            reward_text: "".into(),
            completed: false,
        });

        log.progress_gather("q1", "wood", 100); // Complete only gather task

        let q = log.quests.get("q1").unwrap();
        assert!(!q.completed, "Quest should not be completed with incomplete tasks");
        assert_eq!(q.completed, false, "completed should be exactly false");
    }

    // --- Hit Boolean Return ---

    #[test]
    fn test_tick_returns_hit_true() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.0, 1.0),
                damage: 10,
                reach: 10.0,
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(hit, "hit should be true when attack lands");
        assert_eq!(hit, true, "hit should be exactly true");
    }

    #[test]
    fn test_tick_returns_hit_false_inactive() {
        use crate::combat::{AttackState, ComboChain};
        use glam::Vec3;

        let chain = ComboChain { name: "test".into(), steps: vec![] };
        let mut state = AttackState::new(chain);
        // Don't call start() - state is inactive
        let mut target = Stats::new(100);

        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::ZERO, &Stats::new(100), None, &mut target);
        assert!(!hit, "hit should be false when state is inactive");
        assert_eq!(hit, false, "hit should be exactly false");
    }

    #[test]
    fn test_tick_returns_hit_false_out_of_reach() {
        use crate::combat::{AttackKind, AttackState, ComboChain, ComboStep};
        use glam::Vec3;

        let chain = ComboChain {
            name: "test".into(),
            steps: vec![ComboStep {
                kind: AttackKind::Light,
                window: (0.0, 1.0),
                damage: 10,
                reach: 1.0, // Short reach
                stagger: 0.1,
            }],
        };

        let mut state = AttackState::new(chain);
        state.start();
        let mut target = Stats::new(100);

        // Target is 100 units away
        let (hit, _) = state.tick(0.0, true, false, Vec3::ZERO, Vec3::new(100.0, 0.0, 0.0), &Stats::new(100), None, &mut target);
        assert!(!hit, "hit should be false when out of reach");
        assert_eq!(hit, false, "hit should be exactly false");
    }
}
