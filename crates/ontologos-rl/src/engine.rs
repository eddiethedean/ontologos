use ontologos_core::Ontology;
use ontologos_rdfs::RdfsEngine;

use crate::report::MaterializationReport;
use crate::rules::{apply_batch_a, apply_batch_b, RuleContext};
use crate::triple_index::TripleIndex;

/// OWL RL forward-chaining engine with optional parallel execution.
#[derive(Debug)]
pub struct RlEngine {
    parallelism: usize,
    record_traces: bool,
}

const MAX_PARALLELISM: usize = 64;

impl RlEngine {
    #[must_use]
    pub fn new(parallelism: usize) -> Self {
        Self {
            parallelism,
            record_traces: false,
        }
    }

    /// Validate parallelism is within supported bounds.
    pub fn try_new(parallelism: usize) -> crate::Result<Self> {
        if parallelism == 0 || parallelism > MAX_PARALLELISM {
            return Err(crate::Error::Core(ontologos_core::Error::Message(format!(
                "parallelism must be in 1..={MAX_PARALLELISM}, got {parallelism}"
            ))));
        }
        Ok(Self {
            parallelism,
            record_traces: false,
        })
    }

    /// Enable recording of individual inference traces (for explain v0.6).
    #[must_use]
    pub fn with_traces(mut self, enabled: bool) -> Self {
        self.record_traces = enabled;
        self
    }

    /// Run RDFS materialization then OWL RL saturation until fixed point.
    pub fn saturate(&self, ontology: &mut Ontology) -> crate::Result<MaterializationReport> {
        let initial_axiom_count = ontology.axiom_count();
        let rdfs_report = RdfsEngine::new()
            .with_traces(self.record_traces)
            .materialize(ontology)
            .map_err(|e| match e {
                ontologos_rdfs::Error::Core(core) => crate::Error::Core(core),
                ontologos_rdfs::Error::WrongProfile { expected, actual } => {
                    crate::Error::WrongProfile { expected, actual }
                }
            })?;
        let rdfs_inferred = rdfs_report.inferred_total();

        let mut report = MaterializationReport {
            initial_axiom_count,
            final_axiom_count: ontology.axiom_count(),
            rdfs_inferred,
            inferred_by_rule: std::collections::BTreeMap::new(),
            traces: Vec::new(),
            clashes: Vec::new(),
        };

        let mut index = TripleIndex::from_ontology(ontology);

        loop {
            let before = ontology.axiom_count();
            let parallelism = self.parallelism;
            let record_traces = self.record_traces;
            let mut ctx = RuleContext {
                ontology,
                index: &mut index,
                report: &mut report,
                record_traces,
                parallelism,
            };
            apply_batch_a(&mut ctx)?;
            apply_batch_b(&mut ctx)?;
            if ontology.axiom_count() == before {
                break;
            }
            index.sync_from_ontology(ontology);
        }

        report.final_axiom_count = ontology.axiom_count();
        Ok(report)
    }
}
