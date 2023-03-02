use anyhow::Result;
use lantern::lumos::Firefly;
use lantern::serialise_res::Source;
use lantern::task_filter::{CompletionStatus, Order, ReadStatus, TaskFilter};
use sqlx::postgres::PgPoolOptions;

fn main() -> Result<()> {
    // let _db = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect("postgresql://localhost:5432")
    //     .await?;
    //
    let mut lumos = Firefly::new();
    let filter = TaskFilter {
        read: ReadStatus::All,
        status: CompletionStatus::Todo,
        sorting: (String::from("DueDate"), Order::Ascending),
        results: 50,
        source: Some(Source::Ff),
    };

    if let Ok(res) = lumos.verify("nlcssingapore", "test", "srohanjd@gmail.com") {
        println!("{:#?}", res);
    }

    lumos.auth().unwrap();
    lumos.get_tasks(filter).unwrap();

    println!("{:#?}", lumos.tasks);
    Ok(())
}
