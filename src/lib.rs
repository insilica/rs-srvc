use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct Step {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub labels: Vec<String>,
    pub run: Option<String>,
    pub run_embedded: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Flow {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Label {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub id: String,
    pub question: String,
    pub required: bool,
    pub r#type: String,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(flatten)]
    pub current_labels: Option<Vec<Label>>,
    pub current_step: Option<Step>,
    pub db: String,
    pub extra: HashMap<String, serde_yaml::Value>,
    pub flows: HashMap<String, Flow>,
    pub labels: HashMap<String, Label>,
    pub reviewer: String,
}

pub struct Opts {
    pub config: String,
}
