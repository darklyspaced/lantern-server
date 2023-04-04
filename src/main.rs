use anyhow::Result;
use dotenvy::dotenv;
use lantern::{error::LanternError, prelude::*};

fn main() -> Result<()> {
    dotenv().ok();
    let mut lumos = match User::attach("nlcssingapore", "avagarde_client", "sample@email.com") {
        Ok(obj) => obj,
        Err(LanternError::InvalidSessionID) => panic!("Invalid Firefly Session ID"),
        Err(LanternError::SchoolCode) => panic!("No associated school found for school code."),
        Err(err) => panic!("Paniced with {err}"),
    };

    let filter = TaskFilter {
        read: ReadStatus::All,
        status: CompletionStatus::Todo,
        sorting: (SortBy::DueDate, Order::Ascending),
        results: 50,
        source: Some(Source::Ff),
    };

    lumos
        .get_tasks(filter)
        .unwrap_or_else(|err| panic!("Failed with {}", err));

    println!("{:?}", lumos.tasks);
    // println!("{:?}", lumos.connection.secret);

    Ok(())
}
