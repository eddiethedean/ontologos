//! Shared JSON report shapes for CLI and Python parity.

use ontologos_core::{Ontology, ParseMetaSummary, Taxonomy};
use ontologos_rl::rdfs::{MaterializationReport as RdfsReport, RdfsRule};
use ontologos_rl::MaterializationReport as RlReport;
use serde::Serialize;
use std::collections::BTreeMap;

use crate::error::{Error, Result};

fn entity_iri(ontology: &Ontology, id: ontologos_core::EntityId) -> Result<String> {
    let record = ontology.entity(id).map_err(Error::Core)?;
    ontology
        .resolve_iri(record.iri)
        .map(|s| s.to_owned())
        .map_err(Error::Core)
}

fn skip_clean_parse_meta(meta: &Option<&ParseMetaSummary>) -> bool {
    meta.is_none_or(|m| m.omit_from_json())
}

fn skip_empty_clashes(clashes: &&[String]) -> bool {
    clashes.is_empty()
}

/// Taxonomy classification JSON (EL/DL/ALC).
#[derive(Serialize)]
pub struct TaxonomyJson<'a> {
    /// Outcome status (`classified`, etc.).
    pub status: &'static str,
    /// Number of subsumption edges.
    pub subsumption_count: usize,
    /// Named class subsumptions as IRI pairs.
    pub subsumptions: Vec<(String, String)>,
    /// Equivalence class clusters.
    pub equivalences: Vec<Vec<String>>,
    /// Unsatisfiable class IRIs.
    pub unsatisfiable: Vec<String>,
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: Option<&'a ParseMetaSummary>,
}

/// RDFS/RL materialization JSON shared by classify and materialize commands.
#[derive(Serialize)]
pub struct MaterializationJson<'a, R> {
    /// Outcome status (`classified`, etc.).
    pub status: &'static str,
    pub initial_axiom_count: usize,
    pub final_axiom_count: usize,
    pub inferred_axioms: usize,
    pub inferred_by_rule: &'a BTreeMap<R, usize>,
    pub clash_count: usize,
    #[serde(skip_serializing_if = "skip_empty_clashes")]
    pub clashes: &'a [String],
    #[serde(skip_serializing_if = "skip_clean_parse_meta")]
    parse_meta: Option<&'a ParseMetaSummary>,
}

/// Build taxonomy JSON fields from a classified ontology.
pub fn taxonomy_json<'a>(
    status: &'static str,
    taxonomy: &Taxonomy,
    ontology: &Ontology,
    parse_meta: Option<&'a ParseMetaSummary>,
) -> Result<TaxonomyJson<'a>> {
    let subsumptions: Vec<(String, String)> = taxonomy
        .subsumptions
        .iter()
        .map(|&(sub, sup)| Ok((entity_iri(ontology, sub)?, entity_iri(ontology, sup)?)))
        .collect::<Result<Vec<_>>>()?;
    let equivalences: Vec<Vec<String>> = taxonomy
        .equivalences
        .iter()
        .map(|cluster| {
            cluster
                .iter()
                .map(|&id| entity_iri(ontology, id))
                .collect::<Result<Vec<_>>>()
        })
        .collect::<Result<Vec<_>>>()?;
    let unsatisfiable: Vec<String> = taxonomy
        .unsatisfiable
        .iter()
        .map(|&id| entity_iri(ontology, id))
        .collect::<Result<Vec<_>>>()?;
    Ok(TaxonomyJson {
        status,
        subsumption_count: taxonomy.subsumption_count(),
        subsumptions,
        equivalences,
        unsatisfiable,
        parse_meta,
    })
}

/// Build RDFS materialization JSON from a saturation report.
pub fn rdfs_materialization_json<'a>(
    status: &'static str,
    report: &'a RdfsReport,
    parse_meta: Option<&'a ParseMetaSummary>,
) -> MaterializationJson<'a, RdfsRule> {
    materialization_json(status, report, parse_meta)
}

/// Build RL materialization JSON from a saturation report.
pub fn rl_materialization_json<'a>(
    status: &'static str,
    report: &'a RlReport,
    parse_meta: Option<&'a ParseMetaSummary>,
) -> MaterializationJson<'a, ontologos_rl::RlRule> {
    materialization_json(status, report, parse_meta)
}

fn materialization_json<'a, R: Serialize>(
    status: &'static str,
    report: &'a impl MaterializationReportView<R>,
    parse_meta: Option<&'a ParseMetaSummary>,
) -> MaterializationJson<'a, R> {
    MaterializationJson {
        status,
        initial_axiom_count: report.initial_axiom_count(),
        final_axiom_count: report.final_axiom_count(),
        inferred_axioms: report.inferred_total(),
        inferred_by_rule: report.inferred_by_rule(),
        clash_count: report.clashes().len(),
        clashes: report.clashes(),
        parse_meta,
    }
}

trait MaterializationReportView<R> {
    fn initial_axiom_count(&self) -> usize;
    fn final_axiom_count(&self) -> usize;
    fn inferred_total(&self) -> usize;
    fn inferred_by_rule(&self) -> &BTreeMap<R, usize>;
    fn clashes(&self) -> &[String];
}

impl MaterializationReportView<RdfsRule> for RdfsReport {
    fn initial_axiom_count(&self) -> usize {
        self.initial_axiom_count
    }
    fn final_axiom_count(&self) -> usize {
        self.final_axiom_count
    }
    fn inferred_total(&self) -> usize {
        self.inferred_total()
    }
    fn inferred_by_rule(&self) -> &BTreeMap<RdfsRule, usize> {
        &self.inferred_by_rule
    }
    fn clashes(&self) -> &[String] {
        &self.clashes
    }
}

impl MaterializationReportView<ontologos_rl::RlRule> for RlReport {
    fn initial_axiom_count(&self) -> usize {
        self.initial_axiom_count
    }
    fn final_axiom_count(&self) -> usize {
        self.final_axiom_count
    }
    fn inferred_total(&self) -> usize {
        self.inferred_total()
    }
    fn inferred_by_rule(&self) -> &BTreeMap<ontologos_rl::RlRule, usize> {
        &self.inferred_by_rule
    }
    fn clashes(&self) -> &[String] {
        &self.clashes
    }
}
