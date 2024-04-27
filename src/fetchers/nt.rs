use std::fs;

use anyhow::{Context, Result};
use scraper::{Html, Selector};
use serde_json::{Map, Value};
use ureq::Agent;

pub fn run() -> Result<()> {
    let agent = Agent::new();

    // all data is returned regardless of params, only seem to be used by the client
    let body = agent.get("https://myfuelnt.nt.gov.au/Home/Results?searchOptions=region&Suburb=&SuburbId=0&RegionId=1&FuelCode=DL&BrandIdentifier=").call()?.into_string()?;
    let html = Html::parse_document(&body);
    for x in html.select(&Selector::parse("#serverJson").expect("hardcoded")) {
        let data: Map<String, Value> =
            serde_json::from_str(x.attr("value").context("json missing")?)?;
        fs::write(format!("nt.json"), serde_json::to_string_pretty(&data)?)?;
    }

    Ok(())
}
