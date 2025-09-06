use anyhow::Result;
use astraweave_core::*;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::{broadcast, Mutex};
use tokio_tungstenite::tungstenite::Message;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum Msg {
    ClientHello { name: String },
    ServerWelcome { id: u32 },
    ServerSnapshot { t: f32, entities: Vec<(u32, IVec2)> },
    ClientProposePlan { actor_id: u32, intent: PlanIntent },
    ServerApplyResult { ok: bool, err: Option<String> },
}

pub struct GameServer {
    pub world: Mutex<World>,
    pub player_id: u32,
    pub companion_id: u32,
    pub enemy_id: u32,
    pub tx: broadcast::Sender<String>,
}

impl GameServer {
    pub fn new() -> Self {
        let mut w = World::new();
        for y in 1..=8 {
            w.obstacles.insert((6, y));
        }
        let player = w.spawn("P", IVec2 { x: 2, y: 2 }, Team { id: 0 }, 100, 0);
        let comp = w.spawn("C", IVec2 { x: 2, y: 3 }, Team { id: 1 }, 80, 30);
        let enemy = w.spawn("E", IVec2 { x: 12, y: 2 }, Team { id: 2 }, 60, 0);
        let (tx, _) = broadcast::channel(64);
        Self {
            world: Mutex::new(w),
            player_id: player,
            companion_id: comp,
            enemy_id: enemy,
            tx,
        }
    }

    pub async fn run_ws(self: &std::sync::Arc<Self>, addr: &str) -> Result<()> {
        use tokio::net::TcpListener;
        let listener = TcpListener::bind(addr).await?;
        println!("Server on {addr}");
        while let Ok((stream, _)) = listener.accept().await {
            let me = self.clone();
            tokio::spawn(async move {
                if let Err(e) = me.handle_conn(stream).await {
                    eprintln!("conn error: {e:?}");
                }
            });
        }
        Ok(())
    }

    async fn handle_conn(self: std::sync::Arc<Self>, stream: tokio::net::TcpStream) -> Result<()> {
        let ws = tokio_tungstenite::accept_async(stream).await?;
        let (mut tx, mut rx) = ws.split();
        let mut rx_bcast = self.tx.subscribe();

        // send welcome
        tx.send(Message::Text(serde_json::to_string(&Msg::ServerWelcome {
            id: 1,
        })?))
        .await?;

        // spawn a task to push snapshots
        let _me = self.clone();
        tokio::spawn(async move {
            loop {
                match rx_bcast.recv().await {
                    Ok(txt) => {
                        // ignore TX errors (client might disconnect)
                        let _ = tx.send(Message::Text(txt)).await;
                    }
                    Err(_) => break,
                }
            }
        });

        while let Some(msg) = rx.next().await {
            let msg = msg?;
            if msg.is_text() {
                let text = msg.into_text()?;
                let m: Msg = serde_json::from_str(&text)?;
                match m {
                    Msg::ClientHello { name } => {
                        println!("Hello from {name}");
                        // send immediate snapshot
                        let w = self.world.lock().await;
                        let ents = vec![
                            (self.player_id, w.pos_of(self.player_id).unwrap()),
                            (self.companion_id, w.pos_of(self.companion_id).unwrap()),
                            (self.enemy_id, w.pos_of(self.enemy_id).unwrap()),
                        ];
                        let snap = Msg::ServerSnapshot {
                            t: w.t,
                            entities: ents,
                        };
                        let _ = self.tx.send(serde_json::to_string(&snap).unwrap());
                    }
                    Msg::ClientProposePlan { actor_id, intent } => {
                        let mut w = self.world.lock().await;
                        let mut log = |s: String| println!("{}", s);
                        let vcfg = ValidateCfg {
                            world_bounds: (0, 0, 19, 9),
                        };
                        let res = validate_and_execute(&mut w, actor_id, &intent, &vcfg, &mut log);
                        let ok = res.is_ok();
                        let err = res.err().map(|e| e.to_string());
                        let reply = Msg::ServerApplyResult { ok, err };
                        // broadcast state update + reply
                        let ents = vec![
                            (self.player_id, w.pos_of(self.player_id).unwrap()),
                            (self.companion_id, w.pos_of(self.companion_id).unwrap()),
                            (self.enemy_id, w.pos_of(self.enemy_id).unwrap()),
                        ];
                        let snap = Msg::ServerSnapshot {
                            t: w.t,
                            entities: ents,
                        };
                        let _ = self.tx.send(serde_json::to_string(&snap).unwrap());
                        let _ = self.tx.send(serde_json::to_string(&reply).unwrap());
                    }
                    Msg::ServerWelcome { .. }
                    | Msg::ServerSnapshot { .. }
                    | Msg::ServerApplyResult { .. } => {
                        // ignore from clients
                    }
                }
            } else if msg.is_close() {
                break;
            }
        }
        Ok(())
    }
}
