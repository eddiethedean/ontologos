//! Reasoner-facing DL classification helpers.

use ontologos_core::{Profile, Reasoner, Taxonomy};

use crate::{Error, classify};

/// DL classification report.
#[derive(Debug)]
pub struct DlReport {
    /// Extracted taxonomy.
    pub taxonomy: Taxonomy,
}

/// Classify via [`Reasoner`] when profile is DL.
pub fn classify_reasoner(reasoner: &Reasoner) -> Result<DlReport, Error> {
    if !matches!(
        reasoner.profile(),
        Profile::Dl | Profile::DlPreview | Profile::Auto
    ) {
        return Err(Error::WrongProfile(reasoner.profile()));
    }
    let taxonomy = classify(reasoner.ontology())?;
    Ok(DlReport { taxonomy })
}
