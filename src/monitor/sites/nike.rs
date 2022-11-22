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

fn get_in_stock_id(s: &Value) -> Option<String> {
    if s["level"].as_str().unwrap() == "OOS" {
        return None;
    }

    Some(s["id"].as_str().unwrap().to_string())
}

pub struct NikeProduct {
    pub product_id: String,
    pub product_handle: String,
}

impl NikeProduct {
    // TODO: Get product_id from website.
    pub fn new(_client: &reqwest::blocking::Client, site_info: &Value) -> Self {
        NikeProduct {
            product_id: site_info["product_id"].as_str().unwrap().to_string(),
            product_handle: site_info["handle"].as_str().unwrap().to_string(),
        }
    }
}

impl MonitorProcessing for NikeProduct {
    // To monitor sneakrs, links need to be in the following format where the
    // product id indicates the shoe:
    // https://api.nike.com/deliver/available_skus/v1/?filter=productIds(8f40f86c-b3db-5a6b-8bfd-8e056fad6b6e)
    //
    // This url takes you straight to payment.
    // https://www.nike.com/nl/en/launch/t/dunk-low-plum/?productId=8f40f86c-b3db-5a6b-8bfd-8e056fad6b6e&size=8.5
    //                       https://api.nike.com/merch/products/v2/8f40f86c-b3db-5a6b-8bfd-8e056fad6b6e
    //                                                              vc6fea0e6-06bc-4042-a4c2-5a4b007059b0
    //
    // To check if it is active:
    // https://api.nike.com/merch/products/v2/dbfe409d-262b-5144-845f-50895007d959
    // CODE 200 JSON response means active.
    // CODE 404 means inactive.
    //
    // So the process is:
    // 1. Check active:
    //    https://api.nike.com/merch/products/v2/[Product-id]
    //    CODE 200 JSON response means active.
    //    CODE 404 means inactive.
    // 2. Check stock:
    //    https://api.nike.com/deliver/available_skus/v1/?filter=productIds([Product-id])
    // 3. Get size US (nikeSize):
    //    https://api.nike.com/merch/skus/v2/?filter=productId([product-id])&filter=country(NL)
    // 4. Buy:
    //    https://www.nike.com/nl/en/launch/t/[product-handle]/?productId=[Product-id]&size=[size]
    //
    // Multiples:
    // https://api.nike.com/deliver/available_skus/v1/?filter=productIds([Product-id1],[Product-id2])
    //
    // Get product id:
    // xpath:
    // //head/meta[@name='branch:deeplink:productId']
    // And then the element's "content" attribute.
    //
    // Another launch state provider (maybe better):
    // https://api.nike.com/launch/launch_views/v2/?filter=productId(8f40f86c-b3db-5a6b-8bfd-8e056fad6b6e)
    fn process(&mut self, client: &reqwest::blocking::Client) ->
            Result<HashSet<Url>, Box<dyn std::error::Error>> {
        let main_url = Url::parse(
            &format!("https://www.nike.com/nl/en/launch/t/{}",
            self.product_handle))?;

        // client.

        // Check the product page.
        let res = client
            .get(main_url.as_ref())
            .headers(get_headers())
            .send()?
            .text()?;

        println!("body = {:?}", res);

        if res.contains("Sold Out") {
            println!("Sold out.");

            return Ok(HashSet::new());
        } else if res.contains("Access Denied") {
            println!("Blocked!");

            return Ok(HashSet::new());
        }

        let info_url = Url::parse(
            &format!("https://api.nike.com/merch/products/v2/{}",
            self.product_id))?;

        // Check if product is active (contains info on release type aswell).
        if !client
            .get(info_url.as_ref())
            .headers(get_headers())
            .send()?
            .status()
            .is_success() {
            println!("Done! Not available.");

            return Ok(HashSet::new());
        }

        let stock_url = Url::parse(&format!("https://api.nike.com/deliver/avail\
            able_skus/v1/?filter=productIds({})", self.product_id))?;

        // Get the stock information.
        let v: Value = client
            .get(stock_url.as_ref())
            .headers(get_headers())
            .send()?
            .json()?;

        // Get the available size ids.
        let objects = &v["objects"];
        let available_size_ids: Vec<String> = objects
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p| get_in_stock_id(p))
            .collect();

        let products = Url::parse(&format!("https://api.nike.com/merch/skus/v2/\
            ?filter=productId({})&filter=country(NL)", self.product_id))?;

        // Get the product information.
        let v: Value = client
            .get(products.as_ref())
            .headers(get_headers())
            .send()?
            .json()?;

        println!("Done!");

        // Get the available size ids.
        let objects = &v["objects"];
        let sizes: Vec<&str> = objects
            .as_array()
            .unwrap()
            .iter()
            .filter_map(|p|
            if available_size_ids.contains(&p["id"].as_str().unwrap().to_string()) {
                p["nikeSize"].as_str()
            } else {
                None
            })
            .collect();

        let links: HashSet<Url> = sizes
            .iter()
            .filter_map(|x| Url::parse(&format!(
                "https://www.nike.com/nl/en/launch/t/{}/?productId={}&size={}",
                self.product_handle, self.product_id, x)).ok())
            .collect();

        Ok(links)
    }
}
