//! OWL RL forward-chaining rule engine.

use ontologos_core::{EntityId, Error as CoreError, Ontology};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("owl rl reasoning not yet implemented")]
    NotImplemented,
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// Indexed triple store used by the RL engine.
#[derive(Debug, Default)]
pub struct TripleIndex {
    #[allow(dead_code)]
    entries: std::collections::HashMap<EntityId, Vec<u32>>,
}

impl TripleIndex {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

/// OWL RL rule engine with optional parallel execution.
#[derive(Debug)]
pub struct RlEngine {
    parallelism: usize,
}

impl RlEngine {
    #[must_use]
    pub fn new(parallelism: usize) -> Self {
        Self { parallelism }
    }

    /// Run forward chaining until saturation.
    pub fn saturate(&self, ontology: &Ontology) -> Result<()> {
        let _ = ontology;
        let _ = self.parallelism;
        Err(Error::NotImplemented)
    }
}
