use firefly_api_driver::Lumos;

fn main() {
    let mut lumos = Lumos::new();
    if lumos.attach("nlcssingapore", "test").is_ok() {
        lumos.auth();
        lumos.get_tasks();
    } else {
        panic!("Failed to attach to school");
    }
}
