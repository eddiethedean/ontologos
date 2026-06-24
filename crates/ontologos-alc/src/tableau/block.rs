//! Blocking for ALC tableau (prevents infinite expansion).

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ontologos_core::{CeId, RoleExpr};

use super::Branch;

/// Maximum tableau expansions before returning [`Error::ResourceLimit`].
pub(crate) const MAX_EXPANSIONS: u32 = 4096;

/// Cap on tableau worlds created during pre-expansion materialization.
pub(crate) const MAX_WORLDS: usize = 256;

/// Maximum stall iterations while every pending world is blocked.
pub(crate) const MAX_STALL_STEPS: u32 = 256;

/// Mark the branch incomplete so the next expansion returns [`Error::ResourceLimit`].
pub(crate) fn signal_resource_limit(branch: &mut super::Branch<'_>) {
    branch.expansions = MAX_EXPANSIONS;
}

/// Whether the expansion budget has been exhausted.
#[must_use]
pub fn is_budget_exhausted(branch: &Branch<'_>) -> bool {
    branch.expansions >= MAX_EXPANSIONS
}

/// Canonical signature for any-style blocking on a tableau world.
#[must_use]
pub fn world_signature(branch: &Branch<'_>, world: usize) -> u64 {
    let Some(w) = branch.worlds.get(world) else {
        return 0;
    };
    let mut hasher = DefaultHasher::new();
    let mut labels: Vec<CeId> = w.labels.iter().copied().collect();
    labels.sort_by_key(|ce| ce.0);
    labels.hash(&mut hasher);
    let mut negated: Vec<CeId> = w.negated.iter().copied().collect();
    negated.sort_by_key(|ce| ce.0);
    negated.hash(&mut hasher);
    w.queue.len().hash(&mut hasher);
    let mut edges: Vec<(u64, usize)> = branch
        .edges
        .iter()
        .filter(|(from, _, _)| *from == world)
        .map(|(_, role, to)| (role_key(role), *to))
        .collect();
    edges.sort_by_key(|(role, to)| (*role, *to));
    edges.hash(&mut hasher);
    hasher.finish()
}

fn role_key(role: &RoleExpr) -> u64 {
    match role {
        RoleExpr::Atomic(id) => id.0 as u64,
        RoleExpr::Inverse(id) => (id.0 as u64) | (1 << 31),
    }
}

/// Whether expansion should stop on this world (budget exhaustion or feature blocking).
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

/// Any-style blocking: if this world's signature was seen before, mark it blocked.
pub fn apply_signature_blocking(branch: &mut Branch<'_>, world: usize) {
    let sig = world_signature(branch, world);
    if branch.blocked_signatures.contains(&sig) {
        mark_blocked(branch, world);
    } else {
        branch.blocked_signatures.insert(sig);
    }
}
