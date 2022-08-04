use std::fs::File;
use std::io::BufReader;
use std::io::Write;
use std::path::PathBuf;

use serde::Serialize;

use crate::embedded;
use crate::errors::*;

pub fn run(filename: PathBuf) -> Result<()> {
    let env = embedded::get_env().chain_err(|| "Env var processing failed")?;
    let input = File::open(filename).chain_err(|| "Cannot open generator file")?;
    let reader = BufReader::new(input);
    let in_events = embedded::events(reader);
    let output_addr = env.output.ok_or("Missing value for SR_OUTPUT")?;
    let mut writer = embedded::output_writer(&output_addr)?;

    for result in in_events {
        let mut event = result.chain_err(|| "Cannot parse line as JSON")?;
        let expected_hash = crate::event::event_hash(event.clone())?;
        let hash = event.hash.clone().unwrap_or("".to_string());
        if hash == "" {
            event.hash = Some(expected_hash);
        } else if expected_hash != hash {
            return Err(format!(
                "Incorrect event hash. Expected: \"{}\". Found: \"{}\".",
                expected_hash, hash
            )
            .into());
        }
        event
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .chain_err(|| "Event serialization failed")?;
        writer.write(b"\n").chain_err(|| "Buffer write failed")?;
    }

    Ok(())
}
