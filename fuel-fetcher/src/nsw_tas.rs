use anyhow::{bail, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use geo::Point;
use serde::Deserialize;

use crate::{CurrentPrice, Fuel, State, Station};

fn data(client_id: &str, client_secret: &str) -> Result<RawData> {
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

    Ok(data)
}

pub fn prices(client_id: &str, client_secret: &str) -> Result<Vec<CurrentPrice>> {
    let mut prices = Vec::new();
    for raw in data(client_id, client_secret)?.prices {
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
        prices.push(CurrentPrice {
            state: raw.state,
            station: raw.stationcode,
            fuel,
            price: Some(raw.price),
        })
    }

    Ok(prices)
}

pub fn stations(client_id: &str, client_secret: &str) -> Result<Vec<Station>> {
    let mut stations = Vec::new();
    for raw in data(client_id, client_secret)?.stations {
        stations.push(Station {
            state: raw.state,
            id: raw.code.parse()?,
            point: Point::new(raw.location.latitude, raw.location.longitude),
        })
    }
    Ok(stations)
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct RawData {
    stations: Vec<RawStation>,
    prices: Vec<RawPrice>,
}

#[derive(Deserialize)]
struct RawStation {
    code: String,
    location: Location,
    state: State,
}

#[derive(Deserialize)]
struct Location {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct RawPrice {
    stationcode: u32,
    state: State,
    fueltype: String,
    price: f64,
    // "lastupdated": "17/04/2024 01:15:49"
}
