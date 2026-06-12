use ontologos_bridge::{core_to_triples, merge_triples_into_ontology};
use ontologos_core::Ontology;
use reasonable::reasoner::ReasonerBuilder;

use crate::report::MaterializationReport;

/// RDFS materialization facade over `reasonable`.
#[derive(Debug, Default)]
pub struct RdfsEngine {
    _record_traces: bool,
}

impl RdfsEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_traces(mut self, enabled: bool) -> Self {
        self._record_traces = enabled;
        self
    }

    /// Materialize RDFS/RL inferences into `ontology` via reasonable.
    pub fn materialize(&self, ontology: &mut Ontology) -> crate::Result<MaterializationReport> {
        let initial_axiom_count = ontology.axiom_count();
        let triples = core_to_triples(ontology).map_err(crate::Error::Bridge)?;
        let mut reasoner = ReasonerBuilder::new()
            .with_triples(triples)
            .build()
            .map_err(crate::Error::Reasonable)?;
        reasoner.reason_full();
        let output = reasoner.view_output().to_vec();
        let diagnostics = reasoner.diagnostics();
        let merge = merge_triples_into_ontology(ontology, &output, diagnostics)
            .map_err(crate::Error::Bridge)?;
        let _ = merge.inferred_axioms;

        Ok(MaterializationReport {
            initial_axiom_count,
            final_axiom_count: ontology.axiom_count(),
            inferred_by_rule: std::collections::BTreeMap::new(),
            trace: ontologos_core::InferenceTrace::new(),
        })
    }
}
