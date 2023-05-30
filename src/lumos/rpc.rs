pub mod light {
    tonic::include_proto!("light");
}
use super::task::AVTask;
use super::user::User;
use crate::models::TasksPG;
use crate::prelude::*;

use anyhow::Result;
use diesel::prelude::*;
use light::lantern_server::Lantern;
pub use light::lantern_server::LanternServer;
use light::{Filter, PTasks, StatusCode};
use std::str::FromStr;
use tokio::sync::Mutex;
use tonic::{Code, Request, Response, Status};

pub struct TaskService {
    inner: Mutex<User>,
}

#[tonic::async_trait]
impl Lantern for TaskService {
    async fn get_tasks(&self, request: Request<Filter>) -> Result<Response<PTasks>, Status> {
        use crate::schema::tasks::dsl::*;

        let filter = construct_filter(request.get_ref());
        let mut user = self.inner.lock().await;
        let mut db_conn = user.db_conn.get().unwrap();
        let mut all_tasks = vec![];

        let loc_tasks = &tasks
            .filter(user_email.eq(user.connection.email.clone()))
            .load::<TasksPG>(&mut db_conn)
            .expect("failed to get local tasks")[0];
        let loc_tasks = serde_json::from_value::<Vec<AVTask>>(loc_tasks.local_tasks.clone());

        if let Ok(l_tasks) = loc_tasks {
            all_tasks.extend(l_tasks);
        } else {
            return Err(Status::new(Code::Unknown, "failed to retrieve local tasks"));
        }
        if let Ok(filter) = filter {
            user.get_ff_tasks(filter).await.unwrap();
        } else {
            return Err(Status::new(Code::InvalidArgument, "filter is malformed"));
        }

        all_tasks.extend(user.tasks.clone());
        Ok(Response::new(PTasks {
            body: serde_json::to_string(&all_tasks).unwrap(),
        }))
    }

    async fn add_tasks(&self, request: Request<PTasks>) -> Result<Response<StatusCode>, Status> {
        use crate::schema::tasks::dsl::*;

        let user = self.inner.lock().await;
        let mut db_conn = user.db_conn.get().unwrap();
        let loc_tasks = &tasks
            .filter(user_email.eq(user.connection.email.clone()))
            .load::<TasksPG>(&mut db_conn)
            .expect("failed to get local tasks")[0];
        let loc_tasks =
            serde_json::from_value::<Vec<AVTask>>(loc_tasks.local_tasks.clone()).unwrap();
        let mut all_tasks = serde_json::from_str::<Vec<AVTask>>(&request.get_ref().body).unwrap();

        all_tasks.extend(loc_tasks);

        diesel::update(tasks)
            .filter(user_email.eq(&user.connection.email))
            .set(local_tasks.eq(serde_json::to_value(all_tasks).unwrap()))
            .execute(&mut (user.db_conn.clone().get().unwrap()))
            .unwrap();

        Ok(Response::new(StatusCode { success: true }))
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
