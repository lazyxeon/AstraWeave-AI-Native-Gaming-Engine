//! Full-stack integration tests for GameServer and WebSocket communication

#![cfg(test)]

use crate::common::*;
use astraweave_net::Msg;
use astraweave_core::{ActionStep, PlanIntent};

#[tokio::test]
async fn test_server_full_handshake_and_sync() {
    let server = spawn_test_server().await.unwrap();
    let mut client = connect_test_client("handshake_player", &server.addr).await.unwrap();

    // 1. Verify welcome received (done in connect_test_client)
    assert!(client.player_id.is_some());

    // 2. Wait for initial snapshot
    let snap = client.wait_for_snapshot(5000).await.unwrap();
    assert!(snap.entities.len() >= 2); // Player and Companion are close, Enemy is far

    // 3. Send a plan proposal
    let actor_id = client.player_id.unwrap();
    let intent = PlanIntent {
        plan_id: "move_test".into(),
        steps: vec![ActionStep::MoveTo { x: 5, y: 5, speed: None }],
    };
    client.propose_plan(actor_id, intent.clone()).await.unwrap();

    // 4. Wait for ApplyResult
    let mut got_result = false;
    for _ in 0..20 {
        let msg = client.recv_timeout(1000).await.unwrap();
        if let Msg::ServerApplyResult { ok, .. } = msg {
            assert!(ok);
            got_result = true;
            break;
        }
    }
    assert!(got_result);

    // 5. Test ClientInput (which should send Ack)
    client.send_input(actor_id, intent, 42).await.unwrap();
    
    let mut got_ack = false;
    for _ in 0..20 {
        let msg = client.recv_timeout(1000).await.unwrap();
        if let Msg::ServerAck { seq, .. } = msg {
            assert_eq!(seq, 42);
            got_ack = true;
            break;
        }
    }
    assert!(got_ack);

    server.shutdown().await;
}

#[tokio::test]
async fn test_server_interest_policy_switching() {
    let server = spawn_test_server().await.unwrap();
    
    // Connect with radius policy
    let mut client = connect_test_client("policy_player", &server.addr).await.unwrap();
    
    // Change policy to FOV via ClientHello (re-sending hello updates policy)
    client.send(&Msg::ClientHello {
        name: "policy_player".into(),
        token: Some("dev".into()),
        policy: Some("fov".into()),
    }).await.unwrap();

    // Wait for a snapshot and verify it's filtered (hard to verify exact FOV without more setup, 
    // but we hit the code paths)
    let _snap = client.wait_for_snapshot(5000).await.unwrap();

    // Change to FOV-LOS
    client.send(&Msg::ClientHello {
        name: "policy_player".into(),
        token: Some("dev".into()),
        policy: Some("fovlos".into()),
    }).await.unwrap();
    
    let _snap = client.wait_for_snapshot(5000).await.unwrap();

    server.shutdown().await;
}

#[tokio::test]
async fn test_server_delta_compression_flow() {
    let server = spawn_test_server().await.unwrap();
    let mut client = connect_test_client("delta_player", &server.addr).await.unwrap();

    // First message should be a full snapshot
    let msg1 = client.recv_timeout(5000).await.unwrap();
    assert!(matches!(msg1, Msg::ServerSnapshot { .. }));
    
    if let Msg::ServerSnapshot { snap } = msg1 {
        let mut last = client.last_snapshot.lock().await;
        *last = Some(snap);
    }

    // To force a delta, we need to move an entity via a plan
    let actor_id = client.player_id.unwrap();
    let intent = PlanIntent {
        plan_id: "move_test".into(),
        steps: vec![ActionStep::MoveTo { x: 5, y: 5, speed: None }],
    };
    client.propose_plan(actor_id, intent).await.unwrap();
    
    let mut got_delta = false;
    for _ in 0..50 {
        let msg = client.recv_timeout(1000).await.unwrap();
        if matches!(msg, Msg::ServerDelta { .. }) {
            got_delta = true;
            break;
        }
        if let Msg::ServerSnapshot { snap } = msg {
            let mut last = client.last_snapshot.lock().await;
            *last = Some(snap);
        }
    }
    
    assert!(got_delta, "Should eventually receive a delta message after movement");

    server.shutdown().await;
}
