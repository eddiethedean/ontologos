use std::collections::BTreeMap;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ontologos_core::ReasonerConfig;
use ontologos_core::{EntityId, Ontology, ParseMetaSummary, Profile, Reasoner, Taxonomy};
use ontologos_el::{classify_with_profile, ClassifyOutcome};
use ontologos_explain::{explain_with_profile, render_text, ProofGraph};
use ontologos_parser::load_ontology;
use ontologos_profile::{detect_profile, ProfileReport};
use ontologos_rdfs::MaterializationReport as RdfsReport;
use ontologos_rl::MaterializationReport as RlReport;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "ontologos",
    about = "Modular Rust ontology reasoner",
    after_help = "v0.8.0: profile (detect), materialize (RDFS), classify (EL/RL/RDFS), explain (proof graphs). \
                  Use --incremental for delta re-classify. \
                  Docs: https://ontologos.readthedocs.io/en/latest/reference/cli/"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,

    /// OWL profile for `classify` (default: auto)
    #[arg(long, value_enum, default_value_t = CliProfile::Auto)]
    profile: CliProfile,

    /// Enable incremental re-classification / materialization when axioms change
    #[arg(long, default_value_t = false)]
    incremental: bool,

    /// Output format
    #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
    format: OutputFormat,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Detect the OWL profile of an ontology
    Profile { ontology: PathBuf },
    /// Classify or saturate by profile (EL taxonomy, RL saturation, or RDFS materialization)
    Classify { ontology: PathBuf },
    /// Materialize RDFS TBox inferences explicitly
    Materialize { ontology: PathBuf },
    /// Explain inferences as a proof graph (JSON or text)
    Explain { ontology: PathBuf },
}

#[derive(Debug, Clone, Copy, ValueEnum, Default)]
enum CliProfile {
    #[default]
    Auto,
    El,
    Rl,
    Rdfs,
}

