use firefly_api_driver::Lumos;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lumos = Lumos::new();
    lumos.build("fuck", "yeah").await;
    Ok(())
}
