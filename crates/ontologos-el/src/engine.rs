//! EL profile engine adapter (DIP unit struct).

use ontologos_core::{Axiom, EntityId, EntityKind, Ontology, Reasoner, Taxonomy};
use ontologos_rl::same_as_closure;

use crate::{ElClassifier, ElReport};

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

    /// Check consistency via EL classification (no unsatisfiable classes) plus disjoint clashes.
    pub fn is_consistent(&self, ontology: &Ontology) -> crate::Result<bool> {
        let taxonomy = ElClassifier::new().classify(ontology)?;
        if !taxonomy.unsatisfiable.is_empty() {
            return Ok(false);
        }
        if el_disjoint_abox_clash(ontology, &taxonomy) {
            return Ok(false);
        }
        Ok(!el_disjoint_tbox_clash(ontology, &taxonomy))
    }
}

fn expand_taxonomy_ancestors(
    taxonomy: &Taxonomy,
    start: EntityId,
) -> std::collections::HashSet<EntityId> {
    use std::collections::HashSet;
    let mut expanded = HashSet::from([start]);
    let mut stack = vec![start];
    while let Some(current) = stack.pop() {
        if let Some(cluster) = taxonomy.equivalent_classes(current) {
            for &eq in cluster {
                if expanded.insert(eq) {
                    stack.push(eq);
                }
            }
        }
        for sup in taxonomy.direct_superclasses(current) {
            if expanded.insert(sup) {
                stack.push(sup);
            }
        }
    }
    expanded
}

fn el_disjoint_abox_clash(ontology: &Ontology, taxonomy: &Taxonomy) -> bool {
    use std::collections::{HashMap, HashSet};

    let same_as = same_as_closure(ontology);
    let mut disjoint_pairs: Vec<(EntityId, EntityId)> = Vec::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::DisjointClasses(classes) = axiom {
            for i in 0..classes.len() {
                for j in (i + 1)..classes.len() {
                    let a = classes[i];
                    let b = classes[j];
                    let key = if a.0 <= b.0 { (a, b) } else { (b, a) };
                    disjoint_pairs.push(key);
                }
            }
        }
    }
    if disjoint_pairs.is_empty() {
        return false;
    }

    let mut types: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for (id, record) in ontology.entities().iter() {
        if !record.kind.satisfies(EntityKind::Individual) {
            continue;
        }
        let rep = same_as.representative(id);
        let entry = types.entry(rep).or_default();
        for &class in ontology.classes_of(id) {
            entry.insert(class);
        }
    }

    for classes in types.values() {
        let mut expanded = HashSet::new();
        for &class in classes {
            expanded.extend(expand_taxonomy_ancestors(taxonomy, class));
        }
        for &t in &expanded {
            for &(d1, d2) in &disjoint_pairs {
                if taxonomy.is_subsumed(t, d1) && taxonomy.is_subsumed(t, d2) {
                    return true;
                }
            }
        }
        let expanded: Vec<EntityId> = expanded.into_iter().collect();
        for i in 0..expanded.len() {
            for j in (i + 1)..expanded.len() {
                let a = expanded[i];
                let b = expanded[j];
                for &(d1, d2) in &disjoint_pairs {
                    if (taxonomy.is_subsumed(a, d1) && taxonomy.is_subsumed(b, d2))
                        || (taxonomy.is_subsumed(a, d2) && taxonomy.is_subsumed(b, d1))
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn el_disjoint_tbox_clash(ontology: &Ontology, taxonomy: &Taxonomy) -> bool {
    for (_, axiom) in ontology.axioms().iter() {
        let Axiom::DisjointClasses(classes) = axiom else {
            continue;
        };
        for i in 0..classes.len() {
            for j in (i + 1)..classes.len() {
                let a = classes[i];
                let b = classes[j];
                if taxonomy.is_subsumed(a, b) && taxonomy.is_subsumed(b, a) {
                    return true;
                }
            }
        }
    }
    false
}

#[cfg(test)]
mod dl_abox_index_tests {
    use ontologos_core::{Axiom, ClassExpr, DlAxiom, EntityKind, Ontology};

    use super::ElEngine;

    #[test]
    fn el_disjoint_abox_clash_sees_dl_store_class_assertion() {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://ex/a", EntityKind::Individual)
            .unwrap();
        let d1 = ontology
            .entity_id("http://ex/D1", EntityKind::Class)
            .unwrap();
        let d2 = ontology
            .entity_id("http://ex/D2", EntityKind::Class)
            .unwrap();
        ontology
            .add_axiom(Axiom::DisjointClasses(vec![d1, d2]))
            .unwrap();
        let ce1 = ontology.dl_mut().intern_ce(ClassExpr::Atomic(d1));
        let ce2 = ontology.dl_mut().intern_ce(ClassExpr::Atomic(d2));
        ontology.dl_mut().push_axiom(DlAxiom::ClassAssertion {
            individual: a,
            class: ce1,
        });
        ontology.dl_mut().push_axiom(DlAxiom::ClassAssertion {
            individual: a,
            class: ce2,
        });
        ontology.reindex_dl_abox();

        let engine = ElEngine;
        assert!(
            !engine.is_consistent(&ontology).expect("consistent"),
            "expected disjoint ABox clash from DL-store assertions"
        );
    }
}
