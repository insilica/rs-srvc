use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read};

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
    serde_json::from_str(s).chain_err(|| "Event deserialization failed")
}

pub fn events(reader: BufReader<impl Read>) -> impl Iterator<Item = Result<Event>> {
    reader
        .lines()
        .map(|line| parse_event(line.chain_err(|| "Failed to read line")?.as_str()))
}
