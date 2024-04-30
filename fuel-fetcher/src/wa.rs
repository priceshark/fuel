use anyhow::Result;
use serde::Deserialize;

use crate::{CurrentPrice, Fuel, State};

pub const FUELS: [&str; 7] = ["ULP", "PUP", "DSL", "BDL", "LPG", "98R", "E85"];

pub fn run() -> Result<Vec<CurrentPrice>> {
    let agent = crate::agent();

    let mut prices = Vec::new();
    for fuel in FUELS {
        let data: Vec<RawStation> = agent
            .get(&format!(
                "https://www.fuelwatch.wa.gov.au/api/sites?fuelType={fuel}",
            ))
            .call()?
            .into_json()?;
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStation {
    id: u32,
    product: Product,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Product {
    price_today: Option<f64>,
}
