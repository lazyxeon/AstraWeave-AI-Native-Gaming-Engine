use anyhow::Result;
use astraweave_ai::{Orchestrator, RuleOrchestrator};
use astraweave_core::{PlanIntent, WorldSnapshot};
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;

pub async fn run_ws_server(addr: &str) -> Result<()> {
    use tokio::net::TcpListener;
    let listener = TcpListener::bind(addr).await?;
    println!("Companion WS server listening on {}", addr);
    while let Ok((stream, _)) = listener.accept().await {
        tokio::spawn(handle_conn(stream));
    }
    Ok(())
}

async fn handle_conn(stream: tokio::net::TcpStream) -> Result<()> {
    let ws = tokio_tungstenite::accept_async(stream).await?;
    let (mut tx, mut rx) = ws.split();
    let orch = RuleOrchestrator;

    while let Some(msg) = rx.next().await {
        let msg = msg?;
        if msg.is_text() {
            let txt = msg.into_text()?;
            let snap: WorldSnapshot = serde_json::from_str(&txt)?;
            let plan: PlanIntent = orch.propose_plan(&snap);
            let out = serde_json::to_string(&plan)?;
            tx.send(Message::Text(out)).await?;
        } else if msg.is_close() {
            break;
        }
    }
    Ok(())
}

pub async fn ws_client_roundtrip(addr: &str, snapshot: &WorldSnapshot) -> Result<PlanIntent> {
    let (ws, _) = tokio_tungstenite::connect_async(addr).await?;
    let (mut tx, mut rx) = ws.split();
    let js = serde_json::to_string(snapshot)?;
    tx.send(Message::Text(js)).await?;
    if let Some(msg) = rx.next().await {
        let msg = msg?;
        let txt = msg.into_text()?;
        let plan: PlanIntent = serde_json::from_str(&txt)?;
        return Ok(plan);
    }
    anyhow::bail!("no response")
}
