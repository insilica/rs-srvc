use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;

use reqwest::blocking::Client;
use serde::Serialize;

use lib_sr::errors::*;
use lib_sr::event;
use lib_sr::event::Event;
use lib_sr::Config;

use crate::embedded;
use crate::embedded::Env;
use crate::json_schema;

pub fn read_hashes(file: File) -> Result<HashSet<String>> {
    let reader = BufReader::new(file);
    let events = event::events(reader);
    let mut hashes = HashSet::new();

    for result in events {
        let hash = result?.hash.ok_or("No hash for event")?;
        hashes.insert(hash);
    }

    Ok(hashes)
}

pub fn ensure_hash(event: &mut Event) -> Result<()> {
    let expected_hash = lib_sr::event::event_hash(event.clone())?;
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

fn validation_error_message(e: jsonschema::ValidationError) -> String {
    // Work around lifetime complications caused by jsonschema's
    // ValidationError referencing the schema data
    let instance_path = e.instance_path.to_string();
    let path = if instance_path.is_empty() {
        String::from("root")
    } else {
        instance_path
    };
    format!(
        "JSON schema validation failed at {}: {}",
        path,
        e.to_string()
    )
}

fn validate_answer(
    answer: &Event,
    data: &serde_json::Value,
    schema: &serde_json::Value,
) -> Result<()> {
    let event_hash = answer.hash.as_ref().unwrap();
    match data.get("answer") {
        Some(answer) => {
            let schema = json_schema::compile(schema)?;
            match schema.validate(answer) {
                Ok(_) => (),
                Err(errs) => {
                    for e in errs {
                        Err(format!(
                            "label-answer {} failed JSON schema validation: {}",
                            event_hash,
                            validation_error_message(e)
                        ))?
                    }
                }
            };
        }
        None => (),
    }
    Ok(())
}

fn prep_event(labels: &mut HashMap<String, Event>, result: Result<Event>) -> Result<Event> {
    let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
    ensure_hash(&mut event)?;
    if event.r#type == "label" {
        labels.insert(event.hash.as_ref().unwrap().to_string(), event.clone());
    } else if event.r#type == "label-answer" {
        let data = event.data.as_ref().expect("data");
        let label_hash = data.get("label").expect("label").as_str().expect("string");
        let label = labels
            .get(label_hash)
            .ok_or_else(|| format!("Label not found with hash: {}", label_hash))?;
        match label
            .data
            .as_ref()
            .expect("data")
            .as_object()
            .expect("object")
            .get("json_schema")
        {
            Some(val) => validate_answer(&event, data, val)?,
            None => (),
        }
    }
    Ok(event)
}

pub fn run_remote(env: Env, config: Config) -> Result<()> {
    let mut hashes = HashSet::new();
    let mut labels = HashMap::new();
    let input_addr = env.input.ok_or("Missing value for SR_INPUT")?;
    let in_events = embedded::input_events(&input_addr)?;
    let client = Client::new();
    let url = embedded::api_route(&config.db, "upload");

    for result in in_events {
        let event = prep_event(&mut labels, result)?;
        let hash = event.hash.clone().expect("Hash not set");

        if !hashes.contains(&hash) && event.r#type != "control" || config.sink_all_events {
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
    let mut labels = HashMap::new();
    let db_file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&config.db)
        .chain_err(|| format!("Failed to open db: \"{}\"", config.db))?;
    let input_addr = env.input.ok_or("Missing value for SR_INPUT")?;
    let in_events = embedded::input_events(&input_addr)?;
    let mut writer = LineWriter::new(db_file);

    for result in in_events {
        let event = prep_event(&mut labels, result)?;
        let hash = event.hash.clone().expect("Hash not set");

        if !hashes.contains(&hash) && event.r#type != "control" || config.sink_all_events {
            event
                .serialize(&mut serde_json::Serializer::new(&mut writer))
                .chain_err(|| "Event serialization failed")?;

            #[cfg(unix)]
            let newline = b"\n";
            #[cfg(windows)]
            let newline = b"\r\n";
            writer.write(newline).chain_err(|| "Buffer write failed")?;
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
