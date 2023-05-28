use super::{AVTask, RawFFTask, User};
use crate::models::{NewTasksPG, NewUserPG};

use diesel::prelude::*;
use dotenvy::dotenv;
use quick_xml::{events::Event, reader::Reader};
use reqwest::header;
use serde_json::json;

pub fn parse_xml(response: String) -> Vec<String> {
    let mut reader = Reader::from_str(response.as_str());
    reader.trim_text(true);
    let (mut txt, mut buf) = (Vec::new(), Vec::new());

    loop {
        match reader.read_event_into(&mut buf) {
            Err(e) => panic!("Error at position {} {:?}", reader.buffer_position(), e),
            Ok(Event::Eof) => break,
            Ok(Event::Text(e)) => txt.push(e.unescape().unwrap().into_owned()),
            _ => (),
        }
        buf.clear();
    }
    txt
}

pub async fn auth(instance: &mut User) {
    use crate::schema::users::dsl::*;
    dotenv().ok();

    let mut db_conn = instance.db_conn.clone().get().unwrap();
    let params = [
        ("ffauth_device_id", &instance.connection.device_id),
        ("ffauth_secret", &String::from("")),
        ("device_id", &instance.connection.device_id),
        ("app_id", &instance.connection.app_id),
    ];
    let url = reqwest::Url::parse_with_params(
        &(instance.connection.http_endpoint.to_string() + "Login/api/gettoken"),
        params,
    )
    .unwrap();

    let res = instance
        .http_client
        .get(url)
        .header(
            header::COOKIE,
            header::HeaderValue::from_static("ASP.NET_SessionId=2fkcirvf3envmu40d14fpc12"),
            // HACK: need to make this not hard-coded
        )
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    let txt = parse_xml(res);
    if let Some(secret) = txt.first() {
        if secret != "Invalid token" {
            instance.connection.secret = secret.to_string();
            diesel::update(users)
                .filter(email.eq(&instance.connection.email))
                .set(firefly_secret.eq(secret))
                .execute(&mut db_conn)
                .unwrap();
        } else {
            panic!("invalid sessionid!")
        }
    };
}

pub fn add_user_to_db(instance: &mut User, new_email: &str) {
    use crate::schema::tasks;
    use crate::schema::users;

    let mut db_conn = instance.db_conn.clone().get().unwrap();
    let new_user = NewUserPG {
        email: new_email,
        firefly_secret: &instance.connection.secret,
        device_id: &instance.connection.device_id,
    };
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut db_conn)
        .expect("error creating new user");

    let new_task_user_relation = NewTasksPG {
        user_email: new_email,
        firefly_tasks: json!({"empty": true}),
        local_tasks: json!({"empty": true}),
    };
    diesel::insert_into(tasks::table)
        .values(&new_task_user_relation)
        .execute(&mut db_conn)
        .expect("error create task-user relation");
}

pub fn update_tasks_db(instance: &mut User, conn: &mut PgConnection) {
    use crate::schema::tasks::dsl::*;

    diesel::update(tasks)
        .filter(user_email.eq(&instance.connection.email))
        .set(firefly_tasks.eq::<serde_json::Value>(serde_json::to_value(&instance.tasks).unwrap()))
        .execute(conn)
        .unwrap();
}

/// Converts the serialised response [`RawTask`], that is received from Firefly, into [`Task`]. A
/// more condensed, and relevant format.
///
/// This ensures parity, in format, between tasks that were pulled from Firefly and those that were
/// created by the user. Allows the frontend to have just one parser for tasks.
pub fn rawtask_to_task(tasks: Vec<RawFFTask>) -> Option<Vec<AVTask>> {
    let mut standard_tasks = vec![];
    for task in tasks {
        standard_tasks.push({
            AVTask {
                due_date: task.due_date?,
                is_done: task.is_done?,
                set_date: task.set_date?,
                title: task.title?,
                id: {
                    if let Some(id) = task.id {
                        id.parse::<usize>().unwrap()
                    } else {
                        0
                    }
                },
            }
        })
    }
    Some(standard_tasks)
}
