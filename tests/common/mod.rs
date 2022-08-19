#![allow(dead_code)]

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use assert_cmd::Command;

pub fn cmd(timeout_millis: u64) -> Command {
    let mut cmd = Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.timeout(Duration::from_millis(timeout_millis));
    cmd
}

pub fn check_sink(dir: &str) -> Result<(), std::io::Error> {
    let mut sink = String::new();
    File::open(PathBuf::from(dir).join("sink.jsonl"))?.read_to_string(&mut sink)?;
    Command::new("git")
        .arg("diff")
        .arg("expected.jsonl")
        .arg("sink.jsonl")
        .current_dir(dir)
        .assert()
        .success()
        .stderr("")
        .stdout("");
    Ok(())
}
