// `error_chain!` can recurse deeply
#![recursion_limit = "1024"]

#[macro_use]
extern crate error_chain;

mod errors {
    error_chain! {}
}

use errors::*;

use clap::{Parser, Subcommand};

mod embedded;

/// Sysrev version control CLI
#[derive(Parser)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run an embedded review step
    RunEmbeddedStep {
        /// The name of an embedded review step
        #[clap(subcommand)]
        name: EmbeddedSteps,
    },
}

#[derive(Subcommand, Debug)]
enum EmbeddedSteps {
    /// Remove articles that have already been reviewed
    RemoveReviewed {},
}

fn run_embedded_step(name: EmbeddedSteps) -> Result<()> {
    match name {
        EmbeddedSteps::RemoveReviewed {} => embedded::remove_reviewed::run(),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::RunEmbeddedStep { name } => {
            run_embedded_step(name)
        }
    }
}

quick_main!(run);
