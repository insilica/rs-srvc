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
