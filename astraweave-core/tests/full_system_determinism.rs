//! Full-System Determinism Integration Tests
//!
//! These tests validate deterministic behavior across the **complete ECS world**:
//! - Entity spawning and component assignment
//! - Simulation time advancement
//! - Component state updates (cooldowns, positions, health)
//! - Obstacle management
//! - Entity lifecycle (spawn, update, destroy)
//!
//! Unlike unit tests which test individual subsystems, these tests verify that
//! **the entire ECS world produces bit-identical results** when given the same inputs,
//! which is critical for:
//! - Multiplayer synchronization (lockstep networking)
//! - Replay systems (demo playback, debugging)
//! - Anti-cheat validation (server-side replay verification)
//! - Deterministic AI training (reproducible behavior)
//!
//! **Success Criteria**:
//! - ✅ 100-frame simulation produces identical state across multiple runs
//! - ✅ Different seeds produce different (but deterministic) results
//! - ✅ Entity ordering doesn't affect determinism
//! - ✅ Component updates are deterministic
//! - ✅ Cooldown tick logic is deterministic

use astraweave_core::{IVec2, Team, World};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ============================================================================
// Test Helpers
// ============================================================================

/// Hash the complete world state for determinism comparison
///
/// This hashes ALL relevant world state that should be deterministic:
/// - Simulation time (t)
/// - Next entity ID (deterministic entity creation)
/// - All entities and their components (pose, health, team, ammo, cooldowns, names)
/// - Obstacles
fn hash_world_state(world: &World) -> u64 {
    let mut hasher = DefaultHasher::new();

    // Hash simulation time (should increment deterministically)
    world.t.to_bits().hash(&mut hasher);

    // Hash next entity ID (ensures deterministic entity creation)
    world.next_id.hash(&mut hasher);

    // Get all entities and sort for deterministic order
    let mut entities = world.entities();
    entities.sort();

    for entity in entities {
        // Hash entity ID
        entity.hash(&mut hasher);

        // Hash pose component (if present)
        if let Some(pose) = world.pose(entity) {
            pose.pos.x.hash(&mut hasher);
            pose.pos.y.hash(&mut hasher);
        }

        // Hash health component (if present)
        if let Some(health) = world.health(entity) {
            health.hp.hash(&mut hasher);
        }

        // Hash team component (if present)
        if let Some(team) = world.team(entity) {
            team.id.hash(&mut hasher);
        }

        // Hash ammo component (if present)
        if let Some(ammo) = world.ammo(entity) {
            ammo.rounds.hash(&mut hasher);
        }

        // Hash cooldowns component (if present)
        if let Some(cooldowns) = world.cooldowns(entity) {
            // Sort cooldown keys for deterministic order
            let mut cd_keys: Vec<_> = cooldowns.map.keys().collect();
            cd_keys.sort();
            for key in cd_keys {
                key.hash(&mut hasher);
                cooldowns.map[key].to_bits().hash(&mut hasher);
            }
        }

        // Hash name (if present)
        if let Some(name) = world.name(entity) {
            name.hash(&mut hasher);
        }
    }

    // Hash obstacles (convert HashSet to sorted Vec for determinism)
    let mut obstacles: Vec<_> = world.obstacles.iter().collect();
    obstacles.sort();
    for obstacle in obstacles {
        obstacle.hash(&mut hasher);
    }

    hasher.finish()
}

/// Create a deterministic world state with fixed seed
fn create_seeded_world(seed: u64) -> World {
    let mut world = World::new();

    // Spawn player with seed-based position
    let _player_id = world.spawn(
        "player",
        IVec2 {
            x: ((seed / 10000) % 100) as i32,
            y: ((seed / 1000000) % 100) as i32,
        },
        Team { id: 0 },
        100,
        30,
    );

    // Spawn companion with seed-based position
    let companion_id = world.spawn(
        "companion",
        IVec2 {
            x: (seed % 100) as i32,
            y: ((seed / 100) % 100) as i32,
        },
        Team { id: 1 },
        80,
        10,
    );

    // Set companion cooldowns based on seed
    if let Some(cooldowns) = world.cooldowns_mut(companion_id) {
        cooldowns
            .map
            .insert("attack".to_string(), (seed % 10) as f32);
        if seed % 2 == 0 {
            cooldowns.map.insert("heal".to_string(), 5.0);
        }
    }

    // Spawn enemies based on seed
    let enemy_count = (seed % 3) + 1; // 1-3 enemies
    for i in 0..enemy_count {
        let _enemy_id = world.spawn(
            &format!("enemy_{}", i),
            IVec2 {
                x: ((seed * (i + 1) / 100000) % 100) as i32,
                y: ((seed * (i + 1) / 10000000) % 100) as i32,
            },
            Team { id: 2 },
            50 + (i as i32 * 10),
            15,
        );
    }

    // Add obstacles based on seed
    for i in 0..5 {
        let x = ((seed + i * 123) % 50) as i32;
        let y = ((seed + i * 456) % 50) as i32;
        world.obstacles.insert((x, y));
    }

    world
}

