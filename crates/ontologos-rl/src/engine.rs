use ontologos_bridge::{core_to_triples, merge_triples_into_ontology};
use ontologos_core::Ontology;
use reasonable::reasoner::ReasonerBuilder;

use crate::report::MaterializationReport;

/// OWL RL facade over the `reasonable` engine.
#[derive(Debug)]
pub struct RlEngine {
    _record_traces: bool,
}

impl RlEngine {
    /// Create an engine (`parallelism` is accepted for API compatibility; reasonable manages parallelism internally).
    #[must_use]
    pub fn new(_parallelism: usize) -> Self {
        Self {
            _record_traces: false,
        }
    }

    /// Validate configuration (always succeeds for the reasonable adapter).
    pub fn try_new(parallelism: usize) -> crate::Result<Self> {
        if parallelism == 0 || parallelism > 64 {
            return Err(crate::Error::Core(ontologos_core::Error::Message(format!(
                "parallelism must be in 1..=64, got {parallelism}"
            ))));
        }
        Ok(Self::new(parallelism))
    }

    /// Enable trace recording (no-op for reasonable; traces remain empty until upstream support lands).
    #[must_use]
    pub fn with_traces(mut self, enabled: bool) -> Self {
        self._record_traces = enabled;
        self
    }

    /// Saturate `ontology` via reasonable and merge inferred axioms back into core.
    pub fn saturate(&self, ontology: &mut Ontology) -> crate::Result<MaterializationReport> {
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

        Ok(MaterializationReport {
            initial_axiom_count,
            final_axiom_count: ontology.axiom_count(),
            rdfs_inferred: 0,
            inferred_by_rule: std::collections::BTreeMap::new(),
            trace: ontologos_core::InferenceTrace::new(),
            clashes: merge.clashes,
            disjoint_clash_keys: std::collections::HashSet::new(),
            same_as_clash_keys: std::collections::HashSet::new(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::{Axiom, EntityKind};

    #[test]
    fn saturates_subclass_chain() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        let c = ontology
            .entity_id("http://ex.org/C", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: b,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: b,
                superclass: c,
            })
            .unwrap();

        let report = RlEngine::new(1).saturate(&mut ontology).unwrap();
        assert!(
            report.inferred_total() >= 1 || report.final_axiom_count > report.initial_axiom_count
        );
    }
}
