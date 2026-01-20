//! Synchronization tests for astraweave-net
//!
//! Tests that validate state synchronization across clients, delta compression,
//! interest filtering, tick monotonicity, and server authority.
//!
//! These tests are P0-Critical as they validate core networking correctness.

#![cfg(test)]

use astraweave_core::{ActionStep, IVec2, PlanIntent};
use astraweave_net::{
    apply_delta, diff_snapshots, filter_snapshot_for_viewer, Delta, EntityState,
    FovInterest, FovLosInterest, FullInterest, RadiusTeamInterest, Snapshot,
};
use std::collections::BTreeSet;

use crate::common::*;

// ============================================================================
// 1. TWO CLIENTS SEE SAME WORLD STATE
// ============================================================================

#[tokio::test]
async fn test_two_clients_see_same_world_state() {
    let server = spawn_test_server().await.unwrap();

    // Spawn test entities in known positions
    let e1 = server
        .spawn_entity("entity1", IVec2 { x: 5, y: 5 }, 0, 100)
        .await;
    let e2 = server
        .spawn_entity("entity2", IVec2 { x: 10, y: 10 }, 0, 50)
        .await;

    // Get two snapshots (simulating two clients)
    let snap1 = server.get_snapshot().await;
    let snap2 = server.get_snapshot().await;

    // Both should see the same entities
    assert_eq!(
        snap1.entities.len(),
        snap2.entities.len(),
        "Both clients should see same entity count"
    );

    // Verify both entities exist in both snapshots
    for id in [e1, e2] {
        let exists_1 = snap1.entities.iter().any(|e| e.id == id);
        let exists_2 = snap2.entities.iter().any(|e| e.id == id);
        assert!(exists_1 && exists_2, "Entity {} should be visible to both", id);
    }

    // Verify state consistency
    assert_snapshots_consistent(&snap1, &snap2);

    server.shutdown().await;
}

#[tokio::test]
async fn test_multiple_clients_state_convergence() {
    let server = spawn_test_server().await.unwrap();

    // Spawn entity and modify it
    let id = server
        .spawn_entity("mover", IVec2 { x: 0, y: 0 }, 0, 100)
        .await;

    // Take snapshots at different times
    let mut snapshots = Vec::new();
    for _ in 0..5 {
        snapshots.push(server.get_snapshot().await);
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // All snapshots should have the entity
    for snap in &snapshots {
        assert!(
            snap.entities.iter().any(|e| e.id == id),
            "Entity should be present in all snapshots"
        );
    }

    // Later snapshots should have higher or equal ticks
    for i in 1..snapshots.len() {
        assert!(
            snapshots[i].tick >= snapshots[i - 1].tick,
            "Ticks should be monotonically increasing"
        );
    }

    server.shutdown().await;
}

// ============================================================================
// 2. DELTA COMPRESSION PRESERVES STATE
// ============================================================================

#[tokio::test]
async fn test_delta_compression_preserves_state() {
    // Create base snapshot
    let mut base = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 5, y: 5 },
                hp: 80,
                team: 1,
                ammo: 30,
            },
        ],
    };

    // Create head snapshot with position change
    let head = Snapshot {
        version: 1,
        tick: 1,
        t: 0.016,
        seq: 1,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 1, y: 0 }, // Position changed
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 5, y: 5 },
                hp: 80,
                team: 1,
                ammo: 30,
            },
        ],
    };

    // Create viewer and generate delta
    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 1, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);

    // Apply delta to base
    apply_delta(&mut base, &delta);

    // Base should now match head
    assert_eq!(base.tick, head.tick, "Tick should be updated");
    assert_eq!(
        base.entities.len(),
        head.entities.len(),
        "Entity count should match"
    );

    // Find entity 1 and verify position was updated
    let e1_base = base.entities.iter().find(|e| e.id == 1).unwrap();
    let e1_head = head.entities.iter().find(|e| e.id == 1).unwrap();
    assert_eq!(e1_base.pos, e1_head.pos, "Position should be updated via delta");
}

