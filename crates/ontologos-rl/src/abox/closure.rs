//! `sameAs` equivalence closure over individuals.

use std::collections::HashMap;

use ontologos_core::{Axiom, EntityId, Ontology};

/// Equivalence classes of individuals under `owl:sameAs`.
#[derive(Debug, Clone, Default)]
pub struct SameAsClosure {
    /// Representative → cluster members.
    pub clusters: Vec<Vec<EntityId>>,
    rep: HashMap<EntityId, EntityId>,
}

/// Build `sameAs` closure from asserted axioms.
#[must_use]
pub fn same_as_closure(ontology: &Ontology) -> SameAsClosure {
    let mut parent: HashMap<EntityId, EntityId> = HashMap::new();

    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::SameIndividual(ids) = axiom {
            if ids.len() < 2 {
                continue;
            }
            for w in ids.windows(2) {
                union(&mut parent, w[0], w[1]);
            }
        }
    }

    let mut clusters_map: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
    let keys: Vec<EntityId> = parent.keys().copied().collect();
    for id in keys {
        let rep = find(&mut parent, id);
        clusters_map.entry(rep).or_default().push(id);
    }

    SameAsClosure {
        clusters: clusters_map.into_values().collect(),
        rep: parent,
    }
}

fn find(parent: &mut HashMap<EntityId, EntityId>, x: EntityId) -> EntityId {
    let mut current = x;
    while parent.get(&current).copied().unwrap_or(current) != current {
        current = parent[&current];
    }
    // path compression
    let root = current;
    let mut node = x;
    while node != root {
        let next = parent.get(&node).copied().unwrap_or(node);
        parent.insert(node, root);
        node = next;
    }
    root
}

fn union(parent: &mut HashMap<EntityId, EntityId>, a: EntityId, b: EntityId) {
    let ra = find(parent, a);
    let rb = find(parent, b);
    if ra != rb {
        parent.insert(rb, ra);
    }
}

impl SameAsClosure {
    /// Canonical representative for an individual.
    #[must_use]
    pub fn representative(&self, id: EntityId) -> EntityId {
        *self.rep.get(&id).unwrap_or(&id)
    }
}
