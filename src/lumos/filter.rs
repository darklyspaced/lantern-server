use crate::lumos::task::Response;
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

#[derive(serde::Serialize, Deserialize, Debug)]
struct Sorting {
    column: String,
    order: String,
}

#[derive(serde::Serialize, Deserialize, Debug)]
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
    ///
    /// The API allows you to list a maximum of 100 tasks per request so requesting more tasks than
    /// that would require splitting it up into multiple requests and appending to `filters`
    pub async fn to_json(
        &self,
        client: reqwest::Client,
        url: reqwest::Url,
    ) -> (Option<Vec<JSONFFTaskFilter>>, Option<Response>) {
        let pre_filter = JSONFFTaskFilter {
            ownerType: String::from("OnlySetters"),
            page: 0,
            pageSize: 100, // max 100 per request (API limitation)
            archiveStatus: String::from("All"),
            completionStatus: self.status.to_string(),
            readStatus: self.read.to_string(),
            markingStatus: String::from("All"),
            sortingCriteria: vec![Sorting {
                column: self.sorting.0.to_string(),
                order: self.sorting.1.to_string(),
            }],
        };
        let res = client
            .post(url.clone())
            .json(&pre_filter)
            .send()
            .await
            .unwrap()
            .text()
            .await
            .unwrap();
        let ser_res = serde_json::from_str::<Response>(&res).unwrap();

        if let Some(count) = ser_res.total_count {
            let count = count as u32;
            println!("count: {count}");

            if count > 100 {
                let mut filters: Vec<JSONFFTaskFilter> = vec![];
                let pages = (count - 1) / 100; // 0 is valid, so no need to ceil (303 / 100 = 9 [0, 1, 2, 3])

                for page in 1..=pages {
                    println!("{}", min(count - 100 * page, 100));
                    let pre_json = JSONFFTaskFilter {
                        ownerType: String::from("OnlySetters"),
                        page,
                        pageSize: min(count - 100 * page, 100), // max 100 per request (API limitation)
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
                (Some(filters), Some(ser_res))
            } else {
                (None, Some(ser_res))
            }
        } else {
            eprintln!("total count is not present: probably bad response");
            (None, None)
        }
    }
}
