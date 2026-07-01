use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueEnum};
use ontologos_core::ReasonerConfig;
use ontologos_core::{EntityId, Ontology, ParseMetaSummary, Profile, Reasoner, Taxonomy};
use ontologos_facade::ClassifyOutcome;
use ontologos_explain::{ProofGraph, explain_with_profile, render_text};
use ontologos_parser::load_ontology_lenient as load_ontology;
use ontologos_profile::{ProfileReport, classify_hybrid, detect_profile};
use ontologos_rdfs::MaterializationReport as RdfsReport;
use ontologos_rl::MaterializationReport as RlReport;
use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Parser)]
#[command(
    name = "ontologos",
    about = "Modular Rust ontology reasoner",
    after_help = "v1.0.0: profile (detect), materialize (RDFS), classify (auto|el|rl|rdfs|alc|dl|dl-preview|swrl), \
                  explain (proof graphs). Preview profiles: see docs/guides/preview-profiles.md. \
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

    /// Wall-clock budget in seconds for DL consistency/classify (0 = unlimited)
    #[arg(long)]
    budget_secs: Option<u64>,

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
    /// Answer an OWL QL conjunctive query over a classified ontology
    Query {
        ontology: PathBuf,
        /// Conjunctive query, e.g. `Type(?x, http://ex.org/A)`
        #[arg(long)]
        query: String,
    },
    /// List ABox individuals, types, and sameAs clusters after RL materialization
    Instances { ontology: PathBuf },
    /// Check ontology consistency (OWLReasoner-style)
    Consistent { ontology: PathBuf },
    /// Check entailment (`SubClassOf`, `ClassAssertion`, or `ObjectPropertyAssertion`)
    Entail {
        ontology: PathBuf,
        /// Subclass IRI (`SubClassOf` entailment)
        #[arg(long)]
        sub: Option<String>,
        /// Superclass IRI (`SubClassOf` entailment)
        #[arg(long)]
        sup: Option<String>,
        /// Individual IRI (`ClassAssertion` entailment)
        #[arg(long)]
        individual: Option<String>,
        /// Class IRI (`ClassAssertion` entailment)
        #[arg(long)]
        class: Option<String>,
        /// Subject individual IRI (`ObjectPropertyAssertion` entailment)
        #[arg(long)]
        subject: Option<String>,
        /// Object property IRI (`ObjectPropertyAssertion` entailment)
        #[arg(long)]
        property: Option<String>,
        /// Object individual IRI (`ObjectPropertyAssertion` entailment)
        #[arg(long)]
        object: Option<String>,
    },
    /// List sub-object-properties of a named property (OWL API `getSubObjectProperties`)
    Subproperties {
        ontology: PathBuf,
        /// Object property IRI
        #[arg(long)]
        property: String,
        /// Direct sub-properties only
        #[arg(long, default_value_t = false)]
        direct: bool,
    },
    /// List object property values for an individual (OWL API `getObjectPropertyValues`)
    PropertyValues {
        ontology: PathBuf,
        /// Subject individual IRI
        #[arg(long)]
        subject: String,
        /// Object property IRI
        #[arg(long)]
        property: String,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, Default, PartialEq, Eq)]
enum CliProfile {
    #[default]
    Auto,
    El,
    Rl,
    Rdfs,
    Alc,
    Dl,
    /// OWL 2 DL preview (subset checks, explicit warning)
    DlPreview,
}

