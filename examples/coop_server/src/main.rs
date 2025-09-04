use astraweave_net::GameServer;
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = std::sync::Arc::new(GameServer::new());
    server.run_ws("127.0.0.1:9090").await?;
    Ok(())
}
