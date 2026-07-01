use serde::Serialize;
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
    /// ABox engine error.
    #[error(transparent)]
    Abox(#[from] ontologos_abox::Error),
    /// Core error (e.g. incomplete consistency folded to message).
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
}

/// Axiom-shaped entailment checks for [`crate::is_entailed_axiom`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum EntailmentCheck {
    /// Named class subsumption `SubClassOf(sub, sup)`.
    SubClassOf {
        /// Subclass IRI.
        sub: String,
        /// Superclass IRI.
        sup: String,
    },
    /// `ClassAssertion(individual, class)` with named classes.
    ClassAssertion {
        /// Individual IRI.
        individual: String,
        /// Class IRI.
        class: String,
    },
    /// `ObjectPropertyAssertion(subject, property, object)`.
    ObjectPropertyAssertion {
        /// Subject individual IRI.
        subject: String,
        /// Object property IRI.
        property: String,
        /// Object individual IRI.
        object: String,
    },
}
