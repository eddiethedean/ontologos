//! DL profile engine adapter (DIP unit struct).

use std::collections::HashSet;

use ontologos_core::{Ontology, Reasoner, RoleExpr, Taxonomy};

/// OWL 2 DL profile engine adapter.
#[derive(Debug, Default, Clone, Copy)]
pub struct DlEngine;

impl DlEngine {
    /// Classify an ontology under OWL 2 DL semantics.
    pub fn classify(&self, ontology: &Ontology) -> crate::Result<Taxonomy> {
        crate::classify(ontology)
    }

    /// Classify via reasoner when profile is DL.
    pub fn classify_reasoner(&self, reasoner: &Reasoner) -> crate::Result<crate::reasoner::DlReport> {
        crate::reasoner::classify_reasoner(reasoner)
    }

    /// Check ontology consistency under DL.
    pub fn is_consistent(&self, ontology: &Ontology) -> crate::Result<bool> {
        crate::is_consistent(ontology)
    }

    /// Check ontology consistency with optional wall-clock budget.
    pub fn check_consistency(
        &self,
        ontology: &Ontology,
        budget_secs: Option<u64>,
    ) -> crate::Result<ontologos_core::ConsistencyResult> {
        crate::check_consistency(ontology, budget_secs)
    }

    /// Sub-object-property expressions for a role.
    pub fn sub_object_properties(
        &self,
        ontology: &Ontology,
        role: &RoleExpr,
        direct: bool,
    ) -> crate::Result<HashSet<RoleExpr>> {
        crate::sub_object_property_expressions(ontology, role, direct)
    }
}
