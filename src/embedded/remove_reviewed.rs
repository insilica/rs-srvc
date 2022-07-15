use std::collections::HashSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::LineWriter;
use std::io::Write;

use serde::Serialize;

use crate::embedded;

pub fn read_reviewed_docs(
    file: File,
    reviewer: String,
) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let reader = BufReader::new(file);
    let events = embedded::events(reader);
    let mut hashes = HashSet::new();

    for result in events {
        let event = result?;
        if event.r#type == "label-answer" {
            let data = event.data.unwrap_or(serde_json::Value::Null);
            let document = data["document"].as_str();
            let rvwr = data["reviewer"].as_str();
            if document.and(rvwr).is_some() && rvwr.unwrap() == reviewer {
                hashes.insert(document.unwrap().to_string());
            }
        }
    }

    Ok(hashes)
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let env = embedded::get_env()?;
    let config = embedded::get_config(env.config)?;
    let reviewer = config.reviewer;
    let db_file = File::open(config.db);
    let reviewed_docs = match db_file {
        Err(_) => HashSet::new(), // The file may not exist yet
        Ok(file) => read_reviewed_docs(file, reviewer)?,
    };
    let input = File::open(env.input.unwrap())?;
    let reader = BufReader::new(input);
    let in_events = embedded::events(reader);
    let output = OpenOptions::new().write(true).open(env.output.unwrap())?;
    let mut writer = LineWriter::new(output);

    for result in in_events {
        let event = result?;
        let hash = event.hash.clone().unwrap_or("".to_string());
        if !reviewed_docs.contains(&hash) {
            event.serialize(&mut serde_json::Serializer::new(&mut writer))?;
            writer.write(b"\n")?;
        }
    }

    Ok(())
}
