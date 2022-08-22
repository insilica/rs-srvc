#![allow(dead_code)]

use std::path::PathBuf;
use std::time::Duration;

use assert_cmd::Command;

pub fn cmd(timeout_millis: u64) -> Command {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.timeout(Duration::from_millis(timeout_millis));
    cmd.env_remove("RUST_BACKTRACE");
    cmd
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
