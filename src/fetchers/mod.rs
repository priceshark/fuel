use std::{env, fs};

use anyhow::{Context, Result};
use serde::Deserialize;

mod nsw_tas;
mod nt;
mod wa;

#[derive(Deserialize)]
struct Secrets {
    nsw_client_id: String,     // "api key"
    nsw_client_secret: String, // "api secret"
}

pub fn run() -> Result<()> {
    let secrets: Secrets = serde_json::from_str(
        &fs::read_to_string("secrets.json").context("failed to read secrets")?,
    )
    .context("failed to parse secrets")?;

    nt::run()?;
    nsw_tas::run(&secrets.nsw_client_id, &secrets.nsw_client_secret)?;
    wa::run()?;

    Ok(())
}
