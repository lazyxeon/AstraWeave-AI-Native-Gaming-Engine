//! HUD (Heads-Up Display) system tests
//!
//! Tests HUD updates, persistence, validation, and animations

#![allow(clippy::field_reassign_with_default)]

use astraweave_ui::hud::{
    DamageNumber, DamageType, EnemyData, EnemyFaction, HealthAnimation, PlayerStats,
};

// ===== HUD Health Updates =====

#[test]
fn test_hud_player_health_update() {
    let mut stats = PlayerStats::default();

    stats.health = 75.0;

    assert_eq!(stats.health, 75.0);
    assert_eq!(stats.max_health, 100.0);
}

#[test]
fn test_hud_player_stats_default() {
    let stats = PlayerStats::default();

    assert_eq!(stats.health, 100.0);
    assert_eq!(stats.max_health, 100.0);
    assert_eq!(stats.mana, 100.0);
    assert_eq!(stats.stamina, 100.0);
}

#[test]
fn test_hud_health_percentage_calculation() {
    let stats = PlayerStats {
        health: 50.0,
        max_health: 100.0,
        ..Default::default()
    };

    let percentage = (stats.health / stats.max_health) * 100.0;
    assert_eq!(percentage, 50.0);
}

#[test]
fn test_hud_multiple_resource_updates() {
    let mut stats = PlayerStats::default();

    stats.health = 80.0;
    stats.mana = 60.0;
    stats.stamina = 40.0;

    assert_eq!(stats.health, 80.0);
    assert_eq!(stats.mana, 60.0);
    assert_eq!(stats.stamina, 40.0);
}

// ===== HUD Value Validation =====

#[test]
fn test_hud_health_clamp_to_zero() {
    let mut stats = PlayerStats::default();

    // Set negative health and demonstrate clamping behavior
    stats.health = -50.0;

    // Application would clamp this value, here we test the clamping logic
    let clamped = stats.health.max(0.0);
    assert_eq!(clamped, 0.0);
}

#[test]
fn test_hud_health_clamp_to_max() {
    let mut stats = PlayerStats::default();

    // Set overflow health and demonstrate clamping behavior
    stats.health = 150.0;

    // Application would clamp this value, here we test the clamping logic
    let clamped = stats.health.min(stats.max_health);
    assert_eq!(clamped, 100.0);
}

#[test]
fn test_hud_mana_boundaries() {
    let mut stats = PlayerStats::default();

    stats.mana = 0.0;
    assert_eq!(stats.mana, 0.0);

    stats.mana = stats.max_mana;
    assert_eq!(stats.mana, 100.0);
}

// ===== Health Animation Tests =====

#[test]
fn test_health_animation_initialization() {
    let anim = HealthAnimation::new(100.0);

    assert_eq!(anim.current_visual, 100.0);
    assert_eq!(anim.target, 100.0);
    assert_eq!(anim.animation_time, 0.0);
}

#[test]
fn test_health_animation_set_target() {
    let mut anim = HealthAnimation::new(100.0);

    anim.set_target(75.0);

    assert_eq!(anim.target, 75.0);
    assert_eq!(anim.animation_time, 0.0);
}

#[test]
fn test_health_animation_damage_flash_trigger() {
    let mut anim = HealthAnimation::new(100.0);

    // Take damage
    anim.set_target(50.0);

    assert!(anim.flash_timer > 0.0);
}

#[test]
fn test_health_animation_healing_detection() {
    let mut anim = HealthAnimation::new(50.0);

    anim.set_target(80.0);

    assert!(anim.is_healing());
}

#[test]
fn test_health_animation_update_progresses() {
    let mut anim = HealthAnimation::new(100.0);
    anim.set_target(50.0);

    let initial_visual = anim.visual_health();

    // Update animation (16ms frame)
    anim.update(0.016);

    // Visual health should move toward target
    let after_visual = anim.visual_health();
    assert!(after_visual < initial_visual);
}

#[test]
fn test_health_animation_flash_decay() {
    let mut anim = HealthAnimation::new(100.0);
    anim.set_target(50.0);

    let initial_flash = anim.flash_alpha();
    assert!(initial_flash > 0.0);

    // Update several frames
    for _ in 0..10 {
        anim.update(0.016);
    }

    let after_flash = anim.flash_alpha();
    assert!(after_flash < initial_flash);
}

// ===== Enemy HUD Tests =====

#[test]
fn test_enemy_data_creation() {
    let enemy = EnemyData::new(1, (0.0, 1.0, 0.0), 100.0, EnemyFaction::Hostile);

    assert_eq!(enemy.id, 1);
    assert_eq!(enemy.health, 100.0);
    assert_eq!(enemy.max_health, 100.0);
    assert_eq!(enemy.faction, EnemyFaction::Hostile);
}

#[test]
fn test_enemy_faction_types() {
    let hostile = EnemyData::new(1, (0.0, 0.0, 0.0), 100.0, EnemyFaction::Hostile);
    let neutral = EnemyData::new(2, (0.0, 0.0, 0.0), 100.0, EnemyFaction::Neutral);
    let friendly = EnemyData::new(3, (0.0, 0.0, 0.0), 100.0, EnemyFaction::Friendly);

    assert_eq!(hostile.faction, EnemyFaction::Hostile);
    assert_eq!(neutral.faction, EnemyFaction::Neutral);
    assert_eq!(friendly.faction, EnemyFaction::Friendly);
}

#[test]
fn test_enemy_health_animation() {
    let mut enemy = EnemyData::new(1, (0.0, 0.0, 0.0), 100.0, EnemyFaction::Hostile);

    // Damage enemy
    enemy.health = 70.0;
    enemy.health_animation.set_target(70.0);

    assert_eq!(enemy.health_animation.target, 70.0);
}

// ===== Damage Numbers Tests =====

#[test]
fn test_damage_number_creation() {
    let damage = DamageNumber::new(50, 0.0, (0.0, 1.0, 0.0), DamageType::Normal);

    assert_eq!(damage.value, 50);
    assert_eq!(damage.spawn_time, 0.0);
    assert_eq!(damage.damage_type, DamageType::Normal);
}

#[test]
fn test_damage_number_types() {
    let normal = DamageNumber::new(50, 0.0, (0.0, 0.0, 0.0), DamageType::Normal);
    let critical = DamageNumber::new(100, 0.0, (0.0, 0.0, 0.0), DamageType::Critical);
    let self_damage = DamageNumber::new(30, 0.0, (0.0, 0.0, 0.0), DamageType::SelfDamage);

    assert_eq!(normal.damage_type, DamageType::Normal);
    assert_eq!(critical.damage_type, DamageType::Critical);
    assert_eq!(self_damage.damage_type, DamageType::SelfDamage);
}

#[test]
fn test_damage_number_world_position() {
    let damage = DamageNumber::new(50, 0.0, (10.0, 5.0, -3.0), DamageType::Normal);

    assert_eq!(damage.world_pos, (10.0, 5.0, -3.0));
}

#[test]
fn test_damage_number_has_velocity() {
    let damage = DamageNumber::new(50, 0.0, (0.0, 0.0, 0.0), DamageType::Normal);

    // Should have some horizontal velocity (pseudo-random)
    assert!(damage.velocity_x.abs() > 0.0);

    // Should have upward velocity
    assert!(damage.velocity_y < 0.0); // Negative = up
}
