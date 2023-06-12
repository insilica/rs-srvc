use std::{fs, path::Path};

use anyhow::Result;
use common::test_dir;

mod common;

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
