use std::env;

use crate::errors::*;

pub fn get_timestamp_override() -> Result<Option<u64>> {
    Ok(match env::var("SR_TIMESTAMP_OVERRIDE") {
        Ok(s) => Some(
            s.parse::<u64>()
                .chain_err(|| format!("Invalid value for SR_TIMESTAMP_OVERRIDE: {}", s))?,
        ),
        Err(_) => None,
    })
}

pub fn open_browser(url: &str) -> Result<()> {
    webbrowser::open(url).chain_err(|| format!("Failed to open browser for URL: {}", url))
}
