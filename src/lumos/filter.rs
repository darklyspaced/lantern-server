use serde::{Deserialize, Serialize};
use std::cmp::min;

pub enum CompletionStatus {
    Todo,
    DoneOrArchived,
    All,
}

pub enum ReadStatus {
    All,
    OnlyRead,
    OnlyUnread,
}

pub enum Order {
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum Source {
    #[serde(rename = "FF")]
    Ff,

    #[serde(rename = "GC")]
    Gc,
}

pub enum SortBy {
    DueDate,
    SetDate,
}

#[allow(dead_code)]
pub struct TaskFilter {
    pub status: CompletionStatus,
    pub read: ReadStatus,
    pub sorting: (SortBy, Order), // String = DueDate or SetDate; bool is True or False
    pub source: Option<Source>,   // Google Classroom or Firefly; sometimes not present -_-
}

#[derive(serde::Serialize, Deserialize)]
struct Sorting {
    column: String,
    order: String,
}

#[derive(serde::Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct JSONTaskFilter {
    ownerType: String,
    page: u32,
    pageSize: u32,
    archiveStatus: String,
    completionStatus: String,
    readStatus: String,
    markingStatus: String,
    sortingCriteria: Vec<Sorting>,
}

impl TaskFilter {
    /// Converts the more ergonomic [`TaskFilter`]to a `Vec<JSONTaskFilter>` a vector of filters (one
    /// for each page) that can then be serialised into a JSON for each request.
    pub fn to_json(&self) -> Vec<JSONTaskFilter> {
        let mut filters: Vec<JSONTaskFilter> = vec![];
        let results = 1000;
        let pages: u32 = (results - 1) / 50;
        for page in 0..pages + 1 {
            let pre_json = JSONTaskFilter {
                ownerType: String::from("OnlySetters"),
                page,
                pageSize: min(results - 50 * page, 50),
                archiveStatus: String::from("All"),
                completionStatus: match self.status {
                    CompletionStatus::Todo => String::from("Todo"),
                    CompletionStatus::All => String::from("AllIncludingArchived"),
                    CompletionStatus::DoneOrArchived => String::from("DoneOrArchived"),
                },
                readStatus: match self.read {
                    ReadStatus::All => String::from("All"),
                    ReadStatus::OnlyRead => String::from("OnlyRead"),
                    ReadStatus::OnlyUnread => String::from("OnlyUnread"),
                },
                markingStatus: String::from("All"),
                sortingCriteria: vec![Sorting {
                    column: match self.sorting.0 {
                        SortBy::DueDate => String::from("DueDate"),
                        SortBy::SetDate => String::from("SetDate"),
                    },
                    order: match self.sorting.1 {
                        Order::Ascending => String::from("Ascending"),
                        Order::Descending => String::from("Descending"),
                    },
                }],
            };

            filters.push(pre_json);
        }
        filters
    }
}
