//! SWRL profile engine adapter (DIP unit struct).

use ontologos_core::{Ontology, Taxonomy};

use crate::rules::SwrlReport;

/// DLSafe SWRL profile engine adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct SwrlEngine;

impl SwrlEngine {
    /// Classify with SWRL rules materialized post-DL.
    pub fn classify_with_swrl(&self, ontology: &Ontology) -> crate::Result<(Taxonomy, SwrlReport)> {
        crate::classify_with_swrl(ontology)
    }

    /// Apply SWRL rules and check DL consistency.
    pub fn is_consistent(&self, ontology: &Ontology) -> crate::Result<bool> {
        crate::is_consistent_with_swrl(ontology)
    }
}
