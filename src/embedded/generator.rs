use std::collections::{HashMap, HashSet};
use std::io::Write;

use reqwest::blocking::Client;
use serde_json::{json, Value};

use lib_sr::errors::*;
use lib_sr::event;
use lib_sr::event::Event;
use lib_sr::{Config, Label};

use crate::embedded;
use crate::embedded::GeneratorContext;

fn write_labels(
    writer: &mut Box<dyn Write + Send + Sync>,
    config: &Config,
) -> Result<HashSet<String>> {
    let mut label_hashes = HashSet::new();
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
        embedded::write_event_dedupe(writer, &event, &mut label_hashes)?;
    }

    Ok(label_hashes)
}

pub fn run(file_or_url: &str) -> Result<()> {
    let (reader, _, _) = embedded::get_file_or_url(Client::new(), file_or_url)?;
    let GeneratorContext { config, mut writer } = embedded::get_generator_context()?;
    let in_events = event::events(reader);
    let mut hashes = write_labels(&mut writer, &config)?;

    let mut answers: HashMap<String, Vec<Event>> = HashMap::new();
    let mut events: Vec<Event> = Vec::new();

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

        if event.r#type == "label" {
            embedded::write_event_dedupe(&mut writer, &event, &mut hashes)?
        } else if event.r#type == "label-answer" {
            match &event.data {
                Some(data) => match data.get("document") {
                    Some(doc_hash) => {
                        let hash = doc_hash.as_str().expect("str").to_owned();
                        match answers.get_mut(&hash) {
                            Some(v) => v.push(event.clone()),
                            None => {
                                let mut v = Vec::new();
                                v.push(event.clone());
                                answers.insert(hash, v);
                            }
                        }
                    }
                    None => {
                        return Err(format!(
                        "label-answer is missing the \"data.document\" property. Event hash: {}",
                        event.hash.unwrap()
                    )
                        .into())
                    }
                },
                None => {
                    return Err(format!(
                        "label-answer is missing the \"data\" property. Event hash: {}",
                        event.hash.unwrap()
                    )
                    .into())
                }
            }
        } else {
            events.push(event);
        }
    }

    for event in events {
        embedded::write_event_dedupe(&mut writer, &event, &mut hashes)?;
        if event.r#type == "document" {
            match answers.get(&event.hash.to_owned().expect("hash")) {
                Some(doc_answers) => {
                    for answer in doc_answers {
                        embedded::write_event_dedupe(&mut writer, &answer, &mut hashes)?;
                    }
                }
                None => {}
            }
        }
    }

    Ok(())
}
