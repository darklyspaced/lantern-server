use crate::task_filter::task_filter::{CompletionStatus, Order, ReadStatus, TaskFilter};
use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::blocking::Client;
use reqwest::header;
use serde_json::Value;
use std::error::Error;
use uuid::Uuid;

mod task_filter;

#[derive(Debug)]
pub struct Lumos {
    secret: String,
    school_code: String,
    device_id: String,
    app_id: String,
    address: String,
}

fn parse_xml(response: String) -> Vec<String> {
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

impl<'a> Lumos {
    // declares Lumos
    pub fn new() -> Lumos {
        Lumos {
            school_code: String::from(""),
            app_id: String::from(""),
            device_id: Uuid::new_v4().to_string(),
            secret: String::from(""),
            address: String::from(""),
        }
    }

    // initialises the struct
    pub fn attach(
        &mut self,
        school_code: &'a str,
        app_id: &'a str,
    ) -> Result<&Lumos, Box<dyn Error>> {
        let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");
        let res = reqwest::blocking::get(portal + school_code)?.text();

        if let Ok(response) = res {
            let txt = parse_xml(response);
            if txt.len() >= 3 {
                self.school_code = school_code.to_string();
                self.app_id = app_id.to_string();
                self.address = String::from("https://") + &txt[1] + &"/";
            } else {
                return Err("School not found!".into());
            }
        } else {
            return Err("Request failed".into());
        }

        Ok(self)
    }

    pub fn auth(&mut self) {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::COOKIE,
            header::HeaderValue::from_static("ASP.NET_SessionId=oprumwu0migtu2hpsb5jslyl"), // NOTE: change to be dynamic
        );

        let client = Client::builder().default_headers(headers).build().unwrap();

        let params = [
            ("ffauth_device_id", &self.device_id),
            ("ffauth_secret", &self.secret),
            ("device_id", &self.device_id),
            ("app_id", &self.app_id),
        ];
        let url = reqwest::Url::parse_with_params(
            &((&self.address).to_string() + "Login/api/gettoken"),
            params,
        )
        .unwrap();

        let res = client.get(url).send().unwrap().text().unwrap();
        let txt = parse_xml(res);
        self.secret = txt.first().unwrap().to_owned();
    }

    pub fn get_tasks(&self) {
        let filter = TaskFilter {
            status: CompletionStatus::Todo,
            read: ReadStatus::All,
            sorting: (String::from("DueDate"), Order::Ascending),
            results: 100,
        };

        let params = [
            ("ffauth_device_id", &self.device_id),
            ("ffauth_secret", &self.secret),
        ];
        let url = reqwest::Url::parse_with_params(
            &((&self.address).to_string() + "api/v2/taskListing/view/student/tasks/all/filterBy"),
            params,
        )
        .unwrap();

        let filters = filter.to_json();
        let client = Client::new();
        let res = client
            .post(url)
            .json(&filters[0])
            .send()
            .unwrap()
            .text()
            .unwrap();

        let object: Value = serde_json::from_str(&res).unwrap();
        let object_json = serde_json::to_string_pretty(&object).unwrap();

        println!("{}", object_json);
    }
}
