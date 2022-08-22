#![allow(dead_code)]

use std::path::PathBuf;
use std::time::Duration;

use assert_cmd::Command;
use rexpect::session::PtySession;

pub fn cmd(timeout_millis: u64) -> Command {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.timeout(Duration::from_millis(timeout_millis));
    cmd.env_remove("RUST_BACKTRACE");
    cmd
}

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
    Ok(rexpect::session::spawn_command(cmd, Some(timeout_millis))?)
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
