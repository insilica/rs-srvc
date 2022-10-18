use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Step {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub labels: Vec<String>,
    pub run: Option<String>,
    pub run_embedded: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Flow {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub steps: Vec<Step>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Label {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub hash: Option<String>,
    pub id: String,
    pub json_schema: Option<serde_json::Value>,
    pub question: String,
    pub required: bool,
    pub r#type: String,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub current_labels: Option<Vec<Label>>,
    pub current_step: Option<Step>,
    pub db: String,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub flows: HashMap<String, Flow>,
    pub labels: HashMap<String, Label>,
    pub reviewer: String,
    pub sink_all_events: bool,
}

pub struct Opts {
    pub config: String,
}
