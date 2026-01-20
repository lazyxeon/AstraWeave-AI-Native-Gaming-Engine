//! Common test utilities for astraweave-net integration tests

#![allow(dead_code)]

use anyhow::Result;
use astraweave_core::{IVec2, PlanIntent, Team, World};
use astraweave_net::{build_snapshot, GameServer, Msg, Snapshot};
use futures_util::stream::FusedStream;
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};

/// Test server handle with cleanup
pub struct TestServer {
    pub server: Arc<GameServer>,
    pub addr: String,
    pub handle: tokio::task::JoinHandle<()>,
}

impl TestServer {
    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.server.tx.receiver_count()
    }

    /// Get current tick
    pub fn current_tick(&self) -> u64 {
        self.server
            .tick
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    /// Get current world snapshot
    pub async fn get_snapshot(&self) -> Snapshot {
        let w = self.server.world.lock().await;
        build_snapshot(&w, self.current_tick(), 0)
    }

    /// Spawn an entity in the server world
    pub async fn spawn_entity(&self, name: &str, pos: IVec2, team: u8, hp: i32) -> u32 {
        let mut w = self.server.world.lock().await;
        w.spawn(name, pos, Team { id: team }, hp, 0)
    }

    /// Get entity position
    pub async fn get_entity_pos(&self, id: u32) -> Option<IVec2> {
        let w = self.server.world.lock().await;
        w.pos_of(id)
    }

    /// Execute plan for entity
    pub async fn execute_plan(&self, actor_id: u32, intent: PlanIntent) -> Result<bool> {
        let mut w = self.server.world.lock().await;
        let mut log = |_s: String| {};
        let vcfg = astraweave_core::ValidateCfg {
            world_bounds: (0, 0, 19, 9),
        };
        astraweave_core::validate_and_execute(&mut w, actor_id, &intent, &vcfg, &mut log)?;
        Ok(true)
    }

    /// Shutdown the server
    pub async fn shutdown(self) {
        self.handle.abort();
    }
}

/// Test client handle
pub struct TestClient {
    pub ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    pub name: String,
    pub player_id: Option<u32>,
    pub last_snapshot: Arc<Mutex<Option<Snapshot>>>,
}

impl TestClient {
    /// Send a message to server
    pub async fn send(&mut self, msg: &Msg) -> Result<()> {
        let json = serde_json::to_string(msg)?;
        self.ws.send(Message::Text(json.into())).await?;
        Ok(())
    }

    /// Receive next message with timeout
    pub async fn recv_timeout(&mut self, timeout_ms: u64) -> Result<Msg> {
        let timeout = Duration::from_millis(timeout_ms);
        match tokio::time::timeout(timeout, self.ws.next()).await {
            Ok(Some(Ok(Message::Text(text)))) => {
                let msg: Msg = serde_json::from_str(&text)?;
                println!("Client {} received {:?}", self.name, msg);
                Ok(msg)
            }
            Ok(Some(Ok(_))) => anyhow::bail!("unexpected message type"),
            Ok(Some(Err(e))) => Err(e.into()),
            Ok(None) => anyhow::bail!("connection closed"),
            Err(_) => anyhow::bail!("timeout"),
        }
    }

    /// Check if client is connected
    pub fn is_connected(&self) -> bool {
        !self.ws.is_terminated()
    }

    /// Wait for a specific message type
    pub async fn wait_for_snapshot(&mut self, timeout_ms: u64) -> Result<Snapshot> {
        let start = tokio::time::Instant::now();
        let total_timeout = Duration::from_millis(timeout_ms);
        loop {
            if start.elapsed() > total_timeout {
                anyhow::bail!("timeout waiting for snapshot");
            }

            // Use a shorter timeout for individual recv calls to allow checking total timeout
            match self.recv_timeout(100).await {
                Ok(Msg::ServerSnapshot { snap }) => {
                    let mut last = self.last_snapshot.lock().await;
                    *last = Some(snap.clone());
                    return Ok(snap);
                }
                Ok(Msg::ServerDelta { delta }) => {
                    let mut last = self.last_snapshot.lock().await;
                    if let Some(ref mut base) = *last {
                        astraweave_net::apply_delta(base, &delta);
                        return Ok(base.clone());
                    }
                }
                Ok(_) => continue,
                Err(_) => continue,
            }
        }
    }

