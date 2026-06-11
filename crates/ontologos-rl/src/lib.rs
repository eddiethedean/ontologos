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

const MAX_PARALLELISM: usize = 64;

impl RlEngine {
    #[must_use]
    pub fn new(parallelism: usize) -> Self {
        Self { parallelism }
    }

    /// Validate parallelism is within supported bounds.
    pub fn try_new(parallelism: usize) -> Result<Self> {
        if parallelism == 0 || parallelism > MAX_PARALLELISM {
            return Err(Error::Core(CoreError::Message(format!(
                "parallelism must be in 1..={MAX_PARALLELISM}, got {parallelism}"
            ))));
        }
        Ok(Self { parallelism })
    }

    /// Run forward chaining until saturation.
    pub fn saturate(&self, ontology: &Ontology) -> Result<()> {
        let _ = ontology;
        let _ = self.parallelism;
        Err(Error::NotImplemented)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn try_new_rejects_invalid_parallelism() {
        assert!(RlEngine::try_new(0).is_err());
        assert!(RlEngine::try_new(65).is_err());
        assert!(RlEngine::try_new(1).is_ok());
    }
}
