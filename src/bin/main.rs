use color_eyre::Result;
use dotenvy::dotenv;
use lantern::lumos::rpc::{LanternServer, TaskService};
use tokio::sync::mpsc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    color_eyre::install()?;
    let addr = "[::1]:8080".parse().unwrap();
    println!("Starting server on 127.0.0.1:8080");

    let svc = LanternServer::new(TaskService::new().await);
    let (tx, mut rx) = mpsc::unbounded_channel();

    let serve = Server::builder()
        .accept_http1(true)
        .add_service(tonic_web::enable(svc))
        .serve(addr);

    tokio::spawn(async move {
        if let Err(e) = serve.await {
            eprintln!("Error = {:?}", e);
        };

        tx.send(()).unwrap();
    });

    rx.recv().await;
    Ok(())
}
