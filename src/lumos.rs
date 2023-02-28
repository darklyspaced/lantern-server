use crate::serialise_res::Item;
use crate::serialise_res::Task;
use crate::task_filter::TaskFilter;
use anyhow::Result;
use quick_xml::{events::Event, reader::Reader};
use reqwest::{blocking::Client, header};
use uuid::Uuid;
// use serde_json::Value;

// HACK: used async requests instead of blocking

#[derive(Debug)]
pub struct Firefly {
    secret: String,
    school_code: String,
    device_id: String,
    app_id: String,
    address: String,
    client: Client,
    pub tasks: Vec<Item>,
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

fn check_existence(instance: Option<&mut Firefly>, school_code: &str) -> Result<Vec<String>> {
    let res;
    let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");
    let url = reqwest::Url::parse(&(portal + school_code))?;
    if let Some(lumos) = instance {
        res = lumos.client.get(url).send()?.text()?;
    } else {
        res = reqwest::blocking::get(url)?.text()?;
    }
    Ok(parse_xml(res))
}

impl<'a> Firefly {
    // declares Lumos
    pub fn new() -> Self {
        Firefly {
            school_code: String::from(""),
            app_id: String::from(""),
            device_id: Uuid::new_v4().to_string(),
            secret: String::from(""),
            address: String::from(""),
            client: Client::new(),
            tasks: vec![],
        }
    }

    // attaches to an already existing intergration
    pub fn attach(
        &mut self,
        school_code: &'a str,
        app_id: &'a str,
        secret: &'a str,
    ) -> Result<Firefly> {
        let res = check_existence(None, school_code)?;
        let address = String::from("https://") + &res[1] + "/";
        Ok(Firefly {
            school_code: school_code.to_string(),
            app_id: app_id.to_string(),
            device_id: Uuid::new_v4().to_string(),
            secret: secret.to_string(),
            address,
            client: Client::new(),
            tasks: vec![],
        })
    }

    // verifies that school exists
    pub fn verify(
        &mut self,
        school_code: &'a str,
        app_id: &'a str,
    ) -> Result<&mut Firefly, &'static str> {
        let response = check_existence(Some(self), school_code);
        if let Ok(res) = response {
            if res.len() >= 3 {
                self.school_code = school_code.to_string();
                self.app_id = app_id.to_string();
                self.address = String::from("https://") + &res[1] + "/";
            } else {
                return Err("School not found!");
            }
        } else {
            return Err("Request failed");
        }
        Ok(self)
    }

    // creates intergration
    // TODO: Only allow this to be called after new && (attach || verify) have been called
    pub fn auth(&mut self) -> Result<()> {
        let params = [
            ("ffauth_device_id", &self.device_id),
            ("ffauth_secret", &self.secret),
            ("device_id", &self.device_id),
            ("app_id", &self.app_id),
        ];
        let url = reqwest::Url::parse_with_params(
            &(self.address.to_string() + "Login/api/gettoken"),
            params,
        )?;

        let res = self
            .client
            .get(url)
            .header(
                header::COOKIE,
                header::HeaderValue::from_static("ASP.NET_SessionId=l2wkr0lecg4yz2ndqtbbou52"),
            )
            .send()?
            .text()?;

        let txt = parse_xml(res);
        if let Some(secret) = txt.first() {
            self.secret = secret.to_owned();
        }
        Ok(())
    }

    // TODO: Only allow this to be called after auth can been called
    pub fn get_tasks(&mut self, filter: TaskFilter) -> Result<()> {
        let params = [
            ("ffauth_device_id", &self.device_id),
            ("ffauth_secret", &self.secret),
        ];
        let url = reqwest::Url::parse_with_params(
            &(self.address.to_string() + "api/v2/taskListing/view/student/tasks/all/filterBy"),
            params,
        )?;

        let filters = filter.to_json();
        let res = self.client.post(url).json(&filters[0]).send()?.text()?;

        let serialised_response: Task = serde_json::from_str(&res).unwrap();
        let items = serialised_response.items.unwrap();

        if let Some(ref source) = filter.source {
            let parsed_items = items
                .into_iter()
                .filter(|item| {
                    let curr_source = item.task_source.as_ref().unwrap();
                    if source == curr_source {
                        return true;
                    }
                    false
                })
                .collect::<Vec<Item>>();

            self.tasks = parsed_items;
        } else {
            self.tasks = items;
        }
        Ok(())
    }
}

impl Default for Firefly {
    fn default() -> Self {
        Self::new()
    }
}