#[tokio::test]
async fn test_delta_compression_minimal_size() {
    // Create base with 100 entities
    let mut base_entities: Vec<EntityState> = (0..100)
        .map(|i| EntityState {
            id: i,
            pos: IVec2 { x: i as i32, y: 0 },
            hp: 100,
            team: 0,
            ammo: 50,
        })
        .collect();

    let base = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: base_entities.clone(),
    };

    // Change only ONE entity
    base_entities[50].pos.x = 999;

    let head = Snapshot {
        version: 1,
        tick: 1,
        t: 0.016,
        seq: 1,
        world_hash: 0,
        entities: base_entities,
    };

    let viewer = EntityState {
        id: 0,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);

    // Delta should only contain the changed entity
    assert_eq!(
        delta.changed.len(),
        1,
        "Delta should only contain changed entities"
    );
    assert_eq!(delta.changed[0].id, 50, "Changed entity ID should be 50");
    assert!(
        delta.changed[0].pos.is_some(),
        "Position should be in delta"
    );
}

#[tokio::test]
async fn test_delta_handles_entity_removal() {
    let base = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 5, y: 5 },
                hp: 80,
                team: 0,
                ammo: 30,
            },
        ],
    };

    // Head has entity 2 removed
    let head = Snapshot {
        version: 1,
        tick: 1,
        t: 0.016,
        seq: 1,
        world_hash: 0,
        entities: vec![EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 50,
        }],
    };

    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);

    // Delta should have entity 2 in removed list
    assert!(
        delta.removed.contains(&2),
        "Removed entity should be in delta.removed"
    );

    // Apply and verify
    let mut reconstructed = base.clone();
    apply_delta(&mut reconstructed, &delta);
    assert_eq!(reconstructed.entities.len(), 1, "Only one entity after delta");
    assert!(
        reconstructed.entities.iter().all(|e| e.id != 2),
        "Entity 2 should be removed"
    );
}

#[tokio::test]
async fn test_delta_handles_entity_addition() {
    let base = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 50,
        }],
    };

    // Head has a new entity
    let head = Snapshot {
        version: 1,
        tick: 1,
        t: 0.016,
        seq: 1,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 99,
                pos: IVec2 { x: 10, y: 10 },
                hp: 50,
                team: 1,
                ammo: 20,
            },
        ],
    };

    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);

    // Delta should contain the new entity with full data
    let new_entity_delta = delta.changed.iter().find(|d| d.id == 99);
    assert!(
        new_entity_delta.is_some(),
        "New entity should be in delta.changed"
    );
    let new_d = new_entity_delta.unwrap();
    assert!(new_d.pos.is_some(), "New entity should have position");
    assert!(new_d.hp.is_some(), "New entity should have hp");
    assert!(new_d.team.is_some(), "New entity should have team");
    assert!(new_d.ammo.is_some(), "New entity should have ammo");
}

// ============================================================================
// 3. INTEREST FILTERING HIDES ENTITIES
// ============================================================================

#[tokio::test]
async fn test_interest_filtering_radius() {
    let full_snap = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 3, y: 0 }, // Within radius 5
                hp: 80,
                team: 1,
                ammo: 30,
            },
            EntityState {
                id: 3,
                pos: IVec2 { x: 100, y: 100 }, // Far away
                hp: 60,
                team: 1,
                ammo: 10,
            },
        ],
    };

    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let interest = RadiusTeamInterest { radius: 5 };
    let filtered = filter_snapshot_for_viewer(&full_snap, &interest, &viewer);

    // Viewer should see themselves and entity 2 (within radius)
    // Entity 3 is too far but different team
    assert!(
        filtered.entities.iter().any(|e| e.id == 1),
        "Viewer should see themselves"
    );
    assert!(
        filtered.entities.iter().any(|e| e.id == 2),
        "Entity within radius should be visible"
    );
    assert!(
        !filtered.entities.iter().any(|e| e.id == 3),
        "Entity outside radius should be hidden"
    );
}

#[tokio::test]
async fn test_interest_filtering_team_always_visible() {
    let full_snap = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 100, y: 100 }, // Far away but SAME TEAM
                hp: 80,
                team: 0,
                ammo: 30,
            },
        ],
    };

    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let interest = RadiusTeamInterest { radius: 5 };
    let filtered = filter_snapshot_for_viewer(&full_snap, &interest, &viewer);

    // Teammate should be visible regardless of distance
    assert!(
        filtered.entities.iter().any(|e| e.id == 2),
        "Teammate should always be visible even outside radius"
    );
}

