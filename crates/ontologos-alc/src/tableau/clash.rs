//! Clash detection for ALC tableau branches.

use ontologos_core::{CeId, ClassExpr, DlAxiom, EntityId, RoleExpr};

use super::expand::{count_role_successors, effective_cardinality_filler};
use super::{Branch, effective_class_expression};

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
        check_cross_kind_cardinality_bounds(branch, world_idx);
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
                    branch, world_idx, &property, filler,
                ) {
                    branch.clash = true;
                    return;
                }
            }
        }
        for &neg_ce in &world.negated {
            if super::expand::world_structurally_satisfies(branch, world_idx, neg_ce) {
                branch.clash = true;
                return;
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
    let mut seen = std::collections::HashSet::new();
    assert_label_inner(branch, world, ce, &mut seen);
}

fn assert_label_inner(
    branch: &mut Branch<'_>,
    world: usize,
    ce: CeId,
    seen: &mut std::collections::HashSet<(usize, CeId)>,
) {
    let ce = super::effective_class_expression(branch.dl, ce);
    if !seen.insert((world, ce)) {
        return;
    }
    if let Some(ClassExpr::And(ops)) = branch.dl.core().dl().ce(ce).cloned() {
        for op in crate::tableau::expand::and_conjuncts_cardinality_first(branch.dl, ops) {
            assert_label_inner(branch, world, op, seen);
            if branch.clash {
                return;
            }
        }
        return;
    }
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
            if !world_satisfies_sub(branch, world, sub) {
                continue;
            }
            let sup = super::effective_class_expression(branch.dl, sup);
            if branch.worlds[world].labels.contains(&sup) {
                continue;
            }
            let w = &mut branch.worlds[world];
            if let Some(ClassExpr::And(ops)) = branch.dl.core().dl().ce(sup).cloned() {
                if ops.iter().all(|op| w.labels.contains(op)) {
                    if w.labels.insert(sup) {
                        progressed = true;
                    }
                    continue;
                }
                for op in ops {
                    let op = super::effective_class_expression(branch.dl, op);
                    if w.negated.contains(&op) {
                        branch.clash = true;
                        return;
                    }
                    if w.labels.insert(op) {
                        w.queue.push_back(op);
                        progressed = true;
                    }
                }
                w.labels.insert(sup);
            } else {
                if w.negated.contains(&sup) {
                    branch.clash = true;
                    return;
                }
                if w.labels.insert(sup) {
                    w.queue.push_back(sup);
                    progressed = true;
                }
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
    if branch.worlds[world].labels.contains(&ce) {
        branch.clash = true;
        return;
    }
    if super::expand::world_structurally_satisfies(branch, world, ce) {
        branch.clash = true;
        return;
    }
    let w = &mut branch.worlds[world];
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
            if entity_same(branch, min_p, *max_p) && min_n > *max_n {
                branch.clash = true;
                return;
            }
        }
    }
}

fn entity_same(branch: &Branch<'_>, left: EntityId, right: EntityId) -> bool {
    if left == right {
        return true;
    }
    let left_iri = branch
        .dl
        .core()
        .entity(left)
        .ok()
        .and_then(|r| branch.dl.core().resolve_iri(r.iri).ok());
    let right_iri = branch
        .dl
        .core()
        .entity(right)
        .ok()
        .and_then(|r| branch.dl.core().resolve_iri(r.iri).ok());
    left_iri.is_some() && left_iri == right_iri
}

fn role_atomic_same(branch: &Branch<'_>, left: &RoleExpr, right: &RoleExpr) -> bool {
    match (left, right) {
        (RoleExpr::Atomic(a), RoleExpr::Atomic(b)) => entity_same(branch, *a, *b),
        _ => left == right,
    }
}

