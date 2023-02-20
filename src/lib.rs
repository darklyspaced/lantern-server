use quick_xml::events::Event;
use quick_xml::reader::Reader;
use reqwest::blocking::Client;
use reqwest::header;
use uuid::Uuid;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Lumos<'a> {
    secret: String,
    school_code: &'a str,
    device_id: Uuid,
    app_id: &'a str,
    address: String,
}

impl<'a> Lumos<'a> {
    // declares Lumos
    pub fn new() -> Lumos<'a> {
        Lumos {
            school_code: "",
            app_id: "",
            device_id: Uuid::new_v4(),
            secret: String::from(""),
            address: String::from(""),
        }
    }

    // initialises the struct
    pub fn attach(&mut self, school_code: &'a str, app_id: &'a str) {
        // NOTE: Split into attach & auth?

        let mut headers = header::HeaderMap::new();
        headers.insert(
            "ASP.NET_SessionId",
            header::HeaderValue::from_static("oprumwu0migtu2hpsb5jslyl"), // change to be dynamic
        );

        let client = Client::builder().cookie_store(true).build().unwrap();
        let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");
        let res = client.get(portal + &school_code).send().unwrap().text();

        if let Ok(response) = res {
            let mut reader = Reader::from_str(response.as_str());
            reader.trim_text(true);

            let mut txt = Vec::new();
            let mut buf = Vec::new();

            loop {
                match reader.read_event_into(&mut buf) {
                    Err(e) => panic!("Error at position {}: {:?}", reader.buffer_position(), e),

                    Ok(Event::Eof) => break,

                    Ok(Event::Text(e)) => txt.push(e.unescape().unwrap().into_owned()),

                    _ => (),
                }
                buf.clear();
            }

            if txt.len() >= 3 {
                self.school_code = school_code;
                self.app_id = app_id;
                self.address = txt[1].to_owned();
            } else {
                panic!("School not found");
            }
        } else {
            panic!("Request failed");
        }

        let params = [
            ("ffauth_device_id", self.device_id.to_string()),
            ("ffauth_secret", (&self.secret).to_owned()),
            ("device_id", self.device_id.to_string()),
            ("app_id", self.app_id.to_string()),
        ];
        let _url = reqwest::Url::parse_with_params(
            &((&self.address).to_owned() + "Login/api/gettoken"),
            params,
        );
    }
}
