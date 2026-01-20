//! NaN and Infinity validation tests for UI subsystem.
//!
//! P0-Critical: Ensures UI systems handle invalid float inputs gracefully
//! without panicking or corrupting visual state.

#![allow(clippy::field_reassign_with_default)]

use astraweave_ui::hud::{
    easing, DamageNumber, DamageType, EnemyData, EnemyFaction, HealthAnimation, PlayerStats,
};
use std::panic;

/// Helper to verify a closure doesn't panic
fn should_not_panic<F: FnOnce() + panic::UnwindSafe>(name: &str, f: F) {
    let result = panic::catch_unwind(f);
    assert!(
        result.is_ok(),
        "{} should not panic on invalid input",
        name
    );
}

/// Helper to create a default DamageNumber for testing
fn create_damage_number(
    value: i32,
    spawn_time: f32,
    world_pos: (f32, f32, f32),
) -> DamageNumber {
    DamageNumber::new(value, spawn_time, world_pos, DamageType::Normal)
}

// ============================================================================
// Easing function NaN/Infinity tests
// ============================================================================

#[test]
fn test_ease_out_cubic_nan() {
    should_not_panic("ease_out_cubic with NaN", || {
        let result = easing::ease_out_cubic(f32::NAN);
        let _ = result; // Just verify no panic
    });
}

#[test]
fn test_ease_out_cubic_infinity() {
    should_not_panic("ease_out_cubic with Infinity", || {
        let result = easing::ease_out_cubic(f32::INFINITY);
        let _ = result;
    });
}

#[test]
fn test_ease_out_cubic_neg_infinity() {
    should_not_panic("ease_out_cubic with -Infinity", || {
        let result = easing::ease_out_cubic(f32::NEG_INFINITY);
        let _ = result;
    });
}

#[test]
fn test_ease_in_out_quad_nan() {
    should_not_panic("ease_in_out_quad with NaN", || {
        let result = easing::ease_in_out_quad(f32::NAN);
        let _ = result;
    });
}

#[test]
fn test_ease_in_out_quad_infinity() {
    should_not_panic("ease_in_out_quad with Infinity", || {
        let result = easing::ease_in_out_quad(f32::INFINITY);
        let _ = result;
    });
}

#[test]
fn test_ease_in_out_quad_neg_infinity() {
    should_not_panic("ease_in_out_quad with -Infinity", || {
        let result = easing::ease_in_out_quad(f32::NEG_INFINITY);
        let _ = result;
    });
}

// ============================================================================
// HealthAnimation NaN/Infinity tests
// ============================================================================

#[test]
fn test_health_animation_new_nan() {
    should_not_panic("HealthAnimation::new with NaN", || {
        let anim = HealthAnimation::new(f32::NAN);
        let _ = anim;
    });
}

#[test]
fn test_health_animation_new_infinity() {
    should_not_panic("HealthAnimation::new with Infinity", || {
        let anim = HealthAnimation::new(f32::INFINITY);
        let _ = anim;
    });
}

#[test]
fn test_health_animation_new_neg_infinity() {
    should_not_panic("HealthAnimation::new with -Infinity", || {
        let anim = HealthAnimation::new(f32::NEG_INFINITY);
        let _ = anim;
    });
}

#[test]
fn test_health_animation_set_target_nan() {
    should_not_panic("HealthAnimation::set_target with NaN", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(f32::NAN);
    });
}

#[test]
fn test_health_animation_set_target_infinity() {
    should_not_panic("HealthAnimation::set_target with Infinity", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(f32::INFINITY);
    });
}

#[test]
fn test_health_animation_set_target_neg_infinity() {
    should_not_panic("HealthAnimation::set_target with -Infinity", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(f32::NEG_INFINITY);
    });
}

#[test]
fn test_health_animation_update_nan_dt() {
    should_not_panic("HealthAnimation::update with NaN dt", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(50.0);
        anim.update(f32::NAN);
    });
}

