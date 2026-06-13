//! OWL ALC/DL tableau classification.

#![warn(missing_docs)]

mod clause;
mod dl_ontology;
mod normalize;
mod tableau;

use ontologos_core::{Ontology, Taxonomy};
use ontologos_profile::{detect_profile, OwlProfile};
use thiserror::Error;

pub use clause::{Clause, ClauseSet};
pub use dl_ontology::DlOntology;
pub use normalize::clausify;
pub use tableau::AlcClassifier;
pub use tableau::{
    classify as tableau_classify, classify_with_seed as tableau_classify_with_seed,
    is_consistent as tableau_is_consistent, TableauSeed,
};

/// Result type for ALC operations.
pub type Result<T> = std::result::Result<T, Error>;

/// ALC engine errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Profile mismatch.
    #[error("ontology exceeds ALC fragment ({0:?})")]
    NonAlcProfile(OwlProfile),
    /// Core error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// Parser error.
    #[error(transparent)]
    Parser(#[from] ontologos_parser::Error),
    /// Profile detection error.
    #[error("profile detection failed: {0}")]
    Profile(String),
    /// General message.
    #[error("{0}")]
    Message(String),
    /// Tableau expansion budget exhausted (incomplete reasoning).
    #[error("tableau expansion budget exhausted ({0} expansions)")]
    ResourceLimit(u32),
}

/// Classify an ontology under ALC tableau semantics.
pub fn classify(ontology: &Ontology) -> Result<Taxonomy> {
    let report = detect_profile(ontology).map_err(|e| Error::Profile(e.to_string()))?;
    if !matches!(
        report.detected,
        Some(OwlProfile::El | OwlProfile::Ql | OwlProfile::Dl)
    ) {
        if let Some(p) = report.detected {
            if p == OwlProfile::Rl {
                return Err(Error::NonAlcProfile(p));
            }
        }
    }
    tableau::classify(ontology)
}

/// Classify with saturation-derived seed facts.
pub fn classify_with_seed(ontology: &Ontology, seed: &TableauSeed) -> Result<Taxonomy> {
    let report = detect_profile(ontology).map_err(|e| Error::Profile(e.to_string()))?;
    if !matches!(
        report.detected,
        Some(OwlProfile::El | OwlProfile::Ql | OwlProfile::Dl)
    ) {
        if let Some(p) = report.detected {
            if p == OwlProfile::Rl {
                return Err(Error::NonAlcProfile(p));
            }
        }
    }
    tableau::classify_with_seed(ontology, seed)
}

/// Tableau consistency check.
pub fn is_consistent(ontology: &Ontology) -> Result<bool> {
    tableau::is_consistent(ontology)
}

/// Classify via [`ontologos_core::Reasoner`] when profile is ALC-compatible.
pub fn classify_reasoner(reasoner: &ontologos_core::Reasoner) -> Result<Taxonomy> {
    classify(reasoner.ontology())
}
