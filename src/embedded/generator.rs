use std::collections::HashSet;

use url::Url;

use lib_sr::{common, errors::*, generate};

use crate::embedded;
use crate::embedded::GeneratorContext;

pub fn run(file_or_url: &str) -> Result<()> {
    let GeneratorContext {
        config,
        in_events,
        mut writer,
    } = embedded::get_generator_context()?;

    let mut hashes = HashSet::new();

    for event in in_events {
        embedded::write_event_dedupe(&mut writer, &event?, &mut hashes)?;
    }

    match Url::parse(file_or_url) {
        Ok(_) => {
            let mut f_dedupe =
                |event| embedded::write_event_dedupe(&mut writer, &event, &mut hashes);
            generate::run_jsonl(file_or_url, &config, &mut f_dedupe)
        }
        Err(_) => {
            if common::has_sqlite_ext(file_or_url) {
                let mut f = |event| embedded::write_event(&mut writer, &event);
                generate::run_sqlite(file_or_url, &config, &mut f)
            } else {
                let mut f_dedupe =
                    |event| embedded::write_event_dedupe(&mut writer, &event, &mut hashes);
                generate::run_jsonl(file_or_url, &config, &mut f_dedupe)
            }
        }
    }
}
