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
mod flow;
mod hash;
mod json_schema;
mod sr_yaml;

const REV: Option<&'static str> = option_env!("SELF_REV");
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

    /// Run a flow
    #[clap(alias = "review")]
    Flow {
        /// The name of the flow
        #[clap(forbid_empty_values = true)]
        name: String,
    },

    /// Run an embedded step
    RunEmbeddedStep {
        /// The name of an embedded step
        #[clap(subcommand)]
        name: EmbeddedSteps,
    },

    /// Print the srvc version
    Version {},
}

#[derive(Subcommand, Debug)]
enum EmbeddedSteps {
    #[clap(alias = "generator-file")]
    /// Source review events from a file or URL
    Generator {
        /// Path to a file or URL containing review events
        #[clap(forbid_empty_values = true)]
        file_or_url: String,
    },
    /// Serve an HTML file as a map step
    Html {
        /// Path to a file or URL containing an HTML review step
        #[clap(forbid_empty_values = true)]
        file_or_url: String,
    },
    /// Label documents
    Label {},
    /// Label documents using a web interface
    LabelWeb {},
    /// Run a step using a Nix flake
    RunUsing {
        #[clap(forbid_empty_values = true)]
        uses: String,
    },
    /// Store review events in a file
    Sink {},
    #[clap(alias = "remove-reviewed")]
    /// Skip documents that have already been reviewed
    SkipReviewed {},
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
        EmbeddedSteps::Generator { file_or_url } => embedded::generator::run(&file_or_url),
        EmbeddedSteps::Html { file_or_url } => embedded::html::run(&file_or_url),
        EmbeddedSteps::Label {} => embedded::label::run(),
        EmbeddedSteps::LabelWeb {} => embedded::label_web::run(),
        EmbeddedSteps::SkipReviewed {} => embedded::skip_reviewed::run(),
        EmbeddedSteps::RunUsing { uses } => embedded::run_using::run(&uses),
        EmbeddedSteps::Sink {} => embedded::sink::run(),
    }
}

fn version() -> Result<()> {
    writeln!(io::stdout(), "srvc {}", VERSION).chain_err(|| "Failed to write to stdout")?;
    match REV {
        Some(rev) => {
            writeln!(io::stdout(), "Revision {}", rev).chain_err(|| "Failed to write to stdout")?
        }
        _ => {}
    }
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
        Commands::Flow { name } => flow::run(opts, name),
        Commands::RunEmbeddedStep { name } => run_embedded_step(name),
        Commands::Version {} => version(),
    }
}

fn run() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let mut opts = opts(&cli);
    run_command(cli, &mut opts)
}

quick_main!(run);
