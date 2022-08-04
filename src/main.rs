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

#[cfg(test)]
mod tests;

use lib::Opts;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

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

    /// Print the srvc version
    Version {},
}

#[derive(Subcommand, Debug)]
enum EmbeddedSteps {
    /// Source review events from a file
    GeneratorFile { filename: PathBuf },
    /// Label documents
    Label {},
    /// Remove documents that have already been reviewed
    RemoveReviewed {},
    /// Store review events in a file
    Sink {},
}

fn run_embedded_step(opts: &mut Opts, name: EmbeddedSteps) -> Result<()> {
    match name {
        EmbeddedSteps::GeneratorFile { filename } => embedded::generator_file::run(filename),
        EmbeddedSteps::Label {} => embedded::label::run(opts),
        EmbeddedSteps::RemoveReviewed {} => embedded::remove_reviewed::run(),
        EmbeddedSteps::Sink {} => embedded::sink::run(),
    }
}

fn version(opts: &mut Opts) -> Result<()> {
    write!(opts.out_stream, "srvc {}", VERSION)
        .chain_err(|| "Failed to write to opts.out_stream")?;
    Ok(())
}

fn opts(cli: &Cli) -> Opts {
    Opts {
        config: cli.config.to_owned(),
        err_stream: Box::new(std::io::stderr()),
        in_stream: Box::new(std::io::stdin()),
        out_stream: Box::new(std::io::stdout()),
    }
}

fn run_command(cli: Cli, opts: &mut Opts) -> Result<()> {
    match cli.command {
        Commands::Review { name } => review::run(opts, name),
        Commands::RunEmbeddedStep { name } => run_embedded_step(opts, name),
        Commands::Version {} => version(opts),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut opts = opts(&cli);
    run_command(cli, &mut opts)
}

quick_main!(run);
