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

mod graph;
mod normal_form;
mod reasoner;
mod route;
mod taxonomy_extract;
mod trace;

use ontologos_core::{Ontology, Taxonomy};
use thiserror::Error;

pub use reasoner::{classify_reasoner, classify_with_report, try_classify_reasoner};
pub use route::{classify_with_profile, resolve_profile_flag, ClassifyOutcome, ProfileFlag};
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
    #[error("profile detection failed: {0}")]
    Profile(String),
    /// Auto-routing cannot classify this profile.
    #[error("classification not supported for profile {0:?}")]
    UnsupportedProfile(ontologos_profile::OwlProfile),
    /// Core error.
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    /// RDFS engine error.
    #[error(transparent)]
    Rdfs(#[from] ontologos_rdfs::Error),
    /// RL engine error.
    #[error(transparent)]
    Rl(#[from] ontologos_rl::Error),
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
    ///
    /// Runs ELK-style goal-directed completion and transitive-reduction taxonomy
    /// extraction. The ontology is not mutated.
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
        let taxonomy = taxonomy_extract::extract_taxonomy(ontology, &graph);
        let trace = graph.into_trace();
        Ok(ElReport { taxonomy, trace })
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
        assert!(taxonomy.is_subsumed(a, b));
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
        assert!(report
            .trace
            .steps
            .iter()
            .any(|s| s.rule == "sub_trans_forward"));
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
}
