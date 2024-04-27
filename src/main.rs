use std::{collections::BTreeMap, fs::File, io::BufRead, path::Path};

use anyhow::Result;
use chrono::{DateTime, Utc};
use glob::glob;
use serde::Serialize;
use typed_floats::tf64::NonNaN;

mod nsw;
mod nt;
mod qld;
mod wa;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Clone, Copy)]
enum State {
    NSW,
    NT,
    QLD,
    WA,
}

impl State {
    fn slug(&self) -> &'static str {
        match self {
            Self::NSW => "nsw",
            Self::NT => "nt",
            Self::QLD => "qld",
            Self::WA => "wa",
        }
    }
}

fn main() -> Result<()> {
    for state in [State::NSW, State::NT, State::QLD, State::WA] {
        // let mut records: BTreeMap<Site, Vec<Record>> = BTreeMap::new();
        for path in glob(&format!("raw/{}/*.csv.zst", state.slug()))? {
            let path = path?;
            let data = zstd::decode_all(File::open(&path)?)?;
            let data = String::from_utf8_lossy(&data).to_string();

            eprintln!("{path:?}");
            let output = match state {
                State::NSW => nsw::parse(data)?,
                State::NT => nt::parse(data)?,
                State::QLD => qld::parse(data)?,
                State::WA => wa::parse(data)?,
            };
            for record in output {
                // if let Some(x) = records.get_mut(&record.site) {
                //     x.push(record.price);
                // } else {
                //     records.insert(record.site, vec![record.price]);
                // }
                println!("{}", record.price.fuel);
            }

            // eprintln!("{path:?} {}", records.len());
        }

        // for (site, prices) in records {
        //     println!("{}", serde_json::to_string(&OutputRecord { site, prices })?);
        // }
    }

    Ok(())
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Clone)]
struct Site {
    id: Option<u64>,
    name: Option<String>,
    brand: String,
    address: Option<String>,
    suburb: String,
    postcode: String,
    state: State,
    latitude: Option<NonNaN>,
    longitude: Option<NonNaN>,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct FullRecord {
    site: Site,
    price: Record,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Serialize)]
struct Record {
    timestamp: DateTime<Utc>,
    fuel: String,
    price: NonNaN,
}

#[derive(Debug, Serialize)]
struct OutputRecord {
    site: Site,
    prices: Vec<Record>,
}
