use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ontologos_core::Reasoner;
use ontologos_explain::explain;
use ontologos_parser::load_ontology;
use ontologos_profile::{detect_profile, ProfileReport};
use ontologos_rdfs::{MaterializationReport, RdfsEngine};
use serde::Serialize;
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
}

#[derive(Debug, Error)]
enum CliError {
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    #[error(transparent)]
    Parser(#[from] ontologos_parser::Error),
    #[error(transparent)]
    Profile(#[from] ontologos_profile::Error),
    #[error(transparent)]
    Rdfs(#[from] ontologos_rdfs::Error),
    #[error(transparent)]
    Explain(#[from] ontologos_explain::Error),
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match cli.command {
        Command::Profile { ontology } => {
            let ontology = load_ontology(&ontology)?;
            let report = detect_profile(&ontology)?;
            match cli.format {
                OutputFormat::Text => println!("{}", format_profile_text(&report)),
                OutputFormat::Json => emit_json(&report)?,
            }
        }
        Command::Classify { ontology } => {
            let ontology = load_ontology(&ontology)?;
            let reasoner = Reasoner::builder().build(ontology)?;
            reasoner.classify()?;
            emit_status(cli.format, "classified")?;
        }
        Command::Materialize { ontology } => {
            let mut ontology = load_ontology(&ontology)?;
            let report = RdfsEngine::new().materialize(&mut ontology)?;
            emit_materialize_report(cli.format, &report)?;
        }
        Command::Explain { ontology } => {
            let ontology = load_ontology(&ontology)?;
            let graph = explain(&ontology)?;
            emit(cli.format, &graph)?;
        }
    }

    Ok(())
}

fn emit_status(format: OutputFormat, status: &str) -> Result<(), CliError> {
    match format {
        OutputFormat::Text => println!("status: {status}"),
        OutputFormat::Json => emit_json(&serde_json::json!({ "status": status }))?,
    }
    Ok(())
}

#[derive(Serialize)]
struct MaterializeCliOutput<'a> {
    status: &'static str,
    #[serde(flatten)]
    report: &'a MaterializationReport,
}

fn emit_materialize_report(
    format: OutputFormat,
    report: &MaterializationReport,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Text => {
            println!("status: materialized");
            println!("initial_axiom_count: {}", report.initial_axiom_count);
            println!("final_axiom_count: {}", report.final_axiom_count);
            println!("inferred_axioms: {}", report.inferred_total());
            if report.inferred_by_rule.is_empty() {
                println!("inferred_by_rule: none");
            } else {
                println!("inferred_by_rule:");
                for (rule, count) in &report.inferred_by_rule {
                    println!("  {}: {count}", rule.as_str());
                }
            }
        }
        OutputFormat::Json => emit_json(&MaterializeCliOutput {
            status: "materialized",
            report,
        })?,
    }
    Ok(())
}

fn emit_json<T: Serialize>(value: &T) -> Result<(), CliError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| CliError::Core(ontologos_core::Error::Serialization(e.to_string())))?;
    println!("{json}");
    Ok(())
}

fn emit<T: Serialize>(format: OutputFormat, value: &T) -> Result<(), CliError> {
    match format {
        OutputFormat::Text => println!("(use --format json for structured output)"),
        OutputFormat::Json => emit_json(value)?,
    }
    Ok(())
}

fn format_profile_text(report: &ProfileReport) -> String {
    let detected = report
        .detected
        .map(|p| format!("{p:?}"))
        .unwrap_or_else(|| "none".into());
    let mut lines = vec![format!("detected profile: {detected}")];
    if report.diagnostics.is_empty() {
        lines.push("diagnostics: none".into());
    } else {
        lines.push("diagnostics:".into());
        for diag in &report.diagnostics {
            lines.push(format!("  - {}: {}", diag.construct, diag.message));
        }
    }
    lines.join("\n")
}
