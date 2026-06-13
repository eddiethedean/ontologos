//! Role inclusion axiom saturation (complex chains).

use std::collections::{HashMap, HashSet};

use ontologos_core::{EntityId, RoleExpr};

use ontologos_alc::{Clause, ClauseSet};

/// Saturate role hierarchy with complex chain inclusions.
#[derive(Debug, Default)]
pub struct RoleHierarchy {
    subroles: HashMap<EntityId, HashSet<EntityId>>,
    chains: Vec<(Vec<RoleExpr>, EntityId)>,
}

impl RoleHierarchy {
    /// Build from clausal role axioms.
    #[must_use]
    pub fn from_clauses(clauses: &ClauseSet) -> Self {
        let mut h = Self::default();
        for clause in clauses.clauses() {
            match clause {
                Clause::RoleSubsumption { sub, sup } => {
                    h.add_subrole(*sub, *sup);
                }
                Clause::RoleChain { chain, sup } => {
                    h.chains.push((chain.clone(), *sup));
                }
                _ => {}
            }
        }
        h.saturate();
        h
    }

    fn add_subrole(&mut self, sub: EntityId, sup: EntityId) {
        self.subroles.entry(sub).or_default().insert(sup);
    }

    fn saturate(&mut self) {
        let mut changed = true;
        while changed {
            changed = false;
            let pairs: Vec<(EntityId, EntityId)> = self
                .subroles
                .iter()
                .flat_map(|(&a, ss)| ss.iter().map(move |&b| (a, b)))
                .collect();
            for (a, b) in pairs {
                if let Some(bb) = self.subroles.get(&b).cloned() {
                    for c in bb {
                        if self.subroles.entry(a).or_default().insert(c) {
                            changed = true;
                        }
                    }
                }
            }
        }
    }

    /// Whether `sub` ⊑ `sup` in the saturated hierarchy.
    #[must_use]
    pub fn is_subrole(&self, sub: EntityId, sup: EntityId) -> bool {
        if sub == sup {
            return true;
        }
        self.subroles.get(&sub).is_some_and(|ss| ss.contains(&sup))
    }

    /// Complex role chains registered.
    pub fn chains(&self) -> &[(Vec<RoleExpr>, EntityId)] {
        &self.chains
    }

    /// Saturated atomic role subsumptions.
    pub fn subrole_map(&self) -> &HashMap<EntityId, HashSet<EntityId>> {
        &self.subroles
    }
}
