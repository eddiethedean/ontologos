//! Unified reasoner facade — routes all OWL profiles without circular crate deps.

#![warn(missing_docs)]

use ontologos_core::{Profile, Reasoner, Taxonomy};
use ontologos_el::{classify_with_profile as el_classify, ClassifyOutcome};
use ontologos_profile::{detect_profile, OwlProfile};
use thiserror::Error;

/// Result type for facade operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Facade routing errors.
#[derive(Debug, Error)]
pub enum Error {
    /// EL engine error.
    #[error(transparent)]
    El(#[from] ontologos_el::Error),
    /// ALC engine error.
    #[error(transparent)]
    Alc(#[from] ontologos_alc::Error),
    /// DL engine error.
    #[error(transparent)]
    Dl(#[from] ontologos_dl::Error),
    /// SWRL engine error.
    #[error(transparent)]
    Swrl(#[from] ontologos_swrl::Error),
}

/// Classify using any supported profile (EL, RL, RDFS, ALC, DL, SWRL, Auto).
pub fn classify(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    match reasoner.profile() {
        Profile::Alc => Ok(ClassifyOutcome::Taxonomy(ontologos_alc::classify(
            reasoner.ontology(),
        )?)),
        Profile::Dl => Ok(ClassifyOutcome::Taxonomy(ontologos_dl::classify(
            reasoner.ontology(),
        )?)),
        Profile::Swrl => Ok(ClassifyOutcome::Taxonomy(
            ontologos_swrl::classify_with_swrl(reasoner.ontology())?.0,
        )),
        Profile::Auto => classify_auto(reasoner),
        _ => el_classify(reasoner).map_err(Error::El),
    }
}

fn classify_auto(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    let report = detect_profile(reasoner.ontology())
        .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
    if report.detected == Some(OwlProfile::Dl) {
        return Ok(ClassifyOutcome::Taxonomy(ontologos_dl::classify(
            reasoner.ontology(),
        )?));
    }
    el_classify(reasoner).map_err(Error::El)
}

/// Check ontology consistency for the configured profile.
pub fn is_consistent(reasoner: &Reasoner) -> Result<bool> {
    match reasoner.profile() {
        Profile::Alc => Ok(ontologos_alc::is_consistent(reasoner.ontology())?),
        Profile::Dl | Profile::Swrl => Ok(ontologos_dl::is_consistent(reasoner.ontology())?),
        Profile::Auto => is_consistent_auto(reasoner),
        Profile::El => el_is_consistent(reasoner.ontology()),
        Profile::Rl | Profile::Rdfs => Ok(true),
    }
}

fn el_is_consistent(ontology: &ontologos_core::Ontology) -> Result<bool> {
    ontologos_el::ElClassifier::new()
        .classify(ontology)
        .map(|t| t.unsatisfiable.is_empty())
        .map_err(Error::El)
}

fn is_consistent_auto(reasoner: &Reasoner) -> Result<bool> {
    let report = detect_profile(reasoner.ontology())
        .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
    match report.detected {
        Some(OwlProfile::Dl) => Ok(ontologos_dl::is_consistent(reasoner.ontology())?),
        Some(OwlProfile::El) | Some(OwlProfile::Ql) => el_is_consistent(reasoner.ontology()),
        Some(OwlProfile::Rl) => Ok(true),
        None => Err(Error::El(ontologos_el::Error::Profile(
            "no profile detected".into(),
        ))),
    }
}

/// Extract taxonomy when the outcome is classification-shaped.
#[must_use]
pub fn taxonomy_from_outcome(outcome: &ClassifyOutcome) -> Option<&Taxonomy> {
    match outcome {
        ClassifyOutcome::Taxonomy(t) => Some(t),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Ontology, Profile, Reasoner};

    fn el_ontology() -> Ontology {
        Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .class("http://example.org/B")
            .unwrap()
            .subclass_of("http://example.org/A", "http://example.org/B")
            .unwrap()
            .build()
            .unwrap()
    }

    fn el_reasoner() -> Reasoner {
        Reasoner::builder()
            .profile(Profile::El)
            .build(el_ontology())
            .unwrap()
    }

    #[test]
    fn is_consistent_el_uses_el_classifier() {
        let reasoner = el_reasoner();
        assert!(super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_consistent_auto_routes_el_to_el_classifier() {
        let reasoner = Reasoner::builder()
            .profile(Profile::Auto)
            .build(el_ontology())
            .unwrap();
        assert!(super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_consistent_rl_returns_true_without_dl_tableau() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .build()
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::Rl)
            .build(ontology)
            .unwrap();
        assert!(super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_consistent_alc_uses_alc_engine() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .build()
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::Alc)
            .build(ontology)
            .unwrap();
        let err = super::is_consistent(&reasoner).expect_err("alc consistency on bare class");
        assert!(matches!(err, super::Error::Alc(_)));
    }
}
