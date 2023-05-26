use std::io;
use std::io::{BufReader, BufWriter, Write};

use anyhow::{Context, Error, Result};
use serde::Serialize;

use lib_sr::event;

pub fn run() -> Result<()> {
    let reader = BufReader::new(io::stdin());
    let in_events = event::events(reader);
    let mut writer = BufWriter::new(io::stdout());

    for result in in_events {
        let mut event = result.with_context(|| "Cannot parse line as JSON")?;
        let expected_hash = lib_sr::event::event_hash(event.clone())?;
        let hash = event.hash.clone().unwrap_or("".to_string());
        if hash == "" {
            event.hash = Some(expected_hash);
        } else if expected_hash != hash {
            return Err(Error::msg(format!(
                "Incorrect event hash. Expected: \"{}\". Found: \"{}\".",
                expected_hash, hash
            )));
        }
        event
            .serialize(&mut serde_json::Serializer::new(&mut writer))
            .with_context(|| "Event serialization failed")?;
        writer.write(b"\n").with_context(|| "Buffer write failed")?;
        writer.flush().with_context(|| "Flush failed")?;
    }

    Ok(())
}
