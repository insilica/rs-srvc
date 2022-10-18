use std::io::{BufReader, BufWriter, Write};

use serde::Serialize;

use lib_sr::Opts;

use crate::errors::*;
use crate::event;

pub fn run(opts: &mut Opts) -> Result<()> {
    let reader = BufReader::new(&mut opts.in_stream);
    let in_events = event::events(reader);
    let mut writer = BufWriter::new(&mut opts.out_stream);

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
        writer.flush().chain_err(|| "Flush failed")?;
    }

    Ok(())
}
