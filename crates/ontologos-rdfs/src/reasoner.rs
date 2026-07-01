use ontologos_core::{Error as CoreError, Profile, Reasoner};

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
    materialize_routed(reasoner)
}

/// Materialize after profile routing selected RDFS (including [`Profile::Auto`]).
pub fn materialize_routed(reasoner: &mut Reasoner) -> crate::Result<MaterializationReport> {
    let record_traces = reasoner.config().explanations;
    RdfsEngine::new()
        .with_traces(record_traces)
        .materialize_with_reasoner(reasoner)
}

/// Run classification when the reasoner profile is [`Profile::Rdfs`]; otherwise returns
/// [`CoreError::NotImplemented`].
pub fn classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<()> {
    match reasoner.profile() {
        Profile::Rdfs => materialize_reasoner(reasoner).map(|_| ()),
        _ => Err(crate::Error::Core(CoreError::NotImplemented)),
    }
}
