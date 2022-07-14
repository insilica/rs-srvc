use std::collections::HashSet;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufReader;

use crate::embedded;

pub fn read_reviewed_docs(_file: File) -> Result<HashSet<String>, Box<dyn std::error::Error>> {
    let hashes = HashSet::new();

    Ok(hashes)
}

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let env = embedded::get_env()?;
    let config = embedded::get_config(env.config)?;
    let db_file = File::open(config.db.unwrap())?;
    let reviewed_docs = read_reviewed_docs(db_file)?;
    let input = File::open(env.input.unwrap())?;
    let _reader = BufReader::new(input);
    let _output = OpenOptions::new().write(true).open(env.output.unwrap())?;

    println!("{:?}", reviewed_docs);
    Ok(())
}
