#![allow(dead_code)]

use std::io::Read;
use std::process::{Child, Command, Stdio};

pub fn sr_process(args: &Vec<&str>, dir: &str) -> Result<Child, std::io::Error> {
    let bin = env!("CARGO_BIN_EXE_sr");
    Command::new(bin)
        .args(args)
        .current_dir(dir)
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
}

pub fn cmd_out(
    args: &Vec<&str>,
    dir: &str,
) -> Result<(Option<i32>, String, String), std::io::Error> {
    let mut child = sr_process(args, dir)?;
    let exit = child.wait()?.code();
    let mut stdout = String::new();
    child.stdout.expect("stdout").read_to_string(&mut stdout)?;
    let mut stderr = String::new();
    child.stderr.expect("stderr").read_to_string(&mut stderr)?;
    Ok((exit, stdout, stderr))
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
    let mut cmd = Command::new("diff")
        .args([file_a, file_b])
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
