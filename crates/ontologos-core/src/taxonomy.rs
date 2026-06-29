use serde::{Deserialize, Serialize};

use crate::entity::EntityId;

/// Extracted class taxonomy from a classification run.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Taxonomy {
    /// Direct subsumptions `(subclass, superclass)` after transitive reduction.
    pub subsumptions: Vec<(EntityId, EntityId)>,
    /// Equivalence class clusters (each vec has ≥ 2 members when from explicit axioms).
    pub equivalences: Vec<Vec<EntityId>>,
    /// Classes inferred equivalent to `owl:Nothing` / ⊥.
    pub unsatisfiable: Vec<EntityId>,
}

impl Taxonomy {
    /// Number of direct subsumption edges.
    #[must_use]
    pub fn subsumption_count(&self) -> usize {
        self.subsumptions.len()
    }

    /// Direct superclasses of `class` in the reduced taxonomy.
    #[must_use]
    pub fn direct_superclasses(&self, class: EntityId) -> Vec<EntityId> {
        self.subsumptions
            .iter()
            .filter_map(|&(sub, sup)| (sub == class).then_some(sup))
            .collect()
    }

    /// Direct subclasses of `class` in the reduced taxonomy.
    #[must_use]
    pub fn direct_subclasses(&self, class: EntityId) -> Vec<EntityId> {
        self.subsumptions
            .iter()
            .filter_map(|&(sub, sup)| (sup == class).then_some(sub))
            .collect()
    }

    /// Whether `sub` is subsumed by `sup` (direct or indirect) in this taxonomy.
    #[must_use]
    pub fn is_subsumed(&self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup {
            return true;
        }
        let mut stack: Vec<EntityId> = self.direct_superclasses(sub);
        let mut seen = std::collections::HashSet::new();
        while let Some(current) = stack.pop() {
            if current == sup {
                return true;
            }
            if !seen.insert(current) {
                continue;
            }
            stack.extend(self.direct_superclasses(current));
        }
        false
    }

    /// Equivalence cluster containing `class`, if any.
    #[must_use]
    pub fn equivalent_classes(&self, class: EntityId) -> Option<&[EntityId]> {
        self.equivalences
            .iter()
            .find(|cluster| cluster.contains(&class))
            .map(Vec::as_slice)
    }

    /// Remove transitively redundant subsumption edges `(A,C)` when `(A,B)` and `(B,C)` exist.
    ///
    /// Produces a **minimal** cover of the same subsumption relation (HermiT-style direct edges).
    pub fn reduce_transitive_redundancy(&mut self) {
        let edges: Vec<(EntityId, EntityId)> = self.subsumptions.clone();
        if edges.is_empty() {
            return;
        }
        let edge_count = edges.len();
        let mut direct_supers: std::collections::HashMap<EntityId, std::collections::HashSet<EntityId>> =
            std::collections::HashMap::new();
        for (sub, sup) in &edges {
            direct_supers.entry(*sub).or_default().insert(*sup);
        }
        let mut reduced = Vec::with_capacity(edge_count);
        for (sub, sup) in edges {
            let redundant = direct_supers
                .get(&sub)
                .is_some_and(|supers| supers.iter().any(|mid| *mid != sup && self.is_subsumed(*mid, sup)));
            if !redundant {
                reduced.push((sub, sup));
            }
        }
        reduced.sort_by_key(|(a, b)| (a.0, b.0));
        reduced.dedup();
        self.subsumptions = reduced;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduce_transitive_redundancy_removes_implied_edges() {
        let a = EntityId(1);
        let b = EntityId(2);
        let c = EntityId(3);
        let mut tax = Taxonomy {
            subsumptions: vec![(a, b), (b, c), (a, c)],
            ..Taxonomy::default()
        };
        tax.reduce_transitive_redundancy();
        assert_eq!(tax.subsumptions, vec![(a, b), (b, c)]);
    }
}