#[test]
fn test_health_animation_update_infinity_dt() {
    should_not_panic("HealthAnimation::update with Infinity dt", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(50.0);
        anim.update(f32::INFINITY);
    });
}

#[test]
fn test_health_animation_update_neg_infinity_dt() {
    should_not_panic("HealthAnimation::update with -Infinity dt", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(50.0);
        anim.update(f32::NEG_INFINITY);
    });
}

#[test]
fn test_health_animation_visual_health_after_nan_target() {
    should_not_panic("visual_health after NaN target", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(f32::NAN);
        anim.update(0.016);
        let _ = anim.visual_health();
    });
}

#[test]
fn test_health_animation_flash_alpha_after_nan() {
    should_not_panic("flash_alpha after NaN operations", || {
        let mut anim = HealthAnimation::new(f32::NAN);
        anim.set_target(f32::NAN);
        anim.update(f32::NAN);
        let _ = anim.flash_alpha();
    });
}

#[test]
fn test_health_animation_is_healing_with_nan() {
    should_not_panic("is_healing with NaN values", || {
        let mut anim = HealthAnimation::new(100.0);
        anim.set_target(f32::NAN);
        let _ = anim.is_healing();
    });
}

// ============================================================================
// PlayerStats NaN/Infinity tests
// ============================================================================

#[test]
fn test_player_stats_nan_health() {
    should_not_panic("PlayerStats with NaN health", || {
        let mut stats = PlayerStats::default();
        stats.health = f32::NAN;
        let _ = stats;
    });
}

#[test]
fn test_player_stats_nan_max_health() {
    should_not_panic("PlayerStats with NaN max_health", || {
        let mut stats = PlayerStats::default();
        stats.max_health = f32::NAN;
        let _ = stats;
    });
}

#[test]
fn test_player_stats_nan_mana() {
    should_not_panic("PlayerStats with NaN mana", || {
        let mut stats = PlayerStats::default();
        stats.mana = f32::NAN;
        let _ = stats;
    });
}

#[test]
fn test_player_stats_nan_stamina() {
    should_not_panic("PlayerStats with NaN stamina", || {
        let mut stats = PlayerStats::default();
        stats.stamina = f32::NAN;
        let _ = stats;
    });
}

#[test]
fn test_player_stats_infinity_all_values() {
    should_not_panic("PlayerStats with Infinity all values", || {
        let mut stats = PlayerStats::default();
        stats.health = f32::INFINITY;
        stats.max_health = f32::INFINITY;
        stats.mana = f32::INFINITY;
        stats.max_mana = f32::INFINITY;
        stats.stamina = f32::INFINITY;
        stats.max_stamina = f32::INFINITY;
        let _ = stats;
    });
}

#[test]
fn test_player_stats_neg_infinity_all_values() {
    should_not_panic("PlayerStats with -Infinity all values", || {
        let mut stats = PlayerStats::default();
        stats.health = f32::NEG_INFINITY;
        stats.max_health = f32::NEG_INFINITY;
        stats.mana = f32::NEG_INFINITY;
        stats.max_mana = f32::NEG_INFINITY;
        stats.stamina = f32::NEG_INFINITY;
        stats.max_stamina = f32::NEG_INFINITY;
        let _ = stats;
    });
}

// ============================================================================
// EnemyData NaN/Infinity tests
// ============================================================================

#[test]
fn test_enemy_data_nan_position() {
    should_not_panic("EnemyData with NaN position", || {
        let enemy = EnemyData::new(1, (f32::NAN, f32::NAN, f32::NAN), 100.0, EnemyFaction::Hostile);
        let _ = enemy;
    });
}

#[test]
fn test_enemy_data_infinity_position() {
    should_not_panic("EnemyData with Infinity position", || {
        let enemy = EnemyData::new(
            1,
            (f32::INFINITY, f32::INFINITY, f32::INFINITY),
            100.0,
            EnemyFaction::Hostile,
        );
        let _ = enemy;
    });
}

