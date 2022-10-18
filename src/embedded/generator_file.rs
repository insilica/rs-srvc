use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;
use serde_json::{json, Value};

use lib_sr::Label;

use crate::embedded;
use crate::embedded::GeneratorContext;
use crate::errors::*;
use crate::event;
use crate::event::Event;

pub fn run(filename: PathBuf) -> Result<()> {
    let GeneratorContext { config, mut writer } = embedded::get_generator_context()?;
    let input = File::open(&filename)
        .chain_err(|| format!("Cannot open generator file: {:?}", filename))?;
    let reader = BufReader::new(input);
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
        event
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .chain_err(|| "Event serialization failed")?;
        writer.write(b"\n").chain_err(|| "Buffer write failed")?;
    }

    for result in in_events {
        let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
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
        event
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .chain_err(|| "Event serialization failed")?;
        writer.write(b"\n").chain_err(|| "Buffer write failed")?;
    }

    Ok(())
}
