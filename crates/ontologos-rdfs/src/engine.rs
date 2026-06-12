use ontologos_core::Ontology;

use crate::report::MaterializationReport;
use crate::rules::{
    apply_dom_inherit, apply_rng_inherit, apply_sc_trans, apply_sp_trans, RuleContext,
};

/// RDFS forward-chaining engine.
#[derive(Debug, Default)]
pub struct RdfsEngine {
    record_traces: bool,
}

impl RdfsEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable recording of individual inference traces (for explain v0.6).
    #[must_use]
    pub fn with_traces(mut self, enabled: bool) -> Self {
        self.record_traces = enabled;
        self
    }

    /// Materialize RDFS inferences into `ontology` until saturation.
    pub fn materialize(&self, ontology: &mut Ontology) -> crate::Result<MaterializationReport> {
        let initial_axiom_count = ontology.axiom_count();
        let mut report = MaterializationReport {
            initial_axiom_count,
            final_axiom_count: initial_axiom_count,
            inferred_by_rule: std::collections::BTreeMap::new(),
            trace: ontologos_core::InferenceTrace::new(),
        };

        loop {
            let before = ontology.axiom_count();
            let mut ctx = RuleContext {
                ontology,
                report: &mut report,
                record_traces: self.record_traces,
            };
            apply_sc_trans(&mut ctx)?;
            apply_sp_trans(&mut ctx)?;
            apply_dom_inherit(&mut ctx)?;
            apply_rng_inherit(&mut ctx)?;
            if ontology.axiom_count() == before {
                break;
            }
        }

        report.final_axiom_count = ontology.axiom_count();
        Ok(report)
    }
}
