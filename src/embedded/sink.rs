use std::collections::HashSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;

use reqwest::blocking::Client;
use serde::Serialize;

use crate::embedded;
use crate::embedded::Env;
use crate::errors::*;
use crate::event::Event;
use crate::lib::Config;

pub fn read_hashes(file: File) -> Result<HashSet<String>> {
    let reader = BufReader::new(file);
    let events = embedded::events(reader);
    let mut hashes = HashSet::new();

    for result in events {
        let hash = result?.hash.ok_or("No hash for event")?;
        hashes.insert(hash);
    }

    Ok(hashes)
}

pub fn ensure_hash(event: &mut Event) -> Result<()> {
    let expected_hash = crate::event::event_hash(event.clone())?;
    let hash = event.hash.clone().unwrap_or("".to_string());
    if hash == "" {
        event.hash = Some(expected_hash);
    } else if expected_hash != hash {
        return Err(format!(
            "Incorrect event hash. Expected: \"{}\". Found: \"{}\".",
            expected_hash, hash
        )
        .into());
    }
    Ok(())
}

pub fn run_remote(env: Env, config: Config) -> Result<()> {
    let mut hashes = HashSet::new();
    let input_addr = env.input.ok_or("Missing value for SR_INPUT")?;
    let in_events = embedded::input_events(&input_addr)?;
    let client = Client::new();
    let url = embedded::api_route(&config.db, "upload");

    for result in in_events {
        let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
        ensure_hash(&mut event)?;
        let hash = event.hash.clone().expect("Hash not set");

        if !hashes.contains(&hash) && event.r#type != "control" {
            let json = serde_json::to_string(&event).chain_err(|| "Serialization failed")?;
            let response = client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(json)
                .send()
                .chain_err(|| "Error sending event to remote")?;
            let status = response.status().as_u16();
            if status >= 400 {
                let text = response
                    .text()
                    .chain_err(|| "Error getting response text")?;
                return Err(format!("{} response at {} ({})", status, &url, text).into());
            }
            hashes.insert(hash);
        };
    }

    Ok(())
}

pub fn run_local(env: Env, config: Config) -> Result<()> {
    let maybe_db = File::open(&config.db);
    let mut hashes = match maybe_db {
        Err(_) => HashSet::new(), // The file may not exist yet
        Ok(file) => read_hashes(file)?,
    };
    let db_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.db)
        .chain_err(|| format!("Failed to open db: \"{}\"", config.db))?;
    let input_addr = env.input.ok_or("Missing value for SR_INPUT")?;
    let in_events = embedded::input_events(&input_addr)?;
    let mut writer = LineWriter::new(db_file);

    for result in in_events {
        let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
        ensure_hash(&mut event)?;
        let hash = event.hash.clone().expect("Hash not set");

        if !hashes.contains(&hash) && event.r#type != "control" {
            event
                .serialize(&mut serde_json::Serializer::new(&mut writer))
                .chain_err(|| "Event serialization failed")?;
            writer.write(b"\n").chain_err(|| "Buffer write failed")?;
            hashes.insert(hash);
        };
    }

    Ok(())
}

pub fn run() -> Result<()> {
    let env = embedded::get_env().chain_err(|| "Env var processing failed")?;
    let config = embedded::get_config(&env.config)?;
    if embedded::is_remote_target(&config.db) {
        run_remote(env, config)
    } else {
        run_local(env, config)
    }
}
