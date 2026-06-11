use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ontologos_core::{Ontology, Reasoner};
use ontologos_explain::explain;
use ontologos_profile::detect_profile;
use ontologos_rdfs::RdfsEngine;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(name = "ontologos", about = "Modular Rust ontology reasoner")]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Detect the OWL profile of an ontology
    Profile { ontology: PathBuf },
    /// Classify an ontology
    Classify { ontology: PathBuf },
    /// Materialize RDFS inferences
    Materialize { ontology: PathBuf },
    /// Explain inferences in an ontology
    Explain { ontology: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Json,
    Yaml,
}

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    #[error(transparent)]
    Profile(#[from] ontologos_profile::Error),
    #[error(transparent)]
    Rdfs(#[from] ontologos_rdfs::Error),
    #[error(transparent)]
    Explain(#[from] ontologos_explain::Error),
    #[error("yaml output not yet implemented")]
    YamlNotImplemented,
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();
    let ontology = Ontology::from_file(match &cli.command {
        Command::Profile { ontology }
        | Command::Classify { ontology }
        | Command::Materialize { ontology }
        | Command::Explain { ontology } => ontology,
    })?;

    match cli.command {
        Command::Profile { .. } => {
            let report = detect_profile(&ontology)?;
            emit(cli.format, &report)?;
        }
        Command::Classify { .. } => {
            let reasoner = Reasoner::builder().build(ontology)?;
            reasoner.classify()?;
            emit(cli.format, &serde_json::json!({ "status": "classified" }))?;
        }
        Command::Materialize { .. } => {
            RdfsEngine::new().materialize(&ontology)?;
            emit(cli.format, &serde_json::json!({ "status": "materialized" }))?;
        }
        Command::Explain { .. } => {
            let graph = explain(&ontology)?;
            emit(cli.format, &graph)?;
        }
    }

    Ok(())
}

fn emit<T: serde::Serialize>(format: OutputFormat, value: &T) -> Result<(), CliError> {
    match format {
        OutputFormat::Text => println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_default()
        ),
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(value).unwrap_or_default()
        ),
        OutputFormat::Yaml => return Err(CliError::YamlNotImplemented),
    }
    Ok(())
}
