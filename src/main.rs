#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate maplit;

use std::env;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use clap::{Parser, Subcommand};
use url::Url;

use lib_sr::common;
use lib_sr::errors::*;
use lib_sr::Opts;
use url::form_urlencoded;

mod embedded;
mod flow;
mod hash;
mod json_schema;
mod pull;
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

    #[clap(short, long)]
    dir: Option<PathBuf>,
}

#[derive(Subcommand)]
#[clap(version)]
enum Commands {
    /// Open the documentation website
    Docs {
        /// Search query
        #[clap(multiple_values = true)]
        query: Vec<String>,
    },

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
        /// Override the default db file
        #[clap(long)]
        db: Option<String>,

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

    /// Pull events into the project from a file or URL
    Pull {
        /// Override the default db file
        #[clap(long)]
        db: Option<String>,

        /// Path to a file or URL containing review events
        #[clap(forbid_empty_values = true)]
        file_or_url: String,
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

fn open_docs(query: Vec<String>) -> Result<()> {
    if query.is_empty() {
        common::open_browser(&format!("https://docs.sysrev.com/v{}/", VERSION))
    } else {
        let mut url =
            Url::parse(&format!("https://docs.sysrev.com/v{}/search.html", VERSION)).expect("url");
        let params = vec![("q", query.join(" "))];
        let qs = form_urlencoded::Serializer::new(String::new())
            .extend_pairs(params)
            .finish();
        url.set_query(Some(&qs));
        common::open_browser(url.as_str())
    }
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
        Commands::Docs { query } => open_docs(query),
        Commands::Hash {} => hash::run(),
        Commands::PrintConfig { pretty } => print_config(opts, pretty),
        Commands::Flow { db, name } => flow::run(opts, db, name),
        Commands::Pull { db, file_or_url } => pull::run(opts, db, &file_or_url),
        Commands::RunEmbeddedStep { name } => run_embedded_step(name),
        Commands::Version {} => version(),
    }
}

fn run() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let mut opts = opts(&cli);

    match cli.dir.to_owned() {
        Some(path) => env::set_current_dir(&path).chain_err(|| {
            format!(
                "Failed to set working directory: {}",
                path.to_string_lossy()
            )
        })?,
        None => {}
    }

    run_command(cli, &mut opts)
}

quick_main!(run);
