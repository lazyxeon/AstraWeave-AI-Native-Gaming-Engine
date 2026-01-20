// integration_tests.rs
//
// Cross-system integration tests for Veilweaver
// Tests enemy AI + anchor system + combat + spawner interactions

#![allow(clippy::useless_vec)]

use crate::{
    Anchor, AnchorVfxState, CombatEvent, CombatSystem, Enemy, EnemyBehavior, EnemySpawner,
    EnemyState, Killer,
};
use glam::Vec3;

// ============================================================================
// Test 1: Enemy Attacks Anchor (AI + Anchor Decay)
// ============================================================================

#[test]
fn test_enemy_attacks_anchor_reduces_stability() {
    // Setup: Anchor at 1.0 stability (Perfect), enemy nearby
    let mut anchor = Anchor::new(1.0, 50, None);
    let anchor_pos = Vec3::new(10.0, 0.0, 5.0);

    let mut enemy = Enemy::new(anchor_pos + Vec3::new(2.0, 0.0, 0.0), 5.0);
    enemy.target_anchor_id = Some(0);
    enemy.state = EnemyState::AttackAnchor;

    // Initial state
    assert_eq!(enemy.state, EnemyState::AttackAnchor);
    assert_eq!(anchor.vfx_state(), AnchorVfxState::Perfect);

    // Enemy attacks (simplified - in real game, combat event triggers decay)
    let behavior = enemy.update(
        0.016,
        anchor_pos + Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(50.0, 0.0, 50.0),
        &[(0, anchor_pos)],
    );
    assert!(matches!(behavior, EnemyBehavior::Attack(_)));

    // Simulate combat event → decay (20% stability loss per attack in real game)
    anchor.adjust_stability(-0.2);

    // Verify anchor decayed
    assert_eq!(anchor.stability(), 0.8);
    assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
}

#[test]
fn test_enemy_prioritizes_broken_anchors() {
    // Setup: 2 anchors - one Perfect (1.0), one Broken (0.05)
    let broken_anchor_pos = Vec3::new(15.0, 0.0, 8.0);
    let broken_anchors = vec![(1, broken_anchor_pos)];

    let mut enemy = Enemy::new(Vec3::new(12.0, 0.0, 6.0), 5.0);
    enemy.state = EnemyState::Patrol;

    let player_pos = Vec3::new(50.0, 0.0, 50.0); // Player far away

    // Update enemy - should prioritize broken anchor
    let behavior = enemy.update(
        0.016,
        Vec3::new(12.0, 0.0, 6.0),
        player_pos,
        &broken_anchors,
    );

    // Verify enemy transitioned to AttackAnchor
    assert_eq!(enemy.state, EnemyState::AttackAnchor);
    assert_eq!(enemy.target_anchor_id, Some(1));
    assert!(matches!(behavior, EnemyBehavior::MoveTo(_)));
}

// ============================================================================
// Test 2: Player Kills Enemy Near Anchor (Combat + Anchor Protection)
// ============================================================================

#[test]
fn test_player_kills_enemy_near_anchor_reduces_stress() {
    // Setup: Anchor under attack (Unstable), enemy nearby, player intervenes
    let mut anchor = Anchor::new(0.5, 50, None); // Unstable
    let anchor_pos = Vec3::new(10.0, 0.0, 5.0);

    let mut enemy = Enemy::new(anchor_pos + Vec3::new(3.0, 0.0, 0.0), 5.0);
    enemy.target_anchor_id = Some(0);
    enemy.state = EnemyState::AttackAnchor;

    let mut combat = CombatSystem::new();

    // Player attacks enemy
    let event = combat.player_attack(0, &mut enemy, anchor_pos + Vec3::new(3.0, 0.0, 0.0));

    // Verify enemy damaged
    assert!(enemy.health < 100.0);
    assert!(matches!(event, Some(CombatEvent::EnemyDamaged { .. })));

    // Player kills enemy (20 HP attack × 5 = 100 HP)
    for _ in 0..4 {
        combat.player_attack(0, &mut enemy, anchor_pos + Vec3::new(3.0, 0.0, 0.0));
    }

    let final_event = combat.player_attack(0, &mut enemy, anchor_pos + Vec3::new(3.0, 0.0, 0.0));
    assert!(matches!(
        final_event,
        Some(CombatEvent::EnemyKilled {
            killer: Killer::Player,
            ..
        })
    ));

    // In real game: killing enemy near anchor reduces stress
    // Simplified: anchor stability improves slightly (e.g., +0.05)
    anchor.adjust_stability(0.05); // Positive delta = healing

    assert_eq!(anchor.stability(), 0.55);
}

