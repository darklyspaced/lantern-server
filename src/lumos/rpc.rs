pub mod light {
    tonic::include_proto!("light");
}
use super::task::AVTask;
use super::user::User;
use crate::prelude::*;

use anyhow::Result;
use light::lantern_server::Lantern;
pub use light::lantern_server::LanternServer;
use light::{Filter, PTasks, StatusCode};
use std::str::FromStr;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

pub struct TaskService {
    inner: Mutex<User>,
}

#[tonic::async_trait]
impl Lantern for TaskService {
    async fn get_tasks(&self, request: Request<Filter>) -> Result<Response<PTasks>, Status> {
        // TODO: get_tasks should return local_tasks as well
        let filter = construct_filter(request.get_ref());
        let mut user = self.inner.lock().await;

        if let Ok(filter) = filter {
            user.get_ff_tasks(filter).await.unwrap();
        } else {
            panic!("failed to create filter from input");
        }

        Ok(Response::new(PTasks {
            body: serde_json::to_string(&user.tasks).unwrap(),
        }))
    }

    async fn add_tasks(&self, request: Request<PTasks>) -> Result<Response<StatusCode>, Status> {
        // use crate::schema::tasks::dsl::*;

        // let emails = tasks
        //     .filter(user_email.eq("test"))
        //     .load::<crate::models::Tasks>(&mut user.daemon.db)
        //     .expect("Failed to get emails.");

        println!("{}", request.get_ref().body);
        let loc_tasks = serde_json::from_str::<AVTask>(request.get_ref().body.as_ref());
        if loc_tasks.is_ok() {
            return Ok(Response::new(StatusCode {
                success: true,
                msg: String::from("added task"),
            }));
        } else {
            return Ok(Response::new(StatusCode {
                success: false,
                msg: String::from(
                    "failed to serialise task. ensure that task is in current format.",
                ),
            }));
        }
    }
}

impl TaskService {
    pub async fn new() -> Self {
        let user = User::attach("nlcssingapore", "avagarde", "sample@email.com")
            .await
            .unwrap();
        TaskService {
            inner: Mutex::new(user),
        }
    }
}

fn construct_filter(
    filter: &Filter,
) -> Result<FFTaskFilter, Box<dyn std::error::Error + Send + Sync>> {
    Ok(FFTaskFilter {
        source: Some(Source::from_str(filter.source.as_str())?),
        status: CompletionStatus::from_str(filter.status.as_str())?,
        read: ReadStatus::from_str(filter.read.as_str())?,
        sorting: (
            SortBy::from_str(filter.sort_by.as_str())?,
            SortOrder::from_str(filter.sort_order.as_str())?,
        ),
    })
}
