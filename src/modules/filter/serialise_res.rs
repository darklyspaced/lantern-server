// Example code that deserializes and serializes the model.
// extern crate serde;
// #[macro_use]
extern crate serde_derive;
extern crate serde_json;
//
// use generated_module::Welcome;
//
// fn main() {
//     let json = r#"{"answer": 42}"#;
//     let model: Welcome = serde_json::from_str(&json).unwrap();
// }

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Welcome {
    #[serde(rename = "aggregateOffsets")]
    pub aggregate_offsets: AggregateOffsets,

    #[serde(rename = "fromIndex")]
    pub from_index: i64,

    #[serde(rename = "hasValues")]
    pub has_values: bool,

    #[serde(rename = "items")]
    pub items: Vec<Item>,

    #[serde(rename = "toIndex")]
    pub to_index: i64,

    #[serde(rename = "totalCount")]
    pub total_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AggregateOffsets {
    #[serde(rename = "toFfIndex")]
    pub to_ff_index: i64,

    #[serde(rename = "toGcIndex")]
    pub to_gc_index: i64,

    #[serde(rename = "toMsIndex")]
    pub to_ms_index: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "addressees")]
    pub addressees: Vec<Addressee>,

    #[serde(rename = "altLink")]
    pub alt_link: String,

    #[serde(rename = "archived")]
    pub archived: bool,

    #[serde(rename = "classes")]
    pub classes: Vec<Class>,

    #[serde(rename = "descriptionContainsQuestions")]
    pub description_contains_questions: bool,

    #[serde(rename = "dueDate")]
    pub due_date: Option<serde_json::Value>,

    #[serde(rename = "fileSubmissionRequired")]
    pub file_submission_required: bool,

    #[serde(rename = "hasFileSubmission")]
    pub has_file_submission: bool,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "isDone")]
    pub is_done: bool,

    #[serde(rename = "isExcused")]
    pub is_excused: bool,

    #[serde(rename = "isMissingDueDate")]
    pub is_missing_due_date: bool,

    #[serde(rename = "isPersonalTask")]
    pub is_personal_task: bool,

    #[serde(rename = "isResubmissionRequired")]
    pub is_resubmission_required: bool,

    #[serde(rename = "isUnread")]
    pub is_unread: bool,

    #[serde(rename = "lastMarkedAsDoneBy")]
    pub last_marked_as_done_by: Option<serde_json::Value>,

    #[serde(rename = "mark")]
    pub mark: Mark,

    #[serde(rename = "setDate")]
    pub set_date: String,

    #[serde(rename = "setter")]
    pub setter: Setter,

    #[serde(rename = "student")]
    pub student: Setter,

    #[serde(rename = "taskSource")]
    pub task_source: String,

    #[serde(rename = "title")]
    pub title: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Addressee {
    #[serde(rename = "guid")]
    pub guid: String,

    #[serde(rename = "isGroup")]
    pub is_group: bool,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "source")]
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Class {
    #[serde(rename = "classname")]
    pub classname: String,

    #[serde(rename = "id")]
    pub id: String,

    #[serde(rename = "source")]
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Mark {
    #[serde(rename = "grade")]
    pub grade: Option<serde_json::Value>,

    #[serde(rename = "hasFeedback")]
    pub has_feedback: bool,

    #[serde(rename = "isMarked")]
    pub is_marked: bool,

    #[serde(rename = "mark")]
    pub mark: Option<serde_json::Value>,

    #[serde(rename = "markMax")]
    pub mark_max: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Setter {
    #[serde(rename = "deleted")]
    pub deleted: bool,

    #[serde(rename = "guid")]
    pub guid: String,

    #[serde(rename = "name")]
    pub name: String,

    #[serde(rename = "sortKey")]
    pub sort_key: String,
}
