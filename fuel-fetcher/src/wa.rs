use std::{collections::BTreeMap, thread::sleep, time::Duration};

use anyhow::{bail, Result};
use geo::Point;
use serde::Deserialize;

use crate::{CurrentPrice, Fuel, State, Station};

pub const FUELS: [&str; 7] = ["ULP", "PUP", "DSL", "BDL", "LPG", "98R", "E85"];

pub fn prices() -> Result<Vec<CurrentPrice>> {
    let agent = crate::agent();

    let mut prices = Vec::new();
    for fuel in FUELS {
        let mut attempt = 0;
        let data: Vec<RawStation> = loop {
            match agent
                .get(&format!(
                    "https://www.fuelwatch.wa.gov.au/api/sites?fuelType={fuel}",
                ))
                .call()
            {
                Ok(x) => break x.into_json()?,
                Err(ureq::Error::Status(500 | 503, _)) if attempt < 3 => {
                    attempt += 1;
                    eprintln!("Attempt {attempt} failed");
                    sleep(Duration::from_secs(3));
                    continue;
                }
                Err(e) => bail!(e),
            }
        };
        let fuel = match fuel {
            "ULP" => Fuel::Unleaded91,
            "PUP" => Fuel::Unleaded95,
            "DSL" => Fuel::Diesel,
            "BDL" => Fuel::PremiumDiesel,
            "LPG" => Fuel::LPG,
            "98R" => Fuel::Unleaded98,
            "E85" => Fuel::Ethanol85,
            _ => unreachable!(),
        };
        for station in data {
            prices.push(CurrentPrice {
                state: State::WA,
                station: station.id,
                fuel,
                price: station.product.price_today,
            })
        }
    }
    Ok(prices)
}

pub fn stations() -> Result<Vec<Station>> {
    let agent = crate::agent();
    let mut stations = BTreeMap::new();
    for fuel in FUELS {
        let data: Vec<RawStation> = agent
            .get(&format!(
                "https://www.fuelwatch.wa.gov.au/api/sites?fuelType={fuel}",
            ))
            .call()?
            .into_json()?;
        for station in data {
            if !stations.contains_key(&station.id) {
                stations.insert(
                    station.id,
                    Point::new(station.address.latitude, station.address.longitude),
                );
            }
        }
    }

    Ok(stations
        .into_iter()
        .map(|(id, point)| Station {
            state: State::WA,
            id,
            point,
        })
        .collect())
}

#[derive(Deserialize)]
struct RawStation {
    id: u32,
    address: Address,
    product: Product,
}

#[derive(Deserialize)]
struct Address {
    latitude: f64,
    longitude: f64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    price_today: Option<f64>,
}
