use anyhow::Result;
use dotenvy::dotenv;
use lantern::lumos::user::User;

fn main() -> Result<()> {
    dotenv().ok();
    let lumos = User::attach("nlcssingapore", "testing123", "what").unwrap();
    println!("{:?}", &lumos.connection.secret);
    println!("{:?}", &lumos.connection.device_id);

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
