use anyhow::{Context, Result};
use chrono::NaiveDateTime;
use serde::Deserialize;
use typed_floats::tf64::NonNaN;

use crate::{FullRecord, Record, Site, State};

#[derive(Debug, Deserialize)]
struct RawRecord {
    #[serde(rename = "SiteId")]
    site_id: u64,
    #[serde(rename = "Site_Name")]
    site_name: String,
    #[serde(rename = "Site_Brand")]
    site_brand: String,
    #[serde(rename = "Sites_Address_Line_1")]
    site_address: String,
    #[serde(rename = "Site_Suburb")]
    site_suburb: String,
    #[serde(rename = "Site_State")]
    site_state: String, // ???
    #[serde(rename = "Site_Post_Code")]
    site_post_code: String,
    #[serde(rename = "Site_Latitude")]
    site_latitude: NonNaN,
    #[serde(rename = "Site_Longitude")]
    site_longitude: NonNaN,
    #[serde(rename = "Fuel_Type")]
    fuel_type: String,
    #[serde(rename = "Price")]
    price: u64,
    #[serde(rename = "TransactionDateutc")]
    date: String,
}

// #[derive(Debug, Deserialize)]
// enum Fuel {
//     // only appears once in nov 2022, ignored later
//     #[serde(rename = "Liquefied natural gas")]
//     LNG,
//     OPAL,
//     #[serde(rename = "e85")]
//     E85,
//     LPG,
//     Diesel,
//     #[serde(rename = "Premium Diesel")]
//     PremiumDiesel,
//     #[serde(rename = "PULP 95/96 RON")]
//     PULP9596RON,
//     #[serde(rename = "e10")]
//     E10,
//     #[serde(rename = "PULP 98 RON")]
//     PULP98RON,
//     Unleaded,
// }

pub fn parse(data: String) -> Result<Vec<FullRecord>> {
    let mut output = Vec::new();
    let mut reader = csv::Reader::from_reader(data.as_bytes());
    for result in reader.deserialize() {
        let record: RawRecord = result?;
        let site = Site {
            id: Some(record.site_id),
            name: Some(record.site_name),
            brand: record.site_brand,
            address: Some(record.site_address),
            suburb: record.site_suburb,
            postcode: record.site_post_code,
            state: State::QLD,
            longitude: Some(record.site_longitude),
            latitude: Some(record.site_latitude),
        };
        let timestamp = if record.date.contains('T') {
            NaiveDateTime::parse_from_str(&record.date, "%Y-%m-%dT%H:%M:%S")
        } else {
            NaiveDateTime::parse_from_str(&record.date, "%d/%m/%Y %H:%M")
        }
        .with_context(|| format!("Failed to parse date: {}", record.date))?
        .and_utc();
        let price = NonNaN::try_from((record.price as f64) / 100.0)?;
        output.push(FullRecord {
            site,
            price: Record {
                timestamp,
                fuel: record.fuel_type,
                price,
            },
        })
    }

    Ok(output)
}