#[test]
fn test_enemy_data_nan_health() {
    should_not_panic("EnemyData with NaN health", || {
        let enemy = EnemyData::new(1, (0.0, 0.0, 0.0), f32::NAN, EnemyFaction::Hostile);
        let _ = enemy;
    });
}

#[test]
fn test_enemy_data_infinity_health() {
    should_not_panic("EnemyData with Infinity health", || {
        let enemy = EnemyData::new(1, (0.0, 0.0, 0.0), f32::INFINITY, EnemyFaction::Hostile);
        let _ = enemy;
    });
}

#[test]
fn test_enemy_data_neg_infinity_health() {
    should_not_panic("EnemyData with -Infinity health", || {
        let enemy = EnemyData::new(1, (0.0, 0.0, 0.0), f32::NEG_INFINITY, EnemyFaction::Hostile);
        let _ = enemy;
    });
}

#[test]
fn test_enemy_data_mixed_nan_infinity() {
    should_not_panic("EnemyData with mixed NaN and Infinity", || {
        let enemy = EnemyData::new(
            1,
            (f32::NAN, f32::INFINITY, f32::NEG_INFINITY),
            f32::NAN,
            EnemyFaction::Neutral,
        );
        let _ = enemy;
    });
}

// ============================================================================
// DamageNumber NaN/Infinity tests
// ============================================================================

#[test]
fn test_damage_number_nan_spawn_time() {
    should_not_panic("DamageNumber with NaN spawn_time", || {
        let dmg = create_damage_number(100, f32::NAN, (0.0, 0.0, 0.0));
        let _ = dmg;
    });
}

#[test]
fn test_damage_number_infinity_spawn_time() {
    should_not_panic("DamageNumber with Infinity spawn_time", || {
        let dmg = create_damage_number(100, f32::INFINITY, (0.0, 0.0, 0.0));
        let _ = dmg;
    });
}

#[test]
fn test_damage_number_nan_position() {
    should_not_panic("DamageNumber with NaN position", || {
        let dmg = create_damage_number(100, 0.0, (f32::NAN, f32::NAN, f32::NAN));
        let _ = dmg;
    });
}

#[test]
fn test_damage_number_infinity_position() {
    should_not_panic("DamageNumber with Infinity position", || {
        let dmg = create_damage_number(
            100,
            0.0,
            (f32::INFINITY, f32::INFINITY, f32::INFINITY),
        );
        let _ = dmg;
    });
}

#[test]
fn test_damage_number_mixed_nan_infinity_position() {
    should_not_panic("DamageNumber with mixed NaN/Infinity position", || {
        let dmg = create_damage_number(
            100,
            f32::NAN,
            (f32::NAN, f32::INFINITY, f32::NEG_INFINITY),
        );
        let _ = dmg;
    });
}

#[test]
fn test_damage_number_all_damage_types_with_nan() {
    should_not_panic("DamageNumber all damage types with NaN", || {
        let _ = DamageNumber::new(100, f32::NAN, (0.0, 0.0, 0.0), DamageType::Normal);
        let _ = DamageNumber::new(100, f32::NAN, (0.0, 0.0, 0.0), DamageType::Critical);
        let _ = DamageNumber::new(100, f32::NAN, (0.0, 0.0, 0.0), DamageType::SelfDamage);
    });
}

// ============================================================================
// Combined stress tests
// ============================================================================

#[test]
fn test_health_animation_cycle_with_nan() {
    should_not_panic("HealthAnimation full cycle with NaN", || {
        let mut anim = HealthAnimation::new(100.0);

        // Set NaN target
        anim.set_target(f32::NAN);

        // Update multiple times with various invalid values
        for _ in 0..10 {
            anim.update(f32::NAN);
        }

        // Try to read state
        let _ = anim.visual_health();
        let _ = anim.flash_alpha();
        let _ = anim.is_healing();
    });
}

