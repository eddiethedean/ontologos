use ontologos_core::{Error as CoreError, Profile, Reasoner};

use crate::engine::RlEngine;
use crate::report::MaterializationReport;

/// Materialize OWL RL inferences for a reasoner configured with [`Profile::Rl`].
pub fn materialize_reasoner(reasoner: &mut Reasoner) -> crate::Result<MaterializationReport> {
    if reasoner.profile() != Profile::Rl {
        return Err(crate::Error::WrongProfile {
            expected: Profile::Rl,
            actual: reasoner.profile(),
        });
    }
    let parallelism = reasoner.config().parallelism;
    let record_traces = reasoner.config().explanations;
    RlEngine::try_new(parallelism)?
        .with_traces(record_traces)
        .saturate(reasoner.ontology_mut())
}

/// Run classification when the reasoner profile is [`Profile::Rl`].
pub fn classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<()> {
    match reasoner.profile() {
        Profile::Rl => materialize_reasoner(reasoner).map(|_| ()),
        _ => Err(crate::Error::Core(CoreError::NotImplemented)),
    }
}
