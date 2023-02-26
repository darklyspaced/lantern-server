use firefly_api_driver::lumos::Lumos;
use firefly_api_driver::serialise_res::Source;
use firefly_api_driver::task_filter::{CompletionStatus, Order, ReadStatus, TaskFilter};

fn main() {
    let mut lumos = Lumos::new();
    let filter = TaskFilter {
        read: ReadStatus::All,
        status: CompletionStatus::Todo,
        sorting: (String::from("DueDate"), Order::Ascending),
        results: 50,
        source: Some(Source::Ff),
    };

    if lumos.attach("nlcssingapore", "test").is_ok() {
        lumos.auth();
        lumos.get_tasks(filter);
    } else {
        panic!("Failed to attach to school");
    }
    println!("{:#?}", lumos.tasks);
}
