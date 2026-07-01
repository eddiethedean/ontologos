//! EL profile engine adapter (DIP unit struct).

use ontologos_core::{Ontology, Reasoner, Taxonomy};

use crate::{ClassifyOutcome, ElClassifier, ElReport};

/// OWL EL profile engine adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct ElEngine;

impl ElEngine {
    /// Classify and return taxonomy plus optional inference trace.
    pub fn classify_with_report(&self, reasoner: &mut Reasoner) -> crate::Result<ElReport> {
        crate::classify_with_report(reasoner)
    }

    /// Classify and return taxonomy only.
    pub fn classify_taxonomy(&self, reasoner: &mut Reasoner) -> crate::Result<Taxonomy> {
        self.classify_with_report(reasoner).map(|r| r.taxonomy)
    }

    /// Classify an ontology directly (non-incremental).
    pub fn classify_ontology(&self, ontology: &Ontology) -> crate::Result<Taxonomy> {
        ElClassifier::new().classify(ontology)
    }

    /// Profile-routed classification for EL / RDFS / RL / ALC / Auto (non-DL).
    pub fn classify_with_profile(&self, reasoner: &mut Reasoner) -> crate::Result<ClassifyOutcome> {
        crate::route::classify_with_profile(reasoner)
    }

    /// Auto-profile classification (El / Ql / Rl detection).
    pub fn classify_auto(&self, reasoner: &mut Reasoner) -> crate::Result<ClassifyOutcome> {
        crate::route::classify_auto(reasoner)
    }

    /// Check consistency via EL classification (no unsatisfiable classes).
    pub fn is_consistent(&self, ontology: &Ontology) -> crate::Result<bool> {
        ElClassifier::new()
            .classify(ontology)
            .map(|t| t.unsatisfiable.is_empty())
    }
}
