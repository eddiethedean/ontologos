//! Conjunctive query AST and evaluation.

use crate::hierarchy::TaxonomyHierarchy;
use ontologos_core::{EntityId, Ontology};

use crate::{Error, Result};

/// Built-in `owl:Nothing` IRI used by query rewrite.
pub const OWL_NOTHING_IRI: &str = "http://www.w3.org/2002/07/owl#Nothing";

fn lookup_query_class(ontology: &Ontology, class: &str) -> Result<EntityId> {
    ontology
        .lookup_entity(class)
        .or_else(|| match class {
            "owl:Nothing" => ontology.lookup_entity(OWL_NOTHING_IRI),
            "owl:Thing" => ontology.lookup_entity("http://www.w3.org/2002/07/owl#Thing"),
            _ => None,
        })
        .ok_or_else(|| Error::UnknownClass(class.to_owned()))
}

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
    engine: &TaxonomyHierarchy<'_>,
    ontology: &Ontology,
    query: &ConjunctiveQuery,
) -> Result<Vec<QueryAnswer>> {
    if query.atoms.len() > 1 {
        return Err(crate::Error::Parse(
            "conjunctive queries with more than one atom are not supported yet".into(),
        ));
    }
    if query.atoms.is_empty() {
        return Ok(Vec::new());
    }
    let atom = &query.atoms[0];
    match atom {
        QueryAtom::Type { var, class } => {
            let class_id = lookup_query_class(ontology, class)?;
            if engine.taxonomy().unsatisfiable.contains(&class_id) {
                return Ok(Vec::new());
            }
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
            let sup_id = lookup_query_class(ontology, superclass)?;
            if engine.taxonomy().unsatisfiable.contains(&sup_id) {
                return Ok(Vec::new());
            }
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
