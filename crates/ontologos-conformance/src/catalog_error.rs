//! Typed errors for the HermiT conformance harness.

use thiserror::Error;

/// Conformance harness errors (internal); public APIs may still expose `String` for test compatibility.
#[derive(Debug, Error)]
pub enum CatalogError {
    /// Case failed with a formatted message.
    #[error("{case_id}: {message}")]
    CaseFailed {
        /// HermiT or WG case id.
        case_id: String,
        /// Failure detail.
        message: String,
    },
    /// DL engine error.
    #[error(transparent)]
    Dl(#[from] ontologos_dl::Error),
    /// Parser error.
    #[error(transparent)]
    Parser(#[from] ontologos_parser::Error),
    /// Free-form harness message.
    #[error("{0}")]
    Message(String),
}

impl CatalogError {
    /// Format like legacy `Result<(), String>` messages.
    #[must_use]
    pub fn into_legacy_string(self) -> String {
        self.to_string()
    }

    /// Case-scoped failure (HermiT / WG id + detail).
    #[must_use]
    pub fn case_failed(case_id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::CaseFailed {
            case_id: case_id.into(),
            message: message.into(),
        }
    }
}

impl From<String> for CatalogError {
    fn from(message: String) -> Self {
        Self::Message(message)
    }
}
