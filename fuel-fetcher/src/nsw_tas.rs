use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;

use crate::{CurrentPrice, Fuel, State};

pub fn run(client_id: &str, client_secret: &str) -> Result<Vec<CurrentPrice>> {
    let agent = crate::agent();
    let encoded = STANDARD.encode(format!("{client_id}:{client_secret}"));
    let auth: AuthResponse = agent.get( "https://api.onegov.nsw.gov.au/oauth/client_credential/accesstoken?grant_type=client_credentials")
        .set("Authorization", &format!("Basic {encoded}"))
        .call()?
        .into_json()?;

    let data: RawData = agent
        .get("https://api.onegov.nsw.gov.au/FuelPriceCheck/v2/fuel/prices?states=NSW|TAS")
        .set("Authorization", &format!("Bearer {}", auth.access_token))
        .set("apikey", client_id)
        // pretty sure these are only used in the response headers so idc
        .set("transactionid", "a")
        .set("requesttimestamp", "01/01/2001 01:01:01 AM")
        .call()?
        .into_json()?;

    let mut prices = Vec::new();
    for raw in data.prices {
        let fuel = match &*raw.fueltype {
            "B20" | "EV" => continue,
            "DL" => Fuel::Diesel,
            "E10" => Fuel::Ethanol10,
            "E85" => Fuel::Ethanol85,
            "LPG" => Fuel::LPG,
            "P95" => Fuel::Unleaded95,
            "P98" => Fuel::Unleaded98,
            "PDL" => Fuel::PremiumDiesel,
            "U91" => Fuel::Unleaded91,
            x => bail!("unknown fuel {x}"),
        };
        let state = match &*raw.state {
            "NSW" => State::NSW,
            "TAS" => State::TAS,
            x => bail!("unexpected state {x}"),
        };
        prices.push(CurrentPrice {
            state,
            station: raw.stationcode,
            fuel,
            price: Some(raw.price),
        })
    }

    Ok(prices)
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct RawData {
    prices: Vec<Price>,
}

#[derive(Deserialize)]
struct Price {
    stationcode: u32,
    state: String,
    fueltype: String,
    price: f64,
    // "lastupdated": "17/04/2024 01:15:49"
}