impl From<CliProfile> for Profile {
    fn from(value: CliProfile) -> Self {
        match value {
            CliProfile::Auto => Profile::Auto,
            CliProfile::El => Profile::El,
            CliProfile::Rl => Profile::Rl,
            CliProfile::Rdfs => Profile::Rdfs,
        }
    }
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
    El(#[from] ontologos_el::Error),
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
                .profile(cli.profile.into())
                .config(ReasonerConfig {
                    incremental: cli.incremental,
                    ..ReasonerConfig::default()
                })
                .build(ontology)?;
            let outcome = classify_with_profile(&mut reasoner)?;
            emit_classify_outcome(cli.format, &outcome, reasoner.ontology(), &parse_meta)?;
        }
        Command::Materialize { ontology } => {
            let ontology = load_ontology(&ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let mut reasoner = Reasoner::builder()
                .profile(Profile::Rdfs)
                .config(ReasonerConfig {
                    incremental: cli.incremental,
                    ..ReasonerConfig::default()
                })
                .build(ontology)?;
            let report = ontologos_rdfs::materialize_reasoner(&mut reasoner)?;
            emit_rdfs_report(cli.format, "materialized", &report, &parse_meta)?;
        }
        Command::Explain { ontology } => {
            let ontology = load_ontology(&ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let mut reasoner = Reasoner::builder()
                .profile(cli.profile.into())
                .config(ReasonerConfig {
                    explanations: true,
                    ..ReasonerConfig::default()
                })
                .build(ontology)?;
            let graph = explain_with_profile(&mut reasoner)?;
            emit_explain(cli.format, reasoner.ontology(), &graph, &parse_meta)?;
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

fn skip_empty_clashes(clashes: &&[String]) -> bool {
    clashes.is_empty()
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
struct RdfsCliOutput<'a> {
    status: &'static str,
    initial_axiom_count: usize,
    final_axiom_count: usize,
    inferred_axioms: usize,
    inferred_by_rule: &'a BTreeMap<ontologos_rdfs::RdfsRule, usize>,
    clash_count: usize,
    #[serde(skip_serializing_if = "skip_empty_clashes")]
    clashes: &'a [String],
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct RlCliOutput<'a> {
    status: &'static str,
    initial_axiom_count: usize,
    final_axiom_count: usize,
    inferred_axioms: usize,
    inferred_by_rule: &'a BTreeMap<ontologos_rl::RlRule, usize>,
    clash_count: usize,
    #[serde(skip_serializing_if = "skip_empty_clashes")]
    clashes: &'a [String],
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct TaxonomyCliOutput<'a> {
    status: &'static str,
    subsumption_count: usize,
    subsumptions: &'a [(String, String)],
    equivalences: &'a [Vec<String>],
    unsatisfiable: &'a [String],
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

fn emit_classify_outcome(
    format: OutputFormat,
    outcome: &ClassifyOutcome,
    ontology: &Ontology,
    parse_meta: &ParseMetaSummary,
) -> Result<(), CliError> {
    match outcome {
        ClassifyOutcome::Taxonomy(taxonomy) => {
            emit_taxonomy(format, "classified", taxonomy, ontology, parse_meta)
        }
        ClassifyOutcome::Rdfs(report) => emit_rdfs_report(format, "classified", report, parse_meta),
        ClassifyOutcome::Rl(report) => emit_rl_report(format, "classified", report, parse_meta),
    }
}

fn emit_taxonomy(
    format: OutputFormat,
    status: &'static str,
    taxonomy: &Taxonomy,
    ontology: &Ontology,
    parse_meta: &ParseMetaSummary,
) -> Result<(), CliError> {
    let subsumptions: Vec<(String, String)> = taxonomy
        .subsumptions
        .iter()
        .map(|&(sub, sup)| Ok((entity_iri(ontology, sub)?, entity_iri(ontology, sup)?)))
        .collect::<Result<Vec<_>, CliError>>()?;
    let equivalences: Vec<Vec<String>> = taxonomy
        .equivalences
        .iter()
        .map(|cluster| {
            cluster
                .iter()
                .map(|&id| entity_iri(ontology, id))
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<_>, _>>()?;
    let unsatisfiable: Vec<String> = taxonomy
        .unsatisfiable
        .iter()
        .map(|&id| entity_iri(ontology, id))
        .collect::<Result<Vec<_>, _>>()?;

    match format {
        OutputFormat::Text => {
            println!("status: {status}");
            println!("subsumption_count: {}", taxonomy.subsumption_count());
            println!("equivalence_class_count: {}", equivalences.len());
            println!("unsatisfiable_count: {}", unsatisfiable.len());
            if !subsumptions.is_empty() {
                println!("subsumptions:");
                for (sub, sup) in &subsumptions {
                    println!("  {sub} ⊑ {sup}");
                }
            }
            if !equivalences.is_empty() {
                println!("equivalences:");
                for cluster in &equivalences {
                    println!("  {}", cluster.join(" ≡ "));
                }
            }
            if !unsatisfiable.is_empty() {
                println!("unsatisfiable:");
                for iri in &unsatisfiable {
                    println!("  {iri}");
                }
            }
        }
        OutputFormat::Json => emit_json(&TaxonomyCliOutput {
            status,
            subsumption_count: taxonomy.subsumption_count(),
            subsumptions: &subsumptions,
            equivalences: &equivalences,
            unsatisfiable: &unsatisfiable,
            parse_meta,
        })?,
    }
    Ok(())
}

fn emit_rdfs_report(
    format: OutputFormat,
    status: &'static str,
    report: &RdfsReport,
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
            if !report.clashes.is_empty() {
                println!("clash_count: {}", report.clashes.len());
                println!("clashes:");
                for clash in &report.clashes {
                    println!("  {clash}");
                }
            }
        }
        OutputFormat::Json => emit_json(&RdfsCliOutput {
            status,
            initial_axiom_count: report.initial_axiom_count,
            final_axiom_count: report.final_axiom_count,
            inferred_axioms: report.inferred_total(),
            inferred_by_rule: &report.inferred_by_rule,
            clash_count: report.clashes.len(),
            clashes: &report.clashes,
            parse_meta,
        })?,
    }
    Ok(())
}

fn emit_rl_report(
    format: OutputFormat,
    status: &'static str,
    report: &RlReport,
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
            if !report.clashes.is_empty() {
                println!("clash_count: {}", report.clashes.len());
                println!("clashes:");
                for clash in &report.clashes {
                    println!("  {clash}");
                }
            }
        }
        OutputFormat::Json => emit_json(&RlCliOutput {
            status,
            initial_axiom_count: report.initial_axiom_count,
            final_axiom_count: report.final_axiom_count,
            inferred_axioms: report.inferred_total(),
            inferred_by_rule: &report.inferred_by_rule,
            clash_count: report.clashes.len(),
            clashes: &report.clashes,
            parse_meta,
        })?,
    }
    Ok(())
}

fn entity_iri(ontology: &Ontology, id: EntityId) -> Result<String, CliError> {
    let record = ontology.entity(id)?;
    Ok(ontology.resolve_iri(record.iri)?.to_owned())
}

fn emit_json<T: Serialize>(value: &T) -> Result<(), CliError> {
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| CliError::Core(ontologos_core::Error::Serialization(e.to_string())))?;
    println!("{json}");
    Ok(())
}

fn emit_explain(
    format: OutputFormat,
    ontology: &Ontology,
    graph: &ProofGraph,
    parse_meta: &ParseMetaSummary,
) -> Result<(), CliError> {
    match format {
        OutputFormat::Text => {
            println!("status: explained");
            println!("node_count: {}", graph.node_count());
            if graph.node_count() > 0 {
                println!("{}", render_text(ontology, graph));
            }
        }
        OutputFormat::Json => emit_json(&ExplainCliOutput { graph, parse_meta })?,
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
