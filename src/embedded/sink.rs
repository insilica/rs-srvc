use std::collections::HashSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;

use serde::Serialize;

use crate::embedded;
use crate::errors::*;

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

pub fn run() -> Result<()> {
    let env = embedded::get_env().chain_err(|| "Env var processing failed")?;
    let config = embedded::get_config(env.config)?;
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
    let input = File::open(env.input.unwrap()).chain_err(|| "Cannot open SR_INPUT")?;
    let reader = BufReader::new(input);
    let in_events = embedded::events(reader);
    let mut writer = LineWriter::new(db_file);

    for result in in_events {
        let event = result.chain_err(|| "Cannot parse line as JSON")?;
        let hash = event.hash.clone().unwrap_or("".to_string());
        if !hashes.contains(&hash) {
            event
                .serialize(&mut serde_json::Serializer::new(&mut writer))
                .chain_err(|| "Event serialization failed")?;
            writer.write(b"\n").chain_err(|| "Buffer write failed")?;
            hashes.insert(hash);
        };
    }

    Ok(())
}
