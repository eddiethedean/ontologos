use ontologos_core::{Error as CoreError, Profile, Reasoner, Taxonomy};

use crate::{ElClassifier, ElEngine, ElReport, take_el_session};

/// Classify when the reasoner profile is [`Profile::El`].
pub fn classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<Taxonomy> {
    ElEngine.classify_taxonomy(reasoner)
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
    let incremental = reasoner.config().incremental;

    if incremental {
        crate::normal_form::validate_el_profile(reasoner.ontology())?;
        let session = take_el_session(reasoner);
        let (report, session) = ElClassifier::new().classify_incremental(
            reasoner.ontology_mut(),
            session,
            record_traces,
        )?;
        reasoner.set_session(Box::new(session));
        Ok(report)
    } else {
        let report =
            ElClassifier::new().classify_with_options(reasoner.ontology(), record_traces)?;
        reasoner.clear_session();
        Ok(report)
    }
}

/// Classify when the reasoner profile is [`Profile::El`]; otherwise returns
/// [`CoreError::NotImplemented`].
pub fn try_classify_reasoner(reasoner: &mut Reasoner) -> crate::Result<Taxonomy> {
    match reasoner.profile() {
        Profile::El => classify_reasoner(reasoner),
        _ => Err(crate::Error::Core(CoreError::NotImplemented)),
    }
}
