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

pub struct ShopifyVariant {
    pub url: String,
    pub handle: String,
    pub prepend: String,
}

impl ShopifyVariant {
    pub fn new(_client: &reqwest::blocking::Client, site_info: &Value) -> Self {
        ShopifyVariant {
            url: site_info["url"].as_str().unwrap().to_string(),
            handle: site_info["handle"].as_str().unwrap().to_string(),
            prepend: site_info["prepend"].as_str().unwrap().to_string(),
        }
    }

    fn find_target_product(&self, products: std::vec::Vec<serde_json::Value>)
            -> Option<Value> {
        for product in products {
            if product["handle"].as_str().unwrap() == self.handle {
                return Some(product);
            }
        }

        None
    }
}

impl MonitorProcessing for ShopifyVariant {
    fn process(&mut self, client: &reqwest::blocking::Client) ->
            Result<HashSet<Url>, Box<dyn std::error::Error>> {
        let v: Value = client
            .get(&self.url)
            .headers(get_headers()).send()?
            .json()?;

        println!("Done!");

        let products = v["products"].as_array();

        let prod = match products {
            Some(v) => self.find_target_product(v.to_vec()),
            None => {
                println!("No products found!");

                return Ok(HashSet::new())
            }
        };

        let prod2 = match prod {
            Some(v) => v,
            None => {
                println!("Product not found!");

                return Ok(HashSet::new())
            }
        };

        let variants = match prod2["variants"].as_array() {
            Some(x) => {
                x
            },
            None => {
                println!("No variants found!");

                return Ok(HashSet::new())
            }
        };

        let links: std::collections::HashSet<Url> = variants
            .iter()
            .filter_map(|x| {
                let s = match x["id"].as_str() {
                    Some(v) => format!("{}{}:1", self.prepend, v),
                    None => {
                        println!("Variant id not found!");

                        format!("Not Found")
                    }
                };

                Url::parse(&s).ok()
            })
            .collect();

        return Ok(links)
    }
}
