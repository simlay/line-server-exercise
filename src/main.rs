use clap::Parser;
use line_server_exercise::LineServer;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = LineServer::parse();
    env_logger::init();
    args.run().await?;
    Ok(())
}