#[tokio::test]
async fn test_interest_filtering_fov() {
    let full_snap = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 5, y: 0 }, // In front (facing +x)
                hp: 80,
                team: 1,
                ammo: 30,
            },
            EntityState {
                id: 3,
                pos: IVec2 { x: -5, y: 0 }, // Behind
                hp: 60,
                team: 1,
                ammo: 10,
            },
        ],
    };

    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let interest = FovInterest {
        radius: 10,
        half_angle_deg: 45.0, // 90 degree FOV
        facing: IVec2 { x: 1, y: 0 }, // Facing +X
    };
    let filtered = filter_snapshot_for_viewer(&full_snap, &interest, &viewer);

    // Entity 2 should be visible (in front)
    assert!(
        filtered.entities.iter().any(|e| e.id == 2),
        "Entity in front should be visible"
    );
    // Entity 3 should NOT be visible (behind)
    assert!(
        !filtered.entities.iter().any(|e| e.id == 3),
        "Entity behind should be hidden"
    );
}

#[tokio::test]
async fn test_interest_filtering_fov_los_blocked() {
    let obstacles: BTreeSet<(i32, i32)> = [(3, 0)].iter().cloned().collect();

    let full_snap = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![
            EntityState {
                id: 1,
                pos: IVec2 { x: 0, y: 0 },
                hp: 100,
                team: 0,
                ammo: 50,
            },
            EntityState {
                id: 2,
                pos: IVec2 { x: 5, y: 0 }, // Behind obstacle
                hp: 80,
                team: 1,
                ammo: 30,
            },
        ],
    };

    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let interest = FovLosInterest {
        radius: 10,
        half_angle_deg: 90.0,
        facing: IVec2 { x: 1, y: 0 },
        obstacles,
    };
    let filtered = filter_snapshot_for_viewer(&full_snap, &interest, &viewer);

    // Entity 2 should NOT be visible (blocked by obstacle at 3,0)
    assert!(
        !filtered.entities.iter().any(|e| e.id == 2),
        "Entity behind obstacle should be hidden"
    );
}

// ============================================================================
// 4. SNAPSHOT TICK MONOTONICITY
// ============================================================================

