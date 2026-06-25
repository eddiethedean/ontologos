//! Unsatisfiability cache for repeated branch configurations.

use std::collections::HashSet;

use ontologos_core::CeId;

/// Memoizes label sets known to be inconsistent.
#[derive(Debug, Default, Clone)]
pub struct UnsatCache {
    seen: HashSet<u64>,
}

impl UnsatCache {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a world label set as unsatisfiable.
    pub fn record_unsat(&mut self, labels: &HashSet<CeId>) {
        self.seen.insert(hash_labels(labels));
    }

    /// Whether this label set was previously seen as unsatisfiable.
    #[must_use]
    pub fn is_unsat(&self, labels: &HashSet<CeId>) -> bool {
        self.seen.contains(&hash_labels(labels))
    }

    /// Absorb entries from another cache.
    pub fn merge(&mut self, other: &Self) {
        self.seen.extend(other.seen.iter().copied());
    }
}

fn hash_labels(labels: &HashSet<CeId>) -> u64 {
    let mut ids: Vec<u32> = labels.iter().map(|c| c.0).collect();
    ids.sort_unstable();
    let mut hash = 0u64;
    for id in ids {
        hash = hash.wrapping_mul(31).wrapping_add(u64::from(id));
    }
    hash
}
