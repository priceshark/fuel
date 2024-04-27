use anyhow::Result;
use chrono::{NaiveDate, NaiveTime, Utc};
use serde::Deserialize;
use typed_floats::tf64::NonNaN;

use crate::{FullRecord, Record, Site, State};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
struct RawRecord {
    publish_date: String,
    trading_name: String,
    brand_description: String,
    product_description: String, // enum
    // optional: some records in 2007-05 and 2008-07 don't have a price
    product_price: Option<NonNaN>,
    address: String,
    location: String,
    postcode: String,
    area_description: String,
    region_description: String,
}

pub fn parse(csv: String) -> Result<Vec<FullRecord>> {
    let mut output = Vec::new();
    let mut reader = csv::Reader::from_reader(csv.as_bytes());
    for result in reader.deserialize() {
        let record: RawRecord = result?;
        if let Some(price) = record.product_price {
            let site = Site {
                id: None,
                name: Some(record.trading_name),
                brand: record.brand_description,
                address: Some(record.address),
                suburb: record.location,
                postcode: record.postcode,
                state: State::WA,
                latitude: None,
                longitude: None,
            };

            // utc+9
            let date = NaiveDate::parse_from_str(&record.publish_date, "%d/%m/%Y")?;
            let time = NaiveTime::from_hms_opt(9, 0, 0).expect("hardcoded");
            let timestamp = date.and_time(time).and_utc();

            output.push(FullRecord {
                site,
                price: Record {
                    timestamp,
                    fuel: record.product_description,
                    price,
                },
            })
        }
    }

    Ok(output)
}
