use std::collections::BTreeMap;

use ontologos_core::AxiomId;
use serde::Serialize;

/// RDFS TBox rule that produced an inference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RdfsRule {
    /// Transitive `rdfs:subClassOf`.
    ScTrans,
    /// Transitive `rdfs:subPropertyOf`.
    SpTrans,
    /// Domain inheritance along the property hierarchy (RDFS 6).
    DomInherit,
    /// Range inheritance along the property hierarchy (RDFS 8).
    RngInherit,
}

impl RdfsRule {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ScTrans => "sc_trans",
            Self::SpTrans => "sp_trans",
            Self::DomInherit => "dom_inherit",
            Self::RngInherit => "rng_inherit",
        }
    }
}

/// A single recorded inference (optional trace for explain v0.6).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct InferenceRecord {
    pub rule: RdfsRule,
    pub premises: Vec<AxiomId>,
    pub conclusion: AxiomId,
}

/// Summary of RDFS materialization over an ontology.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MaterializationReport {
    pub initial_axiom_count: usize,
    pub final_axiom_count: usize,
    pub inferred_by_rule: BTreeMap<RdfsRule, usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub traces: Vec<InferenceRecord>,
}

impl MaterializationReport {
    #[must_use]
    pub fn inferred_total(&self) -> usize {
        self.final_axiom_count
            .saturating_sub(self.initial_axiom_count)
    }
}
