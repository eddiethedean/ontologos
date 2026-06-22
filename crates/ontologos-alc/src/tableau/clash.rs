//! Clash detection for ALC tableau branches.

use ontologos_core::{CeId, ClassExpr, EntityId, RoleExpr};

use super::expand::{count_role_successors, effective_cardinality_filler};
use super::Branch;

/// Check direct label/negation clashes and disjointness constraints.
pub fn detect_clash(branch: &mut Branch<'_>) {
    if branch.clash {
        return;
    }
    check_negated_cardinality(branch);
    if branch.clash {
        return;
    }
    for world_idx in 0..branch.worlds.len() {
        check_conflicting_cardinality_bounds(branch, world_idx);
        if branch.clash {
            return;
        }
        check_conflicting_datatype_cardinality_bounds(branch, world_idx);
        if branch.clash {
            return;
        }
    }
    for (world_idx, world) in branch.worlds.iter().enumerate() {
        for &ce in &world.labels {
            if world.negated.contains(&ce) {
                branch.clash = true;
                return;
            }
            if matches!(branch.dl.core().dl().ce(ce), Some(ClassExpr::Bottom)) {
                branch.clash = true;
                return;
            }
            if let Some(ClassExpr::Not(inner)) = branch.dl.core().dl().ce(ce) {
                if world.labels.contains(inner) {
                    branch.clash = true;
                    return;
                }
            }
        }
        for &neg_ce in &world.negated {
            let Some(expr) = branch.dl.core().dl().ce(neg_ce).cloned() else {
                continue;
            };
            if let ClassExpr::Some { property, filler } = expr {
                if super::expand::existential_already_satisfied(
                    branch,
                    world_idx,
                    &property,
                    filler,
                ) {
                    branch.clash = true;
                    return;
                }
            }
        }
        for &(left, right) in &branch.disjoint {
            if world.labels.contains(&left) && world.labels.contains(&right) {
                branch.clash = true;
                return;
            }
            if (world.labels.contains(&left)
                && super::expand::world_structurally_satisfies(branch, world_idx, right))
                || (world.labels.contains(&right)
                    && super::expand::world_structurally_satisfies(branch, world_idx, left))
            {
                branch.clash = true;
                return;
            }
        }
    }
}

/// Assert `ce` into a world, detecting immediate clashes.
pub fn assert_label(branch: &mut Branch<'_>, world: usize, ce: CeId) {
    let ce = super::effective_class_expression(branch.dl, ce);
    if matches!(branch.dl.core().dl().ce(ce), Some(ClassExpr::Bottom)) {
        branch.clash = true;
        return;
    }
    let w = &mut branch.worlds[world];
    if w.negated.contains(&ce) {
        branch.clash = true;
        return;
    }
    if w.labels.insert(ce) {
        w.queue.push_back(ce);
        propagate_subsumptions(branch, world, ce);
    }
    detect_clash(branch);
}

fn propagate_subsumptions(branch: &mut Branch<'_>, world: usize, _trigger: CeId) {
    if branch.clash {
        return;
    }
    loop {
        let mut progressed = false;
        for &(sub, sup) in branch.tbox_subsumptions.clone().iter() {
            if world_satisfies_sub(branch, world, sub)
                && !branch.worlds[world].labels.contains(&sup)
            {
                let w = &mut branch.worlds[world];
                if w.negated.contains(&sup) {
                    branch.clash = true;
                    return;
                }
                w.labels.insert(sup);
                w.queue.push_back(sup);
                progressed = true;
            }
        }
        if !progressed {
            break;
        }
        detect_clash(branch);
        if branch.clash {
            return;
        }
    }
}

fn world_satisfies_sub(branch: &Branch<'_>, world: usize, sub: CeId) -> bool {
    if branch.worlds[world].labels.contains(&sub) {
        return true;
    }
    if is_thing_ce(branch, sub) {
        return branch.worlds[world]
            .labels
            .iter()
            .any(|&label| is_thing_ce(branch, label));
    }
    false
}

pub(crate) fn is_thing_ce(branch: &Branch<'_>, ce: CeId) -> bool {
    let store = branch.dl.core().dl();
    match store.ce(ce) {
        Some(ClassExpr::Top) => true,
        Some(ClassExpr::Atomic(id)) => branch
            .dl
            .core()
            .entity(*id)
            .ok()
            .and_then(|record| branch.dl.core().resolve_iri(record.iri).ok())
            .is_some_and(|iri| {
                iri == "http://www.w3.org/2002/07/owl#Thing"
                    || iri.ends_with("#Thing")
                    || iri.ends_with("/Thing")
            }),
        _ => false,
    }
}