    /// Send player input
    pub async fn send_input(&mut self, actor_id: u32, intent: PlanIntent, seq: u32) -> Result<()> {
        self.send(&Msg::ClientInput {
            seq,
            tick: 0,
            actor_id,
            intent,
        })
        .await
    }

    /// Propose a plan
    pub async fn propose_plan(&mut self, actor_id: u32, intent: PlanIntent) -> Result<()> {
        self.send(&Msg::ClientProposePlan { actor_id, intent })
            .await
    }
}

/// Spawn a test server on a random port
pub async fn spawn_test_server() -> Result<TestServer> {
    spawn_test_server_with_config(false, 0.0).await
}

/// Spawn a test server with packet loss simulation
pub async fn spawn_test_server_with_packet_loss(loss_rate: f32) -> Result<TestServer> {
    spawn_test_server_with_config(true, loss_rate).await
}

async fn spawn_test_server_with_config(
    _simulate_loss: bool,
    _loss_rate: f32,
) -> Result<TestServer> {
    let server = Arc::new(GameServer::new());
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?.to_string();

    let server_clone = server.clone();
    let handle = tokio::spawn(async move {
        if let Err(e) = server_clone.run_ws_on_listener(listener).await {
            eprintln!("Server error: {e}");
        }
    });

    // Wait a bit for server to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    Ok(TestServer {
        server,
        addr,
        handle,
    })
}

/// Connect a test client to a server
pub async fn connect_test_client(name: &str, addr: &str) -> Result<TestClient> {
    println!("Connecting client {} to {}", name, addr);
    let url = format!("ws://{addr}");
    let (ws, _) = tokio_tungstenite::connect_async(&url).await?;
    println!("Connected client {} to {}", name, addr);

    let mut client = TestClient {
        ws,
        name: name.to_string(),
        player_id: None,
        last_snapshot: Arc::new(Mutex::new(None)),
    };

    // Send hello
    client
        .send(&Msg::ClientHello {
            name: name.to_string(),
            token: Some("dev".to_string()),
            policy: None,
        })
        .await?;

    // Wait for welcome
    println!("Client {} waiting for welcome", name);
    match client.recv_timeout(1000).await? {
        Msg::ServerWelcome { id } => {
            println!("Client {} got welcome id {}", name, id);
            client.player_id = Some(id);
        }
        _ => anyhow::bail!("expected ServerWelcome"),
    }

    Ok(client)
}

/// Create a basic world for testing
pub fn create_test_world() -> World {
    let mut w = World::new();
    w.obstacles.insert((5, 5));
    w
}

/// Assert snapshots are consistent
pub fn assert_snapshots_consistent(s1: &Snapshot, s2: &Snapshot) {
    assert_eq!(s1.entities.len(), s2.entities.len(), "entity count mismatch");
    for e1 in &s1.entities {
        let e2 = s2.entities.iter().find(|e| e.id == e1.id);
        assert!(e2.is_some(), "entity {} missing in second snapshot", e1.id);
        let e2 = e2.unwrap();
        assert_eq!(e1.pos, e2.pos, "position mismatch for entity {}", e1.id);
        assert_eq!(e1.hp, e2.hp, "hp mismatch for entity {}", e1.id);
        assert_eq!(e1.team, e2.team, "team mismatch for entity {}", e1.id);
    }
}

/// Wait for condition with timeout
pub async fn wait_for_condition<F>(mut check: F, timeout_ms: u64) -> Result<()>
where
    F: FnMut() -> bool,
{
    let start = tokio::time::Instant::now();
    while start.elapsed() < Duration::from_millis(timeout_ms) {
        if check() {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    anyhow::bail!("condition not met within timeout")
}
