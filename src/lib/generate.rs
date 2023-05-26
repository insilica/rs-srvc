use std::collections::HashMap;
use std::path::PathBuf;

use log::trace;
use reqwest::blocking::Client;
use rusqlite::Connection;
use serde_json::{json, Value};
use url::Url;

use crate::event;
use crate::event::Event;
use crate::sqlite;
use crate::{common, errors::*};
use crate::{Config, Label};

const SELECT_DOCUMENTS: &str = "SELECT data, extra, hash, type, uri FROM srvc_event WHERE type = 'document' ORDER BY uri NULLS LAST, hash";
const SELECT_LABELS: &str = "SELECT data, extra, hash, type, uri FROM srvc_event WHERE type = 'label' ORDER BY data->>'$.id', hash";
const SELECT_LABEL_ANSWERS_FOR_DOC: &str = "SELECT data, extra, hash, type, uri FROM srvc_event WHERE type = 'label-answer' AND data->>'$.event' = ? ORDER BY data->>'$.timestamp', hash";

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

pub fn run_jsonl<F>(file_or_url: &str, config: &Config, f: &mut F) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    trace! {"run_jsonl"};
    let (reader, _, _) = common::get_file_or_url(Client::new(), file_or_url)?;
    let in_events = event::events(reader);

    let mut answers: HashMap<String, Vec<Event>> = HashMap::new();
    let mut labels: HashMap<String, Event> = HashMap::new();
    let mut events: Vec<Event> = Vec::new();

    for event in get_label_events(config)? {
        labels.insert(event.hash.clone().expect("hash"), event);
    }

    for (i, result) in in_events.enumerate() {
        let mut event = match result {
            Ok(evt) => {
                trace! {"Parsed event: {} {}", evt.r#type, evt.hash.to_owned().unwrap_or(String::from("No hash"))};
                evt
            }
            Err(e) => {
                trace! {"run_jsonl event parse error"};
                Err(e).chain_err(|| format!("Cannot parse line {} as JSON", i + 1))?
            }
        };
        let expected_hash = event::event_hash(event.clone())?;
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
            labels.insert(event.hash.clone().expect("hash"), event);
        } else if event.r#type == "label-answer" {
            match &event.data {
                Some(data) => match data.get("event") {
                    Some(event_hash) => {
                        let hash = event_hash.as_str().expect("str").to_owned();
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
                            "label-answer is missing the \"data.event\" property. Event hash: {}",
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

    // Emit all labels before any label-answers
    // Because a label-answer can depend on two different labels, it is not
    // always possible to group every label with all of its answers.
    let mut label_hashes: Vec<String> = Vec::new();
    let mut lvec: Vec<_> = labels.into_iter().collect();
    lvec.sort_by_key(|(k, _)| k.to_owned());
    for (_, event) in lvec {
        let event_hash = event.hash.clone().expect("hash");
        label_hashes.push(event_hash);
        f(event)?;
    }

    for event_hash in label_hashes {
        write_event_answers_jsonl(&event_hash, &answers, f)?;
    }

    for event in events {
        let event_hash = event.hash.clone().expect("hash");
        f(event)?;
        write_event_answers_jsonl(&event_hash, &answers, f)?;
    }

    Ok(())
}

fn write_event_answers_jsonl<F>(
    hash: &str,
    answers: &HashMap<String, Vec<Event>>,
    f: &mut F,
) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    match answers.get(hash) {
        Some(event_answers) => {
            for answer in event_answers {
                let answer_hash = answer.hash.clone().expect("hash");
                f(answer.to_owned())?;
                write_event_answers_jsonl(&answer_hash, &answers, f)?;
            }
        }
        None => {}
    };
    Ok(())
}

pub fn write_labels_sqlite<F>(conn: &Connection, config: &Config, f: &mut F) -> Result<()>
where
    F: FnMut(Event) -> Result<()>,
{
    let mut labels: HashMap<String, Event> = HashMap::new();
    for event in get_label_events(config)? {
        labels.insert(event.hash.clone().expect("hash"), event);
    }

    let mut stmt = sqlite::prepare_cached(&conn, SELECT_LABELS)?;
    let mut rows = stmt
        .query([])
        .chain_err(|| format!("Failed to execute prepared statement: {}", SELECT_LABELS))?;
    while let Some(row) = rows.next().chain_err(|| "Failed to get next row")? {
        let event = sqlite::parse_event(row)?;
        labels.insert(event.hash.clone().expect("hash"), event);
    }

    // Emit all labels before any label-answers
    // Because a label-answer can depend on two different labels, it is not
    // always possible to group every label with all of its answers.
    let mut label_hashes: Vec<String> = Vec::new();
    let mut lvec: Vec<_> = labels.into_iter().collect();
    lvec.sort_by_key(|(k, _)| k.to_owned());
    for (_, event) in lvec {
        let event_hash = event.hash.clone().expect("hash");
        label_hashes.push(event_hash);
        f(event)?;
    }

    for event_hash in label_hashes {
        write_event_answers_sqlite(conn, f, &event_hash)?;
    }

    Ok(())
}

fn write_event_answers_sqlite<F>(conn: &Connection, f: &mut F, doc_hash: &str) -> Result<()>
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
        let hash = event.hash.clone();
        f(event)?;
        write_event_answers_sqlite(conn, f, &hash.expect("hash"))?;
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
        write_event_answers_sqlite(&conn, f, &event.hash.expect("hash"))?;
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
