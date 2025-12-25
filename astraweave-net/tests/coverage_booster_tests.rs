use anyhow::Result;
use astraweave_net::*;
use astraweave_core::*;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::time::{sleep, Duration};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn test_pusher_apply_result_and_ack() -> Result<()> {
    let server = Arc::new(GameServer::new());
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    
    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run_ws_on_listener(listener).await;
    });

    let (mut ws_stream, _) = connect_async(format!("ws://{}", addr)).await?;
    
    // Receive welcome
    let _welcome = ws_stream.next().await.unwrap()?;

    // Send ClientHello
    let hello = Msg::ClientHello {
        name: "player".to_string(),
        token: Some("dev".to_string()),
        policy: Some("radius".to_string()),
    };
    ws_stream.send(Message::Text(serde_json::to_string(&hello)?.into())).await?;

    // Trigger ApplyResult and Ack
    server.tx.send(ServerEvent::ApplyResult { ok: true, err: None })?;
    server.tx.send(ServerEvent::Ack { seq: 123, tick_applied: 456 })?;

    // Wait for messages
    let mut found_apply = false;
    let mut found_ack = false;
    
    for _ in 0..10 {
        if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
            if text.contains("ServerApplyResult") {
                found_apply = true;
            }
            if text.contains("ServerAck") {
                found_ack = true;
            }
            if found_apply && found_ack {
                break;
            }
        }
    }

    assert!(found_apply, "Should receive ApplyResult");
    assert!(found_ack, "Should receive Ack");
    
    Ok(())
}

#[tokio::test]
async fn test_handle_conn_client_input_and_propose_plan() -> Result<()> {
    let server = Arc::new(GameServer::new());
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    
    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run_ws_on_listener(listener).await;
    });

    let (mut ws_stream, _) = connect_async(format!("ws://{}", addr)).await?;
    let _welcome = ws_stream.next().await.unwrap()?;

    // Send ClientHello
    let hello = Msg::ClientHello {
        name: "player".to_string(),
        token: Some("dev".to_string()),
        policy: Some("radius".to_string()),
    };
    ws_stream.send(Message::Text(serde_json::to_string(&hello)?.into())).await?;

    // Send ClientProposePlan
    let propose = Msg::ClientProposePlan {
        actor_id: server.player_id,
        intent: PlanIntent {
            plan_id: "test_plan".to_string(),
            steps: vec![ActionStep::MoveTo { x: 5, y: 5, speed: None }],
        },
    };
    ws_stream.send(Message::Text(serde_json::to_string(&propose)?.into())).await?;

    // Send ClientInput
    let input = Msg::ClientInput {
        seq: 1,
        tick: 1,
        actor_id: server.player_id,
        intent: PlanIntent {
            plan_id: "test_input".to_string(),
            steps: vec![ActionStep::MoveTo { x: 6, y: 6, speed: None }],
        },
    };
    ws_stream.send(Message::Text(serde_json::to_string(&input)?.into())).await?;

    // Wait for Acks/Results
    let mut found_apply = false;
    let mut found_ack = false;
    
    for _ in 0..20 {
        if let Some(Ok(Message::Text(text))) = ws_stream.next().await {
            if text.contains("ServerApplyResult") {
                found_apply = true;
            }
            if text.contains("ServerAck") {
                found_ack = true;
            }
            if found_apply && found_ack {
                break;
            }
        }
    }

    assert!(found_apply, "Should receive ApplyResult for ProposePlan/Input");
    assert!(found_ack, "Should receive Ack for ClientInput");
    
    Ok(())
}

#[tokio::test]
async fn test_interest_policy_variants_in_pusher() -> Result<()> {
    let server = Arc::new(GameServer::new());
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;
    
    let server_clone = server.clone();
    tokio::spawn(async move {
        let _ = server_clone.run_ws_on_listener(listener).await;
    });

    let (mut ws_stream, _) = connect_async(format!("ws://{}", addr)).await?;
    let _welcome = ws_stream.next().await.unwrap()?;

    // Test FOV policy
    let hello_fov = Msg::ClientHello {
        name: "player".to_string(),
        token: Some("dev".to_string()),
        policy: Some("fov".to_string()),
    };
    ws_stream.send(Message::Text(serde_json::to_string(&hello_fov)?.into())).await?;
    sleep(Duration::from_millis(100)).await;

    // Test FOV LOS policy
    let hello_fovlos = Msg::ClientHello {
        name: "player".to_string(),
        token: Some("dev".to_string()),
        policy: Some("fovlos".to_string()),
    };
    ws_stream.send(Message::Text(serde_json::to_string(&hello_fovlos)?.into())).await?;
    sleep(Duration::from_millis(100)).await;

    // Test unknown policy (should fallback to radius)
    let hello_unknown = Msg::ClientHello {
        name: "player".to_string(),
        token: Some("dev".to_string()),
        policy: Some("unknown".to_string()),
    };
    ws_stream.send(Message::Text(serde_json::to_string(&hello_unknown)?.into())).await?;
    sleep(Duration::from_millis(100)).await;
    
    Ok(())
}

#[tokio::test]
async fn test_server_run_ws_short_lived() -> Result<()> {
    let server = Arc::new(GameServer::new());
    // We can't easily test run_ws because it blocks, but we can test run_ws_on_listener
    // which we already do in other tests. 
    // To cover run_ws, we'd need a valid addr that we can bind to.
    // Let's try a random port.
    let server_clone = server.clone();
    let handle = tokio::spawn(async move {
        let _ = server_clone.run_ws("127.0.0.1:0").await;
    });
    
    sleep(Duration::from_millis(200)).await;
    handle.abort();
    
    Ok(())
}
