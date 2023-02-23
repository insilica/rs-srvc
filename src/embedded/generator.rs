use std::collections::{HashMap, HashSet};
use std::path::PathBuf;

use reqwest::blocking::Client;
use rusqlite::Connection;
use serde_json::{json, Value};
use url::Url;

use lib_sr::event;
use lib_sr::event::Event;
use lib_sr::sqlite;
use lib_sr::{common, errors::*};
use lib_sr::{Config, Label};

use crate::embedded;
use crate::embedded::GeneratorContext;

const SELECT_DOCUMENTS: &str = "SELECT data, extra, hash, type, uri FROM srvc_event WHERE type = 'document' ORDER BY uri NULLS LAST, hash";
const SELECT_LABELS: &str = "SELECT data, extra, hash, type, uri FROM srvc_event WHERE type = 'label' ORDER BY data->>'$.id', hash";
const SELECT_LABEL_ANSWERS_FOR_DOC: &str = "SELECT data, extra, hash, type, uri FROM srvc_event WHERE type = 'label-answer' AND data->>'$.document' = ? ORDER BY data->>'$.timestamp', hash";

fn get_label_events(config: &Config) -> Result<Vec<Event>> {
    let mut labels: Vec<&Label> = config.labels.values().collect();
    // Provide a consistent ordering for consumers
    labels.sort_by(|a, b| a.id.cmp(&b.id));
    let mut events = Vec::new();

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
        events.push(event);
    }

    Ok(events)
}

fn run_jsonl<F>(file_or_url: &str, config: &Config, f: &mut F) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    let (reader, _, _) = embedded::get_file_or_url(Client::new(), file_or_url)?;
    let in_events = event::events(reader);

    for event in get_label_events(config)? {
        f(event)?;
    }

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
            f(event)?;
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
        f(event.clone())?;
        if event.r#type == "document" {
            match answers.get(&event.hash.to_owned().expect("hash")) {
                Some(doc_answers) => {
                    for answer in doc_answers {
                        f(answer.clone())?;
                    }
                }
                None => {}
            }
        }
    }

    Ok(())
}

pub fn write_labels_sqlite<F>(conn: &Connection, config: &Config, f: &mut F) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    let mut label_hashes = HashSet::new();
    for event in get_label_events(config)? {
        let hash = event.hash.to_owned().expect("hash");
        f(event)?;
        label_hashes.insert(hash);
    }

    let mut stmt = sqlite::prepare_cached(&conn, SELECT_LABELS)?;
    let mut rows = stmt
        .query([])
        .chain_err(|| format!("Failed to execute prepared statement: {}", SELECT_LABELS))?;
    while let Some(row) = rows.next().chain_err(|| "Failed to get next row")? {
        let event = sqlite::parse_event(row)?;
        if !label_hashes.contains(&event.hash.to_owned().expect("hash")) {
            f(event)?;
        }
    }
    Ok(())
}

fn write_document_answers_sqlite<F>(conn: &Connection, f: &mut F, doc_hash: &str) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    let mut stmt = sqlite::prepare_cached(&conn, SELECT_LABEL_ANSWERS_FOR_DOC)?;
    let mut rows = stmt.query([doc_hash]).chain_err(|| {
        format!(
            "Failed to execute prepared statement: {}",
            SELECT_LABEL_ANSWERS_FOR_DOC
        )
    })?;
    while let Some(row) = rows.next().chain_err(|| "Failed to get next row")? {
        let event = sqlite::parse_event(row)?;
        f(event)?;
    }
    Ok(())
}

pub fn write_documents_sqlite<F>(conn: &Connection, f: &mut F) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    let mut stmt = sqlite::prepare_cached(&conn, SELECT_DOCUMENTS)?;
    let mut rows = stmt
        .query([])
        .chain_err(|| format!("Failed to execute prepared statement: {}", SELECT_DOCUMENTS))?;
    while let Some(row) = rows.next().chain_err(|| "Failed to get next row")? {
        let event = sqlite::parse_event(row)?;
        f(event.clone())?;
        write_document_answers_sqlite(&conn, f, &event.hash.expect("hash"))?;
    }
    Ok(())
}

pub fn run_sqlite<F>(file: &str, config: &Config, f: &mut F) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    let conn = sqlite::open_ro(&PathBuf::from(file))?;

    write_labels_sqlite(&conn, config, f)?;
    write_documents_sqlite(&conn, f)?;

    sqlite::close(conn)?;
    Ok(())
}

pub fn run_f<F>(file_or_url: &str, config: &Config, f: &mut F) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    match Url::parse(file_or_url) {
        Ok(_) => run_jsonl(file_or_url, config, f),
        Err(_) => {
            if common::has_sqlite_ext(file_or_url) {
                run_sqlite(file_or_url, config, f)
            } else {
                run_jsonl(file_or_url, config, f)
            }
        }
    }
}

pub fn run(file_or_url: &str) -> Result<()> {
    let GeneratorContext { config, mut writer } = embedded::get_generator_context()?;
    let mut hashes = HashSet::new();
    match Url::parse(file_or_url) {
        Ok(_) => {
            let mut f_dedupe =
                |event| embedded::write_event_dedupe(&mut writer, &event, &mut hashes);
            run_jsonl(file_or_url, &config, &mut f_dedupe)
        }
        Err(_) => {
            if common::has_sqlite_ext(file_or_url) {
                let mut f = |event| embedded::write_event(&mut writer, &event);
                run_sqlite(file_or_url, &config, &mut f)
            } else {
                let mut f_dedupe =
                    |event| embedded::write_event_dedupe(&mut writer, &event, &mut hashes);
                run_jsonl(file_or_url, &config, &mut f_dedupe)
            }
        }
    }
}