#[test]
fn test_player_echo_dash_kills_multiple_enemies() {
    // Setup: 3 enemies clustered around player
    let player_pos = Vec3::new(10.0, 0.0, 5.0);

    let mut enemy1 = Enemy::new(player_pos + Vec3::new(2.0, 0.0, 0.0), 5.0);
    let mut enemy2 = Enemy::new(player_pos + Vec3::new(-1.5, 0.0, 1.0), 5.0);
    let mut enemy3 = Enemy::new(player_pos + Vec3::new(0.5, 0.0, -2.0), 5.0);

    let mut combat = CombatSystem::new();

    // Player uses Echo Dash (50 HP AoE, 3.0 radius)
    let enemies = vec![
        (0, &mut enemy1, player_pos + Vec3::new(2.0, 0.0, 0.0)),
        (1, &mut enemy2, player_pos + Vec3::new(-1.5, 0.0, 1.0)),
        (2, &mut enemy3, player_pos + Vec3::new(0.5, 0.0, -2.0)),
    ];

    let events = combat.echo_dash_attack(player_pos, enemies);

    // Verify all 3 enemies damaged (50 HP each)
    assert_eq!(events.len(), 3);
    assert!(enemy1.health < 100.0);
    assert!(enemy2.health < 100.0);
    assert!(enemy3.health < 100.0);

    // Second Echo Dash kills all (50 + 50 = 100 HP total)
    let enemies2 = vec![
        (0, &mut enemy1, player_pos + Vec3::new(2.0, 0.0, 0.0)),
        (1, &mut enemy2, player_pos + Vec3::new(-1.5, 0.0, 1.0)),
        (2, &mut enemy3, player_pos + Vec3::new(0.5, 0.0, -2.0)),
    ];

    let kill_events = combat.echo_dash_attack(player_pos, enemies2);
    assert_eq!(kill_events.len(), 3);

    let kill_count = kill_events
        .iter()
        .filter(|e| matches!(e, CombatEvent::EnemyKilled { .. }))
        .count();
    assert_eq!(kill_count, 3);
}

// ============================================================================
// Test 3: Spawner Difficulty Scales with Broken Anchors
// ============================================================================

