use std::fs;

use anyhow::Result;
use serde::Deserialize;
use serde_json::{Map, Value};

pub fn run() -> Result<()> {
    let agent = ureq::agent();
    let products: Vec<Product> = agent
        .get("https://www.fuelwatch.wa.gov.au/api/products")
        .call()?
        .into_json()?;

    let mut sites = Vec::new();
    for product in products {
        let data: Vec<Map<String, Value>> = agent
            .get(&format!(
                "https://www.fuelwatch.wa.gov.au/api/sites?fuelType={}",
                product.short_name
            ))
            .call()?
            .into_json()?;
        sites.extend(data);
    }

    fs::write("wa.json", serde_json::to_string_pretty(&sites)?)?;

    Ok(())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    short_name: String,
}
