use anyhow::Result;
use lantern::lumos::Firefly;
use lantern::serialise_res::Source;
use lantern::task_filter::{CompletionStatus, Order, ReadStatus, TaskFilter};
use sqlx::postgres::PgPoolOptions;

#[tokio::main]
async fn main() -> Result<()> {
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect("postgresql://localhost:5432")
        .await?;

}