/// Assert negation of `ce` into a world.
pub fn assert_negation(branch: &mut Branch<'_>, world: usize, ce: CeId) {
    let w = &mut branch.worlds[world];
    if w.labels.contains(&ce) {
        branch.clash = true;
        return;
    }
    w.negated.insert(ce);
    detect_clash(branch);
}

/// Clash when a world satisfies `∃R.C` and `∃R.C ⊑ ⊥` is in the TBox.
pub fn check_existential_bottom_subsumptions(branch: &mut Branch<'_>) {
    if branch.clash {
        return;
    }
    let subs: Vec<(CeId, CeId)> = branch
        .tbox_subsumptions
        .iter()
        .copied()
        .filter(|&(_, sup)| matches!(branch.dl.core().dl().ce(sup), Some(ClassExpr::Bottom)))
        .collect();
    for world in 0..branch.worlds.len() {
        for &(sub, _) in &subs {
            if super::expand::world_structurally_satisfies(branch, world, sub) {
                branch.clash = true;
                return;
            }
        }
    }
}

/// Clash when positive min/max datatype cardinality bounds on the same property disagree.
pub fn check_conflicting_datatype_cardinality_bounds(branch: &mut Branch<'_>, world: usize) {
    if branch.clash {
        return;
    }
    let labels = branch.worlds[world].labels.clone();
    let store = branch.dl.core().dl();
    let mut mins: Vec<(u32, EntityId)> = Vec::new();
    let mut maxs: Vec<(u32, EntityId)> = Vec::new();
    for ce in labels {
        let Some(expr) = store.ce(ce).cloned() else {
            continue;
        };
        match expr {
            ClassExpr::DataMinCardinality { n, property, .. } if n > 0 => {
                mins.push((n, property));
            }
            ClassExpr::DataMaxCardinality { n, property, .. } => {
                maxs.push((n, property));
            }
            ClassExpr::DataExactCardinality { n, property, .. } => {
                mins.push((n, property));
                maxs.push((n, property));
            }
            _ => {}
        }
    }
    for (min_n, min_p) in mins {
        for (max_n, max_p) in &maxs {
            if min_p == *max_p && min_n > *max_n {
                branch.clash = true;
                return;
            }
        }
    }
}

/// Clash when positive min/max cardinality bounds on the same role/filler disagree.
pub fn check_conflicting_cardinality_bounds(branch: &mut Branch<'_>, world: usize) {
    if branch.clash {
        return;
    }
    let labels = branch.worlds[world].labels.clone();
    let store = branch.dl.core().dl();
    let mut mins: Vec<(u32, RoleExpr, Option<CeId>)> = Vec::new();
    let mut maxs: Vec<(u32, RoleExpr, Option<CeId>)> = Vec::new();
    for ce in labels {
        let Some(expr) = store.ce(ce).cloned() else {
            continue;
        };
        match expr {
            ClassExpr::MinCardinality {
                n,
                property,
                filler,
            } if n > 0 => {
                mins.push((n, property, effective_cardinality_filler(branch, filler)));
            }
            ClassExpr::MaxCardinality {
                n,
                property,
                filler,
            } => {
                maxs.push((n, property, effective_cardinality_filler(branch, filler)));
            }
            _ => {}
        }
    }
    for (min_n, min_p, min_f) in mins {
        for (max_n, max_p, max_f) in &maxs {
            if min_p == *max_p && min_f == *max_f && min_n > *max_n {
                branch.clash = true;
                return;
            }
        }
    }
}

/// Clash when negated cardinality bounds are violated by the current successor set.
pub fn check_negated_cardinality(branch: &mut Branch<'_>) {
    if branch.clash {
        return;
    }
    let store = branch.dl.core().dl();
    for (world_idx, world) in branch.worlds.iter().enumerate() {
        for &neg_ce in &world.negated {
            let Some(expr) = store.ce(neg_ce).cloned() else {
                continue;
            };
            match expr {
                ClassExpr::MinCardinality {
                    n,
                    property,
                    filler,
                } if n > 0 => {
                    let filler = super::expand::effective_cardinality_filler(branch, filler);
                    let count = count_role_successors(branch, world_idx, &property, filler);
                    if count >= n as usize {
                        branch.clash = true;
                        return;
                    }
                }
                ClassExpr::MaxCardinality {
                    n,
                    property,
                    filler,
                } => {
                    let filler = super::expand::effective_cardinality_filler(branch, filler);
                    let count = count_role_successors(branch, world_idx, &property, filler);
                    if count <= n as usize {
                        branch.clash = true;
                        return;
                    }
                }
                _ => {}
            }
        }
    }
}
