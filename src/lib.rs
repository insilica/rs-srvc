use std::collections::HashMap;

use serde::Deserialize;
use serde::Serialize;

#[derive(Deserialize, Serialize)]
pub struct Label {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub id: String,
    pub question: String,
    pub required: bool,
    pub r#type: String,
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub labels: HashMap<String, Label>,
}

pub struct Opts {
    pub config: String,
}
