use thiserror::Error;

/// Result type alias for parser operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors returned by the OWL/RDF parser.
#[derive(Debug, Error)]
pub enum Error {
    /// File extension or content is not a supported OWL/RDF format.
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),
    /// Parse or I/O failure (missing file, size limit, horned-owl error).
    #[error("parse error: {0}")]
    Parse(String),
    /// Wrapped error from `ontologos-core` during axiom mapping.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
}
