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
        Profile::Dl | Profile::Swrl | Profile::Auto | Profile::Alc => {
            Ok(ontologos_dl::is_consistent(reasoner.ontology())?)
        }
        Profile::El => Ok(ontologos_el::ElClassifier::new()
            .classify(reasoner.ontology())
            .map(|t| t.unsatisfiable.is_empty())
            .map_err(Error::El)?),
        Profile::Rl | Profile::Rdfs => Ok(true),
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
