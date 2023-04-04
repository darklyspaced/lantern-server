use crate::schema::tasks;
use crate::schema::users;
use diesel::prelude::*;

#[derive(Queryable)]
#[diesel(table_name = users)]
pub struct UserPG {
    pub id: i32,
    pub email: String,
    pub firefly_secret: String,
    pub device_id: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUserPG<'a> {
    pub email: &'a str,
    pub firefly_secret: &'a str,
    pub device_id: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = tasks)]
pub struct NewTask<'a> {
    pub user_email: &'a str,
    pub local_tasks: serde_json::Value,
    pub firefly_tasks: serde_json::Value,
}

// #[derive(Queryable, AsChangeset, Identifiable)]
// #[diesel(table_name = tasks)]
// pub struct Task {
//     pub id: i32,
//     pub user_email: String,
//     pub local_tasks: String,
//     pub firefly_tasks: String,
// }
