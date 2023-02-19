use reqwest::Client;
use uuid::Uuid;

#[allow(dead_code)]
pub struct Lumos<'a> {
    secret: String,
    school_code: &'a str,
    device_id: Uuid,
    app_id: &'a str,
}

impl<'a> Lumos<'a> {
    // Constructs the Lumos struct
    pub fn new() -> Lumos<'a> {
        Lumos {
            school_code: "",
            app_id: "",
            device_id: Uuid::new_v4(),
            secret: String::from(""),
        }
    }

    pub async fn build(&mut self, school_code: &'a str, app_id: &'a str) {
        let client = Client::new();
        let portal = String::from("https://appgateway.fireflysolutions.co.uk/appgateway/school/");

        let res = client.get(portal + &school_code).send().await;
        if let Ok(response) = res {
            // parse received xml, set school code and app id if it exists
            println!("{}", response.text().await.unwrap());
            self.school_code = school_code;
            self.app_id = app_id;
        }
    }
}