impl From<CliProfile> for Profile {
    fn from(value: CliProfile) -> Self {
        match value {
            CliProfile::Auto => Profile::Auto,
            CliProfile::El => Profile::El,
            CliProfile::Rl => Profile::Rl,
            CliProfile::Rdfs => Profile::Rdfs,
            CliProfile::Alc | CliProfile::Dl => Profile::Dl,
            CliProfile::DlPreview => Profile::DlPreview,
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
    Dl(#[from] ontologos_dl::Error),
    #[error(transparent)]
    Rl(#[from] ontologos_rl::Error),
    #[error(transparent)]
    Explain(#[from] ontologos_explain::Error),
}

fn map_facade_error(error: ontologos_facade::Error) -> CliError {
    match error {
        ontologos_facade::Error::El(inner) => CliError::El(inner),
        ontologos_facade::Error::Dl(inner) => CliError::Dl(inner),
        ontologos_facade::Error::Rl(inner) => CliError::Rl(inner),
        ontologos_facade::Error::Core(inner) => CliError::Core(inner),
    }
}

fn reasoner_config(cli: &Cli) -> ReasonerConfig {
    ReasonerConfig {
        incremental: cli.incremental,
        budget_secs: cli.budget_secs,
        ..ReasonerConfig::default()
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), CliError> {
    let cli = Cli::parse();

    match &cli.command {
        Command::Profile { ontology } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let report = detect_profile(&ontology)?;
            let hybrid = classify_hybrid(&ontology).ok();
            match cli.format {
                OutputFormat::Text => {
                    println!("{}", format_profile_text(&report));
                    if let Some(h) = &hybrid {
                        println!("hybrid modules: {}", h.modules.len());
                        for m in &h.modules {
                            println!(
                                "  {:?} engine={} axioms={} classes={}",
                                m.profile,
                                ontologos_profile::engine_for_profile(m.profile),
                                m.axiom_ids.len(),
                                m.signature.len()
                            );
                        }
                    }
                }
                OutputFormat::Json => emit_json(&ProfileCliOutput {
                    report: &report,
                    parse_meta: &parse_meta,
                    hybrid_modules: hybrid.as_ref().map(|h| h.modules.len()).unwrap_or(0),
                })?,
            }
        }
        Command::Classify { ontology } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            if cli.profile == CliProfile::DlPreview {
                eprintln!(
                    "warning: DL preview mode — incomplete reasoning, not suitable for production"
                );
            }
            let mut reasoner = Reasoner::builder()
                .profile(cli.profile.into())
                .config(reasoner_config(&cli))
                .build(ontology)?;
            let outcome = ontologos_facade::classify(&mut reasoner).map_err(map_facade_error)?;
            emit_classify_outcome(cli.format, &outcome, reasoner.ontology(), &parse_meta)?;
        }
        Command::Materialize { ontology } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let mut reasoner = Reasoner::builder()
                .profile(Profile::Rdfs)
                .config(reasoner_config(&cli))
                .build(ontology)?;
            let report = ontologos_rdfs::materialize_reasoner(&mut reasoner)?;
            emit_rdfs_report(cli.format, "materialized", &report, &parse_meta)?;
        }
        Command::Explain { ontology } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let mut reasoner = Reasoner::builder()
                .profile(cli.profile.into())
                .config(ReasonerConfig {
                    explanations: true,
                    ..reasoner_config(&cli)
                })
                .build(ontology)?;
            let graph = explain_with_profile(&mut reasoner)?;
            emit_explain(cli.format, reasoner.ontology(), &graph, &parse_meta)?;
        }
        Command::Query { ontology, query } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let cq = ontologos_ql::parse_conjunctive_query(query)
                .map_err(|e| CliError::Core(ontologos_core::Error::Message(e.to_string())))?;
            let mut reasoner = Reasoner::builder()
                .profile(cli.profile.into())
                .config(reasoner_config(&cli))
                .build(ontology)?;
            let outcome = ontologos_facade::classify(&mut reasoner).map_err(map_facade_error)?;
            let taxonomy = ontologos_facade::taxonomy_from_outcome(&outcome).ok_or_else(|| {
                CliError::Core(ontologos_core::Error::Message(
                    "query requires taxonomy classification outcome".into(),
                ))
            })?;
            let answers = ontologos_ql::answer_query(reasoner.ontology(), taxonomy, &cq)
                .map_err(|e| CliError::Core(ontologos_core::Error::Message(e.to_string())))?;
            match cli.format {
                OutputFormat::Text => {
                    println!("answers: {}", answers.len());
                    for answer in &answers {
                        for (var, id) in &answer.bindings {
                            let iri = entity_iri(reasoner.ontology(), *id)?;
                            println!("  ?{var} -> {iri}");
                        }
                    }
                }
                OutputFormat::Json => {
                    let results: Vec<QueryAnswer> = answers
                        .iter()
                        .map(|answer| {
                            let bindings = answer
                                .bindings
                                .iter()
                                .map(|(var, id)| {
                                    let iri = entity_iri(reasoner.ontology(), *id)?;
                                    Ok(QueryBinding {
                                        variable: var.clone(),
                                        iri,
                                    })
                                })
                                .collect::<Result<Vec<_>, CliError>>()?;
                            Ok(QueryAnswer { bindings })
                        })
                        .collect::<Result<Vec<_>, CliError>>()?;
                    emit_json(&QueryCliOutput {
                        answers: answers.len(),
                        results,
                        parse_meta: &parse_meta,
                    })?
                }
            }
        }
        Command::Instances { ontology } => {
            let mut ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let report = ontologos_abox::materialize_abox(&mut ontology)
                .map_err(|e| CliError::Core(ontologos_core::Error::Message(e.to_string())))?;
            let consistent = ontologos_abox::is_abox_consistent(&ontology)
                .map_err(|e| CliError::Core(ontologos_core::Error::Message(e.to_string())))?;
            match cli.format {
                OutputFormat::Text => {
                    println!("consistent: {consistent}");
                    println!("rl_inferences: {}", report.rl_inferences);
                    println!("same_as_clusters: {}", report.same_as_clusters.len());
                    for (i, cluster) in report.same_as_clusters.iter().enumerate() {
                        let names: Vec<_> = cluster
                            .iter()
                            .filter_map(|id| {
                                ontology
                                    .entity(*id)
                                    .ok()
                                    .and_then(|r| ontology.resolve_iri(r.iri).ok())
                            })
                            .collect();
                        println!("  cluster[{i}]: {}", names.join(", "));
                    }
                    for (_, axiom) in ontology.axioms().iter() {
                        if let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom {
                            let ind = entity_iri(&ontology, *individual)?;
                            let cls = entity_iri(&ontology, *class)?;
                            println!("  {ind} : {cls}");
                        }
                    }
                }
                OutputFormat::Json => emit_json(&InstancesCliOutput {
                    consistent,
                    rl_inferences: report.rl_inferences,
                    same_as_clusters: report.same_as_clusters.len(),
                    parse_meta: &parse_meta,
                })?,
            }
        }
        Command::Consistent { ontology } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let reasoner = Reasoner::builder()
                .profile(cli.profile.into())
                .config(reasoner_config(&cli))
                .build(ontology)?;
            let result =
                ontologos_facade::check_consistency(&reasoner).map_err(map_facade_error)?;
            match cli.format {
                OutputFormat::Text => {
                    if !result.complete {
                        eprintln!("warning: consistency check incomplete (budget or tableau limit)");
                    }
                    println!("consistent: {}", result.consistent);
                    println!("complete: {}", result.complete);
                }
                OutputFormat::Json => emit_json(&ConsistentCliOutput {
                    consistent: result.consistent,
                    complete: result.complete,
                    parse_meta: &parse_meta,
                })?,
            }
        }
        Command::Entail {
            ontology,
            sub,
            sup,
            individual,
            class,
            subject,
            property,
            object,
        } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let check = ontologos_facade::parse_entailment_check(
                sub.clone(),
                sup.clone(),
                individual.clone(),
                class.clone(),
                subject.clone(),
                property.clone(),
                object.clone(),
            )
            .map_err(map_facade_error)?;
            let mut reasoner = Reasoner::builder()
                .profile(cli.profile.into())
                .config(reasoner_config(&cli))
                .build(ontology)?;
            let entailed = ontologos_facade::is_entailed_axiom(&mut reasoner, check.clone())
                .map_err(map_facade_error)?;
            match cli.format {
                OutputFormat::Text => {
                    println!("entailed: {entailed}");
                    match check {
                        ontologos_facade::EntailmentCheck::SubClassOf { sub, sup } => {
                            println!("sub: {sub}");
                            println!("sup: {sup}");
                        }
                        ontologos_facade::EntailmentCheck::ClassAssertion { individual, class } => {
                            println!("individual: {individual}");
                            println!("class: {class}");
                        }
                        ontologos_facade::EntailmentCheck::ObjectPropertyAssertion {
                            subject,
                            property,
                            object,
                        } => {
                            println!("subject: {subject}");
                            println!("property: {property}");
                            println!("object: {object}");
                        }
                    }
                }
                OutputFormat::Json => emit_json(&EntailCliOutput {
                    entailed,
                    check: &check,
                    parse_meta: &parse_meta,
                })?,
            }
        }
        Command::Subproperties {
            ontology,
            property,
            direct,
        } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            let reasoner = Reasoner::builder()
                .profile(cli.profile.into())
                .config(reasoner_config(&cli))
                .build(ontology)?;
            let subs = ontologos_facade::get_sub_object_properties(&reasoner, property, *direct)
                .map_err(map_facade_error)?;
            match cli.format {
                OutputFormat::Text => {
                    println!("property: {property}");
                    println!("direct: {direct}");
                    println!("count: {}", subs.len());
                    for iri in &subs {
                        println!("  {iri}");
                    }
                }
                OutputFormat::Json => emit_json(&SubpropertiesCliOutput {
                    property,
                    direct: *direct,
                    subproperties: &subs,
                    parse_meta: &parse_meta,
                })?,
            }
        }
        Command::PropertyValues {
            ontology,
            subject,
            property,
        } => {
            let ontology = load_ontology(ontology)?;
            let parse_meta = parse_meta_summary(&ontology);
            emit_parse_meta_text(cli.format, &parse_meta);
            // ABox RL lookup; global `--profile` does not affect this command.
            let reasoner = Reasoner::builder().build(ontology)?;
            let values =
                ontologos_facade::get_object_property_values(&reasoner, subject, property)
                    .map_err(map_facade_error)?;
            match cli.format {
                OutputFormat::Text => {
                    println!("subject: {subject}");
                    println!("property: {property}");
                    println!("count: {}", values.len());
                    for iri in &values {
                        println!("  {iri}");
                    }
                }
                OutputFormat::Json => emit_json(&PropertyValuesCliOutput {
                    subject,
                    property,
                    values: &values,
                    parse_meta: &parse_meta,
                })?,
            }
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
    hybrid_modules: usize,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct QueryBinding {
    variable: String,
    iri: String,
}

#[derive(Serialize)]
struct QueryAnswer {
    bindings: Vec<QueryBinding>,
}

#[derive(Serialize)]
struct QueryCliOutput<'a> {
    answers: usize,
    results: Vec<QueryAnswer>,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct InstancesCliOutput<'a> {
    consistent: bool,
    rl_inferences: usize,
    same_as_clusters: usize,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct ConsistentCliOutput<'a> {
    consistent: bool,
    /// Whether the check finished without budget or tableau limits.
    complete: bool,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct EntailCliOutput<'a> {
    entailed: bool,
    #[serde(flatten)]
    check: &'a ontologos_facade::EntailmentCheck,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct SubpropertiesCliOutput<'a> {
    property: &'a str,
    direct: bool,
    subproperties: &'a [String],
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: &'a ParseMetaSummary,
}

#[derive(Serialize)]
struct PropertyValuesCliOutput<'a> {
    subject: &'a str,
    property: &'a str,
    values: &'a [String],
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
        OutputFormat::Json => {
            let output = ontologos_facade::taxonomy_json(status, taxonomy, ontology, Some(parse_meta))
                .map_err(map_facade_error)?;
            emit_json(&output)?;
        }
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
        OutputFormat::Json => {
            let output = ontologos_facade::rdfs_materialization_json(status, report, Some(parse_meta));
            emit_json(&output)?;
        }
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
        OutputFormat::Json => {
            let output = ontologos_facade::rl_materialization_json(status, report, Some(parse_meta));
            emit_json(&output)?;
        }
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
