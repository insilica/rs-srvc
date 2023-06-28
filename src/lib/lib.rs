#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

use std::collections::BTreeMap;

use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

pub mod common;
pub mod event;
pub mod flow;
pub mod json_schema;
pub mod sqlite;
pub mod sr_yaml;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Step {
    pub env: Option<Vec<String>>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    pub labels: Vec<String>,
    pub run: Option<String>,
    #[serde(rename = "run-embedded")]
    pub run_embedded: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Flow {
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    pub steps: Vec<Step>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Label {
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    pub hash: Option<String>,
    pub id: String,
    #[serde(rename = "json-schema")]
    pub json_schema: Option<serde_json::Value>,
    pub question: String,
    pub required: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Source {
    pub step: Step,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Srvc {
    pub version: String,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(rename = "current-labels")]
    pub current_labels: Option<Vec<Label>>,
    #[serde(rename = "current-step")]
    pub current_step: Option<Step>,
    pub db: String,
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_json::Value>,
    pub flows: BTreeMap<String, Flow>,
    pub labels: BTreeMap<String, Label>,
    pub reviewer: Option<String>,
    #[serde(
        alias = "sink-all-events",
        alias = "sink_all_events",
        rename = "sink-control-events"
    )]
    pub sink_control_events: bool,
    pub sources: Vec<Source>,
    pub srvc: Srvc,
}

pub struct Opts {
    pub config: String,
}
