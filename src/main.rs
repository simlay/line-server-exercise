
use line_server_exercise::LineServer;
use clap::Parser;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let args = LineServer::parse();
    env_logger::init();
    args.run().await?;
    Ok(())
}
