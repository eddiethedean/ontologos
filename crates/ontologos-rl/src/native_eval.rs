//! Evaluation harness for entity-native RL rules (subset) without triple round-trips.

use ontologos_core::{Axiom, EntityId, Ontology};

/// Materialize transitive `SubClassOf` closure using core axiom indexes only.
pub fn transitive_subclass_closure(ontology: &mut Ontology) -> usize {
    let mut added = 0_usize;
    loop {
        let edges: Vec<(EntityId, EntityId)> = ontology
            .axioms()
            .iter()
            .filter_map(|(_, axiom)| match axiom {
                Axiom::SubClassOf {
                    subclass,
                    superclass,
                } => Some((*subclass, *superclass)),
                _ => None,
            })
            .collect();
        let mut batch = 0_usize;
        for &(a, b) in &edges {
            for &(b2, c) in &edges {
                if b == b2 && a != c {
                    let before = ontology.axiom_count();
                    if ontology
                        .add_inferred_axiom(Axiom::SubClassOf {
                            subclass: a,
                            superclass: c,
                        })
                        .is_ok()
                        && ontology.axiom_count() > before
                    {
                        batch += 1;
                    }
                }
            }
        }
        if batch == 0 {
            break;
        }
        added += batch;
    }
    added
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::{EntityKind, Ontology};

    #[test]
    fn native_transitive_subclass_matches_chain() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex.org/A", EntityKind::Class)
            .unwrap();
        let b = ontology
            .entity_id("http://ex.org/B", EntityKind::Class)
            .unwrap();
        let c = ontology
            .entity_id("http://ex.org/C", EntityKind::Class)
            .unwrap();
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
        let inferred = transitive_subclass_closure(&mut ontology);
        assert!(inferred >= 1);
        let supers = ontology.direct_superclasses(a);
        assert!(
            supers.contains(&c),
            "expected transitive superclass C in direct supers, got {supers:?}"
        );
    }
}