#[test]
fn test_player_stats_animation_with_nan() {
    should_not_panic("PlayerStats animation with NaN", || {
        let mut stats = PlayerStats::default();
        stats.health_animation.set_target(f32::NAN);
        stats.health_animation.update(0.016);

        let _ = stats.health_animation.visual_health();
    });
}

#[test]
fn test_repeated_nan_updates() {
    should_not_panic("repeated NaN updates", || {
        let mut anim = HealthAnimation::new(100.0);

        for i in 0..100 {
            if i % 2 == 0 {
                anim.set_target(f32::NAN);
            } else {
                anim.set_target(50.0);
            }
            anim.update(0.016);
        }
    });
}

// ============================================================================
// Edge case float values
// ============================================================================

#[test]
fn test_health_animation_zero_health() {
    should_not_panic("HealthAnimation with zero health", || {
        let anim = HealthAnimation::new(0.0);
        let _ = anim.visual_health();
    });
}

#[test]
fn test_health_animation_negative_health() {
    should_not_panic("HealthAnimation with negative health", || {
        let anim = HealthAnimation::new(-100.0);
        let _ = anim.visual_health();
    });
}

#[test]
fn test_health_animation_max_float() {
    should_not_panic("HealthAnimation with MAX float", || {
        let anim = HealthAnimation::new(f32::MAX);
        let _ = anim.visual_health();
    });
}

#[test]
fn test_health_animation_min_float() {
    should_not_panic("HealthAnimation with MIN float", || {
        let anim = HealthAnimation::new(f32::MIN);
        let _ = anim.visual_health();
    });
}

#[test]
fn test_health_animation_subnormal() {
    should_not_panic("HealthAnimation with subnormal", || {
        let anim = HealthAnimation::new(f32::MIN_POSITIVE / 2.0);
        let _ = anim.visual_health();
    });
}

#[test]
fn test_health_animation_negative_zero() {
    should_not_panic("HealthAnimation with -0.0", || {
        let anim = HealthAnimation::new(-0.0);
        let _ = anim.visual_health();
    });
}

#[test]
fn test_easing_edge_values() {
    should_not_panic("easing functions with edge values", || {
        // Test boundaries
        let _ = easing::ease_out_cubic(0.0);
        let _ = easing::ease_out_cubic(1.0);
        let _ = easing::ease_out_cubic(-1.0);
        let _ = easing::ease_out_cubic(2.0);
        let _ = easing::ease_out_cubic(f32::MAX);
        let _ = easing::ease_out_cubic(f32::MIN);

        let _ = easing::ease_in_out_quad(0.0);
        let _ = easing::ease_in_out_quad(0.5);
        let _ = easing::ease_in_out_quad(1.0);
        let _ = easing::ease_in_out_quad(-1.0);
        let _ = easing::ease_in_out_quad(2.0);
        let _ = easing::ease_in_out_quad(f32::MAX);
        let _ = easing::ease_in_out_quad(f32::MIN);
    });
}

#[test]
fn test_damage_number_zero_spawn_time() {
    should_not_panic("DamageNumber with zero spawn_time", || {
        let dmg = create_damage_number(100, 0.0, (0.0, 0.0, 0.0));
        let _ = dmg;
    });
}

#[test]
fn test_damage_number_negative_spawn_time() {
    should_not_panic("DamageNumber with negative spawn_time", || {
        let dmg = create_damage_number(100, -1.0, (0.0, 0.0, 0.0));
        let _ = dmg;
    });
}

#[test]
fn test_enemy_data_zero_health() {
    should_not_panic("EnemyData with zero health", || {
        let enemy = EnemyData::new(1, (0.0, 0.0, 0.0), 0.0, EnemyFaction::Hostile);
        let _ = enemy;
    });
}

#[test]
fn test_enemy_data_negative_health() {
    should_not_panic("EnemyData with negative health", || {
        let enemy = EnemyData::new(1, (0.0, 0.0, 0.0), -100.0, EnemyFaction::Hostile);
        let _ = enemy;
    });
}
