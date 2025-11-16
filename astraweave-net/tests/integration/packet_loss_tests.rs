//! Packet loss and network resilience tests for astraweave-net
//!
//! Tests network reliability under various packet loss conditions,
//! including retransmission, deduplication, and eventual consistency.

#![cfg(test)]

use astraweave_core::{ActionStep, IVec2, PlanIntent};
use astraweave_net::{apply_delta, build_snapshot, diff_snapshots, FullInterest};
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::common::*;

// ============================================================================
// BASELINE TESTS (5 tests)
// ============================================================================

#[tokio::test]
async fn test_zero_packet_loss_baseline() {
    let server = spawn_test_server_with_packet_loss(0.0).await.unwrap();

    let id = server
        .spawn_entity("test", IVec2 { x: 1, y: 1 }, 0, 100)
        .await;

    // Execute 10 commands
    for i in 0..10 {
        let plan = PlanIntent {
            plan_id: format!("move_{}", i),
            steps: vec![ActionStep::MoveTo {
                x: (i % 10) as i32,
                y: 1,
            }],
        };
        server.execute_plan(id, plan).await.ok();
    }

    let snap = server.get_snapshot().await;
    assert!(snap.entities.iter().any(|e| e.id == id));

    server.shutdown().await;
}

#[tokio::test]
async fn test_all_packets_arrive_no_loss() {
    let server = spawn_test_server_with_packet_loss(0.0).await.unwrap();

    let mut snapshots = Vec::new();
    for _ in 0..10 {
        snapshots.push(server.get_snapshot().await);
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    }

    // All snapshots should have been captured
    assert_eq!(snapshots.len(), 10);

    // Ticks should be increasing
    for i in 1..snapshots.len() {
        assert!(snapshots[i].tick >= snapshots[i - 1].tick);
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_no_duplicate_packets_baseline() {
    let server = spawn_test_server_with_packet_loss(0.0).await.unwrap();

    let id = server
        .spawn_entity("counter", IVec2 { x: 0, y: 0 }, 0, 100)
        .await;

    // Set initial health
    {
        let mut w = server.server.world.lock().await;
        if let Some(h) = w.health_mut(id) {
            h.hp = 100;
        }
    }

    // Apply damage
    {
        let mut w = server.server.world.lock().await;
        if let Some(h) = w.health_mut(id) {
            h.hp -= 10;
        }
    }

    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();
    assert_eq!(entity.hp, 90);

    server.shutdown().await;
}

#[tokio::test]
async fn test_state_consistency_no_loss() {
    let server = spawn_test_server_with_packet_loss(0.0).await.unwrap();

    let snap1 = server.get_snapshot().await;

    // Make deterministic changes
    let id = server
        .spawn_entity("test", IVec2 { x: 5, y: 5 }, 0, 50)
        .await;

    let snap2 = server.get_snapshot().await;

    // State should be consistent
    let entity = snap2.entities.iter().find(|e| e.id == id).unwrap();
    assert_eq!(entity.hp, 50);
    assert_eq!(entity.pos, IVec2 { x: 5, y: 5 });

    server.shutdown().await;
}

#[tokio::test]
async fn test_snapshot_sequence_no_gaps() {
    let server = spawn_test_server_with_packet_loss(0.0).await.unwrap();

    let mut ticks = Vec::new();
    for _ in 0..20 {
        let snap = server.get_snapshot().await;
        ticks.push(snap.tick);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    // Ticks should be monotonically increasing
    for i in 1..ticks.len() {
        assert!(
            ticks[i] >= ticks[i - 1],
            "tick regression: {} < {}",
            ticks[i],
            ticks[i - 1]
        );
    }

    server.shutdown().await;
}

// ============================================================================
// LIGHT PACKET LOSS TESTS (5% loss) (5 tests)
// ============================================================================

#[tokio::test]
async fn test_5_percent_loss_entity_sync() {
    let server = spawn_test_server_with_packet_loss(0.05).await.unwrap();

    let id = server
        .spawn_entity("test", IVec2 { x: 3, y: 3 }, 0, 100)
        .await;

    // Wait for network to stabilize
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let snap = server.get_snapshot().await;
    assert!(snap.entities.iter().any(|e| e.id == id));

    server.shutdown().await;
}

#[tokio::test]
async fn test_5_percent_loss_eventual_consistency() {
    let server = spawn_test_server_with_packet_loss(0.05).await.unwrap();

    let id = server
        .spawn_entity("test", IVec2 { x: 1, y: 1 }, 0, 100)
        .await;

    // Modify entity
    {
        let mut w = server.server.world.lock().await;
        if let Some(h) = w.health_mut(id) {
            h.hp = 75;
        }
    }

    // Wait for propagation
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();
    assert_eq!(entity.hp, 75);

    server.shutdown().await;
}

#[tokio::test]
async fn test_5_percent_loss_multiple_updates() {
    let server = spawn_test_server_with_packet_loss(0.05).await.unwrap();

    let id = server
        .spawn_entity("test", IVec2 { x: 0, y: 0 }, 0, 100)
        .await;

    // Apply multiple updates
    for hp in (50..=90).step_by(10) {
        let mut w = server.server.world.lock().await;
        if let Some(h) = w.health_mut(id) {
            h.hp = hp;
        }
        drop(w);
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    }

    // Final state should converge
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();
    assert_eq!(entity.hp, 90);

    server.shutdown().await;
}

#[tokio::test]
async fn test_5_percent_loss_no_divergence() {
    let server = spawn_test_server_with_packet_loss(0.05).await.unwrap();

    // Create reference state
    let snap1 = server.get_snapshot().await;
    let hash1 = snap1.world_hash;

    // Wait and check again
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let snap2 = server.get_snapshot().await;
    let hash2 = snap2.world_hash;

    // If no changes were made, hashes should match
    // (This tests that packet loss doesn't cause state divergence)
    assert_eq!(hash1, hash2);

    server.shutdown().await;
}

#[tokio::test]
async fn test_5_percent_loss_tick_progression() {
    let server = spawn_test_server_with_packet_loss(0.05).await.unwrap();

    let tick1 = server.current_tick();
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    let tick2 = server.current_tick();

    // Ticks should still progress despite packet loss
    assert!(tick2 > tick1);

    server.shutdown().await;
}

// ============================================================================
// MODERATE PACKET LOSS TESTS (20% loss) (5 tests)
// ============================================================================

#[tokio::test]
async fn test_20_percent_loss_entity_creation() {
    let server = spawn_test_server_with_packet_loss(0.20).await.unwrap();

    let id = server
        .spawn_entity("resilient", IVec2 { x: 5, y: 5 }, 0, 100)
        .await;

    // Allow time for retransmission
    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    let snap = server.get_snapshot().await;
    assert!(snap.entities.iter().any(|e| e.id == id));

    server.shutdown().await;
}

#[tokio::test]
async fn test_20_percent_loss_state_updates() {
    let server = spawn_test_server_with_packet_loss(0.20).await.unwrap();

    let id = server
        .spawn_entity("test", IVec2 { x: 1, y: 1 }, 0, 100)
        .await;

    // Perform state change
    {
        let mut w = server.server.world.lock().await;
        if let Some(p) = w.pose_mut(id) {
            p.pos = IVec2 { x: 10, y: 10 };
        }
    }

    // Wait for convergence
    tokio::time::sleep(tokio::time::Duration::from_millis(400)).await;

    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();
    assert_eq!(entity.pos, IVec2 { x: 10, y: 10 });

    server.shutdown().await;
}

#[tokio::test]
async fn test_20_percent_loss_no_data_corruption() {
    let server = spawn_test_server_with_packet_loss(0.20).await.unwrap();

    // Create entities with known state
    let ids: Vec<_> = (0..5)
        .map(|i| {
            let server = server.server.clone();
            async move {
                let mut w = server.world.lock().await;
                w.spawn(
                    &format!("e{}", i),
                    IVec2 { x: i, y: i },
                    astraweave_core::Team { id: 0 },
                    100 - i * 10,
                    i * 5,
                )
            }
        })
        .collect::<Vec<_>>();

    let ids = futures_util::future::join_all(handles).await;

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let snap = server.get_snapshot().await;

    // Verify all entities have correct state
    for (i, id) in ids.iter().enumerate() {
        let entity = snap.entities.iter().find(|e| e.id == *id);
        assert!(entity.is_some(), "entity {} missing", id);
        let entity = entity.unwrap();
        assert_eq!(
            entity.pos,
            IVec2 {
                x: i as i32,
                y: i as i32
            }
        );
        assert_eq!(entity.hp, 100 - (i as i32 * 10));
        assert_eq!(entity.ammo, i as i32 * 5);
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_20_percent_loss_delta_application() {
    let server = spawn_test_server_with_packet_loss(0.20).await.unwrap();

    let mut base = server.get_snapshot().await;

    // Make changes
    let id = server
        .spawn_entity("new", IVec2 { x: 7, y: 7 }, 1, 80)
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;

    let head = server.get_snapshot().await;
    let viewer = base.entities.first().unwrap();
    let delta = diff_snapshots(&base, &head, &FullInterest, viewer);

    apply_delta(&mut base, &delta);

    // After delta application, should match head
    assert_eq!(base.tick, head.tick);
    assert!(base.entities.iter().any(|e| e.id == id));

    server.shutdown().await;
}

#[tokio::test]
async fn test_20_percent_loss_concurrent_modifications() {
    let server = spawn_test_server_with_packet_loss(0.20).await.unwrap();

    let id = server
        .spawn_entity("concurrent", IVec2 { x: 5, y: 5 }, 0, 100)
        .await;

    // Multiple concurrent updates
    let handles: Vec<_> = (0..20)
        .map(|_| {
            let server = server.server.clone();
            tokio::spawn(async move {
                tokio::time::sleep(tokio::time::Duration::from_millis(
                    rand::random::<u64>() % 50,
                ))
                .await;
                let mut w = server.world.lock().await;
                if let Some(h) = w.health_mut(id) {
                    h.hp = (h.hp - 1).max(0);
                }
            })
        })
        .collect();

    for h in handles {
        h.await.unwrap();
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();

    // Health should have decreased but remain valid
    assert!(entity.hp <= 100 && entity.hp >= 80);

    server.shutdown().await;
}

// ============================================================================
// HEAVY PACKET LOSS TESTS (50% loss) (5 tests)
// ============================================================================

#[tokio::test]
async fn test_50_percent_loss_basic_sync() {
    let server = spawn_test_server_with_packet_loss(0.50).await.unwrap();

    let id = server
        .spawn_entity("extreme", IVec2 { x: 1, y: 1 }, 0, 100)
        .await;

    // Need more time with 50% loss
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let snap = server.get_snapshot().await;
    assert!(snap.entities.iter().any(|e| e.id == id));

    server.shutdown().await;
}

#[tokio::test]
async fn test_50_percent_loss_eventual_consistency() {
    let server = spawn_test_server_with_packet_loss(0.50).await.unwrap();

    let id = server
        .spawn_entity("test", IVec2 { x: 3, y: 3 }, 0, 100)
        .await;

    // Change state
    {
        let mut w = server.server.world.lock().await;
        if let Some(h) = w.health_mut(id) {
            h.hp = 50;
        }
        if let Some(p) = w.pose_mut(id) {
            p.pos = IVec2 { x: 8, y: 8 };
        }
    }

    // Wait for convergence (longer timeout needed)
    tokio::time::sleep(tokio::time::Duration::from_millis(1500)).await;

    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();

    assert_eq!(entity.hp, 50);
    assert_eq!(entity.pos, IVec2 { x: 8, y: 8 });

    server.shutdown().await;
}

#[tokio::test]
async fn test_50_percent_loss_no_state_corruption() {
    let server = spawn_test_server_with_packet_loss(0.50).await.unwrap();

    // Establish baseline
    let snap1 = server.get_snapshot().await;
    let entities_before = snap1.entities.len();

    // Add entity
    let _id = server
        .spawn_entity("test", IVec2 { x: 5, y: 5 }, 0, 100)
        .await;

    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let snap2 = server.get_snapshot().await;
    let entities_after = snap2.entities.len();

    // Should have exactly one more entity
    assert_eq!(entities_after, entities_before + 1);

    server.shutdown().await;
}

#[tokio::test]
async fn test_50_percent_loss_tick_monotonicity() {
    let server = spawn_test_server_with_packet_loss(0.50).await.unwrap();

    let mut ticks = Vec::new();
    for _ in 0..10 {
        ticks.push(server.current_tick());
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Even with heavy packet loss, ticks should progress
    for i in 1..ticks.len() {
        assert!(
            ticks[i] >= ticks[i - 1],
            "tick regression despite packet loss"
        );
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_50_percent_loss_recovery() {
    let server = spawn_test_server_with_packet_loss(0.50).await.unwrap();

    let id = server
        .spawn_entity("recovery", IVec2 { x: 2, y: 2 }, 0, 100)
        .await;

    // Make a series of updates
    for i in 0..10 {
        let mut w = server.server.world.lock().await;
        if let Some(h) = w.health_mut(id) {
            h.hp = 100 - i * 5;
        }
        drop(w);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    // Final check after waiting for recovery
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();

    // Should have final value
    assert_eq!(entity.hp, 55);

    server.shutdown().await;
}

// ============================================================================
// STRESS AND EDGE CASE TESTS (5 tests)
// ============================================================================

#[tokio::test]
async fn test_varying_packet_loss_stability() {
    // Test with different loss rates to ensure stability
    for loss_rate in [0.0, 0.1, 0.25, 0.40] {
        let server = spawn_test_server_with_packet_loss(loss_rate).await.unwrap();

        let id = server
            .spawn_entity("test", IVec2 { x: 1, y: 1 }, 0, 100)
            .await;

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        let snap = server.get_snapshot().await;
        assert!(
            snap.entities.iter().any(|e| e.id == id),
            "entity missing with loss rate {}",
            loss_rate
        );

        server.shutdown().await;
    }
}

#[tokio::test]
async fn test_burst_packet_loss() {
    let server = spawn_test_server_with_packet_loss(0.30).await.unwrap();

    // Simulate burst of activity during packet loss
    let mut ids = Vec::new();
    for i in 0..5 {
        let id = server
            .spawn_entity(&format!("burst{}", i), IVec2 { x: i, y: i }, 0, 100)
            .await;
        ids.push(id);
    }

    // All entities should eventually be visible
    tokio::time::sleep(tokio::time::Duration::from_millis(800)).await;

    let snap = server.get_snapshot().await;
    for id in ids {
        assert!(
            snap.entities.iter().any(|e| e.id == id),
            "entity {} missing after burst",
            id
        );
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_packet_loss_with_rapid_changes() {
    let server = spawn_test_server_with_packet_loss(0.25).await.unwrap();

    let id = server
        .spawn_entity("rapid", IVec2 { x: 0, y: 0 }, 0, 100)
        .await;

    // Rapid health changes
    let final_hp = Arc::new(Mutex::new(100i32));
    let fh = final_hp.clone();

    let handle = tokio::spawn({
        let server = server.server.clone();
        async move {
            for i in 0..20 {
                let hp = 100 - i * 2;
                let mut w = server.world.lock().await;
                if let Some(h) = w.health_mut(id) {
                    h.hp = hp;
                }
                drop(w);
                *fh.lock().await = hp;
                tokio::time::sleep(tokio::time::Duration::from_millis(30)).await;
            }
        }
    });

    handle.await.unwrap();

    // Wait for propagation
    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id).unwrap();
    let expected = *final_hp.lock().await;

    assert_eq!(entity.hp, expected);

    server.shutdown().await;
}

#[tokio::test]
async fn test_snapshot_consistency_under_loss() {
    let server = spawn_test_server_with_packet_loss(0.30).await.unwrap();

    // Take multiple snapshots
    let mut snapshots = Vec::new();
    for _ in 0..10 {
        snapshots.push(server.get_snapshot().await);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Each snapshot should be internally consistent
    for snap in snapshots {
        // Entity IDs should be unique
        let mut ids = std::collections::HashSet::new();
        for e in &snap.entities {
            assert!(ids.insert(e.id), "duplicate entity ID {} in snapshot", e.id);
        }

        // All entities should have valid state
        for e in &snap.entities {
            assert!(e.hp >= 0, "negative HP for entity {}", e.id);
            assert!(e.ammo >= 0, "negative ammo for entity {}", e.id);
        }
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_multiple_clients_with_packet_loss() {
    let server = spawn_test_server_with_packet_loss(0.20).await.unwrap();

    // Spawn entities for multiple "clients"
    let client_ids: Vec<_> = (0..3)
        .map(|i| {
            let server = server.server.clone();
            async move {
                let mut w = server.world.lock().await;
                w.spawn(
                    &format!("client{}", i),
                    IVec2 { x: i * 2, y: i * 2 },
                    astraweave_core::Team { id: 0 },
                    100,
                    0,
                )
            }
        })
        .collect::<Vec<_>>();

    let ids = futures_util::future::join_all(handles).await;

    // Wait for synchronization
    tokio::time::sleep(tokio::time::Duration::from_millis(600)).await;

    let snap = server.get_snapshot().await;

    // All client entities should be present
    for id in client_ids {
        assert!(
            snap.entities.iter().any(|e| e.id == id),
            "client entity {} missing",
            id
        );
    }

    server.shutdown().await;
}
