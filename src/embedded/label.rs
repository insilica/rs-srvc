use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::json;

use crate::embedded;
use crate::embedded::MapContext;
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
        json!(doc.hash.as_ref().expect("Document must have a hash")),
    );
    data.insert(
        String::from("label"),
        json!(label.hash.as_ref().expect("Label must have a hash")),
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

fn print_doc(opts: &mut Opts, doc: &Event) -> Result<()> {
    serde_json::to_writer_pretty(&mut opts.out_stream, &doc.data)
        .chain_err(|| "Document write failed")?;
    match &doc.uri {
        Some(s) => write!(opts.out_stream, "\n{}", s).chain_err(|| "Document write failed")?,
        None => {}
    }
    write!(opts.out_stream, "\n\n").chain_err(|| "Document write failed")?;
    opts.out_stream.flush().chain_err(|| "Flush failed")?;
    Ok(())
}

pub fn run(opts: &mut Opts) -> Result<()> {
    let MapContext {
        config,
        in_events,
        mut writer,
    } = embedded::get_map_context()?;
    let labels = config.current_labels.unwrap_or(Vec::new());
    let reviewer = config.reviewer;

    for result in in_events {
        let event = result?;
        embedded::write_event(&mut writer, &event)?;

        if event.r#type == "document" {
            print_doc(opts, &event)?;
            for label in &labels {
                let mut answer = read_answer(opts, label, &event, reviewer.clone())?;
                answer.hash = Some(event::event_hash(answer.clone())?);
                embedded::write_event(&mut writer, &answer)?;
            }
        }
    }

    Ok(())
}
