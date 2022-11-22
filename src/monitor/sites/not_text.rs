use crate::monitor::sites::processing::MonitorProcessing as MonitorProcessing;

use std::collections::HashSet;

use url::{Url};
use serde_json::{Value};

fn get_headers() -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();

    headers.insert(reqwest::header::USER_AGENT,
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/79.0.3945.130 Safari/537.36"
            .parse()
            .unwrap()
    );

    headers
}

pub struct NotText {
    pub text: String,
    pub url: String,
}

impl NotText {
    // TODO: Get product_id from website.
    pub fn new(_client: &reqwest::blocking::Client, site_info: &Value) -> Self {
        NotText {
            text: site_info["text"].as_str().unwrap().to_string(),
            url: site_info["url"].as_str().unwrap().to_string(),
        }
    }
}

impl MonitorProcessing for NotText {
    fn process(&mut self, client: &reqwest::blocking::Client) ->
            Result<HashSet<Url>, Box<dyn std::error::Error>> {
        let res = client
            .get(&self.url)
            .headers(get_headers())
            .send()?
            .text()?;

        println!("Done! (Not Text)");

        println!("body = {:?}", &res);

        let mut result = HashSet::new();

        match res.find("Error 1020") {
            Some(_) => {
            },
            None => {
                result.insert(Url::parse(&self.url)?);
            }
        }

        return Ok(result)
    }
}
