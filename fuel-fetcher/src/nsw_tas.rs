use anyhow::{bail, Result};
use geo::Point;
use serde::{Deserialize, Serialize};

use crate::{CurrentPrice, Fuel, State, Station};

fn data(state: State) -> Result<RawData> {
    let agent = crate::agent();
    let url = match state {
        State::NSW => "https://api.onegov.nsw.gov.au/FuelCheckApp/v1/fuel/prices",
        State::TAS => "https://api.onegov.nsw.gov.au/FuelCheckTasApp/v1/fuel/prices",
        _ => panic!("unexpected state {state:?}"),
    };

    let data: RawData = agent
        .get(url)
        // pretty sure these are only used in the response headers so idc
        .set("transactionid", "a")
        .set("requesttimestamp", "01/01/2001 01:01:01 AM")
        .call()?
        .into_json()?;

    Ok(data)
}

pub fn prices(state: State) -> Result<Vec<CurrentPrice>> {
    let mut prices = Vec::new();
    for raw in data(state)?.prices {
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
            state,
            station: raw.stationcode.parse()?,
            fuel,
            price: Some(raw.price),
        })
    }

    Ok(prices)
}

pub fn stations(state: State) -> Result<Vec<Station>> {
    let mut stations = Vec::new();
    for raw in data(state)?.stations {
        stations.push(Station {
            state,
            id: raw.code.parse()?,
            point: Point::new(raw.location.latitude, raw.location.longitude),
        })
    }
    Ok(stations)
}

#[derive(Deserialize, Serialize)]
struct AuthCache {
    access_token: String,
    expires_at: u64,
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
}

#[derive(Deserialize)]
struct Location {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
struct RawPrice {
    stationcode: String,
    fueltype: String,
    price: f64,
    // "lastupdated": "17/04/2024 01:15:49"
}
