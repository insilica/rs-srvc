use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
    path::PathBuf,
};

use reqwest::blocking::Client;
use url::Url;

use crate::errors::*;

pub fn get_file_or_url(
    client: Client,
    file_or_url: &str,
) -> Result<(Box<dyn BufRead + Send + Sync>, Option<PathBuf>, Option<Url>)> {
    match Url::parse(file_or_url) {
        Ok(url) => {
            let mut request = client.get(url.clone());

            if let Ok(token) = env::var("SRVC_TOKEN") {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request
                .send()
                .chain_err(|| format!("Failed to complete HTTP request to {}", url))?;
            let status = response.status().as_u16();
            if status == 200 {
                Ok((Box::new(BufReader::new(response)), None, Some(url)))
            } else {
                Err(format!("Unexpected {} status for {}", status, url).into())
            }
        }
        Err(_) => {
            let path = PathBuf::from(file_or_url);
            let file =
                File::open(&path).chain_err(|| format!("Failed to open file {}", file_or_url))?;
            let reader = BufReader::new(file);
            Ok((Box::new(reader), Some(path), None))
        }
    }
}

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
