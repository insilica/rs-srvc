#![allow(dead_code)]

use std::io::Read;
use std::process::{Command, Stdio};
use std::time::Duration;

pub fn cmd(timeout_millis: u64) -> assert_cmd::Command {
    let mut cmd = assert_cmd::Command::cargo_bin(env!("CARGO_PKG_NAME")).unwrap();
    cmd.timeout(Duration::from_millis(timeout_millis));
    cmd
}

pub fn delete(dir: &str, filename: &str) -> () {
    let mut path = dir.to_owned();
    path.push_str(filename);
    match std::fs::remove_file(path) {
        Ok(_) => (),
        Err(_) => (),
    };
}

pub fn file_diff(dir: &str, file_a: &str, file_b: &str) -> Result<(i32, String), std::io::Error> {
    let mut cmd = Command::new("git")
        .args(["diff", file_a, file_b])
        .current_dir(dir)
        .stdout(Stdio::piped())
        .spawn()?;
    match cmd.wait()?.code() {
        Some(code) => {
            let mut stdout = String::new();
            cmd.stdout.expect("stdout").read_to_string(&mut stdout)?;
            Ok((code, stdout))
        }
        None => panic!("diff process exited early"),
    }
}
