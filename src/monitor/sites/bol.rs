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

pub struct Bol {
    pub url: String,
    pub json: String,
}

impl Bol {
    pub fn new(_client: &reqwest::blocking::Client, site_info: &Value) -> Self {
        Bol {
            url: site_info["url"].as_str().unwrap().to_string(),
            json: site_info["json"].as_str().unwrap().to_string(),
        }
    }
}

impl MonitorProcessing for Bol {
    fn process(&mut self, client: &reqwest::blocking::Client) ->
            Result<HashSet<Url>, Box<dyn std::error::Error>> {
        let v: Value = client
            .get(&self.json)
            .headers(get_headers())
            .send()?
            .json()?;

        println!("Done!");

        let price_data = v["analytics"]["price"].as_f64();

        let price = match price_data {
            Some(v) => v,
            None => {
                println!("Price not found!");

                return Ok(HashSet::new())
            }
        };

        let mut result = HashSet::new();

        if price == 0.0_f64 {
            println!("Price 0");

            return Ok(result)
        }

        result.insert(Url::parse(&self.url)?);

        return Ok(result)
    }
}
