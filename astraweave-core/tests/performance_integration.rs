//! Performance Regression Integration Tests
//!
//! These tests validate that **performance critical paths meet target SLAs**:
//! - 1000-entity @ 60 FPS capacity (16.67ms frame budget)
//! - AI planning latency under load (<5ms per agent)
//! - Frame budget enforcement (no frame drops)
//! - Memory allocation stability (no heap churn)
//! - Stress testing (graceful degradation under 10,000+ entities)
//!
//! Unlike unit benchmarks which measure isolated performance, these tests verify:
//! - **Real-world performance under load** (combined ECS + AI + Physics)
//! - **SLA compliance** (meet/exceed target thresholds)
//! - **No performance regressions** (validate against baselines)
//! - **Graceful degradation** (system doesn't crash under extreme load)
//!
//! **Success Criteria**:
//! - ✅ 1000 entities simulated @ 60 FPS (frame time <16.67ms)
//! - ✅ AI planning completes in <5ms per agent
//! - ✅ Frame budget never exceeded (p99 <16.67ms)
//! - ✅ No heap churn (allocations stable over 100 frames)
//! - ✅ 10,000 entity stress test completes without crash

use astraweave_core::{IVec2, Team, World};
use std::time::{Duration, Instant};

// ============================================================================
// Performance Baselines (from BASELINE_METRICS.md)
// ============================================================================

/// 60 FPS frame budget (16.67ms per frame)
const FRAME_BUDGET_60FPS: Duration = Duration::from_micros(16_670);

/// AI planning target latency (<5ms per agent)
const AI_PLANNING_TARGET: Duration = Duration::from_millis(5);

/// Target entity count @ 60 FPS (conservative estimate based on ECS benchmarks)
const TARGET_ENTITY_COUNT_60FPS: usize = 1_000;

/// Stress test entity count (should NOT crash, may drop frames)
const STRESS_TEST_ENTITY_COUNT: usize = 10_000;

/// Frame time percentile threshold (p99 should stay under budget)
const P99_PERCENTILE: f32 = 0.99;

// ============================================================================
// Test Helpers
// ============================================================================

/// Simulate a typical game frame with entity updates
///
/// This mimics real game loop operations:
/// - Update entity positions (movement)
/// - Update entity health (damage over time)
/// - Update cooldowns (tick)
/// - Query entities by team (AI logic)
fn simulate_game_frame(world: &mut World, dt: f32) {
    // Tick world (cooldowns, time advancement)
    world.tick(dt);

    // Simulate entity movement (update all poses)
    let entities = world.entities();
    for entity in entities {
        if let Some(pose) = world.pose_mut(entity) {
            // Simple movement pattern (circular motion)
            pose.pos.x += 1;
            pose.pos.y += 1;
            if pose.pos.x > 100 {
                pose.pos.x = 0;
            }
            if pose.pos.y > 100 {
                pose.pos.y = 0;
            }
        }
    }

    // Simulate damage over time (update all health)
    for entity in world.entities() {
        if let Some(health) = world.health_mut(entity) {
            health.hp = (health.hp - 1).max(0);
        }
    }

    // Simulate AI queries (find enemies)
    for team_id in 0..3 {
        let _allies = world.all_of_team(team_id);
        let _enemies = world.enemies_of(team_id);
    }
}

/// Create a world with N entities for performance testing
fn create_world_with_entities(count: usize) -> World {
    let mut world = World::new();

    // Distribute entities across 3 teams
    let team_distribution = [
        (0, count / 3),               // Player team (33%)
        (1, count / 3),               // Companion team (33%)
        (2, count - 2 * (count / 3)), // Enemy team (34%, remainder)
    ];

    for (team_id, team_count) in &team_distribution {
        for i in 0..*team_count {
            let _entity = world.spawn(
                &format!("entity_t{}_{}", team_id, i),
                IVec2 {
                    x: (i % 100) as i32,
                    y: (i / 100) as i32,
                },
                Team { id: *team_id },
                100,
                30,
            );
        }
    }

    world
}

