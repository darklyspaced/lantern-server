#![allow(unused)]
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
    connection: Info,
    daemon: Daemon,
    pub tasks: Vec<Item>,
}

struct Info {
    school_code: String,
    device_id: String,
    app_id: String,
    email: String,
    http_endpoint: String,
    secret: String,
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

fn create_user(instance: &mut User, email: &str) -> UserPG {
    use crate::schema::users;

    let new_user = NewUserPG {
        email,
        firefly_secret: "",
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .get_result(&mut instance.daemon.db)
        .expect("Error creating new user")
}

impl<'a> User {
    // creates empty instance of Firefly; do not have intergation
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let db = PgConnection::establish(&database_url)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        User {
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
        }
    }

    // attaches to an already existing intergration
    pub fn attach(
        &mut self,
        school_code: &'a str,
        app_id: &'a str,
        temp_email: &'a str,
    ) -> Result<(), &'static str> {
        use crate::schema::users::dsl::*; // imports useful aliases for diesel

        let http_endpoint = get_http_endpoint(self, school_code);
        if let Ok(endpoint) = http_endpoint {
            self.connection.http_endpoint = endpoint;
            self.connection.school_code = school_code.to_string();
            self.connection.app_id = app_id.to_string();
        } else {
            return Err("Failed to find school!");
        }

        let results = users
            .filter(crate::schema::users::email.eq(temp_email))
            .load::<UserPG>(&mut self.daemon.db)
            .expect("Error loading emails");

        if results.len() >= 1 {
            self.connection.email = temp_email.to_string();
            // TODO: add phantom data to differentiate between authenticated state and not
            //authenticated state
            self.connection.secret = results[0].firefly_secret.clone(); // HACK: used clone :(
        } else {
            create_user(self, temp_email);
            self.connection.email = temp_email.to_string();
        }

        Ok(())
    }

    // verifies that school exists
    // TODO: remove once attach encompasses this functionality
    //
    // pub fn verify(
    //     &mut self,
    //     school_code: &'a str,
    //     app_id: &'a str,
    //     email: &'a str,
    // ) -> Result<&mut Firefly, &'static str> {
    //     let response = check_existence(Some(self), school_code);
    //     if let Ok(res) = response {
    //         if res.len() >= 3 {
    //             self.school_code = school_code.to_string();
    //             self.app_id = app_id.to_string();
    //             self.address = String::from("https://") + &res[1] + "/";
    //             self.email = email.to_string();
    //         } else {
    //             return Err("School not found!");
    //         }
    //     } else {
    //         return Err("Request failed");
    //     }
    //     Ok(self)
    // }
    //
    // creates intergration
    // TODO: Only allow this to be called after new && (attach || verify) have been called
    pub fn auth(&mut self) -> Result<()> {
        let params = [
            ("ffauth_device_id", &self.connection.device_id),
            ("ffauth_secret", &self.connection.secret),
            ("device_id", &self.connection.device_id),
            ("app_id", &self.connection.app_id),
        ];
        let url = reqwest::Url::parse_with_params(
            &(self.connection.http_endpoint.to_string() + "Login/api/gettoken"),
            params,
        )?;

        let res = self
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
            self.connection.secret = secret.to_owned();
        }
        Ok(())
    }

    // TODO: Only allow this to be called after auth can been called
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

// impl Default for User {
//     fn default() -> Self {
//         Self::new()
//     }
// }
