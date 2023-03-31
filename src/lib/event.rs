use std::collections::HashMap;
use std::io::BufRead;

use log::trace;
use multihash::MultihashDigest;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::errors::*;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Event {
    pub data: Option<serde_json::Value>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub hash: Option<String>,
    pub r#type: String,
    pub uri: Option<String>,
}

pub fn event_hash(mut event: Event) -> Result<String> {
    event.hash = None;
    let bytes = serde_ipld_dagcbor::to_vec(&event).chain_err(|| "Failed to serialize event")?;
    let hash = multihash::Code::Sha2_256.digest(&bytes);
    let base58 = bs58::encode(hash.to_bytes());
    Ok(base58.into_string())
}

pub fn parse_event(s: &str) -> Result<Event> {
    match serde_json::from_str(s) {
        Ok(event) => Ok(event),
        Err(e) => {
            if s.len() == 0 {
                Err(e).chain_err(|| "Event deserialization failed (blank line)")
            } else {
                Err(e).chain_err(|| "Event deserialization failed")
            }
        }
    }
}

pub fn parse_event_opt(s: &str) -> Result<Option<Event>> {
    match serde_json::from_str(s) {
        Ok(event) => Ok(Some(event)),
        Err(e) => {
            if s.len() == 0 {
                Ok(None)
            } else {
                Err(e).chain_err(|| "Event deserialization failed")
            }
        }
    }
}

pub fn events(reader: impl BufRead) -> impl Iterator<Item = Result<Event>> {
    reader
        .lines()
        .map(|line| match line.chain_err(|| "Failed to read line") {
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
        return Err(format!(
            "Incorrect event hash. Expected: \"{}\". Found: \"{}\".",
            expected_hash, hash
        )
        .into());
    }
    Ok(())
}
