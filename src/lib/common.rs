use std::{
    env,
    fs::File,
    io::{self, BufRead, BufReader},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Error, Result};
use reqwest::blocking::Client;
use url::Url;

pub fn get_epoch_sec() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("Failed to calculate timestamp")?
        .as_secs())
}

pub fn get_timestamp_override() -> Result<Option<u64>> {
    Ok(match env::var("SR_TIMESTAMP_OVERRIDE") {
        Ok(s) => Some(
            s.parse::<u64>()
                .with_context(|| format!("Invalid value for SR_TIMESTAMP_OVERRIDE: {}", s))?,
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
    webbrowser::open(url).with_context(|| format!("Failed to open browser for URL: {}", url))
}

pub fn get_file_or_url(
    client: &Client,
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
                .with_context(|| format!("Failed to complete HTTP request to {}", url))?;
            let status = response.status().as_u16();
            if status == 200 {
                Ok((Box::new(BufReader::new(response)), None, Some(url)))
            } else {
                Err(Error::msg(format!(
                    "Unexpected {} status for {}",
                    status, url
                )))
            }
        }
        Err(_) => {
            if file_or_url == "-" {
                Ok((Box::new(BufReader::new(io::stdin())), None, None))
            } else {
                let path = PathBuf::from(file_or_url);
                let file = File::open(&path)
                    .with_context(|| format!("Failed to open file {}", file_or_url))?;
                let reader = BufReader::new(file);
                Ok((Box::new(reader), Some(path), None))
            }
        }
    }
}

pub fn get_file_or_url_string(
    client: &Client,
    file_or_url: &str,
) -> Result<(String, Option<PathBuf>, Option<Url>)> {
    let (mut reader, pathbuf, url) = get_file_or_url(client, file_or_url)?;
    let mut s = String::new();
    reader
        .read_to_string(&mut s)
        .with_context(|| format!("Buffer read failed for file {}", file_or_url))?;
    Ok((s, pathbuf, url))
}
