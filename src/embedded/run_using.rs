use std::process::Command;

use lib_sr::errors::*;

pub fn run(uses: &str) -> Result<()> {
    Command::new("nix")
        .args(vec!["run", &uses])
        .status()
        .chain_err(|| "Failed to start step sub-process")?;

    Ok(())
}
