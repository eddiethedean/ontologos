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
}
