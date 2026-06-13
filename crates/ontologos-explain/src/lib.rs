//! Explanation engine with proof graphs and export formats.

mod build;
mod format;
mod query;

use ontologos_core::{
    EntityId, Error as CoreError, InferenceTrace, Ontology, Profile, Reasoner, ReasonerConfig,
};
use ontologos_el::ElClassifier;
use ontologos_profile::{detect_profile, OwlProfile};
use ontologos_rl::RlEngine;
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub use build::build_proof_graph;
pub use format::render_text;
pub use query::{
    explain_subsumption, explain_unsatisfiable, find_bottom_subsumption, find_subsumption_step,
    subsumption_from_axioms,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("explanation generation not yet implemented")]
    NotImplemented,
    #[error("profile detection failed: {0}")]
    Profile(String),
    #[error("classification not supported for profile {0:?}")]
    UnsupportedProfile(OwlProfile),
    #[error(transparent)]
    Core(#[from] CoreError),
    #[error(transparent)]
    Rdfs(#[from] ontologos_rdfs::Error),
    #[error(transparent)]
    Rl(#[from] ontologos_rl::Error),
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
    pub conclusion_sub: Option<(EntityId, EntityId)>,
    /// EL existential conclusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion_existential: Option<(EntityId, EntityId, EntityId)>,
    /// EL subproperty conclusion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conclusion_subproperty: Option<(EntityId, EntityId)>,
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
            .map_err(|e| Error::Core(CoreError::Message(e.to_string())))
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
    match reasoner.profile() {
        Profile::Rdfs => Ok(ontologos_rdfs::RdfsEngine::new()
            .with_traces(true)
            .materialize(reasoner.ontology_mut())?
            .trace),
        Profile::Rl => Ok(RlEngine::try_new(reasoner.config().parallelism)?
            .with_traces(true)
            .saturate(reasoner.ontology_mut())?
            .trace),
        Profile::El => Ok(ElClassifier::new()
            .classify_with_options(reasoner.ontology(), true)?
            .trace),
        Profile::Alc | Profile::Dl | Profile::Swrl => collect_trace_dl(reasoner.ontology()),
        Profile::Auto => collect_trace_auto(reasoner),
    }
}

fn collect_trace_auto(reasoner: &mut Reasoner) -> Result<InferenceTrace> {
    let report = detect_profile(reasoner.ontology()).map_err(|e| Error::Profile(e.to_string()))?;
    let detected = report
        .detected
        .ok_or_else(|| Error::Profile("no profile detected".into()))?;

    match detected {
        OwlProfile::El | OwlProfile::Ql => Ok(ElClassifier::new()
            .classify_with_options(reasoner.ontology(), true)?
            .trace),
        OwlProfile::Rl => Ok(RlEngine::try_new(reasoner.config().parallelism)?
            .with_traces(true)
            .saturate(reasoner.ontology_mut())?
            .trace),
        OwlProfile::Dl => collect_trace_dl(reasoner.ontology()),
    }
}

/// Generate a full proof graph using automatic profile detection.
pub fn explain(ontology: &Ontology) -> Result<ProofGraph> {
    let mut reasoner = Reasoner::builder()
        .profile(Profile::Auto)
        .config(ReasonerConfig {
            explanations: true,
            ..ReasonerConfig::default()
        })
        .build(ontology.clone())?;
    explain_with_profile(&mut reasoner)
}

/// Generate proof graph for RDFS materialization traces.
pub fn explain_rdfs(ontology: &mut Ontology) -> Result<ProofGraph> {
    let trace = ontologos_rdfs::RdfsEngine::new()
        .with_traces(true)
        .materialize(ontology)?
        .trace;
    build_proof_graph(ontology, &trace)
}

/// Generate proof graph for OWL RL saturation traces.
pub fn explain_rl(ontology: &mut Ontology) -> Result<ProofGraph> {
    let trace = RlEngine::try_new(1)?
        .with_traces(true)
        .saturate(ontology)?
        .trace;
    build_proof_graph(ontology, &trace)
}

fn collect_trace_dl(ontology: &Ontology) -> Result<InferenceTrace> {
    use ontologos_core::{TraceConclusion, TraceStep};

    let taxonomy = ontologos_alc::classify(ontology).map_err(|e| Error::Profile(e.to_string()))?;
    let mut trace = InferenceTrace::default();
    for &(sub, sup) in &taxonomy.subsumptions {
        trace.push(TraceStep {
            rule: "dl_tableau_subsumption".into(),
            premises: vec![],
            conclusion: TraceConclusion::SubClassOf { sub, sup },
        });
    }
    Ok(trace)
}

/// Generate proof graph for OWL DL/ALC tableau subsumptions.
pub fn explain_dl(ontology: &Ontology) -> Result<ProofGraph> {
    let trace = collect_trace_dl(ontology)?;
    build_proof_graph(ontology, &trace)
}

/// Generate proof graph for OWL EL classification traces.
pub fn explain_el(ontology: &Ontology) -> Result<ProofGraph> {
    let trace = ElClassifier::new()
        .classify_with_options(ontology, true)?
        .trace;
    build_proof_graph(ontology, &trace)
}

/// Route explanation collection by reasoner profile (like classify).
pub fn explain_with_profile(reasoner: &mut Reasoner) -> Result<ProofGraph> {
    let trace = collect_trace(reasoner)?;
    build_proof_graph(reasoner.ontology(), &trace)
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::*;

    fn class(ontology: &mut Ontology, iri: &str) -> EntityId {
        ontology.entity_id(iri, EntityKind::Class).expect("class")
    }

    #[test]
    fn explain_rdfs_builds_graph() {
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

        let graph = explain_rdfs(&mut ontology).expect("graph");
        assert!(graph.node_count() > 2);
        graph.to_json().expect("json");
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
    }
}
