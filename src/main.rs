#[macro_use]
extern crate maplit;

use std::env;
use std::io;
use std::io::Write;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use url::{form_urlencoded, Url};

use lib_sr::{common, flow, sr_yaml, Opts};

mod embedded;
mod hash;
mod pull;

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

    /// Run a flow
    #[clap(alias = "review")]
    Flow {
        /// Override the default db file
        #[clap(long)]
        db: Option<String>,

        /// Provide the flow definition in JSON format
        #[clap(long)]
        def: Option<String>,

        /// The name of the flow
        #[clap(forbid_empty_values = true)]
        name: String,

        /// Override the default reviewer URI
        #[clap(long)]
        reviewer: Option<String>,

        /// Instruct steps to use free ports, ignoring any port set in sr.yaml
        #[clap(long)]
        use_free_ports: bool,
    },

    /// Add hashes to a stream of events
    Hash {},

    /// Print the full, canonicalized config in JSON format
    PrintConfig {
        /// Whether to pretty-print the JSON
        #[clap(long)]
        pretty: bool,
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
    /// Serve an HTTP endpoint as a map step
    Http {
        /// URL of the HTTP endpoint
        #[clap(forbid_empty_values = true)]
        url: String,
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
    .with_context(|| "Failed to serialize config")?;
    writeln!(io::stdout(), "").with_context(|| "Failed to write newline")?;
    Ok(())
}

fn run_embedded_step(name: EmbeddedSteps) -> Result<()> {
    match name {
        EmbeddedSteps::Generator { file_or_url } => embedded::generator::run(&file_or_url),
        EmbeddedSteps::Html { file_or_url } => embedded::html::run(&file_or_url),
        EmbeddedSteps::Http { url } => embedded::http::run(&url),
        EmbeddedSteps::Label {} => embedded::label::run(),
        EmbeddedSteps::LabelWeb {} => embedded::label_web::run(),
        EmbeddedSteps::RunUsing { uses } => embedded::run_using::run(&uses),
        EmbeddedSteps::Sink {} => embedded::sink::run(),
        EmbeddedSteps::SkipReviewed {} => embedded::skip_reviewed::run(),
    }
}

fn version() -> Result<()> {
    writeln!(io::stdout(), "srvc {}", VERSION).with_context(|| "Failed to write to stdout")?;
    match REV {
        Some(rev) => writeln!(io::stdout(), "Revision {}", rev)
            .with_context(|| "Failed to write to stdout")?,
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
        Commands::Flow {
            db,
            def,
            name,
            reviewer,
            use_free_ports,
        } => flow::run(opts, db, def, name, reviewer, use_free_ports),
        Commands::Hash {} => hash::run(),
        Commands::PrintConfig { pretty } => print_config(opts, pretty),
        Commands::Pull { db, file_or_url } => pull::run(opts, db, &file_or_url),
        Commands::RunEmbeddedStep { name } => run_embedded_step(name),
        Commands::Version {} => version(),
    }
}

fn run() -> Result<()> {
    env_logger::init();
    let cli = Cli::parse();
    let mut opts = opts(&cli);

    if let Some(path) = cli.dir.to_owned() {
        env::set_current_dir(&path).context(format!(
            "Failed to set working directory: {}",
            path.to_string_lossy()
        ))?;
    }

    run_command(cli, &mut opts)
}

fn main() {
    if let Err(err) = run() {
        eprintln!("Error: {:?}", err);
        std::process::exit(1);
    }
}