#[test]
fn test_spawner_increases_difficulty_when_anchors_break() {
    // Setup: 3 anchors - Perfect, Stable, Broken
    let mut perfect = Anchor::new(1.0, 50, None);
    let mut stable = Anchor::new(0.75, 50, None);
    let mut broken = Anchor::new(0.05, 50, None);

    // Verify VFX states
    assert_eq!(perfect.vfx_state(), AnchorVfxState::Perfect);
    assert_eq!(stable.vfx_state(), AnchorVfxState::Stable);
    assert_eq!(
        broken.vfx_state(),
        AnchorVfxState::Broken,
        "Anchor with 0.05 stability should be Broken"
    );

    let _anchors = vec![(0, &mut perfect), (1, &mut stable), (2, &mut broken)];

    let mut spawner = EnemySpawner::new();

    // Add spawn points BEFORE update (so wave can spawn)
    spawner.add_spawn_point(Vec3::new(10.0, 0.0, 5.0), 5.0, Some(0));
    spawner.add_spawn_point(Vec3::new(20.0, 0.0, 10.0), 5.0, Some(1));
    spawner.add_spawn_point(Vec3::new(30.0, 0.0, 15.0), 5.0, Some(2));

    // Set active enemy count FIRST
    spawner.set_active_enemy_count(0);

    // Trigger wave spawn to calculate difficulty (1 broken anchor)
    let anchors_immut: Vec<(usize, &Anchor)> = vec![(0, &perfect), (1, &stable), (2, &broken)];
    let _ = spawner.update(31.0, &anchors_immut); // 31s > wave_interval (30s) → spawns wave + calculates difficulty

    // Verify difficulty updated (1 broken anchor): 1.0 + 0.5 = 1.5
    let initial_difficulty = spawner.difficulty();
    assert_eq!(
        initial_difficulty,
        1.5,
        "Initial difficulty should be 1.5 with 1 broken anchor (stability {}, state {:?})",
        broken.stability(),
        broken.vfx_state()
    );

    // Break another anchor (Perfect → Broken)
    perfect.adjust_stability(-1.0); // Stability 1.0 → 0.0 (Broken)

    // Update with 2 broken anchors (trigger wave spawn again)
    let anchors_immut2: Vec<(usize, &Anchor)> = vec![(0, &perfect), (1, &stable), (2, &broken)];
    let _ = spawner.update(31.0, &anchors_immut2); // Another wave spawn → recalculates difficulty

    // New difficulty (2 broken anchors): 1.0 + 1.0 = 2.0
    let new_difficulty = spawner.difficulty();
    assert_eq!(
        new_difficulty, 2.0,
        "New difficulty should be 2.0 with 2 broken anchors"
    );
}

#[test]
fn test_spawner_prioritizes_broken_anchor_spawn_points() {
    // Setup: 2 anchors - Perfect and Broken
    let mut perfect = Anchor::new(1.0, 50, None);
    let mut broken = Anchor::new(0.05, 50, None);

    let _anchors = vec![(0, &mut perfect), (1, &mut broken)];

    let mut spawner = EnemySpawner::new();

    // Add spawn points (broken anchor first)
    spawner.add_spawn_point(Vec3::new(10.0, 0.0, 5.0), 5.0, Some(0)); // Perfect
    spawner.add_spawn_point(Vec3::new(20.0, 0.0, 10.0), 5.0, Some(1)); // Broken

    // Set active enemy count to allow spawning
    spawner.set_active_enemy_count(0);

    // Force wave spawn
    let anchors_immut: Vec<(usize, &Anchor)> = vec![(0, &perfect), (1, &broken)];
    let spawn_requests = spawner.update(31.0, &anchors_immut);

    // Verify spawn requests were generated
    assert!(!spawn_requests.is_empty());

    // In real game: Priority spawning means broken anchor spawn point used first
    // Simplified: First spawn request should be from broken anchor spawn point
    // (This test verifies spawn request generation, priority logic is internal)
    let first_spawn = &spawn_requests[0];
    assert_eq!(first_spawn.anchor_id, Some(1)); // Broken anchor
}

// ============================================================================
// Test 4: Full Gameplay Loop (Enemy Spawn → Attack → Player Kill → Anchor Heal)
// ============================================================================

