use firefly_api_driver::Lumos;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut lumos = Lumos::new();
    let temp = lumos.attach("nlcssingapore", "test");
    if let Ok(strut) = temp {
        println!("{strut:?}");
    };
    Ok(())
}
