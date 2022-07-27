// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {}
}

use errors::*;

use std::path::PathBuf;

use clap::{Parser, Subcommand};

mod embedded;
mod event;
mod lib;
mod review;
mod sr_yaml;

/// Sysrev version control CLI
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(short, long, default_value = "sr.yaml")]
    config: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a review flow
    Review {
        /// The name of the review flow
        name: String,
    },

    /// Run an embedded review step
    RunEmbeddedStep {
        /// The name of an embedded review step
        #[clap(subcommand)]
        name: EmbeddedSteps,
    },
}

#[derive(Subcommand, Debug)]
enum EmbeddedSteps {
    /// Add hashes to events
    AddHashes {},
    /// Source review events from a file
    GeneratorFile { filename: PathBuf },
    /// Remove documents that have already been reviewed
    RemoveReviewed {},
    /// Store review events in a file
    Sink {},
}

fn run_embedded_step(name: EmbeddedSteps) -> Result<()> {
    match name {
        EmbeddedSteps::AddHashes {} => embedded::add_hashes::run(),
        EmbeddedSteps::GeneratorFile { filename } => embedded::generator_file::run(filename),
        EmbeddedSteps::RemoveReviewed {} => embedded::remove_reviewed::run(),
        EmbeddedSteps::Sink {} => embedded::sink::run(),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let opts = lib::Opts { config: cli.config };

    match cli.command {
        Commands::Review { name } => review::run(opts, name),
        Commands::RunEmbeddedStep { name } => run_embedded_step(name),
    }
}

quick_main!(run);
