use anyhow::{bail, Context, Result};
use serde::Deserialize;
use ureq::Agent;

use crate::{CurrentPrice, Fuel, State};

pub fn run(state: State, token: &str) -> Result<Vec<CurrentPrice>> {
    let auth = format!("fpdapi subscribertoken={token}");
    let url = match state {
        State::QLD =>"https://fppdirectapi-prod.fuelpricesqld.com.au/Price/GetSitesPrices?countryId=21&geoRegionLevel=3&geoRegionId=1",
        State::SA => "https://fppdirectapi-prod.safuelpricinginformation.com.au/Price/GetSitesPrices?countryId=21&geoRegionLevel=3&geoRegionId=4",
        _ => panic!("unexpected state {state:?}")
    };

    // all data is returned regardless of params, only seem to be used by the client
    let response: Response = crate::agent()
        .get(url)
        .set("authorization", &auth)
        .call()?
        .into_json()?;
    let mut prices = Vec::new();
    for raw in response.site_prices {
        let fuel = match raw.fuel_id {
            2 => Fuel::Unleaded91,
            3 => Fuel::Diesel,
            4 => Fuel::LPG,
            5 => Fuel::Unleaded95,
            8 => Fuel::Unleaded98,
            12 => Fuel::Ethanol10,
            14 => Fuel::PremiumDiesel,
            19 => Fuel::Ethanol85,
            21 => Fuel::Unleaded91, // https://en.wikipedia.org/wiki/Opal_(fuel)
            x => bail!("unknown fuel type: {x}"),
        };
        let price = match raw.price {
            9999.0 => None,
            x => Some(x / 10.0),
        };
        prices.push(CurrentPrice {
            state,
            station: raw.site_id,
            fuel,
            price,
        });
    }
    Ok(prices)
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Response {
    site_prices: Vec<Price>,
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Price {
    site_id: u32,
    fuel_id: u32,
    price: f64,
    // transactiondate
}
