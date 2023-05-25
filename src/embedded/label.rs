use std::collections::{HashMap, HashSet};
use std::io;
use std::io::{BufRead, BufReader, Write};

use serde_json::json;

use lib_sr::errors::*;
use lib_sr::event;
use lib_sr::event::Event;
use lib_sr::Label;

use crate::embedded;
use crate::embedded::MapContext;

fn answer_data(label: &Label, doc: &Event, reviewer: String) -> HashMap<String, serde_json::Value> {
    let mut data: HashMap<String, serde_json::Value> = HashMap::new();
    data.insert(
        String::from("event"),
        json!(doc.hash.as_ref().expect("Document must have a hash")),
    );
    data.insert(
        String::from("label"),
        json!(label.hash.as_ref().expect("Label must have a hash")),
    );
    data.insert(String::from("reviewer"), json!(reviewer));
    data
}

fn read_boolean(
    label: &Label,
    doc: &Event,
    reviewer: String,
    timestamp_override: Option<u64>,
) -> Result<Option<Event>> {
    let out = &mut io::stdout();
    let mut reader = BufReader::new(io::stdin());
    write!(out, "{} ", label.question).chain_err(|| "Write failed")?;

    let mut data = answer_data(label, doc, reviewer);
    let mut event = Event {
        data: None,
        extra: HashMap::new(),
        hash: None,
        r#type: String::from("label-answer"),
        uri: None,
    };
    loop {
        if label.required {
            write!(out, "[Yes/No]").chain_err(|| "Write failed")?;
        } else {
            write!(out, "[Yes/No/Skip]").chain_err(|| "Write failed")?;
        };
        out.flush().chain_err(|| "Flush failed")?;
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .chain_err(|| "read_line failed")?;
        let s = line.trim().to_lowercase();
        if s.is_empty() {
        } else if "yes".starts_with(&s) {
            data.insert(String::from("answer"), json!(true));
            embedded::insert_timestamp(&mut data, timestamp_override)?;
            event.data = Some(json!(data));
            break Ok(Some(event));
        } else if "no".starts_with(&s) {
            data.insert(String::from("answer"), json!(false));
            embedded::insert_timestamp(&mut data, timestamp_override)?;
            event.data = Some(json!(data));
            break Ok(Some(event));
        } else if !label.required && "skip".starts_with(&s) {
            break Ok(None);
        }
    }
}

fn read_categorical(
    label: &Label,
    doc: &Event,
    reviewer: String,
    timestamp_override: Option<u64>,
) -> Result<Option<Event>> {
    let out = &mut io::stdout();
    let mut reader = BufReader::new(io::stdin());
    writeln!(out, "{}", label.question).chain_err(|| "Write failed")?;

    let empty_vec = Vec::new();
    let categories = match label.extra.get("categories") {
        Some(x) => x.as_array().unwrap_or(&empty_vec),
        None => &empty_vec,
    };
    let mut i = 1;
    for cat in categories {
        writeln!(out, "{}. {}", i, cat).chain_err(|| "Write failed")?;
        i += 1;
    }
    if !label.required {
        writeln!(out, "{}. Skip Question", i).chain_err(|| "Write failed")?;
    }

    let mut data = answer_data(label, doc, reviewer);
    let mut event = Event {
        data: None,
        extra: HashMap::new(),
        hash: None,
        r#type: String::from("label-answer"),
        uri: None,
    };
    loop {
        write!(out, "? ").chain_err(|| "Write failed")?;
        out.flush().chain_err(|| "Flush failed")?;
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .chain_err(|| "read_line failed")?;
        match line.trim().parse::<usize>() {
            Ok(n) => {
                if n == 0 {
                } else if n < i {
                    let cat = &categories[n - 1];
                    data.insert(String::from("answer"), json!(cat));
                    embedded::insert_timestamp(&mut data, timestamp_override)?;
                    event.data = Some(json!(data));
                    break Ok(Some(event));
                } else if !label.required && i == n {
                    break Ok(None);
                }
            }
            Err(_) => {}
        }
    }
}

fn read_string(
    label: &Label,
    doc: &Event,
    reviewer: String,
    timestamp_override: Option<u64>,
) -> Result<Option<Event>> {
    let out = &mut io::stdout();
    let mut reader = BufReader::new(io::stdin());
    write!(out, "{}? ", label.question).chain_err(|| "Write failed")?;
    out.flush().chain_err(|| "Flush failed")?;

    let mut data = answer_data(label, doc, reviewer);
    let mut event = Event {
        data: None,
        extra: HashMap::new(),
        hash: None,
        r#type: String::from("label-answer"),
        uri: None,
    };
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .chain_err(|| "read_line failed")?;
        let s = line.trim();
        if !s.is_empty() {
            data.insert(String::from("answer"), json!(s));
            embedded::insert_timestamp(&mut data, timestamp_override)?;
            event.data = Some(json!(data));
            break Ok(Some(event));
        } else if !label.required {
            break Ok(None);
        }
    }
}

fn read_answer(
    label: &Label,
    doc: &Event,
    reviewer: String,
    timestamp_override: Option<u64>,
) -> Result<Option<Event>> {
    // label.type is allowed for backwards-compatibility, but new types
    // should not be added. Use json_schema instead.
    match label.extra.get("type").map(|v| v.as_str()).flatten() {
        Some(t) => {
            let t = t.to_lowercase();
            if "boolean" == t {
                read_boolean(label, doc, reviewer, timestamp_override)
            } else if "categorical" == t {
                read_categorical(label, doc, reviewer, timestamp_override)
            } else if "string" == t {
                read_string(label, doc, reviewer, timestamp_override)
            } else {
                Err(format!("Unknown label type ({}): {}", label.id, t).into())
            }
        }
        None => Err(format!("Unknown label type ({})", label.id).into()),
    }
}

fn print_doc(doc: &Event) -> Result<()> {
    serde_json::to_writer_pretty(&mut io::stdout(), &doc.data)
        .chain_err(|| "Document write failed")?;
    match &doc.uri {
        Some(s) => write!(io::stdout(), "\n{}", s).chain_err(|| "Document write failed")?,
        None => {}
    }
    write!(io::stdout(), "\n\n").chain_err(|| "Document write failed")?;
    io::stdout().flush().chain_err(|| "Flush failed")?;
    Ok(())
}

pub fn run() -> Result<()> {
    let MapContext {
        config,
        in_events,
        timestamp_override,
        mut writer,
    } = embedded::get_map_context()?;
    let mut hashes = HashSet::new();
    let labels = config.current_labels.unwrap_or(Vec::new());
    let reviewer = config.reviewer.ok_or("\"reviewer\" not set in config")?;

    for result in in_events {
        let event = result?;
        embedded::write_event_dedupe(&mut writer, &event, &mut hashes)?;

        if event.r#type == "document" {
            print_doc(&event)?;
            for label in &labels {
                match read_answer(label, &event, reviewer.clone(), timestamp_override)? {
                    Some(mut answer) => {
                        answer.hash = Some(event::event_hash(answer.clone())?);
                        embedded::write_event_dedupe(&mut writer, &answer, &mut hashes)?;
                    }
                    None => {}
                };
            }
        }
    }

    Ok(())
}
