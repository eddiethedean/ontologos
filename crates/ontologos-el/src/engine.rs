//! EL profile engine adapter (DIP unit struct).

use ontologos_core::{Axiom, EntityId, Ontology, Reasoner, Taxonomy};

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
        if el_disjoint_abox_clash(ontology) {
            return Ok(false);
        }
        let taxonomy = ElClassifier::new().classify(ontology)?;
        if !taxonomy.unsatisfiable.is_empty() {
            return Ok(false);
        }
        Ok(!el_disjoint_tbox_clash(ontology, &taxonomy))
    }
}

fn el_disjoint_abox_clash(ontology: &Ontology) -> bool {
    use std::collections::HashMap;

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
    let disjoint: std::collections::HashSet<(EntityId, EntityId)> =
        disjoint_pairs.into_iter().collect();

    let mut types: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ClassAssertion { individual, class } = axiom {
            types.entry(*individual).or_default().push(*class);
        }
    }
    for classes in types.values() {
        for i in 0..classes.len() {
            for j in (i + 1)..classes.len() {
                let a = classes[i];
                let b = classes[j];
                let key = if a.0 <= b.0 { (a, b) } else { (b, a) };
                if disjoint.contains(&key) {
                    return true;
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
