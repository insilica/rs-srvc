use std::process::Command;

use anyhow::{Context, Result};

pub fn run(uses: &str) -> Result<()> {
    Command::new("nix")
        .args(vec![
            "--extra-experimental-features",
            "nix-command",
            "--extra-experimental-features",
            "flakes",
            "run",
            &uses,
        ])
        .status()
        .with_context(|| "Failed to start step sub-process")?;

    Ok(())
}
