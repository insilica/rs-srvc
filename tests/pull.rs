use std::{fs, path::Path};

use anyhow::Result;
use common::test_dir;

mod common;

#[test]
fn test_pull_stdin() -> Result<()> {
    let resource_dir = "pull-stdin";
    let timeout_millis = 10000;
    let dir = test_dir(resource_dir);
    let stdin = fs::read_to_string(Path::new(&dir).join("stdin.jsonl"))?;
    common::remove_sink(&dir)?;
    common::cmd(timeout_millis)
        .current_dir(&dir)
        .args(&["pull", "-"])
        .write_stdin(stdin.clone())
        .assert()
        .success()
        .stdout("")
        .stderr("");
    common::check_sink(&dir, true)?;

    common::remove_sink(&dir)?;
    common::cmd(timeout_millis)
        .current_dir(&dir)
        .args(&["pull", "--db", "sink.db", "-"])
        .write_stdin(stdin)
        .assert()
        .success()
        .stdout("")
        .stderr("");
    common::cmd(timeout_millis)
        .current_dir(&dir)
        .args(&["pull", "--db", "sink.jsonl", "sink.db"])
        .assert()
        .success()
        .stdout("")
        .stderr("");
    common::check_sink(&dir, false)?;
    Ok(())
}

#[test]
fn test_pull_stdout() -> Result<()> {
    let resource_dir = "pull-stdout";
    let target = "docs.jsonl";
    let timeout_millis = 2000;
    let dir = test_dir(resource_dir);
    let expected_stdout = fs::read_to_string(Path::new(&dir).join("stdout.jsonl"))?;
    common::remove_sink(&dir)?;
    common::cmd(timeout_millis)
        .current_dir(&dir)
        .args(&["pull", &target])
        .assert()
        .success()
        .stdout(expected_stdout.clone())
        .stderr("");

    common::remove_sink(&dir)?;
    common::cmd(timeout_millis)
        .current_dir(&dir)
        .args(&["pull", "--db", "-", &target])
        .assert()
        .success()
        .stdout(expected_stdout)
        .stderr("");
    Ok(())
}
