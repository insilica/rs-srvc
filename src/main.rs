#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

use std::io;
use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use lib_sr::errors::*;
use lib_sr::Opts;

mod embedded;
mod hash;
mod json_schema;
mod review;
mod sr_yaml;

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
#[clap(version)]
enum Commands {
    /// Add hashes to a stream of events
    Hash {},

    /// Print the full, canonicalized config in JSON format
    PrintConfig {
        /// Whether to pretty-print the JSON
        #[clap(long)]
        pretty: bool,
    },

    /// Run a review flow
    Review {
        /// The name of the review flow
        #[clap(forbid_empty_values = true)]
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
    GeneratorFile {
        /// Path to a file containing review events
        #[clap(forbid_empty_values = true)]
        filename: PathBuf,
    },
    /// Serve an HTML file as a map step
    Html {
        /// Path to a file or URL containing an HTML review step
        #[clap(forbid_empty_values = true)]
        file_or_url: String,
    },
    /// Label documents
    Label {},
    /// Remove documents that have already been reviewed
    RemoveReviewed {},
    /// Store review events in a file
    Sink {},
}

fn print_config(opts: &mut Opts, pretty: bool) -> Result<()> {
    let yaml_config = sr_yaml::get_config(PathBuf::from(&opts.config))?;
    let config = sr_yaml::parse_config(yaml_config)?;
    if pretty {
        serde_json::to_writer_pretty(&mut io::stdout(), &config)
    } else {
        serde_json::to_writer(&mut io::stdout(), &config)
    }
    .chain_err(|| "Failed to serialize config")?;
    writeln!(io::stdout(), "").chain_err(|| "Failed to write newline")?;
    Ok(())
}

fn run_embedded_step(name: EmbeddedSteps) -> Result<()> {
    match name {
        EmbeddedSteps::GeneratorFile { filename } => embedded::generator_file::run(filename),
        EmbeddedSteps::Html { file_or_url } => embedded::html::run(&file_or_url),
        EmbeddedSteps::Label {} => embedded::label::run(),
        EmbeddedSteps::RemoveReviewed {} => embedded::remove_reviewed::run(),
        EmbeddedSteps::Sink {} => embedded::sink::run(),
    }
}

fn version() -> Result<()> {
    writeln!(io::stdout(), "srvc {}", VERSION).chain_err(|| "Failed to write to stdout")?;
    Ok(())
}

fn opts(cli: &Cli) -> Opts {
    Opts {
        config: cli.config.to_owned(),
    }
}

fn run_command(cli: Cli, opts: &mut Opts) -> Result<()> {
    match cli.command {
        Commands::Hash {} => hash::run(),
        Commands::PrintConfig { pretty } => print_config(opts, pretty),
        Commands::Review { name } => review::run(opts, name),
        Commands::RunEmbeddedStep { name } => run_embedded_step(name),
        Commands::Version {} => version(),
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let mut opts = opts(&cli);
    run_command(cli, &mut opts)
}

quick_main!(run);
