//! Query interface over classified ontologies.

use ontologos_core::{EntityId, Error as CoreError, Ontology};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("query engine not yet implemented")]
    NotImplemented,
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// Query handle over a materialized ontology.
#[derive(Debug)]
pub struct QueryEngine<'a> {
    ontology: &'a Ontology,
}

impl<'a> QueryEngine<'a> {
    #[must_use]
    pub fn new(ontology: &'a Ontology) -> Self {
        Self { ontology }
    }

    /// Return direct subclasses of the given class.
    pub fn direct_subclasses(&self, class: EntityId) -> Result<Vec<EntityId>> {
        let _ = class;
        let _ = self.ontology;
        Err(Error::NotImplemented)
    }
}
