//! Unsatisfiability cache for repeated branch configurations.

use std::collections::HashSet;

use ontologos_core::CeId;

/// Canonical sorted label-set key (collision-safe).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct LabelKey(Vec<u32>);

impl LabelKey {
    fn from_labels(labels: &HashSet<CeId>) -> Self {
        let mut ids: Vec<u32> = labels.iter().map(|c| c.0).collect();
        ids.sort_unstable();
        Self(ids)
    }
}

/// Memoizes label sets known to be inconsistent.
#[derive(Debug, Default, Clone)]
pub struct UnsatCache {
    seen: HashSet<LabelKey>,
}

impl UnsatCache {
    /// Create an empty unsatisfiability cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a world label set as unsatisfiable.
    pub fn record_unsat(&mut self, labels: &HashSet<CeId>) {
        self.seen.insert(LabelKey::from_labels(labels));
    }

    /// Whether this label set was previously seen as unsatisfiable.
    #[must_use]
    pub fn is_unsat(&self, labels: &HashSet<CeId>) -> bool {
        self.seen.contains(&LabelKey::from_labels(labels))
    }

    /// Absorb entries from another cache.
    pub fn merge(&mut self, other: &Self) {
        self.seen.extend(other.seen.iter().cloned());
    }
}
