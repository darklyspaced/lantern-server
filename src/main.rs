use dotenvy::dotenv;
use lantern::prelude::*;

fn main() {
    dotenv().ok();
    let mut lumos = User::attach("nlcssingapore", "avagarde_client", "sample@email.com").unwrap();

    let filter = TaskFilter {
        read: ReadStatus::All,
        status: CompletionStatus::Todo,
        sorting: (SortBy::DueDate, Order::Ascending),
        source: Some(Source::Ff),
    };

    lumos
        .get_tasks(filter)
        .unwrap_or_else(|err| panic!("Failed with {}", err));

    println!("{:?}", lumos.tasks);
}
