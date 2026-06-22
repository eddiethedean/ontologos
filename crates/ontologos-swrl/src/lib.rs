//! DLSafe SWRL rule engine integrated with DL classification.

#![warn(missing_docs)]

mod engine;
mod rules;

use ontologos_core::{Ontology, OwlConstruct};
use ontologos_profile::scanner::scan_constructs;
use thiserror::Error;

pub use engine::materialize_swrl_rules;
pub use rules::{apply_swrl_rules, SwrlReport};

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
    if !scan_constructs(ontology).contains(&OwlConstruct::SwrlRule) && ontology.swrl_rules().is_empty()
    {
        return Err(Error::NotImplemented);
    }
    let mut working = ontology.clone();
    let report = apply_swrl_rules(&mut working)?;
    if report.rules_found == 0 {
        return Err(Error::PreviewLimit(
            "SWRL rules detected in profile scan but not mapped for execution".into(),
        ));
    }
    let taxonomy = ontologos_dl::classify(&working)?;
    Ok((taxonomy, report))
}

/// Apply SWRL rules and check DL consistency on the materialized ontology.
pub fn is_consistent_with_swrl(ontology: &Ontology) -> Result<bool> {
    let mut working = ontology.clone();
    let report = apply_swrl_rules(&mut working)?;
    if report.rules_found == 0 && ontology.swrl_rules().is_empty() {
        return Err(Error::NotImplemented);
    }
    Ok(ontologos_dl::is_consistent(&working)?)
}
