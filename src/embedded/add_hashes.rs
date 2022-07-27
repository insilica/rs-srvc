use std::path::PathBuf;

use crate::embedded;
use crate::errors::*;

pub fn run() -> Result<()> {
    let env = embedded::get_env().chain_err(|| "Env var processing failed")?;
    let input = env.input.unwrap();
    embedded::generator_file::run(PathBuf::from(input))
}
