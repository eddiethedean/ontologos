//! Derivation index keyed by [`EntityId`] / [`AxiomId`] for DL unsat cache and explain scaffolding.

use std::collections::{HashMap, HashSet};

use ontologos_core::{axiom_signature, AxiomId, EntityId, Ontology};

/// Tracks which axioms mention each entity (forward index).
#[derive(Debug, Clone, Default)]
pub struct DependencyIndex {
    by_entity: HashMap<EntityId, HashSet<AxiomId>>,
    by_axiom: HashMap<AxiomId, HashSet<EntityId>>,
}

impl DependencyIndex {
    /// Build a dependency index from all axioms in `ontology`.
    #[must_use]
    pub fn from_ontology(ontology: &Ontology) -> Self {
        let mut index = Self::default();
        for (id, axiom) in ontology.axioms().iter() {
            index.insert(id, axiom);
        }
        index
    }

    /// Record one axiom and its entity signature.
    pub fn insert(&mut self, axiom_id: AxiomId, axiom: &ontologos_core::Axiom) {
        let sig = axiom_signature(axiom);
        self.by_axiom.insert(axiom_id, sig.clone());
        for entity in sig {
            self.by_entity.entry(entity).or_default().insert(axiom_id);
        }
    }

    /// Axioms whose signature contains `entity`.
    #[must_use]
    pub fn axioms_for_entity(&self, entity: EntityId) -> Vec<AxiomId> {
        self.by_entity
            .get(&entity)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Entity signature for `axiom_id`.
    #[must_use]
    pub fn entities_for_axiom(&self, axiom_id: AxiomId) -> Vec<EntityId> {
        self.by_axiom
            .get(&axiom_id)
            .map(|s| s.iter().copied().collect())
            .unwrap_or_default()
    }

    /// Expand `seed` entities to all axioms in their dependency closure.
    #[must_use]
    pub fn closure_axioms(&self, seed: &HashSet<EntityId>) -> HashSet<AxiomId> {
        let mut out = HashSet::new();
        let mut queue: Vec<EntityId> = seed.iter().copied().collect();
        let mut seen = seed.clone();
        while let Some(entity) = queue.pop() {
            for axiom_id in self.axioms_for_entity(entity) {
                out.insert(axiom_id);
                for dep in self.entities_for_axiom(axiom_id) {
                    if seen.insert(dep) {
                        queue.push(dep);
                    }
                }
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_core::{Axiom, Ontology};

    #[test]
    fn closure_follows_subclass_chain() {
        let mut ont = Ontology::builder()
            .class("http://ex/A")
            .unwrap()
            .class("http://ex/B")
            .unwrap()
            .class("http://ex/C")
            .unwrap()
            .subclass_of("http://ex/A", "http://ex/B")
            .unwrap()
            .subclass_of("http://ex/B", "http://ex/C")
            .unwrap()
            .build()
            .unwrap();
        let a = ont.lookup_entity("http://ex/A").unwrap();
        let index = DependencyIndex::from_ontology(&ont);
        let closure = index.closure_axioms(&HashSet::from([a]));
        assert_eq!(closure.len(), 2);
        let _ = &mut ont;
        let _ = Axiom::SubClassOf {
            subclass: a,
            superclass: a,
        };
    }
}
