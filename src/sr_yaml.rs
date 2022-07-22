use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_with::skip_serializing_none;

use crate::errors::*;
use crate::lib;

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Step {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub labels: Option<Vec<String>>,
    pub run: Option<String>,
    pub run_embedded: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Flow {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub steps: Option<Vec<Step>>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Label {
    #[serde(flatten)]
    extra: HashMap<String, serde_yaml::Value>,
    question: Option<String>,
    required: Option<bool>,
    r#type: Option<String>,
}

#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
    pub db: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_yaml::Value>,
    pub flows: Option<HashMap<String, Flow>>,
    pub labels: Option<HashMap<String, Label>>,
    pub reviewer: Option<String>,
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

    let add_hashes_step = lib::Step {
        extra: HashMap::new(),
        labels: Vec::new(),
        run: None,
        run_embedded: Some(String::from("add-hashes")),
    };

    let num_steps = steps.len();
    let mut i = 0;
    let mut vec = Vec::new();
    for step in steps {
        i += 1;
        let step = parse_step(step.to_owned())?;
        vec.push(step);
        if i != num_steps {
            vec.push(add_hashes_step.clone());
        }
    }
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

pub fn parse_label(id: &str, label: &Label) -> Result<lib::Label> {
    Ok(lib::Label {
        extra: label.extra.to_owned(),
        id: id.to_string(),
        question: non_blank(id, "question", &label.question)?.to_lowercase(),
        required: true,
        r#type: non_blank(id, "type", &label.r#type)?.to_lowercase(),
    })
}

pub fn parse_labels(
    labels: &Option<HashMap<String, Label>>,
) -> Result<HashMap<String, lib::Label>> {
    match labels {
        Some(labels) => {
            let mut m = HashMap::new();
            for (id, label) in labels {
                let parsed = parse_label(&id, label)?;
                m.insert(id.to_owned(), parsed);
            }
            Ok(m)
        }
        None => Ok(HashMap::new()),
    }
}

pub fn parse_config(config: Config) -> Result<lib::Config> {
    Ok(lib::Config {
        current_labels: None,
        current_step: None,
        db: config.db.ok_or("\"db\" not set in config")?,
        extra: config.extra,
        flows: parse_flows(config.flows)?,
        labels: parse_labels(&config.labels)?,
        reviewer: config.reviewer.ok_or("\"reviewer\" not set in config")?,
    })
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
