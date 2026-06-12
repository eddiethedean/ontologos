use ontologos_core::{Error as CoreError, Profile, Reasoner, Taxonomy};

use crate::{ElClassifier, ElReport};

/// Classify when the reasoner profile is [`Profile::El`].
pub fn classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<Taxonomy> {
    classify_with_report(reasoner).map(|r| r.taxonomy)
}

/// Classify and return taxonomy plus optional inference trace.
pub fn classify_with_report(reasoner: &mut Reasoner) -> crate::Result<ElReport> {
    if reasoner.profile() != Profile::El {
        return Err(crate::Error::WrongProfile {
            expected: Profile::El,
            actual: reasoner.profile(),
        });
    }
    let record_traces = reasoner.config().explanations;
    ElClassifier::new().classify_with_options(reasoner.ontology(), record_traces)
}

/// Classify when the reasoner profile is [`Profile::El`]; otherwise returns
/// [`CoreError::NotImplemented`].
pub fn try_classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<Taxonomy> {
    match reasoner.profile() {
        Profile::El => classify_reasoner(reasoner),
        _ => Err(crate::Error::Core(CoreError::NotImplemented)),
    }
}
