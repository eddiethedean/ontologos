//! RDFS reasoning via graph closure and property propagation.

use ontologos_core::{Error as CoreError, Ontology};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("rdfs reasoning not yet implemented")]
    NotImplemented,
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// RDFS forward-chaining engine.
#[derive(Debug, Default)]
pub struct RdfsEngine;

impl RdfsEngine {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Materialize RDFS inferences for the given ontology.
    pub fn materialize(&self, ontology: &Ontology) -> Result<()> {
        let _ = ontology;
        Err(Error::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn materialize_returns_not_implemented() {
        let ontology = Ontology::default();
        let err = RdfsEngine::new().materialize(&ontology).expect_err("stub");
        assert!(matches!(err, Error::NotImplemented));
    }
}
