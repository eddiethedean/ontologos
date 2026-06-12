use ontologos_core::{Profile, Reasoner};

use crate::engine::RdfsEngine;
use crate::report::MaterializationReport;

/// Materialize RDFS inferences for a reasoner configured with [`Profile::Rdfs`].
pub fn materialize_reasoner(reasoner: &mut Reasoner) -> crate::Result<MaterializationReport> {
    if reasoner.profile() != Profile::Rdfs {
        return Err(crate::Error::WrongProfile {
            expected: Profile::Rdfs,
            actual: reasoner.profile(),
        });
    }
    let record_traces = reasoner.config().explanations;
    RdfsEngine::new()
        .with_traces(record_traces)
        .materialize(reasoner.ontology_mut())
}
