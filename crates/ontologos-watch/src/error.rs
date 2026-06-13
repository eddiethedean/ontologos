use thiserror::Error;

/// File-watch errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Filesystem notify failure.
    #[error("watch error: {0}")]
    Watch(String),
    /// Ontology parse failure on reload.
    #[error(transparent)]
    Parse(#[from] ontologos_parser::Error),
}

/// Result type for watch operations.
pub type Result<T> = std::result::Result<T, Error>;