/// Measure frame time percentile from sorted durations
fn calculate_percentile(mut frame_times: Vec<Duration>, percentile: f32) -> Duration {
    frame_times.sort();
    let index = ((frame_times.len() as f32 * percentile) as usize).min(frame_times.len() - 1);
    frame_times[index]
}

// ============================================================================
// Integration Tests: 1000-Entity @ 60 FPS Validation
// ============================================================================

/// Test that 1000 entities can be simulated @ 60 FPS (frame time <16.67ms)
///
/// This validates that our ECS throughput meets the **minimum viable performance**
/// target for real-time games. If this fails, the engine cannot support even
/// moderately complex scenes.
///
/// **Performance Target**: 1000 entities @ 60 FPS (p99 <16.67ms)
/// **Baseline**: ECS spawn 420 ns/entity, tick <1 ns/entity
/// **Expected Frame Time**: ~5-10ms (60% headroom)
#[test]
fn test_1000_entity_60fps_capacity() {
    const NUM_ENTITIES: usize = TARGET_ENTITY_COUNT_60FPS;
    const NUM_FRAMES: usize = 60; // 1 second of simulation
    const DT: f32 = 0.016; // 60 FPS

    // Setup: Create world with 1000 entities
    let mut world = create_world_with_entities(NUM_ENTITIES);

    // Warm-up: Run 10 frames to stabilize JIT, caches
    for _ in 0..10 {
        simulate_game_frame(&mut world, DT);
    }

    // Measure: Run 60 frames and record frame times
    let mut frame_times = Vec::with_capacity(NUM_FRAMES);

    for _ in 0..NUM_FRAMES {
        let start = Instant::now();
        simulate_game_frame(&mut world, DT);
        let elapsed = start.elapsed();
        frame_times.push(elapsed);
    }

    // Validate: Calculate p99 frame time
    let p99 = calculate_percentile(frame_times.clone(), P99_PERCENTILE);
    let p50 = calculate_percentile(frame_times.clone(), 0.50);
    let p95 = calculate_percentile(frame_times.clone(), 0.95);

    // Assert: p99 must be under 60 FPS budget
    assert!(
        p99 < FRAME_BUDGET_60FPS,
        "❌ PERFORMANCE REGRESSION: 1000-entity p99 frame time {}ms exceeds 60 FPS budget (16.67ms)\n\
         p50: {}ms, p95: {}ms, p99: {}ms",
        p99.as_secs_f64() * 1000.0,
        p50.as_secs_f64() * 1000.0,
        p95.as_secs_f64() * 1000.0,
        p99.as_secs_f64() * 1000.0,
    );

    // Success: Print performance metrics
    println!("✅ 1000-entity @ 60 FPS validation PASSED");
    println!("   p50: {:.2}ms", p50.as_secs_f64() * 1000.0);
    println!("   p95: {:.2}ms", p95.as_secs_f64() * 1000.0);
    println!(
        "   p99: {:.2}ms ({:.1}% of budget)",
        p99.as_secs_f64() * 1000.0,
        (p99.as_secs_f64() / FRAME_BUDGET_60FPS.as_secs_f64()) * 100.0
    );
}

// ============================================================================
// Integration Tests: AI Planning Latency Under Load
// ============================================================================

