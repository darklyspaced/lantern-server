use crate::lumos::filter::FFTaskFilter;
use crate::lumos::task::{AVTask, RawFFTask, Response};
use crate::models::UserPG;
use utils::*;

use anyhow::Result;
use diesel::prelude::*;
use dotenvy::dotenv;
use reqwest::Client;
use std::error::Error;
use uuid::Uuid;

mod utils;

pub struct User {
    pub connection: Info,
    daemon: Daemon,
    pub tasks: Vec<AVTask>,
}

#[derive(Default)]
pub struct Info {
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

impl<'a> User {
    /// Instansiates a [`User`] that is the channel for commucation with Firefly.
    pub async fn attach(
        school_code: &'a str,
        app_id: &'a str,
        user_email: &'a str,
    ) -> Result<User, Box<dyn Error>> {
        use crate::schema::users::dsl::*; // imports useful aliases for diesel
        dotenv().ok();

        let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let db = PgConnection::establish(&database_url) // make this a connection that is derived
            // from a r2d2::Pool and provided to the struct (in a thread)
            .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));
        let mut user = User {
            daemon: Daemon {
                http_client: Client::new(),
                db,
            },
            connection: Info {
                device_id: Uuid::new_v4().to_string(),
                ..Default::default()
            },
            tasks: vec![],
        };

        let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");
        let url = reqwest::Url::parse(&(portal + school_code))?;
        let res = user
            .daemon
            .http_client
            .get(url)
            .send()
            .await?
            .text()
            .await?;
        let res = parse_xml(res);

        user.connection.http_endpoint = String::from("https://") + &res[1] + "/";
        user.connection.school_code = school_code.to_string();
        user.connection.app_id = app_id.to_string();

        let emails = users
            .filter(email.eq(user_email))
            .load::<UserPG>(&mut user.daemon.db)
            .expect("Failed to get emails.");

        if emails.is_empty() {
            add_user_to_db(&mut user, user_email);
        } else {
            let data = emails.first().unwrap();
            user.connection.secret = data.firefly_secret.to_owned();
            user.connection.device_id = data.device_id.to_owned();
        }
        user.connection.email = user_email.to_string();
        Ok(user)
    }

    /// Gets tasks from Firefly based on authentication done in attach
    ///
    /// This function querys the Firefly API with a POST request to determine how to filter the
    /// tasks that it returns. You can customise this filter with [`TaskFilter`]. The returned JSON
    /// is parsed using [`serde_json`] and stored in connection.tasks.
    ///
    /// ```
    /// use dotenvy::dotenv;
    /// use lantern::prelude::*;
    ///
    /// fn main() {
    ///     dotenv().ok();
    ///     let mut lumos = User::attach("nlcssingapore", "avagarde_client", "sample@email.com").unwrap();
    ///
    ///     let filter = TaskFilter {
    ///         read: ReadStatus::All,
    ///         status: CompletionStatus::Todo,
    ///         sorting: (SortBy::DueDate, Order::Ascending),
    ///         source: Some(Source::Ff),
    ///     };
    ///
    ///     lumos
    ///         .get_tasks(filter)
    ///         .unwrap_or_else(|err| panic!("Failed with {}", err));
    ///
    ///     println!("{:?}", lumos.tasks);
    /// }
    /// ```

    pub async fn get_tasks(&mut self, filter: FFTaskFilter) -> Result<()> {
        fn standardise_ff_tasks(items: Vec<RawFFTask>) -> Vec<AVTask> {
            if let Some(tasks) = rawtask_to_task(items) {
                tasks
            } else {
                eprintln!("Error converting RawTask -> Task");
                vec![AVTask {
                    title: String::from("ERROR 102: RawTask -> Task failed!"),
                    ..Default::default()
                }]
            }
        }

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
            .send()
            .await?
            .text()
            .await?;

        if res == "Invalid token" {
            auth(self).await;
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
                .send()
                .await?
                .text()
                .await?;

            diesel::update(users)
                .filter(email.eq(&self.connection.email))
                .set(firefly_secret.eq(&self.connection.secret))
                .execute(&mut self.daemon.db)?;
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
                .collect::<Vec<RawFFTask>>();

            self.tasks = standardise_ff_tasks(parsed_items);
        } else {
            self.tasks = standardise_ff_tasks(items);
        }
        update_tasks_db(self);
        Ok(())
    }
}
