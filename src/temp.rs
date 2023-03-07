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

fn create_user(instance: &mut User, email: &str, secret: &str) -> UserPG {
    use crate::schema::users;

    let new_user = NewUserPG {
        email,
        firefly_secret: secret,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&mut instance.daemon.db)
        .expect("Error creating new user")
}

pub fn attach(
    &mut self,
    school_code: &'a str,
    app_id: &'a str,
    email: &'a str,
) -> Result<(), &'static str> {
    use crate::schema::users::dsl::*; // imports useful aliases for diesel

    create_user(self, email, "test");

    let results = users
        .load::<UserPG>(&mut self.daemon.db)
        .expect("Error loading emails");
    for user in results {
        println!("{}", user.email);
    }

    let http_endpoint = get_http_endpoint(self, school_code);
    if let Ok(endpoint) = http_endpoint {
        self.connection.http_endpoint = endpoint;
    } else {
        return Err("Failed to find school!");
    }
    Ok(())
}
