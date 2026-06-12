//! OWL EL classification facade over [`whelk`](https://github.com/INCATools/whelk-rs).

mod normal_form;
mod reasoner;
mod route;

use ontologos_bridge::classify_core;
use ontologos_core::{InferenceTrace, Ontology, Taxonomy};
use thiserror::Error;

pub use reasoner::{classify_reasoner, classify_with_report, try_classify_reasoner};
pub use route::{classify_with_profile, resolve_profile_flag, ClassifyOutcome, ProfileFlag};

pub type Result<T> = std::result::Result<T, Error>;

/// EL classification output with optional trace (empty until whelk exposes traces).
#[derive(Debug, Clone)]
pub struct ElReport {
    /// Extracted taxonomy.
    pub taxonomy: Taxonomy,
    /// Inference trace (EL-first; empty when delegating to whelk).
    pub trace: InferenceTrace,
}

/// EL engine errors.
#[derive(Debug, Error)]
pub enum Error {
    #[error("expected profile {expected:?}, got {actual:?}")]
    WrongProfile {
        expected: ontologos_core::Profile,
        actual: ontologos_core::Profile,
    },
    #[error("ontology is not in OWL EL profile (detected {detected:?})")]
    NonElProfile {
        detected: ontologos_profile::OwlProfile,
    },
    #[error("profile detection failed: {0}")]
    Profile(String),
    #[error("classification not supported for profile {0:?}")]
    UnsupportedProfile(ontologos_profile::OwlProfile),
    #[error(transparent)]
    Core(#[from] ontologos_core::Error),
    #[error(transparent)]
    Bridge(#[from] ontologos_bridge::Error),
    #[error(transparent)]
    Rdfs(#[from] ontologos_rdfs::Error),
    #[error(transparent)]
    Rl(#[from] ontologos_rl::Error),
}

/// OWL EL classifier delegating to whelk via `ontologos-bridge`.
#[derive(Debug, Default)]
pub struct ElClassifier;

impl ElClassifier {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Classify the ontology and return the extracted taxonomy.
    pub fn classify(&self, ontology: &Ontology) -> Result<Taxonomy> {
        self.classify_with_options(ontology, false)
            .map(|r| r.taxonomy)
    }

    /// Classify with optional trace recording (trace remains empty until whelk support lands).
    pub fn classify_with_options(
        &self,
        ontology: &Ontology,
        _record_traces: bool,
    ) -> Result<ElReport> {
        normal_form::validate_el_profile(ontology)?;
        let taxonomy = classify_core(ontology)?;
        Ok(ElReport {
            taxonomy,
            trace: InferenceTrace::new(),
        })
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
