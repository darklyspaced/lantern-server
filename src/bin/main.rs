pub mod light {
    tonic::include_proto!("light");
}
use lantern::prelude::*;

use anyhow::Result;
use light::lantern_server::{Lantern, LanternServer};
use light::{Filter, StatusCode, Tasks};
use tonic::transport::Server;
use tonic::{Request, Response, Status};

struct TaskService;

#[tonic::async_trait]
impl Lantern for TaskService {
    async fn get_tasks(&self, request: Request<Filter>) -> Result<Response<Tasks>, Status> {
        let filter = construct_filter(request.get_ref());
        let mut user = User::attach("nlcssingapore", "whatever", "sample@email.com")
            .await
            .unwrap();
        user.get_tasks(filter).await;

        let tasks = user.tasks;

        let t = serde_json::to_string(&tasks).unwrap();
        Ok(Response::new(Tasks {
            body: String::from("Hi"),
        }))
    }

    async fn add_tasks(&self, request: Request<Tasks>) -> Result<Response<StatusCode>, Status> {
        let tasks = request.get_ref();
        Ok(Response::new(StatusCode {
            success: true,
            msg: String::from("101"),
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let addr = "[::1]:8000".parse().unwrap();

    let task_service = TaskService;
    let svc = LanternServer::new(TaskService);

    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}

fn construct_filter(filter: &Filter) -> TaskFilter {
    TaskFilter {
        // HACK: potentially replace with macro that I write
        // HACK: remove the panics
        source: match filter.source.as_str() {
            "FF" => Some(Source::Ff),
            "GC" => Some(Source::Gc),
            _ => None,
        },
        status: match filter.status.as_str() {
            "Todo" => CompletionStatus::Todo,
            "DoneOrArchived" => CompletionStatus::DoneOrArchived,
            "All" => CompletionStatus::All,
            _ => panic!("Invalid status"),
        },
        read: match filter.read.as_str() {
            "All" => ReadStatus::All,
            "OnlyRead" => ReadStatus::OnlyRead,
            "OnlyUnread" => ReadStatus::OnlyUnread,
            _ => panic!("Invalid read status"),
        },
        sorting: (
            match filter.sort_by.as_str() {
                "SetDate" => SortBy::SetDate,
                "DueDate" => SortBy::DueDate,
                _ => panic!("Invalid sort_by"),
            },
            match filter.sort_order.as_str() {
                "Ascending" => Order::Ascending,
                "Descending" => Order::Descending,
                _ => panic!("Invalid order"),
            },
        ),
    }
}