#[test]
fn test_full_gameplay_loop() {
    // Setup: 1 anchor (Unstable), 1 spawn point, 1 enemy
    let mut anchor = Anchor::new(0.5, 50, None); // Unstable
    let anchor_pos = Vec3::new(10.0, 0.0, 5.0);

    let mut spawner = EnemySpawner::new();
    spawner.add_spawn_point(anchor_pos, 5.0, Some(0));
    spawner.set_active_enemy_count(0);

    let mut combat = CombatSystem::new();
    let player_pos = Vec3::new(50.0, 0.0, 50.0); // Player far away initially

    // Step 1: Spawner generates spawn request
    let anchors_immut: Vec<(usize, &Anchor)> = vec![(0, &anchor)];
    let spawn_requests = spawner.update(31.0, &anchors_immut);
    assert!(!spawn_requests.is_empty());

    let spawn_req = &spawn_requests[0];
    let mut enemy = Enemy::new(spawn_req.position, spawn_req.patrol_radius);
    enemy.target_anchor_id = Some(0);

    // Step 2: Enemy transitions to AttackAnchor state (player far away)
    let behavior = enemy.update(0.016, spawn_req.position, player_pos, &[(0, anchor_pos)]);
    assert_eq!(
        enemy.state,
        EnemyState::AttackAnchor,
        "Enemy should attack anchor when player is far"
    );
    assert!(matches!(
        behavior,
        EnemyBehavior::MoveTo(_) | EnemyBehavior::Attack(_)
    ));

    // Step 3: Enemy attacks anchor (simplified - direct stress application)
    anchor.adjust_stability(-0.2); // 20% stability loss
    assert_eq!(anchor.stability(), 0.3); // Unstable → Critical
    assert_eq!(anchor.vfx_state(), AnchorVfxState::Critical);

    // Step 4: Player notices anchor under attack, kills enemy
    let enemy_pos = spawn_req.position;
    for _ in 0..5 {
        combat.player_attack(0, &mut enemy, enemy_pos);
    }

    assert_eq!(enemy.health, 0.0);
    assert_eq!(enemy.state, EnemyState::Dead);

    // Step 5: Killing enemy near anchor reduces stress (simplified)
    anchor.adjust_stability(0.1); // 10% stability gain
    assert_eq!(anchor.stability(), 0.4); // Critical → Unstable
    assert_eq!(anchor.vfx_state(), AnchorVfxState::Unstable);

    // Step 6: Player repairs anchor (requires Echo currency in real game)
    anchor.repair(); // REPAIR_BONUS = 0.3 (30% repair)
    assert!((anchor.stability() - 0.7).abs() < 0.01); // Unstable → Stable (tolerance for f32)
    assert_eq!(anchor.vfx_state(), AnchorVfxState::Stable);
}

#[test]
fn test_multiple_enemies_overwhelm_anchor() {
    // Setup: 1 anchor (Stable), 5 enemies
    let mut anchor = Anchor::new(0.75, 50, None);
    let anchor_pos = Vec3::new(10.0, 0.0, 5.0);

    let mut enemies = vec![];
    for i in 0..5 {
        let mut enemy = Enemy::new(anchor_pos + Vec3::new(i as f32 * 2.0, 0.0, 0.0), 5.0);
        enemy.target_anchor_id = Some(0);
        enemy.state = EnemyState::AttackAnchor;
        enemies.push(enemy);
    }

    // All 5 enemies attack anchor simultaneously (simplified damage calculation)
    // Real game: combat events trigger decay, here we directly adjust stability
    for _ in 0..5 {
        anchor.adjust_stability(-0.1); // Each enemy deals 0.1 stability damage
    }

    // Verify anchor broke
    assert!((anchor.stability() - 0.25).abs() < 0.01); // Stable (0.75) → Critical (0.25) (tolerance for f32)
    assert_eq!(anchor.vfx_state(), AnchorVfxState::Critical);
}

// ============================================================================
// Test 5: Spawn Rate Reduction (Max Concurrent Enemies)
// ============================================================================

#[test]
fn test_spawner_respects_max_concurrent_enemies() {
    // Setup: Max 20 enemies, 19 already spawned
    let mut anchor = Anchor::new(0.5, 50, None);
    let _anchors = vec![(0, &mut anchor)];

    let mut spawner = EnemySpawner::with_settings(
        30.0, // wave_interval
        3,    // base_enemies_per_wave
        20,   // max_concurrent_enemies
    );

    spawner.add_spawn_point(Vec3::new(10.0, 0.0, 5.0), 5.0, Some(0));

    // Set 19 enemies already alive
    spawner.set_active_enemy_count(19);

    // Force wave spawn (should spawn 1 enemy, not 3)
    let anchors_immut: Vec<(usize, &Anchor)> = vec![(0, &anchor)];
    let spawn_requests = spawner.update(31.0, &anchors_immut);

    // Verify only 1 spawn request (capacity limit: 20 - 19 = 1)
    assert_eq!(spawn_requests.len(), 1);
}

