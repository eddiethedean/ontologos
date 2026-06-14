//! Blocking for ALC tableau (prevents infinite expansion).

use super::Branch;

/// Maximum tableau expansions before returning [`Error::ResourceLimit`].
pub(crate) const MAX_EXPANSIONS: u32 = 4096;

/// Whether the expansion budget has been exhausted.
#[must_use]
pub fn is_budget_exhausted(branch: &Branch<'_>) -> bool {
    branch.expansions >= MAX_EXPANSIONS
}

/// Whether expansion should stop on this world (budget exhaustion only).
///
/// Feature-style blocking is disabled until structural clash detection is complete;
/// nominal/cardinality ontologies rely on the expansion budget instead.
#[must_use]
pub fn is_blocked(branch: &Branch<'_>, world: usize) -> bool {
    if branch.expansions >= MAX_EXPANSIONS {
        return true;
    }
    let Some(w) = branch.worlds.get(world) else {
        return true;
    };
    w.blocked
}

/// Mark a world as blocked after an expansion step.
pub fn mark_blocked(branch: &mut Branch<'_>, world: usize) {
    if let Some(w) = branch.worlds.get_mut(world) {
        w.blocked = true;
    }
}
