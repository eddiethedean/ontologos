use ontologos_core::{Error as CoreError, Profile, Reasoner, Taxonomy};

use crate::ElClassifier;

/// Classify when the reasoner profile is [`Profile::El`].
pub fn classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<Taxonomy> {
    if reasoner.profile() != Profile::El {
        return Err(crate::Error::WrongProfile {
            expected: Profile::El,
            actual: reasoner.profile(),
        });
    }
    ElClassifier::new().classify(reasoner.ontology())
}

/// Classify when the reasoner profile is [`Profile::El`]; otherwise returns
/// [`CoreError::NotImplemented`].
pub fn try_classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<Taxonomy> {
    match reasoner.profile() {
        Profile::El => classify_reasoner(reasoner),
        _ => Err(crate::Error::Core(CoreError::NotImplemented)),
    }
}
