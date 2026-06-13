use std::collections::HashSet;

use crate::axiom::{Axiom, AxiomId};
use crate::entity::EntityId;

/// Monotonic ontology edit counter (incremented on every add/remove).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct OntologyRevision(pub u64);

impl OntologyRevision {
    /// Advance to the next revision.
    pub fn bump(&mut self) {
        self.0 = self.0.saturating_add(1);
    }
}

/// Tracks axiom edits since the last `clear_dirty` call.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DirtySet {
    added: Vec<AxiomId>,
    removed: Vec<AxiomId>,
    has_removals: bool,
}

impl DirtySet {
    /// Record a newly added axiom.
    pub fn record_add(&mut self, id: AxiomId) {
        if !self.added.contains(&id) {
            self.added.push(id);
        }
    }

    /// Record a removed axiom.
    pub fn record_remove(&mut self, id: AxiomId) {
        if !self.removed.contains(&id) {
            self.removed.push(id);
        }
        self.has_removals = true;
    }

    /// Axiom ids added since the last flush.
    #[must_use]
    pub fn added(&self) -> &[AxiomId] {
        &self.added
    }

    /// Axiom ids removed since the last flush.
    #[must_use]
    pub fn removed(&self) -> &[AxiomId] {
        &self.removed
    }

    /// Whether any axiom was removed (forces full rematerialization for RL/RDFS).
    #[must_use]
    pub fn has_removals(&self) -> bool {
        self.has_removals
    }

    /// Whether there are pending edits.
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        !self.added.is_empty() || !self.removed.is_empty()
    }

    /// Clear pending edits after engines consume them.
    pub fn clear(&mut self) {
        self.added.clear();
        self.removed.clear();
        self.has_removals = false;
    }
}

/// Entity signature of an axiom (classes, properties, individuals referenced).
#[must_use]
pub fn axiom_signature(axiom: &Axiom) -> HashSet<EntityId> {
    let mut sig = HashSet::new();
    match axiom {
        Axiom::SubClassOf {
            subclass,
            superclass,
        } => {
            sig.insert(*subclass);
            sig.insert(*superclass);
        }
        Axiom::EquivalentClasses(classes) | Axiom::DisjointClasses(classes) => {
            sig.extend(classes.iter().copied());
        }
        Axiom::ObjectPropertyDomain { property, domain } => {
            sig.insert(*property);
            sig.insert(*domain);
        }
        Axiom::ObjectPropertyRange { property, range } => {
            sig.insert(*property);
            sig.insert(*range);
        }
        Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } => {
            sig.insert(*sub_property);
            sig.insert(*super_property);
        }
        Axiom::InverseObjectProperties { left, right } => {
            sig.insert(*left);
            sig.insert(*right);
        }
        Axiom::TransitiveObjectProperty(p)
        | Axiom::SymmetricObjectProperty(p)
        | Axiom::ReflexiveObjectProperty(p)
        | Axiom::FunctionalObjectProperty(p)
        | Axiom::InverseFunctionalObjectProperty(p)
        | Axiom::IrreflexiveObjectProperty(p)
        | Axiom::AsymmetricObjectProperty(p) => {
            sig.insert(*p);
        }
        Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } => {
            sig.insert(*subclass);
            sig.insert(*property);
            sig.insert(*filler);
        }
        Axiom::EquivalentObjectProperties(properties) => {
            sig.extend(properties.iter().copied());
        }
        Axiom::ClassAssertion { individual, class } => {
            sig.insert(*individual);
            sig.insert(*class);
        }
        Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => {
            sig.insert(*subject);
            sig.insert(*property);
            sig.insert(*object);
        }
        Axiom::SameIndividual(individuals) | Axiom::DifferentIndividuals(individuals) => {
            sig.extend(individuals.iter().copied());
        }
    }
    sig
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntityId;

    #[test]
    fn dirty_set_tracks_add_and_remove() {
        let mut dirty = DirtySet::default();
        dirty.record_add(AxiomId(0));
        dirty.record_add(AxiomId(1));
        assert!(dirty.is_dirty());
        assert!(!dirty.has_removals());
        dirty.record_remove(AxiomId(0));
        assert!(dirty.has_removals());
        dirty.clear();
        assert!(!dirty.is_dirty());
    }

    #[test]
    fn axiom_signature_collects_entities() {
        let axiom = Axiom::SubClassOf {
            subclass: EntityId(1),
            superclass: EntityId(2),
        };
        let sig = axiom_signature(&axiom);
        assert_eq!(sig.len(), 2);
        assert!(sig.contains(&EntityId(1)));
        assert!(sig.contains(&EntityId(2)));
    }
}
