use std::{collections::HashMap, net::SocketAddr, sync::Arc, time::Duration};

use anyhow::Result;
use axum::{routing::get, Router};
use futures::{SinkExt, StreamExt};
use parking_lot::Mutex;
use tokio::{net::TcpListener, time::Instant};
use tokio_tungstenite::{accept_hdr_async, tungstenite::handshake::server::Request, tungstenite::protocol::Message};
use tracing::{info, warn};
use tracing_subscriber::EnvFilter;

use aw_net_proto::{
    new_room_id, ClientToServer, Codec, ServerToClient, SessionKey,
    PROTOCOL_VERSION,
};

type PlayerId = String;
type RoomId = String;

#[derive(Clone)]
struct Player {
    id: PlayerId,
    display: String,
    last_input_seq: u32,
    last_seen: Instant,
    // simplistic rate limit
    tokens: f32,
}

#[derive(Clone)]
struct Room {
    id: RoomId,
    region: String,
    game_mode: String,
    session_key: SessionKey,
    tick_hz: u32,
    players: HashMap<PlayerId, Player>,
    // minimal world tick counter
    tick: u64,
    // last snapshot id
    snap_id: u32,
}

#[derive(Clone)]
struct AppState {
    rooms: Arc<Mutex<HashMap<RoomId, Room>>>,
    // sled persistence as key-value: "room:<id>" => JSON, "player:<id>" => JSON
    db: sled::Db,
    codec: Codec,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse().unwrap()))
        .init();

    let db = sled::open("aw_net_server_db")?;
    let state = AppState {
        rooms: Arc::new(Mutex::new(HashMap::new())),
        db,
        codec: Codec::PostcardLz4,
    };

    // Health check and region info (HTTP)
    let http_app = {
        let app_state = state.clone();
        Router::new()
            .route("/healthz", get(|| async { "ok" }))
            .route("/regions", get(|| async { r#"["us-east","us-west","eu-central"]"# }))
            .with_state(app_state)
    };

    // Spawn HTTP server
    tokio::spawn(async move {
        let addr: SocketAddr = "0.0.0.0:8789".parse().unwrap();
        info!("HTTP admin on http://{}", addr);
        let listener = TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, http_app)
            .await
            .unwrap();
    });

    // WS server
    let ws_addr: SocketAddr = "0.0.0.0:8788".parse().unwrap();
    info!("WS listening on ws://{}", ws_addr);

    let listener = TcpListener::bind(ws_addr).await?;
    loop {
        let (stream, _addr) = listener.accept().await?;
        let app = state.clone();
        tokio::spawn(async move {
            let peer = match accept_hdr_async(stream, |_req: &Request, resp| {
                // You can inspect headers for auth/cookies here
                Ok(resp)
            })
            .await
            {
                Ok(ws) => ws,
                Err(e) => {
                    warn!("ws handshake failed: {e}");
                    return;
                }
            };
            if let Err(e) = handle_socket(app, peer).await {
                warn!("ws session error: {e:?}");
            }
        });
    }
}

async fn handle_socket(app: AppState, mut ws: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>) -> Result<()> {
    // Handshake
    let hello = recv::<ClientToServer>(&app, &mut ws).await?;
    match hello {
        ClientToServer::Hello { protocol } if protocol == PROTOCOL_VERSION => {
            send(&app, &mut ws, &ServerToClient::HelloAck { protocol }).await?;
        }
        ClientToServer::Hello { protocol } => {
            send(&app, &mut ws, &ServerToClient::ProtocolError {
                msg: format!("protocol mismatch: client={protocol}, server={PROTOCOL_VERSION}"),
            }).await?;
            return Ok(());
        }
        _ => {
            send(&app, &mut ws, &ServerToClient::ProtocolError { msg: "expected Hello".into() }).await?;
            return Ok(());
        }
    }

    // Matchmaking or direct join
    let mut room_id: Option<RoomId> = None;
    let player_id = uuid::Uuid::new_v4().to_string();
    let mut session_hint = [0u8; 8];
    let mut tick_hz = 30u32;

    if let Ok(msg) = recv::<ClientToServer>(&app, &mut ws).await {
        match msg {
            ClientToServer::FindOrCreate { region, game_mode, .. } => {
                let mut rooms = app.rooms.lock();
                if let Some((rid, _)) = rooms.iter().find(|(_, r)| r.region == region && r.game_mode == game_mode && r.players.len() < 4) {
                    room_id = Some(rid.clone());
                } else {
                    let rid = new_room_id();
                    let key = SessionKey::random();
                    let r = Room {
                        id: rid.clone(),
                        region,
                        game_mode,
                        session_key: key.clone(),
                        tick_hz,
                        players: HashMap::new(),
                        tick: 0,
                        snap_id: 0,
                    };
                    rooms.insert(rid.clone(), r);
                    room_id = Some(rid.clone());
                }
            }
            ClientToServer::JoinRoom { room_id: rid, display_name: _ } => {
                room_id = Some(rid);
            }
            _other => {
                warn!("unexpected message before join: {_other:?}");
            }
        }
    }

    let rid = match room_id {
        Some(x) => x,
        None => {
            send(&app, &mut ws, &ServerToClient::ProtocolError { msg: "no room selected".into() }).await?;
            return Ok(());
        }
    };

    // allocate player into room
    {
        let mut rooms = app.rooms.lock();
        let room = rooms.get_mut(&rid).expect("room exists");
        session_hint.copy_from_slice(&room.session_key.0[0..8]);
        room.players.insert(player_id.clone(), Player {
            id: player_id.clone(),
            display: "player".into(),
            last_input_seq: 0,
            last_seen: tokio::time::Instant::now(),
            tokens: 30.0, // token bucket
        });
        tick_hz = room.tick_hz;
    }

    send(&app, &mut ws, &ServerToClient::MatchResult { room_id: rid.clone(), session_key_hint: session_hint }).await?;
    send(&app, &mut ws, &ServerToClient::JoinAccepted {
        room_id: rid.clone(),
        player_id: player_id.clone(),
        session_key_hint: session_hint,
        tick_hz,
    }).await?;

    // Per-connection game loop
    let tick_dt = Duration::from_millis((1000 / tick_hz.max(1)) as u64);
    let mut _last_snap = 0u32;

    loop {
        tokio::select! {
            biased;

            // Receive client messages
            msg = ws.next() => {
                match msg {
                    Some(Ok(Message::Binary(bytes))) => {
                        if let Ok(m) = aw_net_proto::decode_msg::<ClientToServer>(app.codec, &bytes) {
                            if let Err(e) = on_client_msg(&app, &rid, &player_id, &mut ws, m).await {
                                warn!("client msg error: {e:?}");
                            }
                        }
                    }
                    Some(Ok(Message::Ping(p))) => { let _ = ws.send(Message::Pong(p)).await; }
                    Some(Ok(_)) => {}
                    Some(Err(e)) => { warn!("ws recv: {e}"); break; }
                    None => break,
                }
            }

            // Send authoritative snapshot periodically
            _ = tokio::time::sleep(tick_dt) => {
                let (snap, sid) = build_snapshot(&app, &rid);
                _last_snap = sid;
                send(&app, &mut ws, &snap).await?;
            }
        }
    }

    // Cleanup
    {
        let mut rooms = app.rooms.lock();
        if let Some(room) = rooms.get_mut(&rid) {
            room.players.remove(&player_id);
            if room.players.is_empty() {
                rooms.remove(&rid);
            }
        }
    }
    Ok(())
}

fn build_snapshot(app: &AppState, rid: &str) -> (ServerToClient, u32) {
    let (server_tick, sid, payload) = {
        let mut rooms = app.rooms.lock();
        let room = rooms.get_mut(rid).unwrap();
        room.tick += 1;
        room.snap_id = room.snap_id.wrapping_add(1);

        let server_tick = room.tick;
        let sid = room.snap_id;

        // Payload is engine-owned; here we emit minimal demo payload (tick only)
        #[derive(serde::Serialize)]
        struct DemoState { tick: u64 }
        let demo = DemoState { tick: server_tick };
        let raw = postcard::to_allocvec(&demo).unwrap();

        (server_tick, sid, raw)
    };

    let msg = ServerToClient::Snapshot {
        id: sid,
        server_tick,
        base_id: None,
        compressed: true,
        payload: lz4_flex::compress_prepend_size(&payload),
    };
    (msg, sid)
}

async fn on_client_msg(
    app: &AppState,
    rid: &str,
    pid: &str,
    ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    msg: ClientToServer,
) -> Result<()> {
    match msg {
        ClientToServer::InputFrame { seq, input_blob, sig, .. } => {
            // basic rate limit
            let mut kick = false;
            {
                let mut rooms = app.rooms.lock();
                if let Some(room) = rooms.get_mut(rid) {
                    if let Some(p) = room.players.get_mut(pid) {
                        p.tokens += 8.0; // refill
                        if p.tokens > 60.0 { p.tokens = 60.0; }
                        p.tokens -= 1.0;
                        if p.tokens < 0.0 {
                            kick = true;
                        } else {
                            p.last_input_seq = seq;
                            p.last_seen = tokio::time::Instant::now();
                        }
                        // lightweight tamper check: verify sig with session hint
                        let mut hint = [0u8; 8];
                        hint.copy_from_slice(&room.session_key.0[0..8]);
                        if sig != aw_net_proto::sign16(&input_blob, &hint) {
                            // tamper detected; you can keep counters and kick later
                            // for demo, just warn
                            warn!("tamper-evident signature mismatch for pid={pid}");
                        }
                    }
                }
            }
            if kick {
                send(app, ws, &ServerToClient::RateLimited).await?;
            }
        }
        ClientToServer::Ping { nano } => {
            send(app, ws, &ServerToClient::Pong { nano }).await?;
        }
        ClientToServer::Ack { .. } => { /* optional: track delivery / deltas */ }
        _ => { /* ignore other in-session messages */ }
    }
    Ok(())
}

// Helpers
async fn send(app: &AppState, ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>, msg: &ServerToClient) -> Result<()> {
    let bytes = aw_net_proto::encode_msg(app.codec, msg);
    ws.send(Message::Binary(bytes)).await?;
    Ok(())
}
async fn recv<T: for<'de> serde::Deserialize<'de>>(
    app: &AppState,
    ws: &mut tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
) -> Result<T> {
    let msg = ws.next().await.ok_or_else(|| anyhow::anyhow!("ws closed"))??;
    match msg {
        Message::Binary(b) => {
            let t = aw_net_proto::decode_msg::<T>(app.codec, &b)?;
            Ok(t)
        }
        _ => Err(anyhow::anyhow!("unexpected ws message")),
    }
}