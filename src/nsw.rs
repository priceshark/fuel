use anyhow::{bail, Context, Result};
use chrono::NaiveDateTime;
use serde::Deserialize;
use typed_floats::tf64::NonNaN;

use crate::{FullRecord, Record, Site, State};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct RawRecord {
    service_station_name: String,
    address: String,
    suburb: String,
    postcode: String,
    brand: String,
    fuel_type: String,
    price_updated_date: String,
    price: NonNaN,
}

pub fn parse(data: String) -> Result<Vec<FullRecord>> {
    let mut new = String::new();
    for (i, line) in data.lines().enumerate() {
        let line = line.trim_end_matches(',');
        match line {
            "" => continue,
            "Rows 1 - 82817 (All Rows)" => continue, // last row of june 2019
            _ => (),
        }

        if new.is_empty() {
            if line.starts_with("ServiceStationName") {
                new.push_str(&line.replace("FuelCode", "FuelType"));
            } else {
                if i > 5 {
                    bail!("Failed to find header");
                }
                continue; // no line break
            }
        } else {
            new.push_str(line);
        }
        new.push('\n');
    }

    let mut reader = csv::Reader::from_reader(new.as_bytes());
    let mut records: Vec<RawRecord> = Vec::new();
    for result in reader.deserialize() {
        let mut record: RawRecord = result?;

        // fix cells that are merged above in spreadsheet, lost in csv conversion
        if let Some(prev) = records.last() {
            if record.service_station_name.is_empty()
                && record.address.is_empty()
                && record.suburb.is_empty()
                && record.postcode.is_empty()
            {
                record.service_station_name = prev.service_station_name.clone();
                record.address = prev.address.clone();
                record.suburb = prev.suburb.clone();
                record.postcode = prev.postcode.clone();
            }
            if record.brand.is_empty() {
                record.brand = prev.brand.clone();
            }
            if record.fuel_type.is_empty() {
                record.fuel_type = prev.fuel_type.clone();
            }
            if record.price_updated_date.is_empty() {
                record.price_updated_date = prev.price_updated_date.clone();
            }
        }

        records.push(record);
    }

    let mut output = Vec::new();
    for record in records {
        let date = record.price_updated_date;
        let timestamp = if date.ends_with("M") {
            NaiveDateTime::parse_from_str(&date, "%d/%m/%Y %I:%M:%S %p")
        } else if date.starts_with("2016-") {
            // 2016-12-01 1212":"12":"18
            // -> 2016-12-01 12:12:18 (i think)
            let (date, time) = date.split_once(' ').context("nope")?;
            let mut new = date.to_string();
            new.push(' ');
            for char in time.chars().skip(2) {
                if char != '"' {
                    new.push(char);
                }
            }

            NaiveDateTime::parse_from_str(&new, "%Y-%m-%d %H:%M:%S")
        } else if date.contains("/2016 ") {
            NaiveDateTime::parse_from_str(&date, "%d/%m/%Y %H:%M:%S")
        } else {
            NaiveDateTime::parse_from_str(&date, "%m/%d/%Y %H:%M")
        }
        .with_context(|| format!("Failed to parse date: {date}"))?
        .and_utc();

        let site = Site {
            id: None,
            name: Some(record.service_station_name),
            brand: record.brand,
            address: Some(record.address),
            suburb: record.suburb,
            postcode: record.postcode,
            state: State::NSW,
            latitude: None,
            longitude: None,
        };
        output.push(FullRecord {
            site,
            price: Record {
                timestamp,
                fuel: record.fuel_type,
                price: record.price,
            },
        })
    }

    Ok(output)
}
