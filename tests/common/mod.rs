#![allow(dead_code)]

use std::env;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use assert_cmd::Command;
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
) -> Result<PtySession, rexpect::errors::Error> {
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

pub fn remove_sink(dir: &str) -> Result<(), std::io::Error> {
    let sink = PathBuf::from(dir).join("sink.jsonl");
    if sink.exists() {
        std::fs::remove_file(&sink)
    } else {
        Ok(())
    }
}

pub fn check_sink(dir: &str) -> Result<(), std::io::Error> {
    Command::new("git")
        .args(["diff", "--no-index", "expected.jsonl", "sink.jsonl"])
        .current_dir(dir)
        .assert()
        .code(0)
        .success()
        .stderr("")
        .stdout("");
    remove_sink(dir)?;
    Ok(())
}
