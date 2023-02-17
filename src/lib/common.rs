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

pub fn has_sqlite_ext(filename: &str) -> bool {
    let name = filename.to_lowercase();
    if name.ends_with(".db") || name.ends_with(".sqlite") {
        true
    } else {
        false
    }
}

pub fn open_browser(url: &str) -> Result<()> {
    webbrowser::open(url).chain_err(|| format!("Failed to open browser for URL: {}", url))
}