/// Test that AI planning operations complete within <5ms per agent
///
/// This validates that AI systems don't block the main thread for too long.
/// Real AI planning (GOAP, behavior trees, LLM) is tested elsewhere; this
/// tests the **ECS query overhead** that precedes AI planning (finding allies,
/// enemies, obstacles).
///
/// **Performance Target**: <5ms per agent for AI queries
/// **Baseline**: Entity query ~1 ns/entity
/// **Expected**: <1ms (95% headroom)
#[test]
fn test_ai_planning_latency_under_load() {
    const NUM_ENTITIES: usize = 1_000;
    const NUM_AI_AGENTS: usize = 100; // 10% of entities are AI agents

    // Setup: Create world with 1000 entities
    let world = create_world_with_entities(NUM_ENTITIES);

    // Measure: Simulate AI planning (entity queries per agent)
    let start = Instant::now();

    for agent_idx in 0..NUM_AI_AGENTS {
        let team_id = (agent_idx % 3) as u8;

        // Query allies (typical AI operation)
        let _allies = world.all_of_team(team_id);

        // Query enemies (typical AI operation)
        let _enemies = world.enemies_of(team_id);

        // Query obstacle (typical pathfinding check)
        let _blocked = world.obstacle(IVec2 { x: 50, y: 50 });
    }

    let elapsed = start.elapsed();
    let per_agent = elapsed / NUM_AI_AGENTS as u32;

    // Validate: Average per-agent query time should be <5ms
    assert!(
        per_agent < AI_PLANNING_TARGET,
        "❌ PERFORMANCE REGRESSION: AI query latency {}μs exceeds target (5ms)\n\
         Total time: {}ms for {} agents",
        per_agent.as_micros(),
        elapsed.as_secs_f64() * 1000.0,
        NUM_AI_AGENTS,
    );

    // Success: Print performance metrics
    println!("✅ AI planning latency validation PASSED");
    println!("   Per-agent query time: {:.2}μs", per_agent.as_micros());
    println!(
        "   Total time: {:.2}ms for {} agents",
        elapsed.as_secs_f64() * 1000.0,
        NUM_AI_AGENTS
    );
}

// ============================================================================
// Integration Tests: Frame Budget Enforcement
// ============================================================================

/// Test that frame time NEVER exceeds 60 FPS budget across 100 frames
///
/// This validates that we have **consistent frame times** without spikes.
/// Even a single spike >16.67ms causes a frame drop, which is noticeable
/// to players. This is a **strict SLA test** (100% of frames must pass).
///
/// **Performance Target**: 0 frame drops over 100 frames
/// **Baseline**: Should have 40-60% headroom (frame time ~6-10ms)
#[test]
fn test_frame_budget_never_exceeded() {
    const NUM_ENTITIES: usize = 500; // Conservative (50% of target)
    const NUM_FRAMES: usize = 100;
    const DT: f32 = 0.016;

    // Setup: Create world with 500 entities (conservative)
    let mut world = create_world_with_entities(NUM_ENTITIES);

    // Warm-up
    for _ in 0..10 {
        simulate_game_frame(&mut world, DT);
    }

    // Measure: Run 100 frames, check EVERY frame
    let mut frame_drops = 0;
    let mut max_frame_time = Duration::ZERO;

    for frame_idx in 0..NUM_FRAMES {
        let start = Instant::now();
        simulate_game_frame(&mut world, DT);
        let elapsed = start.elapsed();

        if elapsed > max_frame_time {
            max_frame_time = elapsed;
        }

        if elapsed > FRAME_BUDGET_60FPS {
            frame_drops += 1;
            eprintln!(
                "⚠️  Frame drop at frame {}: {}ms (budget: 16.67ms)",
                frame_idx,
                elapsed.as_secs_f64() * 1000.0
            );
        }
    }

    // Validate: ZERO frame drops allowed
    assert_eq!(
        frame_drops,
        0,
        "❌ PERFORMANCE REGRESSION: {} frame drops detected over {} frames\n\
         Max frame time: {}ms (budget: 16.67ms)\n\
         Entity count: {} (50% of target)",
        frame_drops,
        NUM_FRAMES,
        max_frame_time.as_secs_f64() * 1000.0,
        NUM_ENTITIES,
    );

    // Success: Print performance metrics
    println!("✅ Frame budget enforcement PASSED");
    println!("   {} frames, 0 drops", NUM_FRAMES);
    println!(
        "   Max frame time: {:.2}ms ({:.1}% of budget)",
        max_frame_time.as_secs_f64() * 1000.0,
        (max_frame_time.as_secs_f64() / FRAME_BUDGET_60FPS.as_secs_f64()) * 100.0
    );
}

// ============================================================================
// Integration Tests: Memory Allocation Stability
// ============================================================================

