//! DLSafe SWRL rule engine integrated with DL classification.

#![warn(missing_docs)]

mod rules;

use ontologos_core::{Ontology, OwlConstruct};
use ontologos_profile::scanner::scan_constructs;
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
    /// SWRL profile not yet implemented.
    #[error(
        "SWRL rule execution is not implemented (preview); ontology has no executable SWRL rules"
    )]
    NotImplemented,
    /// Preview-only limitation.
    #[error("SWRL preview: {0}")]
    PreviewLimit(String),
}

/// Classify with SWRL rules materialized post-DL.
pub fn classify_with_swrl(ontology: &Ontology) -> Result<(ontologos_core::Taxonomy, SwrlReport)> {
    if !scan_constructs(ontology).contains(&OwlConstruct::SwrlRule) {
        return Err(Error::NotImplemented);
    }
    let taxonomy = ontologos_dl::classify(ontology)?;
    let report = apply_swrl_rules(ontology)?;
    if report.rules_found == 0 {
        return Err(Error::PreviewLimit(
            "SWRL rules detected in profile scan but not mapped for execution".into(),
        ));
    }
    Ok((taxonomy, report))
}
