use anyhow::{bail, Context, Result};
use scraper::{Html, Selector};
use serde::Deserialize;

use crate::{CurrentPrice, Fuel, State};

pub fn run() -> Result<Vec<CurrentPrice>> {
    let body = crate::agent().get("https://myfuelnt.nt.gov.au/Home/Results?searchOptions=region&Suburb=&SuburbId=0&RegionId=1&FuelCode=DL&BrandIdentifier=").call()?.into_string()?;
    let html = Html::parse_document(&body);
    for x in html.select(&Selector::parse("#serverJson").expect("hardcoded")) {
        let json = x.attr("value").context("json missing")?;

        let mut prices = Vec::new();
        let data: RawData = serde_json::from_str(json)?;
        for station in data.fuel_outlet {
            for raw in station.available_fuels {
                let fuel = match &*raw.fuel_code {
                    "E85" => Fuel::Ethanol85,
                    "LPG" => Fuel::LPG,
                    "PD" => Fuel::PremiumDiesel,
                    "P98" => Fuel::Unleaded98,
                    "P95" => Fuel::Unleaded95,
                    "U91" | "LAF" => Fuel::Unleaded91,
                    "DL" => Fuel::Diesel,
                    x => bail!("unknown fuel code: {x}"),
                };
                let price = if raw.is_available {
                    Some(raw.price)
                } else {
                    None
                };
                prices.push(CurrentPrice {
                    state: State::NT,
                    station: station.fuel_outlet_id,
                    fuel,
                    price,
                })
            }
        }

        return Ok(prices);
    }

    bail!("failed to find json");
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawData {
    fuel_outlet: Vec<RawStation>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawStation {
    available_fuels: Vec<RawFuel>,
    fuel_outlet_id: u32,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawFuel {
    fuel_code: String,
    price: f64,
    #[serde(rename = "isAvailable")]
    is_available: bool,
}