/// Clash when object- and datatype-property cardinality bounds disagree on the same IRI.
fn check_cross_kind_cardinality_bounds(branch: &mut Branch<'_>, world: usize) {
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
            ClassExpr::MinCardinality {
                n,
                property: RoleExpr::Atomic(id),
                filler,
            } if n > 0 && effective_cardinality_filler(branch, filler).is_none() => {
                mins.push((n, id));
            }
            ClassExpr::MaxCardinality {
                n,
                property: RoleExpr::Atomic(id),
                filler,
            } if effective_cardinality_filler(branch, filler).is_none() => {
                maxs.push((n, id));
            }
            ClassExpr::ExactCardinality {
                n,
                property: RoleExpr::Atomic(id),
                filler,
            } if effective_cardinality_filler(branch, filler).is_none() => {
                mins.push((n, id));
                maxs.push((n, id));
            }
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
    for (min_n, min_p) in &mins {
        for (max_n, max_p) in &maxs {
            if entity_same(branch, *min_p, *max_p) && min_n > max_n {
                branch.clash = true;
                return;
            }
        }
    }
}

/// Whether merging two worlds would violate datatype cardinality bounds.
pub fn would_datatype_clash_when_merged(
    branch: &super::Branch<'_>,
    left: usize,
    right: usize,
) -> bool {
    let left_bounds = datatype_bounds_from_world(branch, left);
    let right_bounds = datatype_bounds_from_world(branch, right);
    for (lprop, (lmin, lmax)) in &left_bounds {
        let (rmin, rmax) = matching_datatype_bound(branch, &right_bounds, *lprop);
        if (*lmin).max(rmin) > (*lmax).min(rmax) {
            return true;
        }
    }
    for (rprop, (rmin, rmax)) in &right_bounds {
        if left_bounds
            .keys()
            .any(|lprop| entity_same(branch, *lprop, *rprop))
        {
            continue;
        }
        let (lmin, lmax) = (0, u32::MAX);
        if lmin.max(*rmin) > lmax.min(*rmax) {
            return true;
        }
    }
    false
}

fn matching_datatype_bound(
    branch: &Branch<'_>,
    bounds: &std::collections::HashMap<EntityId, (u32, u32)>,
    prop: EntityId,
) -> (u32, u32) {
    for (candidate, bound) in bounds {
        if entity_same(branch, prop, *candidate) {
            return *bound;
        }
    }
    (0, u32::MAX)
}

/// Whether merging would place an atomic class and its negation on one world.
pub fn would_complement_clash_when_merged(
    branch: &super::Branch<'_>,
    left: usize,
    right: usize,
) -> bool {
    let left_pos = atomic_class_entities(branch, left);
    let left_neg = negated_atomic_entities(branch, left);
    let right_pos = atomic_class_entities(branch, right);
    let right_neg = negated_atomic_entities(branch, right);
    left_pos.iter().any(|c| right_neg.contains(c)) || right_pos.iter().any(|c| left_neg.contains(c))
}

fn atomic_class_entities(
    branch: &super::Branch<'_>,
    world: usize,
) -> std::collections::HashSet<EntityId> {
    let mut out = std::collections::HashSet::new();
    for &label in &branch.worlds[world].labels {
        if let Some(ClassExpr::Atomic(id)) = branch.dl.core().dl().ce(label) {
            out.insert(*id);
        }
    }
    out
}

fn negated_atomic_entities(
    branch: &super::Branch<'_>,
    world: usize,
) -> std::collections::HashSet<EntityId> {
    let mut out = std::collections::HashSet::new();
    for &neg in &branch.worlds[world].negated {
        if let Some(ClassExpr::Atomic(id)) = branch.dl.core().dl().ce(neg) {
            out.insert(*id);
        }
    }
    out
}

fn datatype_bounds_from_world(
    branch: &super::Branch<'_>,
    world: usize,
) -> std::collections::HashMap<EntityId, (u32, u32)> {
    let mut bounds = std::collections::HashMap::new();
    for &label in &branch.worlds[world].labels {
        collect_datatype_bounds(branch, label, &mut bounds);
    }
    bounds
}

