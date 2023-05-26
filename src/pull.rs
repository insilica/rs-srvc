use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;

use lib_sr::{errors::*, event::Event, sr_yaml, Opts};

use crate::embedded::{generator, sink};

pub fn run(opts: &mut Opts, db: Option<String>, file_or_url: &str) -> Result<()> {
    let yaml_config = sr_yaml::get_config(PathBuf::from(&opts.config))?;
    let mut config = sr_yaml::parse_config(yaml_config)?;
    config.db = db.unwrap_or(config.db);
    // Don't add any sr.yaml labels to the db
    config.labels = HashMap::new();

    let (tx, rx) = mpsc::sync_channel::<Event>(16);

    let cfg = config.clone();
    let thread = thread::spawn(move || {
        match sink::run_with_events(&cfg, rx.iter().map(|event| Ok(event))) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("err! {}", e);
                Err(e)
            }
        }
    });

    let mut f = |event: Event| {
        tx.send(event)
            .chain_err(|| "Failed to send event to channel")
    };
    generator::run_f(file_or_url, &config, &mut f)?;
    drop(tx);
    match thread.join() {
        Ok(Ok(_)) => Ok(()),
        Ok(Err(e)) => Err(e),
        Err(_) => Err("Failed to join thread".into()),
    }
}
