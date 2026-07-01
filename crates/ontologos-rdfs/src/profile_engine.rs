//! RDFS profile engine adapter (DIP unit struct).

use ontologos_core::{Ontology, Reasoner};

use crate::report::MaterializationReport;

/// RDFS profile engine adapter (distinct from [`crate::RdfsEngine`] materializer).
#[derive(Debug, Default, Clone, Copy)]
pub struct RdfsEngineAdapter;

impl RdfsEngineAdapter {
    /// Materialize RDFS inferences for a reasoner configured with RDFS profile.
    pub fn materialize(&self, reasoner: &mut Reasoner) -> crate::Result<MaterializationReport> {
        crate::materialize_reasoner(reasoner)
    }

    /// Check consistency via RDFS materialization (no clashes).
    pub fn is_consistent(&self, ontology: &Ontology) -> crate::Result<bool> {
        let mut working = ontology.clone();
        let report = crate::RdfsEngine::new().materialize(&mut working)?;
        Ok(report.clashes.is_empty())
    }
}