fn collect_datatype_bounds(
    branch: &super::Branch<'_>,
    ce: CeId,
    bounds: &mut std::collections::HashMap<EntityId, (u32, u32)>,
) {
    let ce = effective_class_expression(branch.dl, ce);
    let Some(expr) = branch.dl.core().dl().ce(ce).cloned() else {
        return;
    };
    match expr {
        ClassExpr::DataMinCardinality { n, property, .. } if n > 0 => {
            let entry = bounds.entry(property).or_insert((0, u32::MAX));
            entry.0 = entry.0.max(n);
        }
        ClassExpr::DataMaxCardinality { n, property, .. } => {
            let entry = bounds.entry(property).or_insert((0, u32::MAX));
            entry.1 = entry.1.min(n);
        }
        ClassExpr::DataExactCardinality { n, property, .. } => {
            let entry = bounds.entry(property).or_insert((0, u32::MAX));
            entry.0 = entry.0.max(n);
            entry.1 = entry.1.min(n);
        }
        ClassExpr::And(ops) => {
            for op in ops {
                collect_datatype_bounds(branch, op, bounds);
            }
        }
        _ => {}
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
            ClassExpr::ExactCardinality {
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
            if role_atomic_same(branch, &min_p, max_p) && min_f == *max_f && min_n > *max_n {
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

/// Clash when an unqualified min exceeds the sum of qualified max bounds over an `All` partition.
pub fn check_partition_cardinality_clash(branch: &mut Branch<'_>, world: usize) {
    if branch.clash {
        return;
    }
    use std::collections::HashMap;
    let store = branch.dl.core().dl();
    let labels = branch.worlds[world].labels.clone();
    let mut min_unqualified: HashMap<RoleExpr, u32> = HashMap::new();
    let mut max_qualified: HashMap<RoleExpr, Vec<(CeId, u32)>> = HashMap::new();
    let mut all_fillers: HashMap<RoleExpr, CeId> = HashMap::new();

    for ce in labels {
        let Some(expr) = store.ce(ce).cloned() else {
            continue;
        };
        match expr {
            ClassExpr::MinCardinality {
                n,
                property,
                filler: None,
            } if n > 0 => {
                min_unqualified
                    .entry(property)
                    .and_modify(|m| *m = (*m).max(n))
                    .or_insert(n);
            }
            ClassExpr::MaxCardinality {
                n,
                property,
                filler: Some(filler),
            } => {
                max_qualified.entry(property).or_default().push((filler, n));
            }
            ClassExpr::All { property, filler } => {
                all_fillers.insert(property, filler);
            }
            _ => {}
        }
    }

    for (role, min_n) in min_unqualified {
        let Some(&all_filler) = all_fillers.get(&role) else {
            continue;
        };
        let Some(parts) = union_partition_fillers(branch, all_filler) else {
            continue;
        };
        let caps = max_qualified.get(&role);
        let mut total_cap = 0_u32;
        for part in parts {
            let part_cap = caps
                .and_then(|entries| entries.iter().find(|(f, _)| *f == part).map(|(_, n)| *n))
                .unwrap_or(u32::MAX);
            total_cap = total_cap.saturating_add(part_cap);
            if total_cap == u32::MAX {
                break;
            }
        }
        if min_n > total_cap {
            branch.clash = true;
            return;
        }
    }
}

fn union_partition_fillers(branch: &Branch<'_>, ce: CeId) -> Option<Vec<CeId>> {
    let store = branch.dl.core().dl();
    match store.ce(ce).cloned()? {
        ClassExpr::Or(ops) => Some(ops),
        ClassExpr::Atomic(_class) => {
            for axiom in store.axioms() {
                let DlAxiom::EquivalentClasses(ids) = axiom else {
                    continue;
                };
                if !ids.contains(&ce) {
                    continue;
                }
                for &other in ids {
                    if other == ce {
                        continue;
                    }
                    if let Some(ClassExpr::Or(ops)) = store.ce(other) {
                        return Some(ops.clone());
                    }
                }
            }
            None
        }
        _ => None,
    }
}
