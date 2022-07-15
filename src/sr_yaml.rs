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
#[derive(Deserialize, Serialize)]
pub struct Step {
    #[serde(flatten)]
    extra: HashMap<String, serde_yaml::Value>,
    run: Option<String>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
pub struct Flow {
    #[serde(flatten)]
    extra: HashMap<String, serde_yaml::Value>,
    steps: Option<Vec<Step>>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
pub struct Label {
    #[serde(flatten)]
    extra: HashMap<String, serde_yaml::Value>,
    question: Option<String>,
    required: Option<bool>,
    r#type: Option<String>,
}

#[skip_serializing_none]
#[derive(Deserialize, Serialize)]
pub struct Config {
    db: Option<String>,
    #[serde(flatten)]
    extra: HashMap<String, serde_yaml::Value>,
    flows: Option<HashMap<String, Flow>>,
    labels: Option<HashMap<String, Label>>,
    reviewer: Option<String>,
}

pub fn non_blank(id: & str, k: &str, s: Option<String>) -> Result<String> {
    match s {
        Some(s) => {
            let s = s.trim();
            if s.is_empty() {
                Err(format!("The {} label has a blank {}", id, k).into())
            } else {
                Ok(s.to_string())
            }
        },
        None => Err(format!("The {} label does not have a {}", id, k).into())
    }
}

pub fn parse_label(id: &str, label: Label) -> Result<lib::Label> {
    Ok(lib::Label {
        extra: label.extra,
        id: id.to_string(),
        question: non_blank(id, "question", label.question)?.to_lowercase(),
        required: true,
        r#type: non_blank(id, "type", label.r#type)?.to_lowercase(),
    })
}

pub fn parse_labels(labels: Option<HashMap<String, Label>>) -> Result<HashMap<String, lib::Label>> {
    match labels {
        Some(labels) => {
            let mut m = HashMap::new();
            for (id, label) in labels {
                let parsed = parse_label(&id, label)?;
                m.insert(id, parsed);
            }
            Ok(m)
        }
        None => Ok(HashMap::new()),
    }
}

pub fn parse_config(config: Config) -> Result<lib::Config> {
    Ok(lib::Config {
        extra: config.extra,
        labels: parse_labels(config.labels)?,
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

pub fn run(opts: lib::Opts, _name: String) -> Result<()> {
    let yaml_config = get_config(PathBuf::from(opts.config))?;
    let config = parse_config(yaml_config)?;
    println!("{}", serde_yaml::to_string(&config).chain_err(|| "")?);
    Ok(())
}
