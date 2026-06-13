//! Blocking for ALC tableau (prevents infinite expansion).

use std::collections::HashSet;

use ontologos_core::RoleExpr;

use super::Branch;

/// Maximum tableau expansions before returning [`Error::ResourceLimit`].
pub(crate) const MAX_EXPANSIONS: u32 = 4096;

/// Whether the expansion budget has been exhausted.
#[must_use]
pub fn is_budget_exhausted(branch: &Branch<'_>) -> bool {
    branch.expansions >= MAX_EXPANSIONS
}

/// Whether expansion should stop on this world (feature-style blocking or budget).
#[must_use]
pub fn is_blocked(branch: &Branch<'_>, world: usize) -> bool {
    if branch.expansions >= MAX_EXPANSIONS {
        return true;
    }
    let Some(w) = branch.worlds.get(world) else {
        return true;
    };
    if w.blocked {
        return true;
    }
    is_feature_blocked(branch, world)
}

fn is_feature_blocked(branch: &Branch<'_>, world: usize) -> bool {
    let labels = &branch.worlds[world].labels;
    if labels.is_empty() {
        return false;
    }
    for (other, other_world) in branch.worlds.iter().enumerate() {
        if other == world {
            continue;
        }
        if !labels.is_subset(&other_world.labels) {
            continue;
        }
        if same_successors(branch, world, other) {
            return true;
        }
    }
    false
}

fn same_successors(branch: &Branch<'_>, a: usize, b: usize) -> bool {
    let succ_a = successors(branch, a);
    let succ_b = successors(branch, b);
    succ_a == succ_b
}

fn successors(branch: &Branch<'_>, world: usize) -> HashSet<(RoleExpr, usize)> {
    branch
        .edges
        .iter()
        .filter(|(from, _, _)| *from == world)
        .map(|(_, role, to)| (role.clone(), *to))
        .collect()
}

/// Mark a world as blocked after an expansion step.
pub fn mark_blocked(branch: &mut Branch<'_>, world: usize) {
    if let Some(w) = branch.worlds.get_mut(world) {
        w.blocked = true;
    }
}
