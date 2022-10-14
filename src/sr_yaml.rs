use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use reqwest::blocking::Client;
use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;
use url::Url;

use crate::errors::*;
use crate::event;
use crate::json_schema;
use crate::lib;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Step {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub labels: Option<Vec<String>>,
    pub run: Option<String>,
    pub run_embedded: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Flow {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub steps: Option<Vec<Step>>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Label {
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
    json_schema_url: Option<String>,
    question: Option<String>,
    required: Option<bool>,
    r#type: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub db: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
    pub flows: Option<HashMap<String, Flow>>,
    pub labels: Option<HashMap<String, Label>>,
    pub reviewer: Option<String>,
    pub sink_all_events: Option<bool>,
}

pub fn non_blank(id: &str, k: &str, s: &Option<String>) -> Result<String> {
    match s {
        Some(s) => {
            let s = s.trim();
            if s.is_empty() {
                Err(format!("The {} label has a blank {}", id, k).into())
            } else {
                Ok(s.to_string())
            }
        }
        None => Err(format!("The {} label does not have a {}", id, k).into()),
    }
}

pub fn parse_step(step: Step) -> Result<lib::Step> {
    Ok(lib::Step {
        extra: step.extra,
        labels: step.labels.unwrap_or(Vec::new()),
        run: step.run,
        run_embedded: step.run_embedded,
    })
}

pub fn parse_flow(flow: Flow) -> Result<lib::Flow> {
    let steps = &mut flow.steps.unwrap_or(Vec::new());
    if steps.len() == 0 {
        return Err("No steps in flow".into());
    }

    let mut vec = Vec::new();
    for step in steps {
        let step = parse_step(step.to_owned())?;
        vec.push(step);
    }
    vec.push(lib::Step {
        extra: HashMap::new(),
        labels: Vec::new(),
        run: None,
        run_embedded: Some(String::from("sink")),
    });
    Ok(lib::Flow {
        extra: flow.extra,
        steps: vec,
    })
}

pub fn parse_flows(flows: Option<HashMap<String, Flow>>) -> Result<HashMap<String, lib::Flow>> {
    let flows = flows.unwrap_or(HashMap::new());
    let mut m = HashMap::new();
    for (flow_name, flow) in flows {
        let flow = parse_flow(flow)?;
        m.insert(flow_name, flow);
    }
    Ok(m)
}

pub fn parse_label(
    id: &str,
    label: &Label,
    json_schema: Option<serde_json::Value>,
) -> Result<lib::Label> {
    let mut label = lib::Label {
        extra: label.extra.clone(),
        hash: None,
        id: id.to_string(),
        json_schema,
        question: non_blank(id, "question", &label.question)?.to_lowercase(),
        required: label.required.unwrap_or(false),
        r#type: non_blank(id, "type", &label.r#type)?.to_lowercase(),
    };
    let data_s = serde_json::to_string(&label).chain_err(|| "Serialization failed")?;
    let data = serde_json::from_str(&data_s).chain_err(|| "Deserialization failed")?;
    let event = event::Event {
        data: data,
        extra: HashMap::new(),
        hash: None,
        r#type: String::from("label"),
        uri: None,
    };
    label.hash = Some(event::event_hash(event)?);
    Ok(label)
}

pub fn get_label_schema(client: &Client, label: &Label) -> Result<Option<serde_json::Value>> {
    let json = label
        .json_schema_url
        .as_ref()
        .map(|url| json_schema::get_schema_for_url(client, &url))
        .transpose()?;
    // Fail fast if an invalid schema is supplied
    match json.as_ref() {
        Some(v) => Some(json_schema::compile(v)?),
        None => None,
    };
    Ok(json)
}

pub fn parse_labels(
    labels: &Option<HashMap<String, Label>>,
) -> Result<HashMap<String, lib::Label>> {
    match labels {
        Some(labels) => {
            let client = Client::new();
            let mut m = HashMap::new();
            for (id, label) in labels {
                let json_schema = get_label_schema(&client, &label)?;
                let parsed = parse_label(&id, label, json_schema)?;
                m.insert(id.to_owned(), parsed);
            }
            Ok(m)
        }
        None => Ok(HashMap::new()),
    }
}

pub fn parse_config(config: Config) -> Result<lib::Config> {
    let reviewer = config.reviewer.ok_or("\"reviewer\" not set in config")?;
    let mut reviewer_err = String::from("\"reviewer\" is not a valid URI: ");
    reviewer_err.push_str(&format!("{:?}", reviewer));
    let reviewer_uri = Url::parse(&reviewer);
    match reviewer_uri {
        Ok(_) => Ok(lib::Config {
            current_labels: None,
            current_step: None,
            db: config.db.ok_or("\"db\" not set in config")?,
            extra: config.extra,
            flows: parse_flows(config.flows)?,
            labels: parse_labels(&config.labels)?,
            reviewer,
            sink_all_events: config.sink_all_events.unwrap_or(false),
        }),
        Err(_) => {
            if !reviewer.contains(":") && reviewer.contains("@") && reviewer.contains(".") {
                let mut try_reviewer = String::from("mailto:");
                try_reviewer.push_str(&reviewer);
                reviewer_err.push_str(&format!("\n  Try {:?}", try_reviewer));
            }
            Err(reviewer_err.into())
        }
    }
}

pub fn get_config(filename: PathBuf) -> Result<Config> {
    let file = File::open(filename.clone())
        .chain_err(|| format!("Failed to open config file: {}", filename.to_string_lossy()))?;
    let reader = BufReader::new(file);
    serde_yaml::from_reader(reader).chain_err(|| {
        format!(
            "Failed to parse config file as YAML: {}",
            filename.to_string_lossy()
        )
    })
}
