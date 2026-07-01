//! ALC profile engine adapter (DIP unit struct).

use ontologos_core::{Ontology, Reasoner, Taxonomy};

/// OWL ALC profile engine adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct AlcEngine;

impl AlcEngine {
    /// Classify an ontology under ALC tableau semantics.
    pub fn classify(&self, ontology: &Ontology) -> crate::Result<Taxonomy> {
        crate::classify(ontology)
    }

    /// Classify via reasoner when profile is ALC-compatible.
    pub fn classify_reasoner(&self, reasoner: &Reasoner) -> crate::Result<Taxonomy> {
        crate::classify_reasoner(reasoner)
    }

    /// Tableau consistency check.
    pub fn is_consistent(&self, ontology: &Ontology) -> crate::Result<bool> {
        crate::is_consistent(ontology)
    }
}
