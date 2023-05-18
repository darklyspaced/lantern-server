use serde::{Deserialize, Serialize};
use std::cmp::min;
use std::string::ToString;
use strum::EnumString;
use strum_macros::Display;

#[derive(Debug, PartialEq, EnumString, Display)]
pub enum CompletionStatus {
    Todo,
    DoneOrArchived,
    AllIncludingArchived,
}

#[derive(Debug, PartialEq, EnumString, Display)]
pub enum ReadStatus {
    All,
    OnlyRead,
    OnlyUnread,
}

#[derive(Debug, PartialEq, EnumString, Display)]
pub enum SortOrder {
    Ascending,
    Descending,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq, EnumString, Display)]
pub enum Source {
    #[serde(rename = "FF")]
    #[strum(serialize = "FF")]
    Ff,

    #[serde(rename = "GC")]
    #[strum(serialize = "GC")]
    Gc,
}

#[derive(Debug, PartialEq, EnumString, Display)]
pub enum SortBy {
    DueDate,
    SetDate,
}

#[allow(dead_code)]
pub struct FFTaskFilter {
    pub status: CompletionStatus,
    pub read: ReadStatus,
    pub sorting: (SortBy, SortOrder), // String = DueDate or SetDate; bool is True or False
    pub source: Option<Source>,       // Google Classroom or Firefly; sometimes not present -_-
}

#[derive(serde::Serialize, Deserialize)]
struct Sorting {
    column: String,
    order: String,
}

#[derive(serde::Serialize, Deserialize)]
#[allow(non_snake_case)]
pub struct JSONFFTaskFilter {
    ownerType: String,
    page: u32,
    pageSize: u32,
    archiveStatus: String,
    completionStatus: String,
    readStatus: String,
    markingStatus: String,
    sortingCriteria: Vec<Sorting>,
}

impl FFTaskFilter {
    /// Converts the more ergonomic [`FFTaskFilter`] to a `Vec<JSONTaskFilter>` a vector of filters (one
    /// for each page) that can then be serialised into a JSON for each request.
    pub fn to_json(&self) -> Vec<JSONFFTaskFilter> {
        let mut filters: Vec<JSONFFTaskFilter> = vec![];
        let results = 1000;
        let pages: u32 = (results - 1) / 50;
        for page in 0..pages + 1 {
            let pre_json = JSONFFTaskFilter {
                ownerType: String::from("OnlySetters"),
                page,
                pageSize: min(results - 50 * page, 50),
                archiveStatus: String::from("All"),
                completionStatus: self.status.to_string(),
                readStatus: self.read.to_string(),
                markingStatus: String::from("All"),
                sortingCriteria: vec![Sorting {
                    column: self.sorting.0.to_string(),
                    order: self.sorting.1.to_string(),
                }],
            };

            filters.push(pre_json);
        }
        filters
    }
}
