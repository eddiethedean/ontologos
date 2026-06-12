use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ontologos_core::{Ontology, ParseMetaSummary, Reasoner};
use ontologos_explain::{explain, ProofGraph};
use ontologos_parser::load_ontology;
use ontologos_profile::{detect_profile, ProfileReport};
use ontologos_rdfs::{materialize_reasoner, MaterializationReport, RdfsEngine};
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
    /// Run RDFS TBox materialization (not OWL EL/DL taxonomy classification; v0.5)
    Classify { ontology: PathBuf },
    /// Materialize RDFS TBox inferences
    Materialize { ontology: PathBuf },
    /// Explain inferences (stub until v0.6 — returns not implemented)
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
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let report = detect_profile(&ontology)?;
            match cli.format {
                OutputFormat::Text => println!("{}", format_profile_text(&report)),
                OutputFormat::Json => emit_json(&ProfileCliOutput {
                    report: &report,
                    parse_meta: &parse_meta,
                })?,
            }
        }
        Command::Classify { ontology } => {
            let ontology = load_ontology(&ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let mut reasoner = Reasoner::builder()
                .profile(ontologos_core::Profile::Rdfs)
                .build(ontology)?;
            let report = materialize_reasoner(&mut reasoner)?;
            emit_inference_report(cli.format, "classified", &report, &parse_meta)?;
        }
        Command::Materialize { ontology } => {
            let mut ontology = load_ontology(&ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let report = RdfsEngine::new().materialize(&mut ontology)?;
            emit_inference_report(cli.format, "materialized", &report, &parse_meta)?;
        }
        Command::Explain { ontology } => {
            let ontology = load_ontology(&ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let graph = explain(&ontology)?;
            emit(
                cli.format,
                &ExplainCliOutput {
                    graph: &graph,
                    parse_meta: &parse_meta,
                },
            )?;
        }
    }

    Ok(())
}

fn parse_meta_summary(ontology: &Ontology) -> ParseMetaSummary {
    ontology
        .parse_meta()
        .map(ParseMetaSummary::from)
        .unwrap_or_default()
}

fn emit_parse_meta_text(format: OutputFormat, parse_meta: &ParseMetaSummary) {
    if matches!(format, OutputFormat::Text) {
        parse_meta.emit_stderr();
    }
}

fn skip_clean_parse_meta(meta: &&ParseMetaSummary) -> bool {
    meta.omit_from_json()
}

#[derive(Serialize)]
struct ProfileCliOutput<'a> {
    #[serde(flatten)]
    report: &'a ProfileReport,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct InferenceCliOutput<'a> {
    status: &'static str,
    initial_axiom_count: usize,
    final_axiom_count: usize,
    inferred_axioms: usize,
    inferred_by_rule: &'a std::collections::BTreeMap<ontologos_rdfs::RdfsRule, usize>,
    #[serde(skip_serializing_if = "inference_traces_empty")]
    traces: &'a [ontologos_rdfs::InferenceRecord],
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct ExplainCliOutput<'a> {
    #[serde(flatten)]
    graph: &'a ProofGraph,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

fn inference_traces_empty(traces: &&[ontologos_rdfs::InferenceRecord]) -> bool {
    traces.is_empty()
}

fn emit_inference_report(
    format: OutputFormat,
    status: &'static str,
    report: &MaterializationReport,
    parse_meta: &ParseMetaSummary,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Text => {
            println!("status: {status}");
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
        OutputFormat::Json => emit_json(&InferenceCliOutput {
            status,
            initial_axiom_count: report.initial_axiom_count,
            final_axiom_count: report.final_axiom_count,
            inferred_axioms: report.inferred_total(),
            inferred_by_rule: &report.inferred_by_rule,
            traces: &report.traces,
            parse_meta,
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
