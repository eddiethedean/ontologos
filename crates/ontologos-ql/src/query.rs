//! Conjunctive query AST and evaluation.

use ontologos_core::{EntityId, Ontology};
use ontologos_query::QueryEngine;

use crate::{Error, Result};

/// A conjunctive query atom: `Class(var)` or `SubClassOf(var, class)`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryAtom {
    /// Individual/class variable has type `class`.
    Type {
        /// Variable name.
        var: String,
        /// Class IRI or local name.
        class: String,
    },
    /// `var` is subsumed by named class.
    Subsumed {
        /// Variable name.
        var: String,
        /// Superclass IRI or local name.
        superclass: String,
    },
}

/// Conjunctive query (AND of atoms).
#[derive(Debug, Clone, Default)]
pub struct ConjunctiveQuery {
    /// Query atoms.
    pub atoms: Vec<QueryAtom>,
}

/// One answer binding (variable → entity id).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryAnswer {
    /// Variable bindings.
    pub bindings: Vec<(String, EntityId)>,
}

/// Evaluate a CQ with a single type atom per variable (QL subset).
pub fn evaluate(
    engine: &QueryEngine<'_>,
    ontology: &Ontology,
    query: &ConjunctiveQuery,
) -> Result<Vec<QueryAnswer>> {
    if query.atoms.is_empty() {
        return Ok(Vec::new());
    }
    let atom = &query.atoms[0];
    match atom {
        QueryAtom::Type { var, class } => {
            let class_id = ontology
                .lookup_entity(class)
                .ok_or_else(|| Error::UnknownClass(class.clone()))?;
            let subs = engine.direct_subclasses(class_id)?;
            let mut answers = Vec::new();
            for sub in subs {
                answers.push(QueryAnswer {
                    bindings: vec![(var.clone(), sub)],
                });
            }
            Ok(answers)
        }
        QueryAtom::Subsumed { var, superclass } => {
            let sup_id = ontology
                .lookup_entity(superclass)
                .ok_or_else(|| Error::UnknownClass(superclass.clone()))?;
            let subs = engine.direct_subclasses(sup_id)?;
            let mut answers = Vec::new();
            for sub in subs {
                answers.push(QueryAnswer {
                    bindings: vec![(var.clone(), sub)],
                });
            }
            Ok(answers)
        }
    }
}
