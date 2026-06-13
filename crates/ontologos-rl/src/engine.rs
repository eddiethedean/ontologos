use ontologos_bridge::{
    materialize_with_session, take_reasonable_session, MaterializeOutcome, MergeLimits,
};
use ontologos_core::{Ontology, Profile, Reasoner};

use crate::report::MaterializationReport;

/// OWL RL facade over the `reasonable` engine.
#[derive(Debug)]
pub struct RlEngine {
    _record_traces: bool,
    merge_limits: MergeLimits,
}

impl RlEngine {
    /// Create an engine (`parallelism` is accepted for API compatibility; reasonable manages parallelism internally).
    #[must_use]
    pub fn new(_parallelism: usize) -> Self {
        Self {
            _record_traces: false,
            merge_limits: MergeLimits::default(),
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

    /// Cap axioms after saturation (default: 10_000_000).
    #[must_use]
    pub fn with_merge_limits(mut self, limits: MergeLimits) -> Self {
        self.merge_limits = limits;
        self
    }

    /// Saturate `ontology` via reasonable and merge inferred axioms back into core.
    pub fn saturate(&self, ontology: &mut Ontology) -> crate::Result<MaterializationReport> {
        let initial_axiom_count = ontology.axiom_count();
        let (outcome, _) = materialize_with_session(
            ontology,
            ontologos_bridge::ReasonableSession::new(),
            false,
            self.merge_limits,
        )
        .map_err(|boxed| crate::Error::Bridge(boxed.0))?;
        Ok(report_from_outcome(
            initial_axiom_count,
            ontology.axiom_count(),
            outcome,
        ))
    }

    /// Saturate using incremental session state on `reasoner`.
    pub fn saturate_reasoner(
        &self,
        reasoner: &mut Reasoner,
    ) -> crate::Result<MaterializationReport> {
        let initial_axiom_count = reasoner.ontology().axiom_count();
        let incremental = reasoner.config().incremental;
        let session = take_reasonable_session(reasoner, Profile::Rl);
        match materialize_with_session(
            reasoner.ontology_mut(),
            session,
            incremental,
            self.merge_limits,
        ) {
            Ok((outcome, session)) => {
                reasoner.set_session(Box::new(session));
                Ok(report_from_outcome(
                    initial_axiom_count,
                    reasoner.ontology().axiom_count(),
                    outcome,
                ))
            }
            Err(boxed) => {
                let (e, session) = *boxed;
                reasoner.set_session(Box::new(session));
                Err(crate::Error::Bridge(e))
            }
        }
    }
}

fn report_from_outcome(
    initial_axiom_count: usize,
    final_axiom_count: usize,
    outcome: MaterializeOutcome,
) -> MaterializationReport {
    MaterializationReport {
        initial_axiom_count,
        final_axiom_count,
        rdfs_inferred: 0,
        inferred_by_rule: std::collections::BTreeMap::new(),
        trace: ontologos_core::InferenceTrace::new(),
        clashes: outcome.merge.clashes,
        disjoint_clash_keys: std::collections::HashSet::new(),
        same_as_clash_keys: std::collections::HashSet::new(),
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
