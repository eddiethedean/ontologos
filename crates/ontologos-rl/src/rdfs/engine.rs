use ontologos_bridge::{
    MaterializeOutcome, MergeLimits, materialize_with_session, take_reasonable_session,
};
use ontologos_core::{Ontology, Profile, Reasoner};

use crate::rdfs::report::MaterializationReport;

/// RDFS materialization facade over `reasonable`.
#[derive(Debug, Default)]
pub struct RdfsEngine {
    _record_traces: bool,
    merge_limits: MergeLimits,
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

    /// Cap axioms after materialization (default: 10_000_000).
    #[must_use]
    pub fn with_merge_limits(mut self, limits: MergeLimits) -> Self {
        self.merge_limits = limits;
        self
    }

    /// Materialize RDFS/RL inferences into `ontology` via reasonable.
    pub fn materialize(&self, ontology: &mut Ontology) -> super::Result<MaterializationReport> {
        let initial_axiom_count = ontology.axiom_count();
        let (outcome, _) = materialize_with_session(
            ontology,
            ontologos_bridge::ReasonableSession::new_for_profile(Profile::Rdfs),
            false,
            self.merge_limits,
        )
        .map_err(|boxed| super::Error::Bridge(boxed.0))?;
        Ok(report_from_outcome(
            initial_axiom_count,
            ontology.axiom_count(),
            outcome,
        ))
    }

    /// Materialize using incremental session on `reasoner`.
    pub fn materialize_with_reasoner(
        &self,
        reasoner: &mut Reasoner,
    ) -> super::Result<MaterializationReport> {
        let initial_axiom_count = reasoner.ontology().axiom_count();
        let incremental = reasoner.config().incremental;
        let session = take_reasonable_session(reasoner, Profile::Rdfs);
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
                Err(super::Error::Bridge(e))
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
        inferred_by_rule: std::collections::BTreeMap::new(),
        trace: ontologos_core::InferenceTrace::new(),
        clashes: outcome.merge.clashes,
    }
}
