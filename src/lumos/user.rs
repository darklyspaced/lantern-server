use super::parse_xml;
use crate::error::LanternError;
use crate::lumos::filter::TaskFilter;
use crate::lumos::task::{Response, Task};
use crate::models::{NewTask, NewUserPG, UserPG};

use anyhow::Result;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use dotenvy::dotenv;
use reqwest::{blocking::Client, header};
use serde_json::json;
use uuid::Uuid;

pub struct User {
    pub connection: Info,
    daemon: Daemon,
    pub tasks: Vec<Task>,
}

pub struct Info {
    school_code: String,
    pub device_id: String,
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
    /// Authenticates user with Firefly
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
            return Err(LanternError::SchoolCode);
        };

        let emails = users
            .filter(email.eq(user_email))
            .load::<UserPG>(&mut user.daemon.db)
            .expect("Error loading emails");

        if emails.is_empty() {
            if let Ok(()) = auth(&mut user) {
                add_user_to_db(&mut user, user_email);
            } else {
                return Err(LanternError::InvalidSessionID);
            }
        } else {
            let data = emails.first().unwrap();
            user.connection.secret = data.firefly_secret.to_owned();
            user.connection.device_id = data.device_id.to_owned();
        }
        user.connection.email = user_email.to_string();
        Ok(user)
    }

    /// Gets tasks from Firefly based on authentication done previously
    ///
    /// This function querys the Firefly API with a POST request to determine how to filter the
    /// tasks that it returns. You can customise this filter with [`TaskFilter`]. The returned JSON
    /// is parsed using [`serde_json`] and stored in connection.tasks.
    ///
    /// ```
    /// use dotenvy::dotenv;
    /// use lantern::prelude::*;
    ///
    /// dotenv().ok();
    /// let mut lumos = User::attach("nlcssingapore", "avagarde_client", "sample@email.com").unwrap();
    ///
    /// let filter = TaskFilter {
    ///     read: ReadStatus::All,
    ///     status: CompletionStatus::Todo,
    ///     sorting: (SortBy::DueDate, Order::Ascending),
    ///     results: 50,
    ///     source: Some(Source::Ff),
    /// };
    ///
    /// lumos
    ///     .get_tasks(filter)
    ///     .unwrap_or_else(|err| panic!("Failed with {}", err));
    ///
    /// println!("{:?}", lumos.tasks);
    /// ```

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
        let mut res = self
            .daemon
            .http_client
            .post(url)
            .json(&filters[0])
            .send()?
            .text()?;

        if res == "Invalid token" {
            if let Ok(()) = auth(self) {
                use crate::schema::users::dsl::*;
                let params = [
                    ("ffauth_device_id", &self.connection.device_id),
                    ("ffauth_secret", &self.connection.secret),
                ];
                let url = reqwest::Url::parse_with_params(
                    &(self.connection.http_endpoint.to_string()
                        + "api/v2/taskListing/view/student/tasks/all/filterBy"),
                    params,
                )?;

                res = self
                    .daemon
                    .http_client
                    .post(url)
                    .json(&filters[0])
                    .send()?
                    .text()?;

                diesel::update(users)
                    .filter(email.eq(&self.connection.email))
                    .set(firefly_secret.eq(&self.connection.secret))
                    .execute(&mut self.daemon.db)?;
            } else {
                panic!("Refreshing secret failed.")
            }
        }

        let serialised_response: Response = serde_json::from_str(&res).unwrap();
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
                .collect::<Vec<Task>>();

            self.tasks = parsed_items;
        } else {
            self.tasks = items;
        }
        update_tasks_db(self);
        Ok(())
    }
}

fn auth(instance: &mut User) -> Result<(), LanternError> {
    dotenv().ok();
    let params = [
        ("ffauth_device_id", &instance.connection.device_id),
        ("ffauth_secret", &String::from("")),
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
            header::HeaderValue::from_static("ASP.NET_SessionId=hpk3341e5kkmcay2smayowxv"),
        )
        .send()?
        .text()?;

    let txt = parse_xml(res);
    if let Some(secret) = txt.first() {
        if secret != "Invalid token" {
            instance.connection.secret = secret.to_string();
        } else {
            return Err(LanternError::FireflyAPI);
        }
    };
    Ok(())
}

fn add_user_to_db(instance: &mut User, new_email: &str) {
    use crate::schema::tasks;
    use crate::schema::users;

    let new_user = NewUserPG {
        email: new_email,
        firefly_secret: &instance.connection.secret,
        device_id: &instance.connection.device_id,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut instance.daemon.db)
        .expect("error creating new user");

    let new_task_user_relation = NewTask {
        user_email: new_email,
        firefly_tasks: json!({"empty": true}),
        local_tasks: json!({"empty": true}),
    };
    diesel::insert_into(tasks::table)
        .values(&new_task_user_relation)
        .execute(&mut instance.daemon.db)
        .expect("error create task-user relation");
}

fn update_tasks_db(instance: &mut User) {
    use crate::schema::tasks::dsl::*;

    diesel::update(tasks)
        .filter(user_email.eq(&instance.connection.email))
        .set(firefly_tasks.eq::<serde_json::Value>(serde_json::to_value(&instance.tasks).unwrap()))
        .execute(&mut instance.daemon.db)
        .unwrap();
}
