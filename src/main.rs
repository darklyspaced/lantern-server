use anyhow::Result;
use dotenvy::dotenv;
use lantern::lumos::User;

fn main() -> Result<()> {
    dotenv().ok();
    let mut lumos = User::new();

    lumos
        .attach("nlcssingapore", "yes", "srohanjd@gmail.com")
        .unwrap();

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
