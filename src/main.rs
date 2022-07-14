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

fn run_embedded_step(name: EmbeddedSteps) -> Result<(), Box<dyn std::error::Error>> {
    match name {
        EmbeddedSteps::RemoveReviewed {} => return embedded::remove_reviewed::run(),
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::RunEmbeddedStep { name } => {
            return run_embedded_step(name);
        }
    }
}
