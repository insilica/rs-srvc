use std::collections::HashMap;
use std::io::BufReader;
use std::io::Write;

use reqwest::blocking::Client;
use serde::Serialize;
use serde_json::{json, Value};

use lib_sr::errors::*;
use lib_sr::event;
use lib_sr::event::Event;
use lib_sr::Label;

use crate::embedded;
use crate::embedded::GeneratorContext;

pub fn run(file_or_url: &str) -> Result<()> {
    let (input, _) = embedded::get_file_or_url(Client::new(), file_or_url)?;
    let GeneratorContext { config, mut writer } = embedded::get_generator_context()?;
    let reader = BufReader::new(input.as_bytes());
    let in_events = event::events(reader);
    let mut labels: Vec<&Label> = config.labels.values().collect();
    labels.sort_by(|a, b| a.id.cmp(&b.id));

    for label in labels {
        let data: Value = serde_json::from_str(
            &serde_json::to_string(&label).chain_err(|| "Serialization failure")?,
        )
        .chain_err(|| "Deserialization failure")?;
        let mut data_m = data.as_object().unwrap().clone();
        let hash = data_m
            .get("hash")
            .expect("hash")
            .as_str()
            .map(|s| String::from(s));
        data_m.remove("hash");
        let event = Event {
            data: Some(json!(data_m)),
            extra: HashMap::new(),
            hash,
            r#type: String::from("label"),
            uri: None,
        };
        embedded::write_event(&mut writer, &event)?
    }

    for result in in_events {
        let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
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
        event
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .chain_err(|| "Event serialization failed")?;
        writer.write(b"\n").chain_err(|| "Buffer write failed")?;
    }

    Ok(())
}