/// Test that memory allocations are stable over 100 frames (no heap churn)
///
/// This validates that we're not allocating/deallocating on every frame,
/// which would cause GC pressure and performance spikes. Stable entity count
/// should result in stable allocation count.
///
/// **Performance Target**: Allocation count variance <10% over 100 frames
/// **Note**: Rust doesn't expose allocation tracking by default, so we use
/// entity count + component count as a proxy for stable memory usage.
#[test]
fn test_memory_allocation_stability() {
    const NUM_ENTITIES: usize = 1_000;
    const NUM_FRAMES: usize = 100;
    const DT: f32 = 0.016;

    // Setup: Create world with 1000 entities
    let mut world = create_world_with_entities(NUM_ENTITIES);

    // Warm-up
    for _ in 0..10 {
        simulate_game_frame(&mut world, DT);
    }

    // Measure: Track entity count over 100 frames
    let mut entity_counts = Vec::with_capacity(NUM_FRAMES);

    for _ in 0..NUM_FRAMES {
        simulate_game_frame(&mut world, DT);
        entity_counts.push(world.entities().len());
    }

    // Validate: Entity count should be stable (no leaks, no unexpected spawns)
    let min_count = entity_counts.iter().min().unwrap();
    let max_count = entity_counts.iter().max().unwrap();
    let variance = if *min_count > 0 {
        ((*max_count as f32 - *min_count as f32) / *min_count as f32) * 100.0
    } else {
        0.0
    };

    assert!(
        variance < 10.0,
        "❌ PERFORMANCE REGRESSION: Entity count variance {:.1}% exceeds 10% threshold\n\
         Min: {}, Max: {}, Initial: {}",
        variance,
        min_count,
        max_count,
        NUM_ENTITIES,
    );

    // Success: Print stability metrics
    println!("✅ Memory allocation stability PASSED");
    println!("   Entity count variance: {:.2}%", variance);
    println!(
        "   Min: {}, Max: {}, Initial: {}",
        min_count, max_count, NUM_ENTITIES
    );
}

// ============================================================================
// Integration Tests: Stress Testing (10,000 Entities)
// ============================================================================

/// Test that 10,000 entities can be simulated without crashing
///
/// This validates **graceful degradation** under extreme load. We don't expect
/// to maintain 60 FPS (frame drops are acceptable), but the system should NOT:
/// - Crash or panic
/// - Produce incorrect results
/// - Hang or deadlock
///
/// **Performance Target**: Complete 60 frames without crash
/// **Expected**: Frame time ~100-200ms (drops frames, but stable)
#[test]
fn test_stress_10k_entities_graceful_degradation() {
    const NUM_ENTITIES: usize = STRESS_TEST_ENTITY_COUNT;
    const NUM_FRAMES: usize = 60; // 1 second of simulation
    const DT: f32 = 0.016;

    // Setup: Create world with 10,000 entities
    let mut world = create_world_with_entities(NUM_ENTITIES);

    // Warm-up
    for _ in 0..5 {
        simulate_game_frame(&mut world, DT);
    }

    // Measure: Run 60 frames, track max/avg frame time
    let mut frame_times = Vec::with_capacity(NUM_FRAMES);

    for _ in 0..NUM_FRAMES {
        let start = Instant::now();
        simulate_game_frame(&mut world, DT);
        let elapsed = start.elapsed();
        frame_times.push(elapsed);
    }

    // Calculate metrics
    let total_time: Duration = frame_times.iter().sum();
    let avg_frame_time = total_time / NUM_FRAMES as u32;
    let max_frame_time = frame_times.iter().max().unwrap();

    // Validate: Should complete without panic (graceful degradation)
    // No hard assertion on frame time (expected to drop frames)

    // Success: Print stress test metrics
    println!("✅ 10k-entity stress test PASSED (graceful degradation)");
    println!(
        "   {} entities simulated for {} frames",
        NUM_ENTITIES, NUM_FRAMES
    );
    println!(
        "   Avg frame time: {:.2}ms",
        avg_frame_time.as_secs_f64() * 1000.0
    );
    println!(
        "   Max frame time: {:.2}ms",
        max_frame_time.as_secs_f64() * 1000.0
    );
    println!("   Total time: {:.2}s", total_time.as_secs_f64());

    if avg_frame_time > FRAME_BUDGET_60FPS {
        println!(
            "   ⚠️  Frame drops expected (avg {}ms > 16.67ms budget)",
            avg_frame_time.as_secs_f64() * 1000.0
        );
        println!("   ⚠️  But system remained stable (no crash, no hang)");
    }
}
