//! Error types for JavaScript bindings.

use ontologos_core::Error as CoreError;
use ontologos_facade::Error as FacadeError;
use thiserror::Error;

/// Binding error surfaced to JavaScript callers.
#[derive(Debug, Error)]
pub enum JsError {
    /// Parse or serialization failure.
    #[error("{0}")]
    Parse(String),
    /// Resource limit exceeded.
    #[error("{0}")]
    ResourceLimit(String),
    /// Reasoning did not complete (budget/tableau limit).
    #[error("incomplete reasoning")]
    IncompleteReasoning,
    /// General failure.
    #[error("{0}")]
    Other(String),
    /// Shared ontology was modified concurrently.
    #[error("shared ontology was modified concurrently; re-sync or use a single writer")]
    OntologyConflict,
}

impl JsError {
    /// Error code for typed exception mapping in host bindings.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Parse(_) => "ParseError",
            Self::ResourceLimit(_) => "ResourceLimitError",
            Self::IncompleteReasoning => "IncompleteReasoningError",
            Self::OntologyConflict => "OntologyConflictError",
            Self::Other(_) => "Error",
        }
    }
}

impl From<CoreError> for JsError {
    fn from(error: CoreError) -> Self {
        match error {
            CoreError::Parse(msg) | CoreError::Serialization(msg) => Self::Parse(msg),
            CoreError::ResourceLimit(msg) => Self::ResourceLimit(msg),
            CoreError::IncompleteConsistency => Self::IncompleteReasoning,
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<ontologos_parser::Error> for JsError {
    fn from(error: ontologos_parser::Error) -> Self {
        match error {
            ontologos_parser::Error::Core(e) => e.into(),
            other => Self::Parse(other.to_string()),
        }
    }
}

impl From<FacadeError> for JsError {
    fn from(error: FacadeError) -> Self {
        match error {
            FacadeError::Dl(ontologos_dl::Error::IncompleteReasoning(_)) => {
                Self::IncompleteReasoning
            }
            FacadeError::Dl(ontologos_dl::Error::Alc(ontologos_alc::Error::ResourceLimit(
                inner,
            ))) => Self::ResourceLimit(inner.to_string()),
            FacadeError::Core(e) => e.into(),
            other => Self::Other(other.to_string()),
        }
    }
}

impl From<ontologos_explain::Error> for JsError {
    fn from(error: ontologos_explain::Error) -> Self {
        match error {
            ontologos_explain::Error::Core(e) => e.into(),
            other => Self::Other(other.to_string()),
        }
    }
}

pub type Result<T> = std::result::Result<T, JsError>;
