// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
// extern crate serde_derive;
// extern crate serde_json;
//
// use generated_module::Welcome;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: Welcome = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    #[serde(rename = "aggregateOffsets")]
    pub aggregate_offsets: Option<AggregateOffsets>,

    #[serde(rename = "fromIndex")]
    pub from_index: Option<i64>,

    #[serde(rename = "hasValues")]
    pub has_values: Option<bool>,

    #[serde(rename = "items")]
    pub items: Option<Vec<Item>>,

    #[serde(rename = "toIndex")]
    pub to_index: Option<i64>,

    #[serde(rename = "totalCount")]
    pub total_count: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateOffsets {
    #[serde(rename = "toFfIndex")]
    pub to_ff_index: Option<i64>,

    #[serde(rename = "toGcIndex")]
    pub to_gc_index: Option<i64>,

    #[serde(rename = "toMsIndex")]
    pub to_ms_index: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "addressees")]
    pub addressees: Option<Vec<Addressee>>,

    #[serde(rename = "altLink")]
    pub alt_link: Option<String>,

    #[serde(rename = "archived")]
    pub archived: Option<bool>,

    #[serde(rename = "classes")]
    pub classes: Option<Vec<Class>>,

    #[serde(rename = "descriptionContainsQuestions")]
    pub description_contains_questions: Option<bool>,

    #[serde(rename = "dueDate")]
    pub due_date: Option<String>,

    #[serde(rename = "fileSubmissionRequired")]
    pub file_submission_required: Option<bool>,

    #[serde(rename = "hasFileSubmission")]
    pub has_file_submission: Option<bool>,

    #[serde(rename = "id")]
    pub id: Option<String>,

    #[serde(rename = "isDone")]
    pub is_done: Option<bool>,

    #[serde(rename = "isExcused")]
    pub is_excused: Option<bool>,

    #[serde(rename = "isMissingDueDate")]
    pub is_missing_due_date: Option<bool>,

    #[serde(rename = "isPersonalTask")]
    pub is_personal_task: Option<bool>,

    #[serde(rename = "isResubmissionRequired")]
    pub is_resubmission_required: Option<bool>,

    #[serde(rename = "isUnread")]
    pub is_unread: Option<bool>,

    #[serde(rename = "lastMarkedAsDoneBy")]
    pub last_marked_as_done_by: Option<serde_json::Value>,

    #[serde(rename = "mark")]
    pub mark: Option<Mark>,

    #[serde(rename = "setDate")]
    pub set_date: Option<String>,

    #[serde(rename = "setter")]
    pub setter: Option<Setter>,

    #[serde(rename = "student")]
    pub student: Option<Setter>,

    #[serde(rename = "taskSource")]
    pub task_source: Option<Source>,

    #[serde(rename = "title")]
    pub title: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Addressee {
    #[serde(rename = "guid")]
    pub guid: Option<String>,

    #[serde(rename = "isGroup")]
    pub is_group: Option<bool>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "source")]
    pub source: Option<Source>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Class {
    #[serde(rename = "classname")]
    pub classname: Option<String>,

    #[serde(rename = "id")]
    pub id: Option<String>,

    #[serde(rename = "source")]
    pub source: Option<Source>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mark {
    #[serde(rename = "grade")]
    pub grade: Option<serde_json::Value>,

    #[serde(rename = "hasFeedback")]
    pub has_feedback: Option<bool>,

    #[serde(rename = "isMarked")]
    pub is_marked: Option<bool>,

    #[serde(rename = "mark")]
    pub mark: Option<serde_json::Value>,

    #[serde(rename = "markMax")]
    pub mark_max: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setter {
    #[serde(rename = "deleted")]
    pub deleted: Option<bool>,

    #[serde(rename = "guid")]
    pub guid: Option<String>,

    #[serde(rename = "name")]
    pub name: Option<String>,

    #[serde(rename = "sortKey")]
    pub sort_key: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Source {
    #[serde(rename = "FF")]
    Ff,

    #[serde(rename = "GC")]
    Gc,
}
