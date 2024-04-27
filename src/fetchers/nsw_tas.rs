use std::fs;

use anyhow::Result;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde::Deserialize;

pub fn run(client_id: &str, client_secret: &str) -> Result<()> {
    let agent = ureq::agent();
    let encoded = STANDARD.encode(format!("{client_id}:{client_secret}"));
    let auth: AuthResponse = agent.get( "https://api.onegov.nsw.gov.au/oauth/client_credential/accesstoken?grant_type=client_credentials")
        .set("Authorization", &format!("Basic {encoded}"))
        .call()?
        .into_json()?;

    let x = agent
        .get("https://api.onegov.nsw.gov.au/FuelPriceCheck/v2/fuel/prices?states=NSW|TAS")
        .set("Authorization", &format!("Bearer {}", auth.access_token))
        .set("apikey", client_id)
        // pretty sure these are only used in the response headers so idc
        .set("transactionid", "a")
        .set("requesttimestamp", "01/01/2001 01:01:01 AM")
        .call()?
        .into_string()?;
    fs::write("nsw_tas.json", x)?;

    Ok(())
}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
}