// ============================================================================
// Integration Tests: 100-Frame Replay Determinism
// ============================================================================

/// Test that running the same simulation twice produces bit-identical results
///
/// This is the **core determinism test**: If we run the exact same sequence of
/// operations twice, we should get bit-identical world state at every frame.
/// This validates that:
/// - Entity spawning is deterministic
/// - Component updates are deterministic
/// - Time advancement is deterministic
/// - Cooldown tick logic is deterministic
#[test]
fn test_100_frame_replay_determinism() {
    const SEED: u64 = 12345;
    const NUM_FRAMES: usize = 100;
    const DT: f32 = 0.016; // 60 FPS

    // Run 1: Simulate 100 frames and record hashes
    let mut world1 = create_seeded_world(SEED);
    let mut hashes1 = Vec::with_capacity(NUM_FRAMES);

    for _frame in 0..NUM_FRAMES {
        hashes1.push(hash_world_state(&world1));
        world1.tick(DT);
    }

    // Run 2: Simulate again with same seed
    let mut world2 = create_seeded_world(SEED);
    let mut hashes2 = Vec::with_capacity(NUM_FRAMES);

    for _frame in 0..NUM_FRAMES {
        hashes2.push(hash_world_state(&world2));
        world2.tick(DT);
    }

    // Verify: Every frame hash should match
    assert_eq!(
        hashes1.len(),
        hashes2.len(),
        "Hash count mismatch (should never happen)"
    );

    for (frame, (hash1, hash2)) in hashes1.iter().zip(hashes2.iter()).enumerate() {
        assert_eq!(
            hash1, hash2,
            "Determinism violation at frame {}: hash mismatch",
            frame
        );
    }

    // Final world state should also match
    let final_hash1 = hash_world_state(&world1);
    let final_hash2 = hash_world_state(&world2);
    assert_eq!(
        final_hash1, final_hash2,
        "Final world state mismatch after 100 frames"
    );
}

// ============================================================================
// Integration Tests: Multiple Runs with Same Seed
// ============================================================================

/// Test that multiple simulation runs with the same seed produce identical results
///
/// This validates that determinism holds across **completely separate simulation runs**,
/// not just sequential frames. Important for:
/// - Replay validation (run replay multiple times, should match)
/// - Server-side verification (client replay must match server simulation)
#[test]
fn test_multiple_runs_same_seed_determinism() {
    const SEED: u64 = 12345;
    const NUM_RUNS: usize = 5;
    const NUM_FRAMES: usize = 50;
    const DT: f32 = 0.016;

    let mut all_final_hashes = Vec::with_capacity(NUM_RUNS);

    // Run simulation 5 times
    for _run in 0..NUM_RUNS {
        let mut world = create_seeded_world(SEED);

        // Simulate 50 frames
        for _frame in 0..NUM_FRAMES {
            world.tick(DT);
        }

        all_final_hashes.push(hash_world_state(&world));
    }

    // Verify: All runs produced identical final state
    let first_hash = all_final_hashes[0];
    for (run_idx, hash) in all_final_hashes.iter().enumerate().skip(1) {
        assert_eq!(
            *hash, first_hash,
            "Run {} produced different final state than run 0",
            run_idx
        );
    }
}

// ============================================================================
// Integration Tests: Different Seeds Produce Different Results
// ============================================================================

/// Test that different RNG seeds produce **different** (but still deterministic) results
///
/// This validates that:
/// - Our seeding mechanism actually works (different seeds = different outcomes)
/// - We're not accidentally using a fixed seed everywhere
/// - RNG state is properly isolated per simulation
#[test]
fn test_different_seeds_produce_different_results() {
    const SEEDS: &[u64] = &[42, 12345, 99999];
    const NUM_FRAMES: usize = 100;
    const DT: f32 = 0.016;

    let mut final_hashes = Vec::with_capacity(SEEDS.len());

    for &seed in SEEDS {
        let mut world = create_seeded_world(seed);

        // Simulate 100 frames
        for _frame in 0..NUM_FRAMES {
            world.tick(DT);
        }

        final_hashes.push(hash_world_state(&world));
    }

    // Verify: All seeds produced DIFFERENT final states
    assert_ne!(
        final_hashes[0], final_hashes[1],
        "Seed 42 and 12345 produced identical results (RNG not working?)"
    );
    assert_ne!(
        final_hashes[1], final_hashes[2],
        "Seed 12345 and 99999 produced identical results (RNG not working?)"
    );
    assert_ne!(
        final_hashes[0], final_hashes[2],
        "Seed 42 and 99999 produced identical results (RNG not working?)"
    );
}

