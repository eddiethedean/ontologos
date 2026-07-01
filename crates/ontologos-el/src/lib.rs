//! OWL EL completion-based classification.
//!
//! # Example
//!
//! ```no_run
//! use ontologos_el::ElClassifier;
//! use ontologos_parser::load_ontology;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let ontology = load_ontology(std::path::Path::new("ontology.owl"))?;
//! let taxonomy = ElClassifier::new().classify(&ontology)?;
//! println!("subsumptions: {}", taxonomy.subsumption_count());
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

mod engine;
mod graph;
mod normal_form;
mod partition;
mod reasoner;
mod session;
mod taxonomy_extract;
mod trace;

use ontologos_core::{Ontology, Taxonomy};
use thiserror::Error;

pub use engine::ElEngine;
pub use reasoner::{classify_reasoner, classify_with_report};
pub use session::{ElSession, take_el_session};
pub use trace::ElReport;

/// Result type for EL operations.
pub type Result<T> = std::result::Result<T, Error>;

/// EL engine errors.
#[derive(Debug, Error)]
pub enum Error {
    /// Reasoner profile mismatch.
    #[error("expected profile {expected:?}, got {actual:?}")]
    WrongProfile {
        /// Expected profile.
        expected: ontologos_core::Profile,
        /// Actual profile.
        actual: ontologos_core::Profile,
    },
    /// Mapped axioms fall outside OWL EL.
    #[error("ontology is not in OWL EL profile (detected {detected:?})")]
    NonElProfile {
        /// Profile detected by `ontologos-profile`.
        detected: ontologos_profile::OwlProfile,
    },
    /// Profile detection failed.
    #[error(transparent)]
    Profile(#[from] ontologos_profile::Error),
    /// Core error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// General configuration or validation error.
    #[error("{0}")]
    Message(String),
}

/// OWL EL classifier using completion rules.
#[derive(Debug, Default)]
pub struct ElClassifier;

impl ElClassifier {
    /// Create a new EL classifier instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Classify the ontology and return the extracted taxonomy.
    pub fn classify(&self, ontology: &Ontology) -> Result<Taxonomy> {
        self.classify_with_options(ontology, false)
            .map(|r| r.taxonomy)
    }

    /// Classify with optional inference trace recording.
    pub fn classify_with_options(
        &self,
        ontology: &Ontology,
        record_traces: bool,
    ) -> Result<ElReport> {
        normal_form::validate_el_profile(ontology)?;
        let mut graph = graph::CompletionGraph::seed(ontology).with_traces(record_traces);
        graph.saturate();
        let mut taxonomy = taxonomy_extract::extract_taxonomy(ontology, &graph);
        taxonomy.canonicalize_entity_aliases(ontology);
        let trace = graph.into_trace();
        Ok(ElReport { taxonomy, trace })
    }

    /// Classify and return report plus session for subsequent incremental runs.
    pub fn classify_with_session(
        &self,
        ontology: &mut Ontology,
        session: Option<ElSession>,
        record_traces: bool,
    ) -> Result<(ElReport, ElSession)> {
        self.classify_incremental(ontology, session, record_traces)
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology};

    use super::*;

    fn class(ontology: &mut Ontology, iri: &str) -> ontologos_core::EntityId {
        ontology.entity_id(iri, EntityKind::Class).expect("class")
    }

    #[test]
    fn transitive_subclass_chain() {
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

        let taxonomy = ElClassifier::new().classify(&ontology).unwrap();
        assert!(taxonomy.is_subsumed(a, c));
        assert!(taxonomy.direct_superclasses(a).contains(&b));
        assert!(taxonomy.direct_superclasses(a).contains(&c) || taxonomy.is_subsumed(a, c));
    }

    #[test]
    fn existential_filler_subsumption_in_taxonomy() {
        use ontologos_core::TraceConclusion;

        let mut ontology = Ontology::new();
        let a = class(&mut ontology, "http://ex.org/A");
        let b = class(&mut ontology, "http://ex.org/B");
        let c = class(&mut ontology, "http://ex.org/C");
        let r = ontology
            .entity_id("http://ex.org/r", EntityKind::ObjectProperty)
            .expect("property");
        ontology
            .add_axiom(Axiom::SubClassOfExistential {
                subclass: a,
                property: r,
                filler: b,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: b,
                superclass: c,
            })
            .unwrap();

        let report = ElClassifier::new()
            .classify_with_options(&ontology, true)
            .expect("classify");
        assert!(report.trace.steps.iter().any(|step| {
            step.rule == "ex_filler_sub"
                && matches!(
                    &step.conclusion,
                    TraceConclusion::Existential {
                        class,
                        property,
                        filler
                    } if *class == a && *property == r && *filler == c
                )
        }));
    }

    #[test]
    fn el_trace_records_transitive_subsumption() {
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

        let report = ElClassifier::new()
            .classify_with_options(&ontology, true)
            .unwrap();
        assert!(
            report
                .trace
                .steps
                .iter()
                .any(|s| s.rule == "sub_trans_forward")
        );
    }

    #[test]
    fn equivalent_classes_cluster() {
        let mut ontology = Ontology::new();
        let a = class(&mut ontology, "http://ex.org/A");
        let b = class(&mut ontology, "http://ex.org/B");
        ontology
            .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
            .unwrap();

        let taxonomy = ElClassifier::new().classify(&ontology).unwrap();
        assert!(
            taxonomy.equivalent_classes(a).is_some()
                || taxonomy.is_subsumed(a, b) && taxonomy.is_subsumed(b, a)
        );
    }

    #[test]
    fn el_classification_forbidden_includes_complex_tbox_constructs() {
        let mut constructs = std::collections::BTreeSet::new();
        constructs.insert(ontologos_core::OwlConstruct::ObjectUnionOf);
        assert!(!ontologos_profile::el_classification_forbidden_in(&constructs).is_empty());
    }

    #[test]
    fn symmetric_property_does_not_block_forced_el_classification() {
        let mut ontology = Ontology::new();
        let p = ontology
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .expect("property");
        ontology
            .add_axiom(Axiom::SymmetricObjectProperty(p))
            .unwrap();

        ElClassifier::new()
            .classify(&ontology)
            .expect("ignored characteristic axioms do not block EL");
    }

    #[test]
    fn multiple_property_domains_infer_all() {
        let mut ontology = Ontology::new();
        let a = class(&mut ontology, "http://ex.org/A");
        let d1 = class(&mut ontology, "http://ex.org/D1");
        let d2 = class(&mut ontology, "http://ex.org/D2");
        let p = ontology
            .entity_id("http://ex.org/p", EntityKind::ObjectProperty)
            .expect("property");
        ontology
            .add_axiom(Axiom::SubClassOfExistential {
                subclass: a,
                property: p,
                filler: a,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::ObjectPropertyDomain {
                property: p,
                domain: d1,
            })
            .unwrap();
        ontology
            .add_axiom(Axiom::ObjectPropertyDomain {
                property: p,
                domain: d2,
            })
            .unwrap();

        let taxonomy = ElClassifier::new().classify(&ontology).unwrap();
        assert!(taxonomy.is_subsumed(a, d1));
        assert!(taxonomy.is_subsumed(a, d2));
    }
}
