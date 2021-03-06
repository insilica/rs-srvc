use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, LineWriter, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::json;

use crate::embedded;
use crate::errors::*;
use crate::event;
use crate::event::Event;
use crate::lib::{Label, Opts};

fn get_epoch_sec() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .chain_err(|| "Failed to calculate timestamp")?
        .as_secs())
}

fn answer_data(label: &Label, doc: &Event, reviewer: String) -> HashMap<String, serde_json::Value> {
    let mut data: HashMap<String, serde_json::Value> = HashMap::new();
    data.insert(
        String::from("document"),
        json!(doc.hash.as_ref().expect("Document mush have a hash")),
    );
    data.insert(
        String::from("label"),
        json!(label.hash.as_ref().expect("Label mush have a hash")),
    );
    data.insert(String::from("reviewer"), json!(reviewer));
    data
}

fn read_boolean(opts: &mut Opts, label: &Label, doc: &Event, reviewer: String) -> Result<Event> {
    let out = &mut opts.out_stream;
    let mut reader = BufReader::new(&mut opts.in_stream);
    write!(out, "{} ", label.question).chain_err(|| "Write failed")?;

    let mut data = answer_data(label, doc, reviewer);
    let mut event = Event {
        data: None,
        extra: HashMap::new(),
        hash: None,
        r#type: String::from("label-answer"),
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
            data.insert(String::from("timestamp"), json!(get_epoch_sec()?));
            event.data = Some(json!(data));
            break Ok(event);
        } else if "no".starts_with(&s) {
            data.insert(String::from("answer"), json!(false));
            data.insert(String::from("timestamp"), json!(get_epoch_sec()?));
            event.data = Some(json!(data));
            break Ok(event);
        } else if !label.required && "skip".starts_with(&s) {
            data.insert(String::from("timestamp"), json!(get_epoch_sec()?));
            event.data = Some(json!(data));
            break Ok(event);
        }
    }
}

fn read_categorical(
    opts: &mut Opts,
    label: &Label,
    doc: &Event,
    reviewer: String,
) -> Result<Event> {
    let out = &mut opts.out_stream;
    let mut reader = BufReader::new(&mut opts.in_stream);
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
                if n < i {
                    let cat = &categories[n - 1];
                    data.insert(String::from("answer"), json!(cat));
                    data.insert(String::from("timestamp"), json!(get_epoch_sec()?));
                    event.data = Some(json!(data));
                    break Ok(event);
                } else if !label.required && i == n {
                    data.insert(String::from("timestamp"), json!(get_epoch_sec()?));
                    event.data = Some(json!(data));
                    break Ok(event);
                }
            }
            Err(_) => {}
        }
    }
}

fn read_string(opts: &mut Opts, label: &Label, doc: &Event, reviewer: String) -> Result<Event> {
    let out = &mut opts.out_stream;
    let mut reader = BufReader::new(&mut opts.in_stream);
    write!(out, "{}? ", label.question).chain_err(|| "Write failed")?;
    out.flush().chain_err(|| "Flush failed")?;

    let mut data = answer_data(label, doc, reviewer);
    let mut event = Event {
        data: None,
        extra: HashMap::new(),
        hash: None,
        r#type: String::from("label-answer"),
    };
    loop {
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .chain_err(|| "read_line failed")?;
        let s = line.trim();
        if !s.is_empty() {
            data.insert(String::from("answer"), json!(s));
            data.insert(String::from("timestamp"), json!(get_epoch_sec()?));
            event.data = Some(json!(data));
            break Ok(event);
        } else if !label.required {
            data.insert(String::from("timestamp"), json!(get_epoch_sec()?));
            event.data = Some(json!(data));
            break Ok(event);
        }
    }
}

fn read_answer(opts: &mut Opts, label: &Label, doc: &Event, reviewer: String) -> Result<Event> {
    if "boolean" == label.r#type {
        read_boolean(opts, label, doc, reviewer)
    } else if "categorical" == label.r#type {
        read_categorical(opts, label, doc, reviewer)
    } else if "string" == label.r#type {
        read_string(opts, label, doc, reviewer)
    } else {
        Err(format!("Unknown label type: {}", label.r#type).into())
    }
}

pub fn run(opts: &mut Opts) -> Result<()> {
    let env = embedded::get_env().chain_err(|| "Env var processing failed")?;
    let config = embedded::get_config(env.config)?;
    let labels = config.current_labels.unwrap_or(Vec::new());
    let reviewer = config.reviewer;
    let input = File::open(env.input.unwrap()).chain_err(|| "Cannot open SR_INPUT")?;
    let reader = BufReader::new(input);
    let in_events = embedded::events(reader);
    let output = OpenOptions::new()
        .write(true)
        .open(env.output.unwrap())
        .chain_err(|| "Cannot open SR_OUTPUT")?;
    let mut writer = LineWriter::new(output);
    for result in in_events {
        let event = result.chain_err(|| "Cannot parse line as JSON")?;
        event
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .chain_err(|| "Event serialization failed")?;
        writer.write(b"\n").chain_err(|| "Buffer write failed")?;

        if event.r#type == "document" {
            for label in &labels {
                let mut answer = read_answer(opts, label, &event, reviewer.clone())?;
                answer.hash = Some(event::event_hash(answer.clone())?);
                answer
                    .serialize(&mut serde_json::Serializer::new(&mut writer))
                    .chain_err(|| "Event serialization failed")?;
                writer.write(b"\n").chain_err(|| "Buffer write failed")?;
            }
        }
    }

    Ok(())
}