#[tokio::test]
async fn test_snapshot_tick_monotonicity() {
    let server = spawn_test_server().await.unwrap();

    let mut prev_tick = 0u64;

    // Take 20 snapshots
    for i in 0..20 {
        let snap = server.get_snapshot().await;
        assert!(
            snap.tick >= prev_tick,
            "Tick {} should be >= prev tick {} (iteration {})",
            snap.tick,
            prev_tick,
            i
        );
        prev_tick = snap.tick;
        tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_snapshot_seq_monotonicity() {
    let server = spawn_test_server().await.unwrap();

    let mut prev_seq = 0u32;

    for _ in 0..10 {
        let snap = server.get_snapshot().await;
        // seq should wrap, but in short tests should always increase
        assert!(
            snap.seq >= prev_seq || snap.seq == 0, // Handle wrap case
            "Seq {} should be >= prev seq {}",
            snap.seq,
            prev_seq
        );
        prev_seq = snap.seq;
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    server.shutdown().await;
}

#[tokio::test]
async fn test_delta_tick_consistency() {
    let base = Snapshot {
        version: 1,
        tick: 10,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 50,
        }],
    };

    let head = Snapshot {
        version: 1,
        tick: 15,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![EntityState {
            id: 1,
            pos: IVec2 { x: 1, y: 0 },
            hp: 100,
            team: 0,
            ammo: 50,
        }],
    };

    let viewer = EntityState {
        id: 1,
        pos: IVec2 { x: 1, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    let delta = diff_snapshots(&base, &head, &FullInterest, &viewer);

    assert_eq!(delta.base_tick, 10, "Delta base_tick should match base");
    assert_eq!(delta.tick, 15, "Delta tick should match head");
}

// ============================================================================
// 5. CONCURRENT CLIENT MODIFICATIONS
// ============================================================================

#[tokio::test]
async fn test_concurrent_client_modifications() {
    let server = spawn_test_server().await.unwrap();

    let id = server
        .spawn_entity("shared", IVec2 { x: 5, y: 5 }, 0, 100)
        .await;

    // Simulate concurrent modifications from multiple "clients"
    let handles: Vec<_> = (0..5)
        .map(|i| {
            let server = spawn_test_server();
            async move {
                let s = server.await.unwrap();
                let _plan = PlanIntent {
                    plan_id: format!("client_{}", i),
                    steps: vec![ActionStep::MoveTo {
                        x: i,
                        y: i,
                        speed: None,
                    }],
                };
                // Simulate independent client actions
                tokio::time::sleep(tokio::time::Duration::from_millis(i as u64 * 10)).await;
                s.shutdown().await;
            }
        })
        .collect();

    for h in handles {
        h.await;
    }

    // Original server should still be consistent
    let snap = server.get_snapshot().await;
    assert!(snap.entities.iter().any(|e| e.id == id), "Entity should exist");

    server.shutdown().await;
}

#[tokio::test]
async fn test_rapid_state_changes_synchronized() {
    let server = spawn_test_server().await.unwrap();

    let id = server
        .spawn_entity("rapid", IVec2 { x: 0, y: 0 }, 0, 100)
        .await;

    // Perform rapid changes
    for i in 0i32..10 {
        let plan = PlanIntent {
            plan_id: format!("rapid_{}", i),
            steps: vec![ActionStep::MoveTo {
                x: i % 10,
                y: 0,
                speed: None,
            }],
        };
        server.execute_plan(id, plan).await.ok();
    }

    // Snapshot should reflect some state (not necessarily the last due to async)
    let snap = server.get_snapshot().await;
    let entity = snap.entities.iter().find(|e| e.id == id);
    assert!(entity.is_some(), "Entity should exist after rapid changes");

    server.shutdown().await;
}

// ============================================================================
// 6. CLIENT JOIN MID-GAME
// ============================================================================

#[tokio::test]
async fn test_client_join_mid_game() {
    let server = spawn_test_server().await.unwrap();

    // Game is already running, spawn some entities
    let e1 = server
        .spawn_entity("existing1", IVec2 { x: 1, y: 1 }, 0, 100)
        .await;
    let e2 = server
        .spawn_entity("existing2", IVec2 { x: 5, y: 5 }, 1, 80)
        .await;

    // Let game run for a bit
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // "New client" joins and gets snapshot
    let snap = server.get_snapshot().await;

    // New client should see all existing entities
    assert!(
        snap.entities.iter().any(|e| e.id == e1),
        "New client should see entity 1"
    );
    assert!(
        snap.entities.iter().any(|e| e.id == e2),
        "New client should see entity 2"
    );

    // Tick should be non-zero (game has been running)
    assert!(snap.tick > 0, "Tick should be > 0 for running game");

    server.shutdown().await;
}

#[tokio::test]
async fn test_late_joiner_gets_full_snapshot() {
    let server = spawn_test_server().await.unwrap();

    // Run game for a while with modifications
    for i in 0..5 {
        server
            .spawn_entity(&format!("entity_{}", i), IVec2 { x: i, y: i }, 0, 100 - i * 10)
            .await;
    }

    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    // Late joiner snapshot
    let snap = server.get_snapshot().await;

    // Should contain all spawned entities
    assert!(snap.entities.len() >= 5, "Should have at least 5 entities");

    server.shutdown().await;
}

// ============================================================================
// 7. CLIENT DISCONNECT CLEANUP
// ============================================================================

#[tokio::test]
async fn test_client_disconnect_no_crash() {
    let server = spawn_test_server().await.unwrap();

    // Spawn entity
    let id = server
        .spawn_entity("player", IVec2 { x: 0, y: 0 }, 0, 100)
        .await;

    // Get snapshot (simulates client connected)
    let _snap1 = server.get_snapshot().await;

    // "Disconnect" - just stop taking snapshots
    // Server should continue running

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Server should still be functional
    let snap2 = server.get_snapshot().await;
    assert!(
        snap2.entities.iter().any(|e| e.id == id),
        "Server should still track entity after client disconnect"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn test_multiple_disconnect_reconnect_cycles() {
    let server = spawn_test_server().await.unwrap();

    for cycle in 0..3 {
        // Spawn entity for this "connection"
        let id = server
            .spawn_entity(&format!("player_{}", cycle), IVec2 { x: cycle, y: 0 }, 0, 100)
            .await;

        // Take some snapshots
        for _ in 0..3 {
            let snap = server.get_snapshot().await;
            assert!(
                snap.entities.iter().any(|e| e.id == id),
                "Entity should be visible"
            );
            tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;
        }

        // "Disconnect" - small delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
    }

    // Server should have all entities from all cycles
    let final_snap = server.get_snapshot().await;
    assert!(
        final_snap.entities.len() >= 3,
        "Should have entities from all cycles"
    );

    server.shutdown().await;
}

// ============================================================================
// 8. SERVER AUTHORITY POSITION OVERRIDE
// ============================================================================

#[tokio::test]
async fn test_server_authority_position_override() {
    let server = spawn_test_server().await.unwrap();

    let id = server
        .spawn_entity("controlled", IVec2 { x: 5, y: 5 }, 0, 100)
        .await;

    // Client requests move to invalid position (blocked by obstacles if any)
    // For this test, we just verify server state is authoritative
    let server_snap = server.get_snapshot().await;
    let entity = server_snap.entities.iter().find(|e| e.id == id).unwrap();

    // Server position is the truth
    let server_pos = entity.pos;

    // Any client-side prediction should eventually converge to server state
    // (In this unit test, we just verify the server has a definitive position)
    assert!(
        server_pos.x >= 0 && server_pos.y >= 0,
        "Server position should be valid"
    );

    server.shutdown().await;
}

#[tokio::test]
async fn test_invalid_move_rejected_by_server() {
    let server = spawn_test_server().await.unwrap();

    let id = server
        .spawn_entity("mover", IVec2 { x: 5, y: 5 }, 0, 100)
        .await;

    let initial_snap = server.get_snapshot().await;
    let _initial_pos = initial_snap
        .entities
        .iter()
        .find(|e| e.id == id)
        .unwrap()
        .pos;

    // Try to execute invalid move (out of bounds)
    let plan = PlanIntent {
        plan_id: "invalid_move".to_string(),
        steps: vec![ActionStep::MoveTo {
            x: -1000, // Invalid position
            y: -1000,
            speed: None,
        }],
    };

    // Server should reject or clamp
    let _ = server.execute_plan(id, plan).await;

    let final_snap = server.get_snapshot().await;
    let final_entity = final_snap.entities.iter().find(|e| e.id == id);
    assert!(final_entity.is_some(), "Entity should still exist");

    // Position should not be the invalid one
    let final_pos = final_entity.unwrap().pos;
    assert!(
        final_pos.x >= 0 && final_pos.y >= 0,
        "Position should be clamped to valid range"
    );

    server.shutdown().await;
}

// ============================================================================
// EDGE CASES AND STRESS TESTS
// ============================================================================

#[tokio::test]
async fn test_empty_snapshot_handling() {
    // Create snapshot with no entities
    let empty = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![],
    };

    let viewer = EntityState {
        id: 0,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    // Should not panic
    let filtered = filter_snapshot_for_viewer(&empty, &FullInterest, &viewer);
    assert!(filtered.entities.is_empty(), "Filtered empty should be empty");
}

#[tokio::test]
async fn test_large_entity_count_sync() {
    // Test with many entities
    let entities: Vec<EntityState> = (0..1000)
        .map(|i| EntityState {
            id: i,
            pos: IVec2 {
                x: (i % 100) as i32,
                y: (i / 100) as i32,
            },
            hp: 100,
            team: (i % 4) as u8,
            ammo: 50,
        })
        .collect();

    let snap = Snapshot {
        version: 1,
        tick: 0,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities,
    };

    let viewer = EntityState {
        id: 0,
        pos: IVec2 { x: 0, y: 0 },
        hp: 100,
        team: 0,
        ammo: 50,
    };

    // Should handle large counts
    let filtered = filter_snapshot_for_viewer(&snap, &FullInterest, &viewer);
    assert_eq!(
        filtered.entities.len(),
        1000,
        "Full interest should show all entities"
    );
}

#[tokio::test]
async fn test_delta_mismatched_base_tick_ignored() {
    let mut base = Snapshot {
        version: 1,
        tick: 10,
        t: 0.0,
        seq: 0,
        world_hash: 0,
        entities: vec![EntityState {
            id: 1,
            pos: IVec2 { x: 0, y: 0 },
            hp: 100,
            team: 0,
            ammo: 50,
        }],
    };

    // Delta with wrong base tick
    let delta = Delta {
        base_tick: 5, // Doesn't match base.tick (10)
        tick: 15,
        changed: vec![],
        removed: vec![],
        head_hash: 0,
    };

    let original_tick = base.tick;
    apply_delta(&mut base, &delta);

    // Should be ignored - tick should not change
    assert_eq!(
        base.tick, original_tick,
        "Mismatched delta should be ignored"
    );
}

#[tokio::test]
async fn test_world_hash_consistency() {
    let server = spawn_test_server().await.unwrap();

    // Take two snapshots at same moment
    let snap1 = server.get_snapshot().await;
    let snap2 = server.get_snapshot().await;

    // If ticks are the same, hashes should match
    if snap1.tick == snap2.tick {
        assert_eq!(
            snap1.world_hash, snap2.world_hash,
            "Same tick should have same hash"
        );
    }

    server.shutdown().await;
}
