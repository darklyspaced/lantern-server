#![allow(unused)]
use std::error::Error;
use std::fmt;
use std::thread::panicking;

use crate::models::{NewUserPG, UserPG};
use crate::serialise_res::Item;
use crate::serialise_res::Task;
use crate::task_filter::TaskFilter;
use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use quick_xml::{events::Event, reader::Reader};
use reqwest::{blocking::Client, header};
use uuid::Uuid;

// HACK: used blocking requests instead of async

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

fn parse_xml(response: String) -> Vec<String> {
    let mut reader = Reader::from_str(response.as_str());
    reader.trim_text(true);
    let (mut txt, mut buf) = (Vec::new(), Vec::new());

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {} {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Text(e)) => txt.push(e.unescape().unwrap().into_owned()),
            _ => (),
        }
        buf.clear();
    }
    txt
}

fn get_http_endpoint(instance: &mut User, school_code: &str) -> Result<String> {
    let res;
    let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");
    let url = reqwest::Url::parse(&(portal + school_code))?;

    res = instance.daemon.http_client.get(url).send()?.text()?;
    let res = parse_xml(res);
    Ok(String::from("https://") + &res[1] + "/")
}

fn add_user_to_db(instance: &mut User, email: &str) -> UserPG {
    use crate::schema::users;

    let new_user = NewUserPG {
        email,
        firefly_secret: &instance.connection.secret,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&mut instance.daemon.db)
        .expect("Error creating new user")
}

#[derive(Debug)]
pub enum LanternError {
    InvalidSessionID, // cannot auth due to invalid session ID
    Firefly,          // something went wrong interacting with Firefly
    // FIXME: Decide whether to use Box<> or & to make the size of dyn Error known.
    Misc(&'static dyn Error), // anything from a database to a dotenvy error (third party errors essentially)
}

impl Error for LanternError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Misc(e) => Some(e),
            _ => None,
        }
    }
}

impl fmt::Display for LanternError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:?})", {
            match self {
                Self::InvalidSessionID => {
                    String::from("could not auth with firefly; invalid session id")
                }
                Self::Firefly => String::from("something went wrong interacting with firefly"),
                Self::Misc(e) => e.to_string(),
            }
        })
    }
}

fn auth(instance: &mut User) -> Result<()> {
    let params = [
        ("ffauth_device_id", &instance.connection.device_id),
        ("ffauth_secret", &instance.connection.secret),
        ("device_id", &instance.connection.device_id),
        ("app_id", &instance.connection.app_id),
    ];
    let url = reqwest::Url::parse_with_params(
        &(instance.connection.http_endpoint.to_string() + "Login/api/gettoken"),
        params,
    )?;

    let res = instance
        .daemon
        .http_client
        .get(url)
        .header(
            header::COOKIE,
            header::HeaderValue::from_static("ASP.NET_SessionId=l2wkr0lecg4yz2ndqtbbou52"),
        )
        .send()?
        .text()?;

    let txt = parse_xml(res);
    if let Some(secret) = txt.first() {
        if secret != "Invalid token" {
            instance.connection.secret = secret.to_owned();
        } else {
        }
    };
    Ok(())
}

impl<'a> User {
    pub fn attach(
        school_code: &'a str,
        app_id: &'a str,
        temp_email: &'a str,
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

        if let Ok(endpoint) = get_http_endpoint(&mut user, school_code) {
            user.connection.http_endpoint = endpoint;
            user.connection.school_code = school_code.to_string();
            user.connection.app_id = app_id.to_string();
        } else {
            panic!("Failed to find school from provided school code.");
        }

        let emails = users
            .filter(crate::schema::users::email.eq(temp_email))
            .load::<UserPG>(&mut user.daemon.db)
            .expect("Error loading emails");

        if emails.is_empty() {
            auth(&mut user);
            add_user_to_db(&mut user, temp_email);
        } else {
            user.connection.secret = emails.first().unwrap().firefly_secret.to_owned();
        }
        user.connection.email = temp_email.to_string();
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
