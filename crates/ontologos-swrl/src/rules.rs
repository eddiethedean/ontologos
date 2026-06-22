//! SWRL rule application entry points.

use ontologos_core::Ontology;

use crate::engine::materialize_swrl_rules;
use crate::Result;

/// Report from SWRL rule application.
#[derive(Debug, Clone, Default)]
pub struct SwrlReport {
    /// Rules discovered in ontology metadata.
    pub rules_found: usize,
    /// New inferences materialized.
    pub inferences_added: usize,
}

/// Extract and apply DLSafe SWRL rules via forward chaining.
pub fn apply_swrl_rules(ontology: &mut Ontology) -> Result<SwrlReport> {
    materialize_swrl_rules(ontology).map_err(crate::Error::Core)
}
