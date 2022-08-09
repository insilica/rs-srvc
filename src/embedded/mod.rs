use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, LineWriter, Read};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::path::PathBuf;

use crate::errors::*;
use crate::event::Event;
use crate::lib::Config;

pub mod generator_file;
pub mod label;
pub mod remove_reviewed;
pub mod sink;

#[derive(Debug)]
pub struct Env {
    config: PathBuf,
    input: Option<SocketAddr>,
    output: Option<SocketAddr>,
}

pub fn get_config(filename: &PathBuf) -> Result<Config> {
    let file = File::open(filename).chain_err(|| "Cannot open config file")?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).chain_err(|| "Cannot parse config as JSON")
}

fn local_addr(s: &str) -> Result<SocketAddr> {
    s.to_socket_addrs()
        .chain_err(|| format!("Unable to parse as SocketAddrs: {}", s))?
        .next()
        .ok_or("No SocketAddr found".into())
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
        PathBuf::from(env::var("SR_CONFIG").chain_err(|| "SR_CONFIG is not a valid file path")?);
    let input = get_env_addr("SR_INPUT")?;
    let output = get_env_addr("SR_OUTPUT")?;

    Ok(Env {
        config,
        input,
        output,
    })
}

pub fn parse_event(s: &str) -> Result<Event> {
    serde_json::from_str(s).chain_err(|| "Cannot parse event")
}

pub fn events(reader: BufReader<impl Read>) -> impl Iterator<Item = Result<Event>> {
    reader
        .lines()
        .map(|line| parse_event(line.chain_err(|| "Failed to read line")?.as_str()))
}

pub fn input_events(addr: &SocketAddr) -> Result<impl Iterator<Item = Result<Event>>> {
    let stream = TcpStream::connect(addr).chain_err(|| format!("Failed to connect to {}", addr))?;
    let reader = BufReader::new(stream);
    Ok(events(reader))
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
    let stream = TcpStream::connect(addr).chain_err(|| format!("Failed to connect to {}", addr))?;
    Ok(LineWriter::new(stream))
}
