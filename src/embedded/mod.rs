use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

use serde::Deserialize;

pub mod remove_reviewed;

#[derive(Deserialize)]
pub struct Config {
    db: Option<String>,
}

pub struct Env {
    config: PathBuf,
    input: Option<PathBuf>,
    output: Option<PathBuf>,
}

pub fn get_config(
    filename: PathBuf,
) -> Result<Config, Box<dyn std::error::Error>> {
    let file = File::open(filename)?;
    let reader = BufReader::new(file);
    let result = serde_json::from_reader(reader).or_else(|err| Err(Box::new(err)));

    // Force conversion to the correct return type
    // https://stackoverflow.com/a/57426673
    return result.or_else(|err| Err(Box::new(err) as Box<dyn std::error::Error>));
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
