pub mod sites;
extern crate reqwest;
extern crate select;
extern crate url;
extern crate serde_json;

use std::collections::HashSet;

use url::{Url};
use serde_json::{Value};

pub fn build_client(server: &str, username: &str, password: &str) ->
        Result<reqwest::blocking::Client, Box<dyn std::error::Error>> {
    let proxy = reqwest::Proxy::all(server)?.basic_auth(username, password);

    let wc = reqwest::blocking::Client::builder()
        .proxy(proxy)
        .build()?;

    match wc.get("http://api.ipify.org/").send() {
        Ok(response) => println!("New IP: {}", response.text()?),
        Err(_) => (),
    };

    Ok(wc)
}

fn get_processing(client: &reqwest::blocking::Client, site_info: &Value) ->
        Box<dyn sites::processing::MonitorProcessing> {
    let s = site_info["data_type"].as_str().unwrap();

    if s == "standard" {
        panic!();
    }

    if s == "json" {
        panic!();
    }

    if s == "not_text" {
        return Box::new(sites::not_text::NotText::new(&client, site_info))
    }

    if s == "nike" {
        return Box::new(sites::nike::NikeProduct::new(&client, site_info))
    }

    if s == "shopify_variant" {
        return Box::new(sites::shopify_variant::ShopifyVariant::new(&client, site_info))
    }

    if s == "bol" {
        return Box::new(sites::bol::Bol::new(&client, site_info))
    }

    if s == "nvidia" {
        return Box::new(sites::nvidia::Nvidia::new(&client, site_info))
    }

    if s == "marktplaats" {
        return Box::new(sites::marktplaats::Marktplaats::new(site_info))
    }

    panic!();
}

pub struct Monitor {
    pub url: Url,
    links: HashSet<Url>,
    pub links_new: HashSet<Url>,
    pub links_rem: HashSet<Url>,
    pub client: reqwest::blocking::Client,
    processor: Box<dyn sites::processing::MonitorProcessing>,
    pub successive_err_count: i32,
}

impl Monitor {
    pub fn new(site_info: &Value) -> Self {
        let url = url::Url::parse(&site_info["url"].as_str().unwrap()).ok().unwrap();

        let client = reqwest::blocking::Client::builder()
            .build()
            .unwrap();

        let processor = get_processing(&client, &site_info);

        Monitor {
            url: url,
            links: HashSet::new(),
            links_new: HashSet::new(),
            links_rem: HashSet::new(),
            client: client,
            processor: processor,
            successive_err_count: 0,
        }
    }

    fn update_links(&mut self, new_links: HashSet<Url>) -> bool {
        let mut links_added: HashSet<&Url> = HashSet::new();
        let mut links_removed: HashSet<&Url> = HashSet::new();

        for link in &self.links {
            if !new_links.contains(link) {
                links_removed.insert(link);
                self.links_rem.insert(link.clone());
            }
        }

        for link in &new_links {
            if !self.links.contains(link) {
                links_added.insert(link);
                self.links_new.insert(link.clone());
            }
        }

        if links_added.len() > 0 {
            println!("Links have been added!");

            self.links = new_links;

            return true;
        }

        self.links = new_links;

        false
    }

    pub fn read_page(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        self.links_new.clear();
        self.links_rem.clear();

        print!("Sending request... ");

        match self.processor.process(&self.client) {
            Ok(links) => return Ok(self.update_links(links)),
            Err(e) => return Err(e),
        }
    }
}
