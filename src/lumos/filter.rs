use color_eyre::Result;
use serde::{Deserialize, Serialize};
use std::string::ToString;
use strum::EnumString;
use strum_macros::Display;

use super::error::FireflyError;
use super::task::Response;

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
    /// Converts the more ergonomic [`FFTaskFilter`] to a `Vec<JSONTaskFilter>` a vector of filters
    ///
    /// The Firefly API allows you to get a maximum of 100 tasks per request; a vector of filters
    /// must be created when the number of tasks being requested exceeds 100.
    pub async fn to_json(
        &self,
        client: reqwest::Client,
        url: reqwest::Url,
    ) -> Result<(Option<Vec<JSONFFTaskFilter>>, Option<Response>), FireflyError> {
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
            .await?
            .text()
            .await?;

        if res == "Invalid token" {
            return Err(FireflyError::InvalidSecret.into());
        }

        let ser_res = serde_json::from_str::<Response>(&res);

        let ser_res = if let Ok(res) = ser_res {
            res
        } else {
            return Err(FireflyError::Misc(format!(
                "malformed response, failed to parse: {}",
                res
            )));
        };

        if let Some(total_tasks) = ser_res.total_count {
            let total_tasks = total_tasks as u32;

            if total_tasks > 100 {
                let mut filters: Vec<JSONFFTaskFilter> = vec![];
                let pages = (total_tasks - 1) / 100; // 0 is valid, so no need to ceil (303 / 100 = 3 [0, 1, 2, 3])

                for page in 1..=pages {
                    let pre_json = JSONFFTaskFilter {
                        ownerType: String::from("OnlySetters"),
                        page,
                        pageSize: 100,
                        // pageSize: min(count - 100 * page, 100), (breaks cause god knows why)
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
                Ok((Some(filters), Some(ser_res)))
            } else {
                Ok((None, Some(ser_res)))
            }
        } else {
            Err(FireflyError::Misc(format!(
                "malformed response, total_count not present: {}",
                res
            )))
        }
    }
}
