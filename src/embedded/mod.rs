use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{env, io};

use anyhow::{Context, Error, Result};
use reqwest::blocking::Client;
use serde_json::json;
use url::Url;

use lib_sr::event::Event;
use lib_sr::{common, event, Config};

pub mod generator;
pub mod html;
pub mod http;
pub mod label;
pub mod label_web;
pub mod run_using;
pub mod sink;
pub mod skip_reviewed;

#[derive(Debug)]
pub struct Env {
    config: PathBuf,
    input: Option<SocketAddr>,
    output: Option<SocketAddr>,
    timestamp_override: Option<u64>,
}

pub struct GeneratorContext {
    config: Config,
    in_events: Box<dyn Iterator<Item = Result<Event>> + Send + Sync>,
    writer: Box<dyn Write + Send + Sync>,
}

pub struct MapContext {
    config: Config,
    in_events: Box<dyn Iterator<Item = Result<Event>> + Send + Sync>,
    timestamp_override: Option<u64>,
    writer: Box<dyn Write + Send + Sync>,
}

pub fn get_config(filename: &PathBuf) -> Result<Config> {
    let file = File::open(filename).with_context(|| "Cannot open config file")?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).with_context(|| "Cannot parse config as JSON")
}

fn local_addr(s: &str) -> Result<SocketAddr> {
    s.to_socket_addrs()
        .with_context(|| format!("Unable to parse as SocketAddrs: {}", s))?
        .next()
        .ok_or(Error::msg("No SocketAddr found"))
}

pub fn get_env_addr(key: &str) -> Result<Option<SocketAddr>> {
    env::var(key)
        .ok()
        .map(|s| {
            if s.is_empty() {
                None
            } else {
                Some(local_addr(&s))
            }
        })
        .flatten()
        .transpose()
}

pub fn get_env() -> Result<Env> {
    let config =
        PathBuf::from(env::var("SR_CONFIG").with_context(|| "SR_CONFIG is not a valid file path")?);
    let input = get_env_addr("SR_INPUT")?;
    let output = get_env_addr("SR_OUTPUT")?;
    let timestamp_override = common::get_timestamp_override()?;

    Ok(Env {
        config,
        input,
        output,
        timestamp_override,
    })
}

pub fn input_events(addr: &SocketAddr) -> Result<impl Iterator<Item = Result<Event>>> {
    let stream =
        TcpStream::connect(addr).with_context(|| format!("Failed to connect to {}", addr))?;
    let reader = BufReader::new(stream);
    Ok(event::events(reader))
}

pub fn is_remote_target(db: &str) -> bool {
    let target = db.to_lowercase();
    target.starts_with("http://") || target.starts_with("https://")
}

pub fn api_route(remote: &str, path: &str) -> String {
    format!(
        "{}{}api/v1/{}",
        remote,
        if remote.ends_with("/") { "" } else { "/" },
        path,
    )
}

pub fn output_writer(addr: &SocketAddr) -> Result<LineWriter<TcpStream>> {
    let stream =
        TcpStream::connect(addr).with_context(|| format!("Failed to connect to {}", addr))?;
    Ok(LineWriter::new(stream))
}

pub fn get_generator_context() -> Result<GeneratorContext> {
    let env = get_env()?;
    let config = get_config(&env.config)?;
    let output_addr = env
        .output
        .ok_or(Error::msg("Missing value for SR_OUTPUT"))?;
    let writer = Box::new(output_writer(&output_addr)?);

    let in_events = match env.input {
        Some(input_addr) => Box::new(input_events(&input_addr)?),
        None => {
            let events: Box<dyn Iterator<Item = Result<Event>> + Send + Sync> =
                Box::new(std::iter::empty());
            events
        }
    };

    Ok(GeneratorContext {
        config,
        in_events,
        writer,
    })
}

pub fn get_map_context() -> Result<MapContext> {
    let env = get_env()?;
    let config = get_config(&env.config)?;
    let input_addr = env.input.ok_or(Error::msg("Missing value for SR_INPUT"))?;
    let in_events = Box::new(input_events(&input_addr)?);
    let output_addr = env
        .output
        .ok_or(Error::msg("Missing value for SR_OUTPUT"))?;
    let writer = Box::new(output_writer(&output_addr)?);

    Ok(MapContext {
        config,
        in_events,
        timestamp_override: env.timestamp_override,
        writer,
    })
}

pub fn write_event(mut writer: &mut Box<dyn Write + Send + Sync>, event: &Event) -> Result<()> {
    let hash = event.hash.clone().unwrap_or(String::from("None"));
    serde_json::to_writer(&mut writer, event)
        .with_context(|| format!("Failed to serialize event with hash: {}", hash))?;
    writer
        .write(b"\n")
        .with_context(|| format!("Buffer write failed for event with hash: {}", hash))?;
    Ok(())
}

pub fn write_event_dedupe(
    writer: &mut Box<dyn Write + Send + Sync>,
    event: &Event,
    hashes: &mut HashSet<String>,
) -> Result<()> {
    let hash = match event.hash.clone() {
        Some(s) => s,
        None => Err(Error::msg("Tried to write event with no hash"))?,
    };
    if !hashes.contains(&hash) {
        write_event(writer, event)?;
        hashes.insert(hash);
    }
    Ok(())
}

fn get_epoch_sec() -> Result<u64> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .with_context(|| "Failed to calculate timestamp")?
        .as_secs())
}

pub fn insert_timestamp(
    data: &mut HashMap<String, serde_json::Value>,
    timestamp_override: Option<u64>,
) -> Result<()> {
    let timestamp = match timestamp_override {
        Some(v) => v,
        None => get_epoch_sec()?,
    };
    data.insert(String::from("timestamp"), json!(timestamp));
    Ok(())
}

fn get_file_or_url(
    client: Client,
    file_or_url: &str,
) -> Result<(Box<dyn BufRead + Send + Sync>, Option<PathBuf>, Option<Url>)> {
    match Url::parse(file_or_url) {
        Ok(url) => {
            let mut request = client.get(url.clone());

            if let Ok(token) = env::var("SRVC_TOKEN") {
                request = request.header("Authorization", format!("Bearer {}", token));
            }

            let response = request
                .send()
                .with_context(|| format!("Failed to complete HTTP request to {}", url))?;
            let status = response.status().as_u16();
            if status == 200 {
                Ok((Box::new(BufReader::new(response)), None, Some(url)))
            } else {
                Err(Error::msg(format!(
                    "Unexpected {} status for {}",
                    status, url
                )))
            }
        }
        Err(_) => {
            if file_or_url == "-" {
                Ok((Box::new(BufReader::new(io::stdin())), None, None))
            } else {
                let path = PathBuf::from(file_or_url);
                let file = File::open(&path)
                    .with_context(|| format!("Failed to open file {}", file_or_url))?;
                let reader = BufReader::new(file);
                Ok((Box::new(reader), Some(path), None))
            }
        }
    }
}

fn get_file_or_url_string(
    client: Client,
    file_or_url: &str,
) -> Result<(String, Option<PathBuf>, Option<Url>)> {
    let (mut reader, pathbuf, url) = get_file_or_url(client, file_or_url)?;
    let mut s = String::new();
    reader
        .read_to_string(&mut s)
        .with_context(|| format!("Buffer read failed for file {}", file_or_url))?;
    Ok((s, pathbuf, url))
}
