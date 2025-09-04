use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::Message;
use astraweave_core::{PlanIntent, ActionStep};
use astraweave_net::Msg;

#[tokio::main]
async fn main() -> Result<()> {
    let (ws, _) = tokio_tungstenite::connect_async("ws://127.0.0.1:9090").await?;
    let (mut tx, mut rx) = ws.split();

    // say hello
    tx.send(Message::Text(serde_json::to_string(&Msg::ClientHello { name: "player1".into() })?)).await?;

    // propose a plan for actor_id=2 (companion in our server)
    let plan = PlanIntent {
        plan_id: "client-plan".into(),
        steps: vec![
            ActionStep::MoveTo { x:4, y:3 },
            ActionStep::Throw { item:"smoke".into(), x:7, y:3 },
            ActionStep::CoverFire { target_id:3, duration:2.0 },
        ]
    };
    tx.send(Message::Text(serde_json::to_string(&Msg::ClientProposePlan { actor_id: 2, intent: plan })?)).await?;

    // read a couple of server messages
    for _ in 0..3 {
        if let Some(msg) = rx.next().await {
            let txt = msg?.into_text()?;
            println!("<< {}", txt);
        }
    }
    Ok(())
}
