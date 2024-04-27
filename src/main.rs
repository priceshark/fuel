use std::{collections::BTreeMap, fs::File, io::BufRead, path::Path};

use anyhow::Result;
use chrono::{DateTime, Utc};
use glob::glob;
use serde::Serialize;
use typed_floats::tf64::NonNaN;

mod fetchers;
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
    return fetchers::run();

    for state in [
        //
        State::NSW,
        State::NT,
        State::QLD,
        State::WA,
    ] {
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
                let fuel = match &*record.price.fuel {
                    "Diesel" | "DL" => Fuel::Diesel,
                    "Premium Diesel" | "Brand Diesel" | "PDL" => Fuel::PremiumDiesel,
                    "Bio Diesel 20" | "B20" => Fuel::Biodiesel,
                    "E10" | "e10" | "Ethanol 94 (E10)" => Fuel::Ethanol10,
                    "E85" | "e85" | "Ethanol 105 (E85)" => Fuel::Ethanol85,
                    "LPG" => Fuel::LPG,
                    "U91" | "Unleaded 91" | "Unleaded" | "ULP" | "OPAL" | "Low Aromatic Fuel" => {
                        Fuel::Unleaded91
                    }
                    "P95" | "Premium 95" | "PULP 95/96 RON" | "PULP" => Fuel::Unleaded95,
                    "P98" | "Premium 98" | "PULP 98 RON" | "98 RON" => Fuel::Unleaded98,

                    // very few, appear to be errors
                    "Liquefied natural gas" | "CNG" | "LNG" | "EV" | "P100" => continue,

                    // phased out 2006
                    "LRP" => continue,

                    x => {
                        println!("{x}");
                        continue;
                    }
                };
                // if let Some(x) = records.get_mut(&record.site) {
                //     x.push(record.price);
                // } else {
                //     records.insert(record.site, vec![record.price]);
                // }
            }

            // eprintln!("{path:?} {}", records.len());
        }

        // for (site, prices) in records {
        //     println!("{}", serde_json::to_string(&OutputRecord { site, prices })?);
        // }
    }

    Ok(())
}

enum Fuel {
    Diesel,
    PremiumDiesel,
    Biodiesel,
    LPG,
    Ethanol10,
    Ethanol85,
    Unleaded91,
    Unleaded95,
    Unleaded98,
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