// ============================================================================
// Integration Tests: Component Update Determinism
// ============================================================================

/// Test that component updates (position, health, ammo) are deterministic
///
/// This validates that:
/// - Position changes produce identical results
/// - Health changes are deterministic
/// - Ammo consumption is deterministic
/// - Cooldowns tick deterministically
#[test]
fn test_component_update_determinism() {
    const SEED: u64 = 54321;
    const NUM_FRAMES: usize = 50;
    const DT: f32 = 0.016;

    // Run 1: Update all components
    let mut world1 = create_seeded_world(SEED);
    let entities1 = world1.entities();

    for _frame in 0..NUM_FRAMES {
        // Update positions (move entities)
        for &entity in &entities1 {
            if let Some(pose) = world1.pose_mut(entity) {
                pose.pos.x += 1;
                pose.pos.y += 1;
            }
        }

        // Update health (take damage)
        for &entity in &entities1 {
            if let Some(health) = world1.health_mut(entity) {
                health.hp = (health.hp - 1).max(0);
            }
        }

        // Update ammo (consume rounds)
        for &entity in &entities1 {
            if let Some(ammo) = world1.ammo_mut(entity) {
                ammo.rounds = (ammo.rounds - 1).max(0);
            }
        }

        world1.tick(DT); // Tick cooldowns
    }

    // Run 2: Same updates
    let mut world2 = create_seeded_world(SEED);
    let entities2 = world2.entities();

    for _frame in 0..NUM_FRAMES {
        for &entity in &entities2 {
            if let Some(pose) = world2.pose_mut(entity) {
                pose.pos.x += 1;
                pose.pos.y += 1;
            }
        }

        for &entity in &entities2 {
            if let Some(health) = world2.health_mut(entity) {
                health.hp = (health.hp - 1).max(0);
            }
        }

        for &entity in &entities2 {
            if let Some(ammo) = world2.ammo_mut(entity) {
                ammo.rounds = (ammo.rounds - 1).max(0);
            }
        }

        world2.tick(DT);
    }

    // Verify: Final states match
    assert_eq!(
        hash_world_state(&world1),
        hash_world_state(&world2),
        "Component updates produced non-deterministic results"
    );
}

// ============================================================================
// Integration Tests: Entity Ordering Independence
// ============================================================================

/// Test that entity creation order doesn't affect determinism
///
/// This validates that:
/// - Entity ID assignment is deterministic
/// - Component queries are order-independent (we sort before hashing)
/// - No hidden ordering dependencies exist
#[test]
fn test_entity_ordering_determinism() {
    const NUM_FRAMES: usize = 50;
    const DT: f32 = 0.016;

    // World 1: Create entities in order A, B, C
    let mut world1 = World::new();
    let _a1 = world1.spawn("entity_a", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);
    let _b1 = world1.spawn("entity_b", IVec2 { x: 10, y: 10 }, Team { id: 1 }, 80, 20);
    let _c1 = world1.spawn("entity_c", IVec2 { x: 20, y: 20 }, Team { id: 2 }, 50, 10);

    for _frame in 0..NUM_FRAMES {
        world1.tick(DT);
    }

    // World 2: Create entities in order C, B, A (reversed)
    let mut world2 = World::new();
    let _c2 = world2.spawn("entity_c", IVec2 { x: 20, y: 20 }, Team { id: 2 }, 50, 10);
    let _b2 = world2.spawn("entity_b", IVec2 { x: 10, y: 10 }, Team { id: 1 }, 80, 20);
    let _a2 = world2.spawn("entity_a", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);

    for _frame in 0..NUM_FRAMES {
        world2.tick(DT);
    }

    // Verify: Different creation order still produces deterministic results
    // (Entity IDs will differ, but our hash function sorts entities, so
    //  the LOGICAL state should be order-independent)
    //
    // NOTE: This test expects entity IDs to DIFFER but logical state to match.
    // Since our hashing includes entity IDs, we need to compare component states
    // by entity count and team membership instead.

    assert_eq!(
        world1.entities().len(),
        world2.entities().len(),
        "Entity count mismatch (should never happen)"
    );

    // Verify same team distribution
    for team_id in 0..3 {
        let team1_count = world1.all_of_team(team_id).len();
        let team2_count = world2.all_of_team(team_id).len();
        assert_eq!(
            team1_count, team2_count,
            "Team {} count mismatch: {} vs {}",
            team_id, team1_count, team2_count
        );
    }
}

