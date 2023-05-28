use anyhow::Result;
use dotenvy::dotenv;
use lantern::lumos::rpc::{LanternServer, TaskService};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let addr = "[::1]:8000".parse().unwrap();
    println!("Starting server on port 127.0.0.1:8000");

    let svc = LanternServer::new(TaskService::new().await);

    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
