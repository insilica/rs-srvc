use std::collections::HashMap;

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
