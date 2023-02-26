use super::serialise_res::Source;
use serde::Deserialize;
use std::cmp::min;

#[allow(dead_code)]
pub enum CompletionStatus {
    Todo,
    DoneOrArchived,
    All,
}

#[allow(dead_code)]
pub enum ReadStatus {
    All,
    OnlyRead,
    OnlyUnread,
}

#[allow(dead_code)]
pub enum Order {
    Ascending,
    Descending,
}

#[allow(dead_code)]
pub struct TaskFilter {
    pub status: CompletionStatus,
    pub read: ReadStatus,
    pub sorting: (String, Order), // String = DueDate or SetDate; bool is True or False
    pub results: u32,             // no. of tasks to retrieve
    pub source: Option<Source>,   // Google Classroom or Firefly or Both
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
    #[allow(dead_code)]
    pub fn to_json(&self) -> Vec<JSONTaskFilter> {
        let mut filters: Vec<JSONTaskFilter> = vec![];
        let pages: u32 = (self.results - 1) / 50;
        for page in 0..pages + 1 {
            let pre_json = JSONTaskFilter {
                ownerType: String::from("OnlySetters"),
                page,
                pageSize: min(self.results - 50 * page, 50),
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
                    column: self.sorting.0.to_owned(),
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
