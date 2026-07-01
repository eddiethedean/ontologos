//! Hybrid multi-module classification orchestration.

use ontologos_core::Ontology;

use crate::error::{Error, Result};
use crate::outcome::ClassifyOutcome;

/// Classify a hybrid DL ontology on the full combined ontology (cross-module entailments).
pub(crate) fn classify_hybrid_modules(ontology: &Ontology) -> Result<ClassifyOutcome> {
    Ok(ClassifyOutcome::Taxonomy(
        ontologos_dl::classify(ontology).map_err(Error::Dl)?,
    ))
}
