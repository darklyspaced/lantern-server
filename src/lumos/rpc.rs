pub mod light {
    tonic::include_proto!("light");
}
use super::task::Task;
use crate::prelude::*;

use anyhow::Result;
use light::lantern_server::Lantern;
pub use light::lantern_server::LanternServer;
use light::{Filter, StatusCode, Tasks};
use std::str::FromStr;
use tonic::{Request, Response, Status};
// use diesel::prelude::*;

#[derive(Default)]
pub struct TaskService; // TODO: allow TaskService to have access to a threadpool for db connection

#[tonic::async_trait]
impl Lantern for TaskService {
    async fn get_tasks(&self, request: Request<Filter>) -> Result<Response<Tasks>, Status> {
        let filter = construct_filter(request.get_ref());
        let mut user = User::attach("nlcssingapore", "whatever", "sample@email.com")
            .await
            .unwrap();

        if let Ok(filter) = filter {
            user.get_tasks(filter).await.unwrap();
        } else {
            panic!("failed to create filter from input");
        }

        Ok(Response::new(Tasks {
            body: serde_json::to_string(&user.tasks).unwrap(),
        }))
    }

    async fn add_tasks(&self, request: Request<Tasks>) -> Result<Response<StatusCode>, Status> {
        // use crate::schema::tasks::dsl::*;

        // let emails = tasks
        //     .filter(user_email.eq("test"))
        //     .load::<crate::models::Tasks>(&mut user.daemon.db)
        //     .expect("Failed to get emails.");

        println!("{}", request.get_ref().body);
        let loc_tasks = serde_json::from_str::<Task>(request.get_ref().body.as_ref());
        if let Ok(_) = loc_tasks {
            return Ok(Response::new(StatusCode {
                success: true,
                msg: String::from("added task"),
            }));
        } else {
            return Ok(Response::new(StatusCode {
                success: false,
                msg: String::from(
                    "failed to serialise task. ensure that task is in corrent format.",
                ),
            }));
        }
    }
}

fn construct_filter(
    filter: &Filter,
) -> Result<TaskFilter, Box<dyn std::error::Error + Send + Sync>> {
    Ok(TaskFilter {
        source: Some(Source::from_str(filter.source.as_str())?),
        status: CompletionStatus::from_str(filter.status.as_str())?,
        read: ReadStatus::from_str(filter.read.as_str())?,
        sorting: (
            SortBy::from_str(filter.sort_by.as_str())?,
            SortOrder::from_str(filter.sort_order.as_str())?,
        ),
    })
}
