use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

pub mod remove_reviewed;

#[derive(Deserialize, Serialize)]
pub struct Config {
    db: String,
    reviewer: String,
}

pub struct Env {
    config: PathBuf,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

#[derive(Deserialize, Serialize)]
pub struct Event {
    data: Option<serde_json::Value>,
    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
    hash: Option<String>,
    r#type: String,
}

pub fn get_config(filename: PathBuf) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader).or_else(|err| Err(Box::new(err)));

    // Force conversion to the correct return type
    // https://stackoverflow.com/a/57426673
    result.or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>))
}

pub fn get_env() -> Result<Env, Box<dyn std::error::Error>> {
    let config = PathBuf::from(env::var("SR_CONFIG")?);
    let input = env::var("SR_INPUT").ok().map(|s| PathBuf::from(s));
    let output = env::var("SR_OUTPUT").ok().map(|s| PathBuf::from(s));

    Ok(Env {
        config,
        input,
        output,
    })
}

pub fn parse_event(s: &str) -> Result<Event, Box<dyn std::error::Error>> {
    let result = serde_json::from_str(s).or_else(|err| Err(Box::new(err)));

    // Force conversion to the correct return type
    // https://stackoverflow.com/a/57426673
    result.or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>))
}

pub fn events(
    reader: BufReader<File>,
) -> impl Iterator<Item = Result<Event, Box<dyn std::error::Error>>> {
    reader.lines().map(|line| parse_event(line?.as_str()))
}
