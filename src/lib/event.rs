use std::collections::BTreeMap;
use std::io::BufRead;

use anyhow::{Context, Error, Result};
use log::trace;
use log::warn;
use multihash::MultihashDigest;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Event {
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    pub hash: Option<String>,
    pub r#type: String,
    pub uri: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct LabelAnswerData {
    pub answer: serde_json::Value,
    #[serde(alias = "document")]
    pub event: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    pub label: String,
    pub reviewer: String,
    pub timestamp: u64,
}

pub fn event_hash(mut event: Event) -> Result<String> {
    event.hash = None;
    let bytes = serde_ipld_dagcbor::to_vec(&event).with_context(|| "Failed to serialize event")?;
    let hash = multihash::Code::Sha2_256.digest(&bytes);
    let base58 = bs58::encode(hash.to_bytes());
    Ok(base58.into_string())
}

pub fn process_event_data(mut event: Event) -> Result<Event> {
    if event.r#type == "label-answer" {
        // Check that label-answers have the required data
        match event.data {
            Some(data) => {
                let answer_data: LabelAnswerData = serde_json::from_value(data.clone())
                    .with_context(|| "Failed to parse label-answer data")?;
                // Canonicalize label-answer data
                event.data = Some(
                    serde_json::to_value(answer_data)
                        .with_context(|| "Failed to serialize label-answer data")?,
                );
                // Update hashes of legacy answers that use document instead of event
                if data.as_object().expect("data").contains_key("document") {
                    let old_hash = event.hash.expect("old hash");
                    warn!(
                        "label-answer {} has deprecated document property instead of event",
                        old_hash
                    );
                    event.hash = None;
                    ensure_hash(&mut event)?;
                    warn!(
                        "Updated label-answer {}. New hash: {}",
                        old_hash,
                        event.hash.clone().expect("new hash")
                    );
                }
                Ok(event)
            }
            None => Err(Error::msg("label-answer must have data")),
        }
    } else {
        Ok(event)
    }
}

pub fn parse_event(s: &str) -> Result<Event> {
    match serde_json::from_str(s) {
        Ok(event) => process_event_data(event),
        Err(e) => {
            if s.len() == 0 {
                Err(e).with_context(|| "Event deserialization failed (blank line)")
            } else {
                Err(e).with_context(|| "Event deserialization failed")
            }
        }
    }
}

pub fn parse_event_opt(s: &str) -> Result<Option<Event>> {
    match serde_json::from_str(s) {
        Ok(event) => Ok(Some(process_event_data(event)?)),
        Err(e) => {
            if s.len() == 0 {
                Ok(None)
            } else {
                Err(e).with_context(|| "Event deserialization failed")
            }
        }
    }
}

pub fn events(reader: impl BufRead) -> impl Iterator<Item = Result<Event>> {
    reader
        .lines()
        .map(|line| match line.with_context(|| "Failed to read line") {
            Ok(line_str) => match parse_event_opt(&line_str) {
                Ok(parsed_line) => Ok(parsed_line),
                Err(e) => {
                    trace! {"Failed to parse line as JSON: {}", line_str};
                    Err(e)
                }
            },
            Err(e) => Err(e),
        })
        // Remove blank lines
        .filter(|x| match x {
            Ok(Some(_)) => true,
            Ok(None) => false,
            Err(_) => true,
        })
        .map(|x| match x {
            Ok(Some(event)) => Ok(event),
            Ok(None) => panic!("Unexpected None"),
            Err(e) => Err(e),
        })
}

pub fn ensure_hash(event: &mut Event) -> Result<()> {
    let expected_hash = event_hash(event.clone())?;
    let hash = event.hash.clone().unwrap_or("".to_string());
    if hash == "" {
        event.hash = Some(expected_hash);
    } else if expected_hash != hash {
        return Err(Error::msg(format!(
            "Incorrect event hash. Expected: \"{}\". Found: \"{}\".",
            expected_hash, hash
        ))
        .into());
    }
    Ok(())
}
