use std::collections::{HashMap, HashSet};
use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;
use std::path::PathBuf;

use log::{debug, info};
use reqwest::blocking::Client;
use serde::Serialize;

use lib_sr::event;
use lib_sr::event::Event;
use lib_sr::sqlite;
use lib_sr::Config;
use lib_sr::{common, errors::*, json_schema};

use crate::embedded;

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
    event::ensure_hash(&mut event)?;
    if event.r#type == "label" {
        labels.insert(event.hash.as_ref().unwrap().to_string(), event.clone());
    } else if event.r#type == "label-answer" {
        let data = event.data.as_ref().expect("data");
        let label_hash = data.get("label").expect("label").as_str().expect("string");
        let label = match labels.get(label_hash) {
            Some(lbl) => Ok(lbl),
            None => {
                debug!("prep_event Label not found with hash: {}", label_hash);
                debug!("prep_event event: {:?}", event);
                debug!("prep_event labels: {:?}", labels);
                Err(format!("Label not found with hash: {}", label_hash))
            }
        }?;
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

fn run_remote(config: &Config, in_events: impl Iterator<Item = Result<Event>>) -> Result<()> {
    let mut hashes = HashSet::new();
    let mut labels = HashMap::new();
    let client = Client::new();
    let url = embedded::api_route(&config.db, "upload");

    for result in in_events {
        let event = prep_event(&mut labels, result)?;
        let hash = event.hash.clone().expect("Hash not set");

        if !hashes.contains(&hash) && event.r#type != "control" || config.sink_all_events {
            let json = serde_json::to_string(&event).chain_err(|| "Serialization failed")?;
            let mut request = client
                .post(&url)
                .header("Content-Type", "application/json")
                .body(json);

            if let Ok(token) = env::var("SRVC_TOKEN") {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            info! {"Sending event to remote: {} {}", event.r#type, event.hash.expect("hash")};
            let response = request
                .send()
                .chain_err(|| "Error sending event to remote")?;
            let status = response.status().as_u16();
            debug! {"Received {} response from remote", status};

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

fn run_local_jsonl(config: &Config, in_events: impl Iterator<Item = Result<Event>>) -> Result<()> {
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
    let mut writer = LineWriter::new(db_file);

    for result in in_events {
        let event = prep_event(&mut labels, result)?;
        let hash = event.hash.clone().expect("Hash not set");

        if !hashes.contains(&hash) && event.r#type != "control" || config.sink_all_events {
            info! {"Writing event to sink: {} {}", event.r#type, hash};
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

fn run_local_sqlite(config: &Config, in_events: impl Iterator<Item = Result<Event>>) -> Result<()> {
    let mut labels = HashMap::new();
    let conn = sqlite::open(&PathBuf::from(&config.db))?;

    for result in in_events {
        let event = prep_event(&mut labels, result)?;

        if event.r#type != "control" || config.sink_all_events {
            info! {"Writing event to sink: {} {}", event.r#type, event.hash.to_owned().expect("hash")};
            sqlite::insert_event(&conn, event)?;
        }
    }

    sqlite::close(conn)
}

pub fn run_with_events(
    config: &Config,
    in_events: impl Iterator<Item = Result<Event>>,
) -> Result<()> {
    if embedded::is_remote_target(&config.db) {
        run_remote(config, in_events)
    } else if common::has_sqlite_ext(&config.db) {
        run_local_sqlite(config, in_events)
    } else {
        run_local_jsonl(config, in_events)
    }
}

pub fn run() -> Result<()> {
    debug! {"Starting sink step"};
    let env = embedded::get_env().chain_err(|| "Env var processing failed")?;
    let config = embedded::get_config(&env.config)?;
    let input_addr = env.input.ok_or("Missing value for SR_INPUT")?;
    let in_events = embedded::input_events(&input_addr)?;
    run_with_events(&config, in_events)
}
