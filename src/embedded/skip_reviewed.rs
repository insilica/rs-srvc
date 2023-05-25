use std::collections::HashSet;
use std::fs::File;
use std::io::BufReader;

use reqwest::blocking::Client;

use lib_sr::errors::*;
use lib_sr::event;
use lib_sr::event::Event;

use crate::embedded;
use crate::embedded::MapContext;

pub fn read_reviewed_docs(file: File, reviewer: &str) -> Result<HashSet<String>> {
    let reader = BufReader::new(file);
    let events = event::events(reader);
    let mut hashes = HashSet::new();

    for result in events {
        let event = result?;
        if event.r#type == "label-answer" {
            let data = event.data.unwrap_or(serde_json::Value::Null);
            let document = data["document"].as_str();
            let rvwr = data["reviewer"].as_str();
            if document.and(rvwr).is_some()
                && rvwr.expect("label-answer must have a reviewer") == reviewer
            {
                hashes.insert(document.unwrap().to_string());
            }
        }
    }

    Ok(hashes)
}

pub fn remote_reviewed(
    client: &Client,
    remote: &str,
    event: &Event,
    reviewer: &str,
) -> Result<bool> {
    let mut path = String::from("document/");
    path.push_str(event.hash.as_ref().expect("Event must have hash"));
    path.push_str("/label-answers");
    let url = embedded::api_route(remote, &path);
    let response = client
        .get(&url)
        .send()
        .chain_err(|| "Error checking hash existence at remote")?;
    let status = response.status().as_u16();
    if status == 200 {
        let text = response
            .text()
            .chain_err(|| "Error getting response text")?;
        for line in text.lines() {
            let answer: Event =
                serde_json::from_str(line).chain_err(|| "Error deserializing label-answer")?;
            match answer
                .data
                .expect("label-answer must have data")
                .get("reviewer")
                .unwrap_or(&serde_json::Value::Null)
                .as_str()
            {
                Some(answer_reviewer) => {
                    if reviewer == answer_reviewer {
                        return Ok(true);
                    }
                }
                None => {}
            }
        }
        Ok(false)
    } else if status == 204 || status == 404 {
        Ok(false)
    } else {
        let text = response
            .text()
            .chain_err(|| "Error getting response text")?;
        Err(format!("Unexpected {} response at {} ({})", status, &url, text).into())
    }
}

pub fn run() -> Result<()> {
    let MapContext {
        config,
        in_events,
        timestamp_override: _,
        mut writer,
    } = embedded::get_map_context()?;
    let reviewer = config.reviewer.ok_or("\"reviewer\" not set in config")?;
    let mut hashes = HashSet::new();
    let is_remote = embedded::is_remote_target(&config.db);
    let client = Client::new();

    if !is_remote {
        let db_file = File::open(&config.db);
        hashes = match db_file {
            Err(_) => hashes, // The file may not exist yet
            Ok(file) => read_reviewed_docs(file, &reviewer)?,
        };
    }

    for result in in_events {
        let event = result?;
        let hash = event.hash.clone().unwrap_or("".to_string());
        if is_remote
            && !hashes.contains(&hash)
            && remote_reviewed(&client, &config.db, &event, &reviewer)?
        {
            hashes.insert(hash.clone());
        } else {
            embedded::write_event_dedupe(&mut writer, &event, &mut hashes)?;
        }
    }

    Ok(())
}
