use crate::lumos::error::FireflyError;
use crate::lumos::filter::FFTaskFilter;
use crate::lumos::task::{AVTask, RawFFTask, Response};
use crate::models::UserPG;
use utils::*;

use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use dotenvy::dotenv;
use reqwest::Client;
use std::error::Error;
use uuid::Uuid;

pub mod utils;

pub struct User {
    pub connection: Info,
    http_client: Client,
    db_conn: Pool<ConnectionManager<PgConnection>>,
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

impl<'a> User {
    /// Instansiates a [`User`] that is the channel for commucation with Firefly.
    pub async fn attach(
        school_code: &'a str,
        app_id: &'a str,
        user_email: &'a str,
    ) -> Result<User, Box<dyn Error>> {
        use crate::schema::users::dsl::*; // imports useful aliases for diesel
        dotenv().ok();

        let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
        let manager = ConnectionManager::<PgConnection>::new(db_url);
        let pool = Pool::builder()
            .test_on_check_out(true)
            .build(manager)
            .expect("Could not build connection pool");

        // let db = PgConnection::establish(&database_url) // potentially add pooling
        //     .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

        let mut user = User {
            connection: Info {
                device_id: Uuid::new_v4().to_string(),
                ..Default::default()
            },
            tasks: vec![],
            http_client: Client::new(),
            db_conn: pool.clone(),
        };

        let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");
        let url = reqwest::Url::parse(&(portal + school_code))?;
        let res = user.http_client.get(url).send().await?.text().await?;
        let res = parse_xml(res);

        user.connection.http_endpoint = String::from("https://") + &res[1] + "/";
        user.connection.school_code = school_code.to_string();
        user.connection.app_id = app_id.to_string();

        let emails = users
            .filter(email.eq(user_email))
            .load::<UserPG>(&mut (pool.clone().get().unwrap()))
            .expect("failed to get emails.");

        if emails.is_empty() {
            auth(&mut user).await;
            add_user_to_db(&mut user, user_email);
        } else {
            let data = emails.first().unwrap();
            user.connection.secret = data.firefly_secret.to_owned();
            user.connection.device_id = data.device_id.to_owned();
        }
        user.connection.email = user_email.to_string();
        Ok(user)
    }

    /// Gets tasks from Firefly based on filter provided.
    ///
    /// This function querys the Firefly API with a POST request. The Firefly API demands a filter to sort
    /// the tasks; sent in the body of the POST request. [`FFTaskFilter`] functions as an
    /// abstraction over this filter (which itself constructed in this function).
    ///
    /// Multiple filters are needed (when more than 100 tasks are being requested) due to technical
    /// limiations with the API. See [`to_json`](FFTaskFilter::to_json) for more details.
    ///
    /// TALK ABOUT THE THREAD SPAWNING ETC
    ///
    /// # Examples
    /// DO THIS
    pub async fn get_ff_tasks(&mut self, filter: FFTaskFilter) -> Result<()> {
        // TODO: check last time since retrieved tasks:
        // if it has been shorter than a day, then just get the x most recent tasks (determined by
        // user setting)
        // if it has be a month since the last fresh 'install', get it all in the background and
        // refresh the cache

        // BUG: Idk how to check when firefly updates its tasks so it might be annoying to update
        // so often; need to figure out some sort of polling mechanism

        // HACK: Potentially make the RPC return a stream so that it can return the first one
        // immeadiately, and then the rest after
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
        let mut items = vec![];
        let res = filter.to_json(self.http_client.clone(), url.clone()).await;
        let mut handles;

        let res = match res {
            Ok(tuple) => tuple,
            Err(e) => match e {
                FireflyError::InvalidSecret => {
                    auth(self).await;
                    let params = [
                        ("ffauth_device_id", &self.connection.device_id),
                        ("ffauth_secret", &self.connection.secret),
                    ];
                    let url = reqwest::Url::parse_with_params(
                        &(self.connection.http_endpoint.to_string()
                            + "api/v2/taskListing/view/student/tasks/all/filterBy"),
                        params,
                    )?;

                    filter
                        .to_json(self.http_client.clone(), url)
                        .await
                        .context("failed while getting filter after refreshing secret")
                        .unwrap() // HACK: can still crash :(
                }
                FireflyError::HTTP(e) => panic!("{e}"),
                FireflyError::Misc(e) => panic!("{e}"), // HACK: remove panics
            },
        };

        match res {
            (Some(filters), Some(res)) => {
                items.extend(res.items.unwrap());
                handles = Vec::with_capacity(filters.len() + 1);

                for filter in filters {
                    let url = url.clone();
                    let client = self.http_client.clone();
                    handles.push(tokio::spawn(async move {
                        let res = client
                            .post(url.clone())
                            .json(&filter)
                            .send()
                            .await
                            .unwrap()
                            .text()
                            .await
                            .unwrap();

                        serde_json::from_str::<Response>(&res)
                            .unwrap()
                            .items
                            .unwrap()
                    }));
                }

                for handle in handles {
                    items.extend(handle.await.unwrap());
                }
            }
            (None, Some(res)) => {
                items = res.items.unwrap();
            }
            _ => {}
        };

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

        // update_tasks_db(self); commented out because there is no point rn :)
        Ok(())
    }
}