#[test]
fn test_spawner_stops_spawning_at_max_capacity() {
    // Setup: Max 20 enemies, 20 already spawned
    let mut anchor = Anchor::new(0.5, 50, None);
    let _anchors = vec![(0, &mut anchor)];

    let mut spawner = EnemySpawner::with_settings(
        30.0, // wave_interval
        3,    // base_enemies_per_wave
        20,   // max_concurrent_enemies
    );

    spawner.add_spawn_point(Vec3::new(10.0, 0.0, 5.0), 5.0, Some(0));

    // Set 20 enemies already alive (max capacity)
    spawner.set_active_enemy_count(20);

    // Force wave spawn (should spawn ZERO enemies)
    let anchors_immut: Vec<(usize, &Anchor)> = vec![(0, &anchor)];
    let spawn_requests = spawner.update(31.0, &anchors_immut);

    // Verify no spawn requests
    assert_eq!(spawn_requests.len(), 0);
}

// ============================================================================
// Test 6: Enemy State Transitions with Anchor Context
// ============================================================================

#[test]
fn test_enemy_transitions_from_patrol_to_attack_anchor() {
    // Setup: Enemy patrolling, broken anchor appears nearby
    let mut enemy = Enemy::new(Vec3::new(10.0, 0.0, 5.0), 5.0);
    enemy.state = EnemyState::Patrol;

    let player_pos = Vec3::new(50.0, 0.0, 50.0); // Player far away
    let broken_anchor_pos = Vec3::new(12.0, 0.0, 6.0);
    let broken_anchors = vec![(0, broken_anchor_pos)];

    // Update enemy - should detect broken anchor and transition
    let behavior = enemy.update(
        0.016,
        Vec3::new(10.0, 0.0, 5.0),
        player_pos,
        &broken_anchors,
    );

    // Verify transition
    assert_eq!(enemy.state, EnemyState::AttackAnchor);
    assert_eq!(enemy.target_anchor_id, Some(0));
    assert!(matches!(behavior, EnemyBehavior::MoveTo(_)));
}

#[test]
fn test_enemy_transitions_from_attack_anchor_to_engage_player() {
    // Setup: Enemy attacking anchor, player enters aggro range
    let mut enemy = Enemy::new(Vec3::new(10.0, 0.0, 5.0), 5.0);
    enemy.state = EnemyState::AttackAnchor;
    enemy.target_anchor_id = Some(0);

    let anchor_pos = Vec3::new(12.0, 0.0, 6.0);
    let player_pos = Vec3::new(15.0, 0.0, 8.0); // Within 10 unit aggro range
    let broken_anchors = vec![(0, anchor_pos)];

    // Update enemy - should detect player and prioritize
    let behavior = enemy.update(
        0.016,
        Vec3::new(10.0, 0.0, 5.0),
        player_pos,
        &broken_anchors,
    );

    // Verify transition (EngagePlayer > AttackAnchor priority)
    assert_eq!(enemy.state, EnemyState::EngagePlayer);
    assert!(matches!(
        behavior,
        EnemyBehavior::MoveTo(_) | EnemyBehavior::Attack(_)
    ));
}

#[test]
fn test_enemy_flees_when_low_health_even_near_anchor() {
    // Setup: Enemy attacking anchor, takes damage below flee threshold
    let mut enemy = Enemy::new(Vec3::new(10.0, 0.0, 5.0), 5.0);
    enemy.state = EnemyState::AttackAnchor;
    enemy.target_anchor_id = Some(0);
    enemy.health = 25.0; // Above flee threshold

    let anchor_pos = Vec3::new(12.0, 0.0, 6.0);
    let player_pos = Vec3::new(50.0, 0.0, 50.0); // Player far away
    let broken_anchors = vec![(0, anchor_pos)];

    // Take damage → below 20 HP flee threshold
    enemy.take_damage(10.0);
    assert_eq!(enemy.health, 15.0);

    // Update enemy - should flee (Flee > AttackAnchor priority)
    let behavior = enemy.update(
        0.016,
        Vec3::new(10.0, 0.0, 5.0),
        player_pos,
        &broken_anchors,
    );

    // Verify flee
    assert_eq!(enemy.state, EnemyState::Flee);
    assert!(matches!(behavior, EnemyBehavior::MoveTo(_)));
}