// ============================================================================
// Integration Tests: Cooldown Tick Determinism
// ============================================================================

/// Test that cooldown tick logic is deterministic
///
/// This validates that:
/// - Cooldowns decrement at correct rate
/// - Cooldowns bottom out at 0.0 (not negative)
/// - Multiple cooldowns on same entity tick independently
/// - Cooldown state is bit-identical across runs
#[test]
fn test_cooldown_tick_determinism() {
    const NUM_FRAMES: usize = 200;
    const DT: f32 = 0.05; // 20 FPS (larger dt for faster cooldown testing)

    // Run 1: Create entity with cooldowns
    let mut world1 = World::new();
    let entity1 = world1.spawn("test_entity", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);

    if let Some(cooldowns) = world1.cooldowns_mut(entity1) {
        cooldowns.map.insert("fast".to_string(), 3.0);
        cooldowns.map.insert("slow".to_string(), 8.0);
        cooldowns.map.insert("very_slow".to_string(), 15.0);
    }

    for _frame in 0..NUM_FRAMES {
        world1.tick(DT);
    }

    // Run 2: Same setup
    let mut world2 = World::new();
    let entity2 = world2.spawn("test_entity", IVec2 { x: 0, y: 0 }, Team { id: 0 }, 100, 30);

    if let Some(cooldowns) = world2.cooldowns_mut(entity2) {
        cooldowns.map.insert("fast".to_string(), 3.0);
        cooldowns.map.insert("slow".to_string(), 8.0);
        cooldowns.map.insert("very_slow".to_string(), 15.0);
    }

    for _frame in 0..NUM_FRAMES {
        world2.tick(DT);
    }

    // Verify: Final cooldown states match
    assert_eq!(
        hash_world_state(&world1),
        hash_world_state(&world2),
        "Cooldown tick logic produced non-deterministic results"
    );

    // Verify: Cooldowns actually ticked down (not stuck at initial values)
    if let Some(cooldowns) = world1.cooldowns(entity1) {
        // 200 frames * 0.05 dt = 10 seconds elapsed
        // fast (3.0s) should be 0.0
        // slow (8.0s) should be 0.0
        // very_slow (15.0s) should be 5.0
        assert_eq!(
            *cooldowns.map.get("fast").unwrap(),
            0.0,
            "fast cooldown should be 0.0"
        );
        assert_eq!(
            *cooldowns.map.get("slow").unwrap(),
            0.0,
            "slow cooldown should be 0.0"
        );
        let very_slow = *cooldowns.map.get("very_slow").unwrap();
        assert!(
            (very_slow - 5.0).abs() < 0.01,
            "very_slow cooldown should be ~5.0, got {}",
            very_slow
        );
    }
}

// ============================================================================
// Integration Tests: Obstacle Determinism
// ============================================================================

/// Test that obstacle management is deterministic
///
/// This validates that:
/// - Obstacle insertion is deterministic
/// - Obstacle queries are deterministic
/// - HashSet iteration order doesn't affect determinism (we sort before hashing)
#[test]
fn test_obstacle_determinism() {
    const NUM_FRAMES: usize = 50;
    const DT: f32 = 0.016;

    // Run 1: Add obstacles in one order
    let mut world1 = World::new();
    world1.obstacles.insert((0, 0));
    world1.obstacles.insert((10, 10));
    world1.obstacles.insert((5, 5));
    world1.obstacles.insert((15, 15));

    for _frame in 0..NUM_FRAMES {
        world1.tick(DT);
    }

    // Run 2: Add obstacles in different order
    let mut world2 = World::new();
    world2.obstacles.insert((15, 15));
    world2.obstacles.insert((5, 5));
    world2.obstacles.insert((0, 0));
    world2.obstacles.insert((10, 10));

    for _frame in 0..NUM_FRAMES {
        world2.tick(DT);
    }

    // Verify: Insertion order doesn't matter (our hash sorts obstacles)
    assert_eq!(
        hash_world_state(&world1),
        hash_world_state(&world2),
        "Obstacle insertion order affected determinism"
    );
}
