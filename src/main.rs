use std::{
    fs,
    path::Path,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use clap::{Parser, Subcommand};
use geo::Point;
use rusqlite::{
    types::{FromSql, FromSqlError},
    Connection, OptionalExtension, ToSql,
};
use serde::{Deserialize, Serialize};
use ureq::{Agent, AgentBuilder};

mod nsw_tas;
mod nt;
mod qld_sa;
mod wa;

#[derive(Debug, Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Stations,
    Prices,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let auth: Auth = toml::from_str(&fs::read_to_string("auth.toml")?)?;

    match cli.command {
        Command::Stations => {
            let mut stations = Vec::new();
            eprintln!("Fetching NSW+TAS");
            stations.extend(nsw_tas::stations(
                &auth.nsw_client_id,
                &auth.nsw_client_secret,
            )?);
            eprintln!("Fetching QLD");
            stations.extend(qld_sa::stations(State::QLD, &auth.qld_token)?);
            eprintln!("Fetching NT");
            stations.extend(nt::stations()?);
            eprintln!("Fetching SA");
            stations.extend(qld_sa::stations(State::SA, &auth.sa_token)?);
            eprintln!("Fetching WA");
            stations.extend(wa::stations()?);
            println!("lat,lon");
            for station in stations {
                let (x, y) = station.point.x_y();
                println!("{x},{y}")
            }
        }

        Command::Prices => {
            let mut prices = Vec::new();
            eprintln!("Fetching NSW+TAS");
            prices.extend(nsw_tas::prices(
                &auth.nsw_client_id,
                &auth.nsw_client_secret,
            )?);
            eprintln!("Fetching NT");
            prices.extend(nt::prices()?);
            eprintln!("Fetching QLD");
            prices.extend(qld_sa::prices(State::QLD, &auth.qld_token)?);
            eprintln!("Fetching SA");
            prices.extend(qld_sa::prices(State::SA, &auth.sa_token)?);
            eprintln!("Fetching WA");
            prices.extend(wa::prices()?);
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

            let path = Path::new("fuel.db");
            let new = !path.exists();
            let mut conn = Connection::open(path)?;
            if new {
                conn.execute_batch(include_str!("../db.sql"))?;
            }

            eprintln!("Updating DB");
            let tx = conn.transaction()?;
            {
                let mut select = tx.prepare(
                    "select price from price where state = ? and station = ? and fuel = ?",
                )?;
                let mut  insert = tx.prepare("insert into price (state, station, fuel, price, changed_at, checked_at) values (?, ?, ?, ?, ?, ?)")?;
                let mut changed = tx.prepare("update price set price = ?, changed_at = ?, checked_at = ? where state = ? and station = ? and fuel = ?")?;
                let mut checked = tx.prepare(
                    "update price set checked_at = ? where state = ? and station = ? and fuel = ?",
                )?;

                for price in prices {
                    // first option: row found?
                    // second option: fuel available?
                    let db_price: Option<Option<f64>> = select
                        .query_row((&price.state, &price.station, &price.fuel), |row| {
                            row.get(0)
                        })
                        .optional()?;

                    if let Some(db_price) = db_price {
                        if price.price == db_price {
                            checked.execute((&now, &price.state, &price.station, &price.fuel))?;
                        } else {
                            changed.execute((
                                &price.price,
                                &now,
                                &now,
                                &price.state,
                                &price.station,
                                &price.fuel,
                            ))?;
                        }
                    } else {
                        insert.execute((
                            &price.state,
                            &price.station,
                            &price.fuel,
                            &price.price,
                            &now,
                            &now,
                        ))?;
                    }
                }
            }
            tx.commit()?;
        }
    }

    Ok(())
}

#[derive(Deserialize)]
struct Auth {
    nsw_client_id: String,
    nsw_client_secret: String,
    qld_token: String,
    sa_token: String,
}

#[derive(Debug)]
struct CurrentPrice {
    state: State,
    station: u32,
    fuel: Fuel,
    // cents per liter
    price: Option<f64>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum State {
    NSW,
    NT,
    QLD,
    SA,
    TAS,
    WA,
}

impl State {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::NSW => "NSW",
            Self::NT => "NT",
            Self::QLD => "QLD",
            Self::SA => "SA",
            Self::TAS => "TAS",
            Self::WA => "WA",
        }
    }
}

impl FromStr for State {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "NSW" => Self::NSW,
            "NT" => Self::NT,
            "QLD" => Self::QLD,
            "SA" => Self::SA,
            "TAS" => Self::TAS,
            "WA" => Self::WA,
            _ => return Err(()),
        })
    }
}

impl ToSql for State {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.as_str().into())
    }
}

impl FromSql for State {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(value
            .as_str()?
            .parse()
            .map_err(|_| FromSqlError::InvalidType)?)
    }
}

#[derive(Debug, Clone, Copy)]
enum Fuel {
    Diesel,
    PremiumDiesel,
    LPG,
    Ethanol10,
    Ethanol85,
    Unleaded91,
    Unleaded95,
    Unleaded98,
}

impl Fuel {
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Diesel => "Diesel",
            Self::PremiumDiesel => "PremiumDiesel",
            Self::LPG => "LPG",
            Self::Ethanol10 => "Ethanol10",
            Self::Ethanol85 => "Ethanol85",
            Self::Unleaded91 => "Unleaded91",
            Self::Unleaded95 => "Unleaded95",
            Self::Unleaded98 => "Unleaded98",
        }
    }
}

impl FromStr for Fuel {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(match s {
            "Diesel" => Self::Diesel,
            "PremiumDiesel" => Self::PremiumDiesel,
            "LPG" => Self::LPG,
            "Ethanol10" => Self::Ethanol10,
            "Ethanol85" => Self::Ethanol85,
            "Unleaded91" => Self::Unleaded91,
            "Unleaded95" => Self::Unleaded95,
            "Unleaded98" => Self::Unleaded98,
            _ => return Err(()),
        })
    }
}

impl ToSql for Fuel {
    fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
        Ok(self.as_str().into())
    }
}

impl FromSql for Fuel {
    fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
        Ok(value
            .as_str()?
            .parse()
            .map_err(|_| FromSqlError::InvalidType)?)
    }
}

#[derive(Deserialize, Serialize)]
struct Station {
    state: State,
    id: u32,
    #[serde(flatten)]
    point: Point,
}

const USER_AGENT: &str = concat!(
    "priceshark-fuel/",
    env!("CARGO_PKG_VERSION"),
    " (mailto:automated@joel.net.au +https://github.com/priceshark/fuel)"
);

fn agent() -> Agent {
    AgentBuilder::new().user_agent(USER_AGENT).build()
}
