use firefly_api_driver::Lumos;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lumos = Lumos::new();
    lumos.attach("nlcssingapore", "test");
    println!("{:?}", lumos);
    Ok(())
}
