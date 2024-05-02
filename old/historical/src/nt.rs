use std::str::FromStr;

use anyhow::{bail, Context, Result};
use chrono::{NaiveDate, NaiveTime};
use serde::Deserialize;
use typed_floats::tf64::NonNaN;

use crate::{FullRecord, Record, Site, State};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawRecord {
    full_date: String,
    #[serde(rename = "Brand Name")]
    brand: String,
    #[serde(rename = "Region Name")]
    region: String,
    suburb: String,
    postcode: String,
    lat: NonNaN,
    long: NonNaN,

    diesel: String,
    #[serde(rename = "Premium 98")]
    premium_98: String,
    #[serde(rename = "Premium 95")]
    premium_95: String,
    #[serde(rename = "Unleaded 91")]
    unleaded_91: String,
    #[serde(rename = "Premium Diesel")]
    premium_diesel: String,
    #[serde(rename = "LPG")]
    lpg: String,
    #[serde(rename = "Low Aromatic Fuel")]
    laf: String,
    #[serde(rename = "Ethanol 105 (E85)")]
    e85: String,
    #[serde(rename = "Ethanol 94 (E10)")]
    e10: String,
    #[serde(rename = "Bio Diesel 20")]
    b20: String,
}

#[derive(Debug, Deserialize)]
enum Fuel {}

pub fn parse(data: String) -> Result<Vec<FullRecord>> {
    let mut output = Vec::new();
    let mut reader = csv::Reader::from_reader(data.as_bytes());
    for result in reader.deserialize() {
        let record: RawRecord = result?;

        let date = if let Some(date) = record.full_date.strip_suffix(" 12:00") {
            NaiveDate::parse_from_str(date, "%m/%d/%Y")
        } else if let Some(date) = record.full_date.strip_suffix(" 12:00:00 AM") {
            NaiveDate::parse_from_str(date, "%d/%m/%Y")
        } else {
            // 2019-10
            NaiveDate::parse_from_str(&record.full_date, "%m-%d-%y")
        }
        .with_context(|| format!("failed to parse date {}", record.full_date))?;
        let timestamp = date
            // midngiht utc+9:30
            .and_time(NaiveTime::from_hms_opt(9, 30, 0).expect("hardcoded"))
            .and_utc();

        let site = Site {
            id: None,
            name: None,
            brand: record.brand,
            address: None,
            suburb: record.suburb,
            postcode: record.postcode,
            state: State::NT,
            latitude: Some(record.lat),
            longitude: Some(record.long),
        };
        for (fuel, price) in [
            ("Diesel", record.diesel),
            ("Premium 98", record.premium_98),
            ("Premium 95", record.premium_95),
            ("Unleaded 91", record.unleaded_91),
            ("Premium Diesel", record.premium_diesel),
            ("LPG", record.lpg),
            ("Low Aromatic Fuel", record.laf),
            ("Ethanol 105 (E85)", record.e85),
            ("Ethanol 94 (E10)", record.e10),
            ("Bio Diesel 20", record.b20),
        ] {
            if price == "0.0" || price == "" || price == "null" {
                continue;
            }

            let price = NonNaN::from_str(&price)?;
            output.push(FullRecord {
                site: site.clone(),
                price: Record {
                    fuel: fuel.to_string(),
                    timestamp: timestamp.clone(),
                    price,
                },
            })
        }
    }

    Ok(output)
}
