//! Explanation engine with proof graphs and export formats.

mod build;
mod format;

use ontologos_core::{EngineKind, InferenceTrace, Ontology, Reasoner};
use ontologos_el::ElClassifier;
use ontologos_profile::resolve_route;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use build::build_proof_graph;
pub use format::render_text;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("explanation generation not yet implemented")]
    NotImplemented,
    #[error(transparent)]
    Profile(#[from] ontologos_profile::Error),
    #[error("explanation is only supported for OWL EL classification (got {0:?})")]
    UnsupportedEngine(EngineKind),
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    #[error(transparent)]
    El(#[from] ontologos_el::Error),
}

/// Node identifier within a proof graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub u32);

/// Single step in a reasoning proof.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofNode {
    /// Rule that produced this step.
    pub rule: String,
    /// Premise nodes in the proof graph.
    pub premises: Vec<NodeId>,
    /// Concluded axiom, when applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion_axiom: Option<ontologos_core::AxiomId>,
    /// EL subsumption conclusion `sub ⊑ sup`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion_sub: Option<(ontologos_core::EntityId, ontologos_core::EntityId)>,
    /// EL existential conclusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion_existential: Option<(ontologos_core::EntityId, ontologos_core::EntityId, ontologos_core::EntityId)>,
    /// EL subproperty conclusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion_subproperty: Option<(ontologos_core::EntityId, ontologos_core::EntityId)>,
}

/// Proof graph backing explanation traces.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ProofGraph {
    /// Nodes in topological construction order.
    pub nodes: Vec<ProofNode>,
}

impl ProofGraph {
    /// Export the proof graph as JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| Error::Core(ontologos_core::Error::Message(e.to_string())))
    }

    /// Number of proof nodes.
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Whether premise links form a directed acyclic graph.
    #[must_use]
    pub fn is_acyclic(&self) -> bool {
        build::validate_proof_acyclic(&self.nodes).is_ok()
    }
}

/// Collect inference trace with explanations enabled on `reasoner`'s ontology.
pub fn collect_trace(reasoner: &mut Reasoner) -> Result<InferenceTrace> {
    let route = resolve_route(reasoner.profile(), reasoner.ontology())?;
    match route.kind {
        EngineKind::El => Ok(ElClassifier::new()
            .classify_with_options(reasoner.ontology(), true)?
            .trace),
        other => Err(Error::UnsupportedEngine(other)),
    }
}

/// Generate proof graph for OWL EL classification traces.
pub fn explain_el(ontology: &Ontology) -> Result<ProofGraph> {
    let trace = ElClassifier::new()
        .classify_with_options(ontology, true)?
        .trace;
    build_proof_graph(ontology, &trace)
}

/// Validate proof graph structure (acyclic, valid axiom ids, premise bounds).
#[doc(hidden)]
pub fn assert_valid_proof_graph(ontology: &Ontology, graph: &ProofGraph) {
    assert!(!graph.nodes.is_empty(), "expected non-empty proof graph");
    assert!(graph.is_acyclic(), "proof graph must be acyclic");
    for node in &graph.nodes {
        if let Some(id) = node.conclusion_axiom {
            ontology
                .axiom(id)
                .unwrap_or_else(|_| panic!("valid conclusion axiom id {}", id.0));
        }
        for premise in &node.premises {
            assert!(
                (premise.0 as usize) < graph.nodes.len(),
                "premise node id must exist"
            );
        }
    }
}

/// Route explanation collection by resolved engine route.
pub fn explain_with_profile(reasoner: &mut Reasoner) -> Result<ProofGraph> {
    let trace = collect_trace(reasoner)?;
    build_proof_graph(reasoner.ontology(), &trace)
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityId, EntityKind, Ontology};

    use super::*;

    fn class(ontology: &mut Ontology, iri: &str) -> EntityId {
        ontology.entity_id(iri, EntityKind::Class).expect("class")
    }

    #[test]
    fn explain_el_builds_graph() {
        let mut ontology = Ontology::new();
        let a = class(&mut ontology, "http://ex.org/A");
        let b = class(&mut ontology, "http://ex.org/B");
        let c = class(&mut ontology, "http://ex.org/C");
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: b,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: b,
                superclass: c,
            })
            .unwrap();

        let graph = explain_el(&ontology).expect("graph");
        assert!(graph.node_count() > 2);
        assert_valid_proof_graph(&ontology, &graph);
        assert!(graph.nodes.iter().any(|node| {
            node.rule == "sub_trans_forward" && node.conclusion_sub == Some((a, c))
        }));
    }
}
