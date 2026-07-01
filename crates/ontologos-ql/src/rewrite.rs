//! Query rewriting over a classified taxonomy (OWL QL fragment).

use ontologos_core::Taxonomy;
use crate::hierarchy::TaxonomyHierarchy;

use crate::query::{ConjunctiveQuery, QueryAtom};
use crate::{Error, Result};

/// Rewrite CQ atoms to subsume queries over named classes using the taxonomy.
pub fn rewrite_query(
    engine: &TaxonomyHierarchy<'_>,
    taxonomy: &Taxonomy,
    query: &ConjunctiveQuery,
) -> Result<ConjunctiveQuery> {
    if !is_ql_shape(engine.ontology(), query) {
        let unknown = query
            .atoms
            .iter()
            .find_map(|atom| match atom {
                QueryAtom::Type { class, .. } | QueryAtom::Subsumed { superclass: class, .. } => {
                    engine.ontology().lookup_entity(class).is_none().then(|| class.clone())
                }
            })
            .unwrap_or_else(|| "<unknown>".into());
        return Err(Error::UnknownClass(unknown));
    }
    let mut atoms = Vec::with_capacity(query.atoms.len());
    for atom in &query.atoms {
        atoms.push(rewrite_atom(engine, taxonomy, atom)?);
    }
    Ok(ConjunctiveQuery { atoms })
}

fn rewrite_atom(
    engine: &TaxonomyHierarchy<'_>,
    taxonomy: &Taxonomy,
    atom: &QueryAtom,
) -> Result<QueryAtom> {
    match atom {
        QueryAtom::Type { var, class } => {
            let class_id = engine
                .ontology()
                .lookup_entity(class)
                .ok_or_else(|| Error::UnknownClass(class.clone()))?;
            if taxonomy.unsatisfiable.contains(&class_id) {
                return Ok(QueryAtom::Type {
                    var: var.clone(),
                    class: "owl:Nothing".into(),
                });
            }
            Ok(atom.clone())
        }
        QueryAtom::Subsumed { var, superclass } => {
            let sup = engine
                .ontology()
                .lookup_entity(superclass)
                .ok_or_else(|| Error::UnknownClass(superclass.clone()))?;
            if taxonomy.unsatisfiable.contains(&sup) {
                return Ok(QueryAtom::Subsumed {
                    var: var.clone(),
                    superclass: "owl:Nothing".into(),
                });
            }
            Ok(atom.clone())
        }
    }
}

/// True when every class named in the query is known in the ontology.
pub(crate) fn is_ql_shape(ontology: &ontologos_core::Ontology, query: &ConjunctiveQuery) -> bool {
    query.atoms.iter().all(|atom| match atom {
        QueryAtom::Type { class, .. }
        | QueryAtom::Subsumed {
            superclass: class, ..
        } => ontology.lookup_entity(class).is_some(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TaxonomyHierarchy;
    use ontologos_core::Ontology;
    use ontologos_el::ElClassifier;

    #[test]
    fn is_ql_shape_requires_known_classes() {
        let ont = Ontology::builder()
            .class("http://ex/A")
            .unwrap()
            .build()
            .unwrap();
        let q = ConjunctiveQuery {
            atoms: vec![QueryAtom::Type {
                var: "x".into(),
                class: "http://ex/A".into(),
            }],
        };
        assert!(is_ql_shape(&ont, &q));
        let bad = ConjunctiveQuery {
            atoms: vec![QueryAtom::Type {
                var: "x".into(),
                class: "http://ex/Missing".into(),
            }],
        };
        assert!(!is_ql_shape(&ont, &bad));
    }

    #[test]
    fn rewrite_preserves_type_atom() {
        let ont = Ontology::builder()
            .class("http://ex/A")
            .unwrap()
            .class("http://ex/B")
            .unwrap()
            .subclass_of("http://ex/A", "http://ex/B")
            .unwrap()
            .build()
            .unwrap();
        let tax = ElClassifier::new().classify(&ont).unwrap();
        let engine = TaxonomyHierarchy::new(&ont, &tax);
        let cq = ConjunctiveQuery {
            atoms: vec![QueryAtom::Type {
                var: "?x".into(),
                class: "http://ex/B".into(),
            }],
        };
        let rewritten = rewrite_query(&engine, &tax, &cq).unwrap();
        assert_eq!(rewritten.atoms.len(), 1);
    }
}
