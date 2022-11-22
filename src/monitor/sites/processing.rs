use std::collections::HashSet;
use url::{Url};

pub trait MonitorProcessing {
    fn process(&mut self, client: &reqwest::blocking::Client) ->
            Result<HashSet<Url>, Box<dyn std::error::Error>>;
}
