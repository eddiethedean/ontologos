//! Explanation engine with proof graphs and export formats.

use ontologos_core::{Error as CoreError, Ontology};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("explanation generation not yet implemented")]
    NotImplemented,
    #[error(transparent)]
    Core(#[from] CoreError),
}

/// Node identifier within a proof graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// Single step in a reasoning proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofNode {
    pub rule: String,
    pub premises: Vec<NodeId>,
}

/// Proof graph backing explanation traces.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProofGraph {
    pub nodes: Vec<ProofNode>,
}

impl ProofGraph {
    /// Export the proof graph as JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::Core(CoreError::Message(e.to_string())))
    }
}

/// Generate explanations for inferences in an ontology.
pub fn explain(_ontology: &Ontology) -> Result<ProofGraph> {
    Err(Error::NotImplemented)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::Ontology;

    #[test]
    fn explain_returns_not_implemented() {
        let ontology = Ontology::default();
        let err = explain(&ontology).expect_err("stub");
        assert!(matches!(err, Error::NotImplemented));
    }
}
