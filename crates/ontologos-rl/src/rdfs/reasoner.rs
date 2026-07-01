use ontologos_core::{Error as CoreError, Profile, Reasoner};

use crate::rdfs::engine::RdfsEngine;
use crate::rdfs::report::MaterializationReport;

/// Materialize RDFS inferences for a reasoner configured with [`Profile::Rdfs`].
pub fn materialize_reasoner(reasoner: &mut Reasoner) -> super::Result<MaterializationReport> {
    if reasoner.profile() != Profile::Rdfs {
        return Err(super::Error::WrongProfile {
            expected: Profile::Rdfs,
            actual: reasoner.profile(),
        });
    }
    materialize_routed(reasoner)
}

/// Materialize after profile routing selected RDFS (including [`Profile::Auto`]).
pub fn materialize_routed(reasoner: &mut Reasoner) -> super::Result<MaterializationReport> {
    let record_traces = reasoner.config().explanations;
    RdfsEngine::new()
        .with_traces(record_traces)
        .materialize_with_reasoner(reasoner)
}

/// Run classification when the reasoner profile is [`Profile::Rdfs`]; otherwise returns
/// [`CoreError::NotImplemented`].
pub fn classify_reasoner(reasoner: &mut Reasoner) -> super::Result<()> {
    match reasoner.profile() {
        Profile::Rdfs => materialize_reasoner(reasoner).map(|_| ()),
        _ => Err(super::Error::Core(CoreError::NotImplemented)),
    }
}
