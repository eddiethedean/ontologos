//! OWL EL completion-based classification.

use ontologos_core::{EntityId, Error as CoreError, Ontology};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("owl el classification not yet implemented")]
    NotImplemented,
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// Extracted taxonomy from an EL classification run.
#[derive(Debug, Default)]
pub struct Taxonomy {
    pub subsumptions: Vec<(EntityId, EntityId)>,
    pub equivalences: Vec<Vec<EntityId>>,
    pub unsatisfiable: Vec<EntityId>,
}

/// OWL EL classifier using completion rules.
#[derive(Debug, Default)]
pub struct ElClassifier;

impl ElClassifier {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Classify the ontology and return the extracted taxonomy.
    pub fn classify(&self, ontology: &Ontology) -> Result<Taxonomy> {
        let _ = ontology;
        Err(Error::NotImplemented)
    }
}
