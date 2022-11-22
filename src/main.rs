mod discord;
mod monitor;
extern crate url;
extern crate serde;
extern crate serde_json;
use std::{
    fs::File,
    io::Read,
    io::{prelude::*, BufReader},
};
use rand::seq::SliceRandom;
use chrono::{Utc};
use std::env;

const MAX_SUCCESSIVE_ERRORS: i32 = 10;
const DELAY_SECONDS: std::time::Duration = std::time::Duration::from_secs(3);

type Sites = std::collections::HashMap<String, serde_json::Value>;


fn load_json() -> Result<Sites, Box<dyn std::error::Error>> {
    let mut file = File::open("sites.json")?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    let json: Sites = serde_json::from_str(&data)?;

    Ok(json)
}

fn load_env() -> serde_json::Value {
    let mut file = File::open("sites.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();

    let json: serde_json::Value = serde_json::from_str(&data).unwrap();

    json
}


#[derive(Debug, Clone)]
struct Proxy {
    pub host: String,
    pub port: String,
    pub username: String,
    pub password: String,
}

fn parse_proxy(line: String) -> Proxy {
    let parts: Vec<&str> = line.trim()
        .split(":")
        .collect();

    if parts.len() == 2 {
        return Proxy {
            host: String::from(parts[0]),
            port:  String::from(parts[1]),
            username: String::from(""),
            password: String::from(""),
        }
    } else if parts.len() != 4 {
        panic!()
    }

    Proxy {
        host: String::from(parts[0]),
        port:  String::from(parts[1]),
        username: String::from(parts[2]),
        password: String::from(parts[3]),
    }
}

fn load_proxies() -> Vec<Proxy> {
    let file = File::open("proxies.txt").expect("no such file");
    let buf = BufReader::new(file);
    buf.lines()
        .map(|l| l.expect("Could not parse line"))
        .map(|x| parse_proxy(x))
        .collect()
}

fn get_random_proxy(proxies: &Vec<Proxy>) -> &Proxy {
    proxies.choose(&mut rand::thread_rng()).unwrap()
}

fn switch_proxy(monitor: &mut monitor::Monitor, proxies: &Vec<Proxy>) {
    let proxy = get_random_proxy(proxies);
    let address = format!("http://{}:{}", proxy.host, proxy.port);

    monitor.client = monitor::build_client(
        address.as_ref(),
        proxy.username.as_ref(), proxy.password.as_ref()).unwrap();
}

fn monitor_array_loop(mut monitors: std::vec::Vec<monitor::Monitor>,
            channel_id: &str, token: &str) ->
            Result<(), Box<dyn std::error::Error>> {
    let mut first_run = true;

    let proxies = load_proxies();

    loop {
        if !first_run {
            std::thread::sleep(DELAY_SECONDS);
        }

        first_run = false;

        for monitor in &mut monitors {
            println!("{}", monitor.url);

            if !proxies.is_empty() {
                switch_proxy(monitor, &proxies);
            }

            match monitor.read_page() {
                Ok(v) => {
                    monitor.successive_err_count = 0;

                    if !v {
                        continue;
                    }
                },
                Err(e) => {
                    println!("{} Error while reading page.", e);
                    println!("Source: {}", e.source().unwrap());

                    // TODO: Handle connection closed before message completed
                    //       Error while reading page
                    //       by trying with a different proxy.

                    if !proxies.is_empty() {
                        switch_proxy(monitor, &proxies);
                    }

                    monitor.successive_err_count += 1;

                    if monitor.successive_err_count > MAX_SUCCESSIVE_ERRORS {
                        discord::print_to_discord(
                                channel_id, token, "Monitor crashed")?;

                        return Err("Maximum successive errors reached".into());
                    }

                    continue;
                }
            };

            let mut s = "Site has been updated!\\n".to_string();

            if monitor.links_new.len() > 0 {
                s += "New Links:\\n";

                monitor.links_new.iter().for_each(|x| {
                    s += x.as_str();
                    s += "\\n";
                });
            }

            if monitor.links_rem.len() > 0 {
                s += "Removed Links:\\n";

                monitor.links_rem.iter().for_each(|x| {
                    s += x.as_str();
                    s += "\\n";
                });
            }

            println!("{}", s);

            if first_run {
                continue;
            }

            if !discord::print_to_discord(channel_id, token, &s).is_ok() {
                return Err("Discord error!".into());
            }
        }

        println!("{}", Utc::now());
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = env::args().collect();
    args.remove(0);
    println!("{:?}", args);

    let site_info: Sites = load_json()?;

    let env = load_env();

    let channel_id = env["channel_id"].to_string();
    let token = env["token"].to_string();

    let sites = args;

    let monitors: std::vec::Vec<monitor::Monitor> =
        sites.iter().map(|m| monitor::Monitor::new(&site_info[m])).collect();

    monitor_array_loop(monitors, &channel_id, &token)?;

    // TODO: Take account of 2000 char message length limit.

    Ok(())
}
