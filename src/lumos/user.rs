use super::parse_xml;
use crate::error::LanternError;
use crate::models::{NewUserPG, UserPG};
use crate::serialise_res::Item;
use crate::serialise_res::Task;
use crate::task_filter::TaskFilter;

use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use reqwest::{blocking::Client, header};
use uuid::Uuid;

pub struct User {
    pub connection: Info,
    daemon: Daemon,
    tasks: Vec<Item>,
}

pub struct Info {
    school_code: String,
    device_id: String,
    app_id: String,
    email: String,
    http_endpoint: String,
    pub secret: String,
}

struct Daemon {
    http_client: Client,
    db: PgConnection,
}

impl<'a> User {
    pub fn attach(
        school_code: &'a str,
        app_id: &'a str,
        user_email: &'a str,
    ) -> Result<User, LanternError> {
        use crate::schema::users::dsl::*; // imports useful aliases for diesel

        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let db = PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        let mut user = User {
            daemon: Daemon {
                http_client: Client::new(),
                db,
            },
            connection: Info {
                school_code: String::from(""),
                app_id: String::from(""),
                email: String::from(""),
                device_id: Uuid::new_v4().to_string(),
                secret: String::from(""),
                http_endpoint: String::from(""),
            },
            tasks: vec![],
        };

        let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");
        let url = reqwest::Url::parse(&(portal + school_code))?;

        let response = user.daemon.http_client.get(url).send()?.text();
        if let Ok(res) = response {
            let res = parse_xml(res);
            user.connection.http_endpoint = String::from("https://") + &res[1] + "/";
            user.connection.school_code = school_code.to_string();
            user.connection.app_id = app_id.to_string();
        } else {
            panic!("Failed to find school from provided school code.");
        };

        let emails = users
            .filter(crate::schema::users::email.eq(user_email))
            .load::<UserPG>(&mut user.daemon.db)
            .expect("Error loading emails");

        if emails.is_empty() {
            if let Ok(()) = auth(&mut user) {
                add_user_to_db(&mut user, user_email);
            } else {
                panic!("Invalid Firefly SessionID!");
            }
        } else {
            user.connection.secret = emails.first().unwrap().firefly_secret.to_owned();
        }
        user.connection.email = user_email.to_string();
        Ok(user)
    }

    pub fn get_tasks(&mut self, filter: TaskFilter) -> Result<()> {
        let params = [
            ("ffauth_device_id", &self.connection.device_id),
            ("ffauth_secret", &self.connection.secret),
        ];
        let url = reqwest::Url::parse_with_params(
            &(self.connection.http_endpoint.to_string()
                + "api/v2/taskListing/view/student/tasks/all/filterBy"),
            params,
        )?;

        let filters = filter.to_json();
        let res = self
            .daemon
            .http_client
            .post(url)
            .json(&filters[0])
            .send()?
            .text()?;

        let serialised_response: Task = serde_json::from_str(&res).unwrap();
        let items = serialised_response.items.unwrap();

        if let Some(ref source) = filter.source {
            let parsed_items = items
                .into_iter()
                .filter(|item| {
                    let curr_source = item.task_source.as_ref().unwrap();
                    if source == curr_source {
                        return true;
                    }
                    false
                })
                .collect::<Vec<Item>>();

            self.tasks = parsed_items;
        } else {
            self.tasks = items;
        }
        Ok(())
    }
}

fn auth(user: &mut User) -> Result<(), LanternError> {
    let params = [
        ("ffauth_device_id", &user.connection.device_id),
        ("ffauth_secret", &user.connection.secret),
        ("device_id", &user.connection.device_id),
        ("app_id", &user.connection.app_id),
    ];
    let url = reqwest::Url::parse_with_params(
        &(user.connection.http_endpoint.to_string() + "Login/api/gettoken"),
        params,
    )?;

    let res = user
        .daemon
        .http_client
        .get(url)
        .header(
            header::COOKIE,
            header::HeaderValue::from_static("ASP.NET_SessionId=llipgya1gswqety0tdeoo10h"),
        )
        .send()?
        .text()?;

    let txt = parse_xml(res);
    if let Some(secret) = txt.first() {
        if secret != "Invalid token" {
            user.connection.secret = secret.to_string();
        } else {
            return Err(LanternError::Firefly);
        }
    };
    Ok(())
}

fn add_user_to_db(instance: &mut User, new_email: &str) -> UserPG {
    use crate::schema::users;

    let new_user = NewUserPG {
        email: new_email,
        firefly_secret: &instance.connection.secret,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&mut instance.daemon.db)
        .expect("Error creating new user")
}
