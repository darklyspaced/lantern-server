use crate::schema::users;
use diesel::prelude::*;

#[derive(Queryable)]
pub struct UserPG {
    pub id: i32,
    pub email: String,
    pub firefly_secret: String,
}

#[derive(Insertable)]
#[diesel(table_name = users)]
pub struct NewUserPG<'a> {
    pub email: &'a str,
    pub firefly_secret: &'a str,
}