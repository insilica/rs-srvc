use std::collections::HashSet;
use std::env;
use std::time::Duration;

use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use url::Url;

use lib_sr::errors::*;
use lib_sr::event::Event;
use lib_sr::{event, Config};

use crate::embedded;
use crate::embedded::MapContext;

#[derive(Clone, Debug, Serialize)]
struct EventsRequest {
    config: Config,
    events: Vec<Event>,
}

#[derive(Clone, Debug, Deserialize)]
struct EventsResponse {
    events: Vec<Event>,
}

fn do_request(
    client: &Client,
    url: &Url,
    config: &Config,
    events: Vec<Event>,
) -> Result<Vec<Event>> {
    let er = EventsRequest {
        config: config.to_owned(),
        events,
    };

    let mut request = client.post(url.clone()).json(&er);

    if let Ok(token) = env::var("SRVC_TOKEN") {
        request = request.header("Authorization", format!("Bearer {}", token));
    }

    let response = request
        .send()
        .chain_err(|| format!("Failed to complete HTTP request to {}", url))?;
    let status = response.status().as_u16();
    if status == 200 {
        let eresp: EventsResponse = serde_json::from_str(
            &response
                .text()
                .chain_err(|| "Failed to get response text")?,
        )
        .chain_err(|| "Failed to parse response")?;
        Ok(eresp.events)
    } else {
        Err(format!("Unexpected {} status for {}", status, url).into())
    }
}

pub fn run(url_str: &str) -> Result<()> {
    let url = Url::parse(url_str).chain_err(|| format! {"Cannot parse URL: {}", url_str})?;

    let MapContext {
        config,
        in_events,
        timestamp_override: _,
        mut writer,
    } = embedded::get_map_context()?;
    let mut hashes = HashSet::new();
    let mut events = Vec::new();
    let client = Client::builder()
        .timeout(Duration::from_secs(300)) // multi-step LLM requests can take a long time
        .build()
        .chain_err(|| "Failed to build reqwest Client")?;

    // Write leading non-docs, then
    // group each document and its related events and send off a request, writing
    // the response body as events.
    for result in in_events {
        let event = result?;

        if event.r#type == "document" {
            if events.len() != 0 {
                for mut ev in do_request(&client, &url, &config, events)? {
                    event::ensure_hash(&mut ev)?;
                    // Write events from response
                    embedded::write_event_dedupe(&mut writer, &ev, &mut hashes)?;
                }
            }
            events = vec![event];
        } else if events.len() == 0 {
            // Write leading non-docs
            embedded::write_event_dedupe(&mut writer, &event, &mut hashes)?;
        } else {
            events.push(event);
        }
    }

    if events.len() != 0 {
        for mut ev in do_request(&client, &url, &config, events)? {
            event::ensure_hash(&mut ev)?;
            // Write events from response
            embedded::write_event_dedupe(&mut writer, &ev, &mut hashes)?;
        }
    }

    Ok(())
}
