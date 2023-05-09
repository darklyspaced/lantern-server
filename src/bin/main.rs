use anyhow::Result;
use lantern::lumos::rpc::{LanternServer, TaskService};
use tonic::transport::Server;

pub struct Request {}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:8000".parse().unwrap();
    println!("Starting server on port 127.0.0.1:8000");

    let task_service = TaskService::default();
    let svc = LanternServer::new(task_service);

    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
