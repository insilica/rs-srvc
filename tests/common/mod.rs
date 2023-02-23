#![allow(dead_code)]

use std::collections::HashSet;
use std::env;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use assert_cmd::Command;
use lib_sr::errors::*;
use lib_sr::event;
#[cfg(unix)]
use rexpect::session::PtySession;

mod http_server;

#[ctor::ctor]
static STATIC_CTOR: () = {
    thread::spawn(move || http_server::run("test-resources", 8877).unwrap());
    http_server::wait_server_ready(8877).unwrap()
};

pub fn cmd(timeout_millis: u64) -> Command {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    if env::var("TEST_SRVC_DISABLE_TIMEOUT").is_err() {
        cmd.timeout(Duration::from_millis(timeout_millis));
    }
    cmd.env_remove("RUST_BACKTRACE");
    cmd
}

#[cfg(unix)]
pub fn spawn(
    dir: &str,
    args: Vec<&str>,
    timestamp_override: u64,
    timeout_millis: u64,
) -> std::result::Result<PtySession, rexpect::errors::Error> {
    let mut cmd = std::process::Command::new(env!("CARGO_BIN_EXE_sr"));
    cmd.args(args);
    cmd.current_dir(dir);
    cmd.env("SR_TIMESTAMP_OVERRIDE", timestamp_override.to_string());
    cmd.env_remove("RUST_BACKTRACE");
    let timeout = if env::var("TEST_SRVC_DISABLE_TIMEOUT").is_ok() {
        None
    } else {
        Some(timeout_millis)
    };
    Ok(rexpect::session::spawn_command(cmd, timeout)?)
}

pub fn sink_path(dir: &str) -> PathBuf {
    PathBuf::from(dir).join("sink.jsonl")
}

pub fn sqlite_sink_path(dir: &str) -> PathBuf {
    PathBuf::from(dir).join("sink.db")
}

pub fn remove_sink(dir: &str) -> Result<()> {
    let sink = sink_path(dir);
    let sqlite_sink = sqlite_sink_path(dir);
    if sink.exists() {
        std::fs::remove_file(&sink).chain_err(|| "Failed to delete old sink")?
    }
    if sqlite_sink.exists() {
        std::fs::remove_file(&sqlite_sink).chain_err(|| "Failed to delete old sink")
    } else {
        Ok(())
    }
}

fn file_hashes(path: &PathBuf) -> Result<HashSet<String>> {
    let reader = BufReader::new(File::open(path).chain_err(|| "Failed to open file")?);
    let mut hashes = HashSet::new();
    for event in event::events(reader) {
        match event {
            Ok(event) => hashes.insert(event.hash.expect("No hash for event")),
            Err(e) => Err(e)?,
        };
    }
    Ok(hashes)
}

fn check_sink_hashes(dir: &str) -> Result<()> {
    let expected = file_hashes(&PathBuf::from(dir).join("expected.jsonl"))?;
    let sink = file_hashes(&PathBuf::from(dir).join("sink.jsonl"))?;

    assert_eq!(
        HashSet::new(),
        expected.difference(&sink).collect(),
        "sink.jsonl contains all of the hashes in expected.jsonl"
    );

    assert_eq!(
        HashSet::new(),
        sink.difference(&expected).collect(),
        "sink.jsonl does contain any hashes that are not in expected.jsonl"
    );

    Ok(())
}

pub fn check_sink(dir: &str, text_diff: bool) -> Result<()> {
    check_sink_hashes(dir)?;
    if text_diff {
        Command::new("git")
            .args(["diff", "--no-index", "expected.jsonl", "sink.jsonl"])
            .current_dir(dir)
            .assert()
            .code(0)
            .success()
            .stderr("")
            .stdout("");
    }
    remove_sink(dir)?;
    Ok(())
}
