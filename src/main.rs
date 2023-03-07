use anyhow::Result;
use dotenvy::dotenv;
use lantern::lumos::User;
// use lantern::serialise_res::Source;
// use lantern::task_filter::{CompletionStatus, Order, ReadStatus, TaskFilter};
// use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    // let _db = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect("postgresql://localhost:5432")
    //     .await?;

    dotenv().ok();
    let lumos = User::new();

    lumos.await.attach("test", "yes", "what").await.unwrap();
    // let filter = TaskFilter {
    //     read: ReadStatus::All,
    //     status: CompletionStatus::Todo,
    //     sorting: (String::from("DueDate"), Order::Ascending),
    //     results: 50,
    //     source: Some(Source::Ff),
    // };
    //
    // if let Ok(res) = lumos.verify("nlcssingapore", "test", "srohanjd@gmail.com") {
    //     println!("{:#?}", res);
    // }
    //
    // lumos.auth().unwrap();
    // lumos.get_tasks(filter).unwrap();
    //
    // println!("{:#?}", lumos.tasks);
    Ok(())
}
