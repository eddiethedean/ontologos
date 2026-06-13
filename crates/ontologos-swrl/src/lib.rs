//! DLSafe SWRL rule engine integrated with DL classification.

#![warn(missing_docs)]

mod rules;

use ontologos_core::Ontology;
use thiserror::Error;

pub use rules::{apply_swrl_rules, SwrlReport, SwrlRule};

/// Result type for SWRL operations.
pub type Result<T> = std::result::Result<T, Error>;

/// SWRL engine errors.
#[derive(Debug, Error)]
pub enum Error {
    /// DL classification error.
    #[error(transparent)]
    Dl(#[from] ontologos_dl::Error),
    /// Core error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// No SWRL rules found.
    #[error("no SWRL rules in ontology")]
    NoRules,
}

/// Classify with SWRL rules materialized post-DL.
pub fn classify_with_swrl(ontology: &Ontology) -> Result<(ontologos_core::Taxonomy, SwrlReport)> {
    let taxonomy = ontologos_dl::classify(ontology)?;
    let report = apply_swrl_rules(ontology)?;
    Ok((taxonomy, report))
}
