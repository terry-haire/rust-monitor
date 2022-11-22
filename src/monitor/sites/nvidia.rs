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

pub struct Nvidia {
    pub url: String,
    pub json: String,
}

impl Nvidia {
    pub fn new(_client: &reqwest::blocking::Client, site_info: &Value) -> Self {
        Nvidia {
            url: site_info["url"].as_str().unwrap().to_string(),
            json: site_info["json"].as_str().unwrap().to_string(),
        }
    }
}

impl MonitorProcessing for Nvidia {
    fn process(&mut self, client: &reqwest::blocking::Client) ->
            Result<HashSet<Url>, Box<dyn std::error::Error>> {
        let v: Value = client
            .get(&self.json)
            .headers(get_headers())
            .send()?
            .json()?;

        println!("Done!");

        let price_data = v["products"]["product"][0]["inventoryStatus"][
            "status"].as_str();

        let inventory_status = match price_data {
            Some(v) => v,
            None => {
                println!("Price not found!");

                return Ok(HashSet::new())
            }
        };

        let mut result = HashSet::new();

        if inventory_status == "PRODUCT_INVENTORY_OUT_OF_STOCK" {
            println!("OOS");

            return Ok(result)
        }

        result.insert(Url::parse(&self.url)?);

        return Ok(result)
    }
}
