use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::*;
use crate::event::Event;

pub mod remove_reviewed;
pub mod sink;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    db: String,
    reviewer: String,
}

#[derive(Debug)]
pub struct Env {
    config: PathBuf,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

pub fn get_config(filename: PathBuf) -> Result<Config> {
    let file = File::open(filename).chain_err(|| "Cannot open config file")?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).chain_err(|| "Cannot parse config as JSON")
}

pub fn get_env() -> Result<Env> {
    let config =
        PathBuf::from(env::var("SR_CONFIG").chain_err(|| "SR_CONFIG is not a valid file path")?);
    let input = env::var("SR_INPUT").ok().map(|s| PathBuf::from(s));
    let output = env::var("SR_OUTPUT").ok().map(|s| PathBuf::from(s));

    Ok(Env {
        config,
        input,
        output,
    })
}

pub fn parse_event(s: &str) -> Result<Event> {
    serde_json::from_str(s).chain_err(|| "Cannot parse event")
}

pub fn events(reader: BufReader<File>) -> impl Iterator<Item = Result<Event>> {
    reader
        .lines()
        .map(|line| parse_event(line.chain_err(|| "Failed to read line")?.as_str()))
}
