//! Tableau expansion rules (∧, ∨, ∃, ∀, ¬).

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, EntityId, RoleExpr};

use super::block;
use super::clash::{self, assert_label, assert_negation};
use super::Branch;

/// Process one queued class expression in `world`.
pub fn process(branch: &mut Branch<'_>, world: usize, ce: CeId) -> Result<(), crate::Error> {
    if branch.clash || block::is_blocked(branch, world) {
        return Ok(());
    }
    let Some(expr) = branch.dl.core().dl().ce(ce).cloned() else {
        return Ok(());
    };
    branch.expansions += 1;
    match expr {
        ClassExpr::Bottom => branch.clash = true,
        ClassExpr::Top | ClassExpr::Atomic(_) => {}
        ClassExpr::Not(inner) => assert_negation(branch, world, inner),
        ClassExpr::And(ops) => {
            for op in ops {
                assert_label(branch, world, op);
                if branch.clash {
                    return Ok(());
                }
            }
        }
        ClassExpr::Or(ops) => {
            if !expand_disjunction(branch, world, ops)? {
                branch.clash = true;
            }
        }
        ClassExpr::Some { property, filler } => {
            apply_existential_clauses(branch, world, &property, filler);
            expand_existential(branch, world, property, filler);
        }
        ClassExpr::All { property, filler } => {
            expand_universal(branch, world, &property, filler);
        }
        ClassExpr::OneOf(individuals) => {
            let mut active_world = world;
            if individuals.len() == 1 {
                if let Some(&named_world) = branch.named_worlds.get(&individuals[0]) {
                    if named_world != active_world {
                        branch.merge_worlds(named_world, active_world);
                        if branch.clash {
                            return Ok(());
                        }
                    }
                    active_world = named_world;
                }
            }
            for ind in individuals {
                let nom = branch
                    .dl
                    .core()
                    .dl()
                    .expressions()
                    .find_map(|(id, e)| match e {
                        ClassExpr::OneOf(v) if v == &[ind] => Some(id),
                        _ => None,
                    });
                if let Some(id) = nom {
                    assert_label(branch, active_world, id);
                }
            }
        }
        ClassExpr::HasValue {
            property,
            individual,
        } => {
            expand_has_value(branch, world, property, individual);
        }
        ClassExpr::HasSelf(property) => {
            branch
                .edges
                .push((world, RoleExpr::Atomic(property), world));
            apply_universal_on_edge(branch, world, world);
            saturate_composed_edges(branch);
        }
        ClassExpr::MinCardinality { n: 0, .. } => {}
        ClassExpr::MinCardinality {
            n,
            property,
            filler,
        } => {
            expand_min_cardinality(branch, world, n, property, filler);
        }
        ClassExpr::MaxCardinality { n: 0, .. } => {}
        ClassExpr::MaxCardinality { .. } => {
            recheck_cardinality_on_world(branch, world);
        }
        ClassExpr::ExactCardinality {
            n,
            property,
            filler,
        } => {
            let filler = effective_cardinality_filler(branch, filler);
            let mut successors = role_successor_worlds(branch, world, &property, filler);
            if successors.len() > n as usize {
                if n == 1 && try_merge_role_successors(branch, &successors) {
                    successors = role_successor_worlds(branch, world, &property, filler);
                }
                if successors.len() > n as usize {
                    branch.clash = true;
                }
            } else if successors.len() < n as usize {
                let filler = filler.or_else(|| top_ce(branch));
                if let Some(f) = filler {
                    for _ in 0..(n as usize - successors.len()) {
                        expand_existential(branch, world, property.clone(), f);
                    }
                }
            }
        }
        ClassExpr::DataAll { .. }
        | ClassExpr::DataSome { .. }
        | ClassExpr::DataHasValue { .. }
        | ClassExpr::DataMinCardinality { .. }
        | ClassExpr::DataMaxCardinality { .. }
        | ClassExpr::DataExactCardinality { .. } => {}
    }
    if !branch.clash {
        propagate_structural_existential_subsumptions(branch);
        materialize_existential_successors(branch);
        let named_worlds: Vec<usize> = branch.named_worlds.values().copied().collect();
        for named_world in named_worlds {
            recheck_cardinality_on_world(branch, named_world);
            if branch.clash {
                return Ok(());
            }
        }
    }
    Ok(())
}

fn expand_min_cardinality(
    branch: &mut Branch<'_>,
    world: usize,
    n: u32,
    property: RoleExpr,
    filler: Option<CeId>,
) {
    let filler = effective_cardinality_filler(branch, filler);
    let count = count_role_successors(branch, world, &property, filler);
    if count >= n as usize {
        return;
    }
    let filler = filler.or_else(|| top_ce(branch));
    let Some(f) = filler else {
        return;
    };
    for _ in 0..(n as usize - count) {
        expand_existential(branch, world, property.clone(), f);
        if branch.clash {
            return;
        }
    }
}

fn top_ce(branch: &Branch<'_>) -> Option<CeId> {
    branch
        .dl
        .core()
        .dl()
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::Top => Some(id),
            _ => None,
        })
}

pub(crate) fn effective_cardinality_filler(
    branch: &Branch<'_>,
    filler: Option<CeId>,
) -> Option<CeId> {
    let f = filler?;
    if is_universal_filler(branch, f) {
        None
    } else {
        Some(f)
    }
}

fn is_universal_filler(branch: &Branch<'_>, ce: CeId) -> bool {
    let store = branch.dl.core().dl();
    match store.ce(ce) {
        Some(ClassExpr::Top) => true,
        Some(ClassExpr::Atomic(id)) => branch
            .dl
            .core()
            .entity(*id)
            .ok()
            .and_then(|rec| branch.dl.core().resolve_iri(rec.iri).ok())
            .is_some_and(|iri| iri == "http://www.w3.org/2002/07/owl#Thing"),
        _ => false,
    }
}

fn expand_has_value(
    branch: &mut Branch<'_>,
    world: usize,
    property: RoleExpr,
    individual: ontologos_core::EntityId,
) {
    if let Some(filler) = branch
        .dl
        .core()
        .dl()
        .expressions()
        .find_map(|(id, e)| match e {
            ClassExpr::OneOf(v) if v == &[individual] => Some(id),
            _ => None,
        })
    {
        expand_existential(branch, world, property, filler);
        return;
    }
    let target = ensure_named_world(branch, individual);
    add_role_edge(branch, world, property, target);
}

fn expand_existential(branch: &mut Branch<'_>, world: usize, property: RoleExpr, filler: CeId) {
    apply_existential_clauses(branch, world, &property, filler);
    if branch.clash {
        return;
    }
    if existential_already_satisfied(branch, world, &property, filler) {
        clash::check_negated_cardinality(branch);
        return;
    }
    if world_satisfies_filler(branch, world, filler)
        && !should_unravel_existential(branch, filler)
    {
        add_role_edge(branch, world, property.clone(), world);
        clash::check_negated_cardinality(branch);
        return;
    }
    if let Some(individual) = nominal_individual(branch, filler) {
        let target = ensure_named_world(branch, individual);
        add_role_edge(branch, world, property, target);
        clash::check_negated_cardinality(branch);
        return;
    }
    if has_universal_has_self_for_role(branch, world, &property) {
        add_role_edge(branch, world, property.clone(), world);
        assert_label(branch, world, filler);
        if branch.clash {
            return;
        }
        clash::check_negated_cardinality(branch);
        return;
    }
    if let Some(target) = existing_role_successor(branch, world, &property) {
        assert_label(branch, target, filler);
        if branch.clash {
            return;
        }
        clash::check_negated_cardinality(branch);
        return;
    }
    let new_world = branch.worlds.len();
    branch.worlds.push(super::World::default());
    branch.edges.push((world, property, new_world));
    assert_label(branch, new_world, filler);
    apply_universal_on_edge(branch, world, new_world);
    saturate_composed_edges(branch);
    clash::check_negated_cardinality(branch);
}

fn existing_role_successor(
    branch: &Branch<'_>,
    world: usize,
    property: &RoleExpr,
) -> Option<usize> {
    branch
        .edges
        .iter()
        .find(|(from, role, _)| *from == world && role_subsumes(branch, property, role))
        .map(|(_, _, to)| *to)
}

pub(crate) fn existential_already_satisfied(
    branch: &Branch<'_>,
    world: usize,
    property: &RoleExpr,
    filler: CeId,
) -> bool {
    branch.edges.iter().any(|(from, role, to)| {
        *from == world
            && role_subsumes(branch, property, role)
            && world_satisfies_filler(branch, *to, filler)
    })
}

/// Unravel `C ⊑ ∃R.C` into fresh successors when `C` also has nominal HasValue constraints.
fn should_unravel_existential(branch: &Branch<'_>, filler: CeId) -> bool {
    let Some(ClassExpr::Atomic(_)) = branch.dl.core().dl().ce(filler) else {
        return false;
    };
    branch.tbox_subsumptions.iter().any(|&(sub, sup)| {
        sub == filler
            && matches!(
                branch.dl.core().dl().ce(sup),
                Some(ClassExpr::HasValue { .. })
            )
    })
}

fn has_universal_has_self_for_role(branch: &Branch<'_>, world: usize, role: &RoleExpr) -> bool {
    let store = branch.dl.core().dl();
    let mut candidates: Vec<(RoleExpr, CeId)> = branch.worlds[world]
        .labels
        .iter()
        .filter_map(|&ce| match store.ce(ce)? {
            ClassExpr::All { property, filler } => Some((property.clone(), *filler)),
            _ => None,
        })
        .collect();
    for (sub, property, filler) in &branch.universals {
        if world_satisfies_subject(branch, world, *sub) {
            candidates.push((property.clone(), *filler));
        }
    }
    candidates.iter().any(|(universal_role, filler)| {
        role_subsumes(branch, universal_role, role)
            && matches!(store.ce(*filler), Some(ClassExpr::HasSelf(has_self_role))
                if role_subsumes(branch, role, &RoleExpr::Atomic(*has_self_role)))
    })
}

fn nominal_individual(branch: &Branch<'_>, filler: CeId) -> Option<EntityId> {
    match branch.dl.core().dl().ce(filler)? {
        ClassExpr::OneOf(v) if v.len() == 1 => Some(v[0]),
        _ => None,
    }
}

fn ensure_named_world(branch: &mut Branch<'_>, id: EntityId) -> usize {
    if let Some(&w) = branch.named_worlds.get(&id) {
        return w;
    }
    let w = branch.worlds.len();
    branch.worlds.push(super::World::default());
    if let Some(nom) = branch
        .dl
        .core()
        .dl()
        .expressions()
        .find_map(|(ce, e)| match e {
            ClassExpr::OneOf(v) if v == &[id] => Some(ce),
            _ => None,
        })
    {
        assert_label(branch, w, nom);
    }
    branch.named_worlds.insert(id, w);
    w
}

fn world_satisfies_filler(branch: &Branch<'_>, world: usize, filler: CeId) -> bool {
    if matches!(branch.dl.core().dl().ce(filler), Some(ClassExpr::Top))
        || clash::is_thing_ce(branch, filler)
    {
        return true;
    }
    let labels = &branch.worlds[world].labels;
    if labels.contains(&filler) {
        return true;
    }
    for &label in labels {
        if ce_subsumes(branch, label, filler) {
            return true;
        }
    }
    false
}

fn ce_subsumes(branch: &Branch<'_>, sub: CeId, sup: CeId) -> bool {
    if sub == sup {
        return true;
    }
    let mut work = vec![sub];
    let mut seen = HashSet::from([sub]);
    while let Some(cur) = work.pop() {
        for &(left, right) in &branch.tbox_subsumptions {
            if left == cur && !seen.contains(&right) {
                if right == sup {
                    return true;
                }
                seen.insert(right);
                work.push(right);
            }
        }
    }
    false
}

/// If a world has a reflexive role edge, assert matching `HasSelf` class expressions.
pub(crate) fn materialize_has_self_from_loops(branch: &mut Branch<'_>) {
    let store = branch.dl.core().dl();
    let has_self: Vec<(CeId, EntityId)> = store
        .expressions()
        .filter_map(|(id, e)| match e {
            ClassExpr::HasSelf(prop) => Some((id, *prop)),
            _ => None,
        })
        .collect();
    if has_self.is_empty() {
        return;
    }
    let edges = branch.edges.clone();
    for (from, role, to) in edges {
        if from != to {
            continue;
        }
        for &(ce, prop) in &has_self {
            if role_subsumes(branch, &RoleExpr::Atomic(prop), &role) {
                clash::assert_label(branch, from, ce);
                if branch.clash {
                    return;
                }
            }
        }
    }
}

/// Connect each named world to itself via `owl:topObjectProperty` when declared.
pub(crate) fn materialize_top_object_property_loops(
    branch: &mut Branch<'_>,
    worlds: &HashMap<EntityId, usize>,
) {
    const TOP: &str = "http://www.w3.org/2002/07/owl#topObjectProperty";
    let top = branch
        .dl
        .core()
        .entities()
        .iter()
        .find_map(|(id, record)| {
            branch
                .dl
                .core()
                .resolve_iri(record.iri)
                .ok()
                .filter(|iri| *iri == TOP)
                .map(|_| RoleExpr::Atomic(id))
        });
    let Some(top) = top else {
        return;
    };
    for &world in worlds.values() {
        add_role_edge(branch, world, top.clone(), world);
        if branch.clash {
            return;
        }
    }
}

/// Apply atomic `C ⊑ ∃R.D` and `C ⊑ HasValue(R, a)` when a world is labelled with `C`.
pub(crate) fn drive_atomic_existential_subsumptions(branch: &mut Branch<'_>) {
    let subs = branch.tbox_subsumptions.clone();
    for world in 0..branch.worlds.len() {
        for &(sub, sup) in &subs {
            if !branch.worlds[world].labels.contains(&sub) {
                continue;
            }
            let Some(ClassExpr::Atomic(_)) = branch.dl.core().dl().ce(sub) else {
                continue;
            };
            match branch.dl.core().dl().ce(sup).cloned() {
                Some(ClassExpr::Some { property, filler }) => {
                    expand_existential(branch, world, property, filler);
                }
                Some(ClassExpr::HasValue {
                    property,
                    individual,
                }) => {
                    expand_has_value(branch, world, property, individual);
                }
                _ => {}
            }
            if branch.clash {
                return;
            }
        }
    }
}

/// Apply `C ⊑ D` when a world structurally satisfies `C` (e.g. nested `∃` chains).
pub(crate) fn propagate_structural_existential_subsumptions(branch: &mut Branch<'_>) {
    let subs = branch.tbox_subsumptions.clone();
    let world_count = branch.worlds.len();
    for world in 0..world_count {
        for &(sub, sup) in &subs {
            if branch.worlds[world].labels.contains(&sup) {
                continue;
            }
            if world_structurally_satisfies(branch, world, sub) {
                assert_label(branch, world, sup);
                if branch.clash {
                    return;
                }
            }
        }
    }
}

pub(crate) fn world_structurally_satisfies(branch: &Branch<'_>, world: usize, ce: CeId) -> bool {
    if branch.worlds[world].labels.contains(&ce) {
        return true;
    }
    if world_satisfies_filler(branch, world, ce) {
        return true;
    }
    let Some(expr) = branch.dl.core().dl().ce(ce).cloned() else {
        return false;
    };
    match expr {
        ClassExpr::Some { property, filler } => branch.edges.iter().any(|(from, role, to)| {
            *from == world
                && role_subsumes(branch, &property, role)
                && world_structurally_satisfies(branch, *to, filler)
        }),
        ClassExpr::HasSelf(property) => branch.edges.iter().any(|(from, role, to)| {
            *from == world
                && *to == world
                && role_subsumes(branch, &RoleExpr::Atomic(property), role)
        }),
        _ => false,
    }
}

/// If `world` is labelled with `∃property.filler`, materialize `filler` on known successors.
pub(crate) fn materialize_existential_successors(branch: &mut Branch<'_>) {
    let store = branch.dl.core().dl();
    let world_count = branch.worlds.len();
    for world in 0..world_count {
        let labels = branch.worlds[world].labels.clone();
        for ce in labels {
            let Some(ClassExpr::Some { property, filler }) = store.ce(ce) else {
                continue;
            };
            for (from, role, to) in branch.edges.clone() {
                if from == world && role_subsumes(branch, property, &role) {
                    assert_label(branch, to, *filler);
                    if branch.clash {
                        return;
                    }
                }
            }
        }
    }
}

/// Insert a role edge and its declared inverse, applying universal propagation.
pub(crate) fn add_role_edge(branch: &mut Branch<'_>, from: usize, property: RoleExpr, to: usize) {
    let to = merge_inverse_functional_successor(branch, from, &property, to);
    branch.edges.push((from, property.clone(), to));
    apply_domain_on_edge(branch, from, &property);
    apply_universal_on_edge(branch, from, to);
    if is_symmetric_role(branch, &property) {
        branch.edges.push((to, property.clone(), from));
        apply_domain_on_edge(branch, to, &property);
        apply_universal_on_edge(branch, to, from);
        recheck_cardinality_on_world(branch, from);
        recheck_cardinality_on_world(branch, to);
    } else if let Some(inverse) = inverse_partner(branch, &property) {
        branch.edges.push((to, inverse.clone(), from));
        apply_domain_on_edge(branch, to, &inverse);
        apply_universal_on_edge(branch, to, from);
        recheck_cardinality_on_world(branch, from);
        recheck_cardinality_on_world(branch, to);
    } else {
        recheck_cardinality_on_world(branch, from);
        recheck_cardinality_on_world(branch, to);
    }
    propagate_structural_existential_subsumptions(branch);
    check_role_disjoint_on_edge(branch, from, &property, to);
    clash::check_existential_bottom_subsumptions(branch);
}

fn check_role_disjoint_on_edge(
    branch: &mut Branch<'_>,
    from: usize,
    property: &RoleExpr,
    to: usize,
) {
    if branch.clash {
        return;
    }
    for (f, prop2, t) in branch.edges.clone() {
        if f == from && t == to && !role_exprs_equal(property, &prop2) && roles_disjoint(branch, property, &prop2) {
            branch.clash = true;
            return;
        }
    }
}

pub(crate) fn role_disjoint_merge_blocked(branch: &Branch<'_>, keep: usize, drop: usize) -> bool {
    if keep == drop {
        return false;
    }
    for &(left, right) in &branch.role_disjoint {
        let p = RoleExpr::Atomic(left);
        let q = RoleExpr::Atomic(right);
        for from in 0..branch.worlds.len() {
            let mut p_targets = HashSet::new();
            let mut q_targets = HashSet::new();
            for (f, role, t) in &branch.edges {
                if *f != from {
                    continue;
                }
                if role_subsumes(branch, &p, role) {
                    p_targets.insert(*t);
                }
                if role_subsumes(branch, &q, role) {
                    q_targets.insert(*t);
                }
            }
            if (p_targets.contains(&keep) && q_targets.contains(&drop))
                || (p_targets.contains(&drop) && q_targets.contains(&keep))
            {
                return true;
            }
        }
    }
    false
}

pub(crate) fn roles_disjoint(branch: &Branch<'_>, r1: &RoleExpr, r2: &RoleExpr) -> bool {
    if role_exprs_equal(r1, r2) {
        return false;
    }
    for &(left, right) in &branch.role_disjoint {
        let p = RoleExpr::Atomic(left);
        let q = RoleExpr::Atomic(right);
        if (role_subsumes(branch, &p, r1) && role_subsumes(branch, &q, r2))
            || (role_subsumes(branch, &p, r2) && role_subsumes(branch, &q, r1))
        {
            return true;
        }
    }
    false
}

fn merge_inverse_functional_successor(
    branch: &mut Branch<'_>,
    from: usize,
    property: &RoleExpr,
    to: usize,
) -> usize {
    if !is_inverse_functional_role(branch, property) {
        return to;
    }
    let duplicate = branch
        .edges
        .iter()
        .find(|(f, role, t)| {
            *f == from
                && *t != to
                && is_inverse_functional_role(branch, role)
                && (role_subsumes(branch, property, role) || role_subsumes(branch, role, property))
        })
        .map(|(_, _, t)| *t);
    if let Some(existing) = duplicate {
        if existing != to {
            branch.merge_worlds(existing, to);
        }
        if branch.clash {
            return to;
        }
        existing
    } else {
        to
    }
}

fn is_inverse_functional_role(branch: &Branch<'_>, role: &RoleExpr) -> bool {
    match role {
        RoleExpr::Atomic(id) => branch.inverse_functional.contains(id),
        RoleExpr::Inverse(id) => branch.inverse_functional.contains(id),
    }
}

pub(crate) fn recheck_cardinality_on_world(branch: &mut Branch<'_>, world: usize) {
    if branch.clash {
        return;
    }
    clash::check_conflicting_cardinality_bounds(branch, world);
    if branch.clash {
        return;
    }
    let labels = branch.worlds[world].labels.clone();
    for ce in labels {
        let Some(expr) = branch.dl.core().dl().ce(ce).cloned() else {
            continue;
        };
        match expr {
            ClassExpr::MaxCardinality {
                n,
                property,
                filler,
            } => {
                let filler = effective_cardinality_filler(branch, filler);
                let mut successors = role_successor_worlds(branch, world, &property, filler);
                if successors.len() > n as usize {
                    if n == 1 && try_merge_role_successors(branch, &successors) {
                        successors = role_successor_worlds(branch, world, &property, filler);
                    }
                    if successors.len() > n as usize {
                        branch.clash = true;
                        return;
                    }
                }
            }
            ClassExpr::ExactCardinality {
                n,
                property,
                filler,
            } => {
                let filler = effective_cardinality_filler(branch, filler);
                let mut successors = role_successor_worlds(branch, world, &property, filler);
                if successors.len() > n as usize {
                    if n == 1 && try_merge_role_successors(branch, &successors) {
                        successors = role_successor_worlds(branch, world, &property, filler);
                    }
                    if successors.len() > n as usize {
                        branch.clash = true;
                        return;
                    }
                }
            }
            _ => {}
        }
    }
    clash::check_negated_cardinality(branch);
}

fn apply_domain_on_edge(branch: &mut Branch<'_>, from: usize, property: &RoleExpr) {
    if let Some(top) = top_ce(branch) {
        apply_existential_clauses(branch, from, property, top);
    }
}

fn is_symmetric_role(branch: &Branch<'_>, role: &RoleExpr) -> bool {
    branch
        .symmetric_roles
        .iter()
        .any(|sym| role_subsumes(branch, sym, role))
}

pub(crate) fn inverse_partner(branch: &Branch<'_>, role: &RoleExpr) -> Option<RoleExpr> {
    match role {
        RoleExpr::Atomic(id) => Some(
            branch
                .role_inverses
                .get(id)
                .map(|partner| RoleExpr::Atomic(*partner))
                .unwrap_or(RoleExpr::Inverse(*id)),
        ),
        RoleExpr::Inverse(id) => Some(
            branch
                .role_inverses
                .get(id)
                .map(|partner| RoleExpr::Atomic(*partner))
                .unwrap_or(RoleExpr::Atomic(*id)),
        ),
    }
}

/// Close role edges under subproperty hierarchy, transitivity, and chain inclusions.
pub(crate) fn saturate_composed_edges(branch: &mut Branch<'_>) {
    let mut changed = true;
    while changed {
        changed = false;
        if saturate_subrole_edges(branch) {
            changed = true;
        }
        if saturate_transitive_edges(branch) {
            changed = true;
        }
        if saturate_chain_edges(branch) {
            changed = true;
        }
    }
}

fn saturate_subrole_edges(branch: &mut Branch<'_>) -> bool {
    let mut added = false;
    let snapshot = branch.edges.clone();
    for (a, r, b) in &snapshot {
        if let RoleExpr::Atomic(sub) = r {
            let supers: Vec<EntityId> = branch
                .role_hierarchy
                .get(sub)
                .map(|s| s.iter().copied().collect())
                .unwrap_or_default();
            for sup in supers {
                let sup_role = RoleExpr::Atomic(sup);
                if !branch
                    .edges
                    .iter()
                    .any(|(x, role, y)| x == a && role == &sup_role && y == b)
                {
                    add_role_edge(branch, *a, sup_role, *b);
                    added = true;
                }
            }
        }
    }
    added
}

fn saturate_transitive_edges(branch: &mut Branch<'_>) -> bool {
    let mut added = false;
    let snapshot = branch.edges.clone();
    for (chain, sup) in &branch.role_chains.clone() {
        if chain.len() == 2 && chain[0] == chain[1] {
            if chain[0] != *sup {
                continue;
            }
            let role = chain[0].clone();
            for (a, r, b) in &snapshot {
                if *r != role {
                    continue;
                }
                for (b2, r2, c) in &snapshot {
                    if b == b2
                        && *r2 == role
                        && !branch
                            .edges
                            .iter()
                            .any(|(x, role2, y)| x == a && role2 == &role && y == c)
                    {
                        add_role_edge(branch, *a, role.clone(), *c);
                        added = true;
                    }
                }
            }
        }
    }
    added
}

fn saturate_chain_edges(branch: &mut Branch<'_>) -> bool {
    let mut added = false;
    let snapshot = branch.edges.clone();
    let chains = branch.role_chains.clone();
    let world_count = branch.worlds.len();
    for (chain, sup) in &chains {
        if chain.is_empty() {
            continue;
        }
        for start in 0..world_count {
            let targets = chain_targets(branch, chain, start, &snapshot);
            for end in targets {
                if !branch
                    .edges
                    .iter()
                    .any(|(f, r, t)| *f == start && role_subsumes(branch, sup, r) && *t == end)
                {
                    add_role_edge(branch, start, sup.clone(), end);
                    added = true;
                }
            }
        }
    }
    added
}

fn chain_targets(
    branch: &Branch<'_>,
    chain: &[RoleExpr],
    from: usize,
    edges: &[(usize, RoleExpr, usize)],
) -> Vec<usize> {
    let mut frontier = vec![from];
    for want in chain {
        let mut next = Vec::new();
        for w in frontier {
            for (f, r, t) in edges {
                if *f == w && role_subsumes(branch, want, r) {
                    next.push(*t);
                }
            }
        }
        if next.is_empty() {
            return vec![];
        }
        frontier = next;
    }
    frontier
}

fn expand_universal(branch: &mut Branch<'_>, world: usize, property: &RoleExpr, filler: CeId) {
    if is_top_object_property(branch, property) {
        let targets: Vec<usize> = branch.named_worlds.values().copied().collect();
        for target in targets {
            assert_label(branch, target, filler);
            if branch.clash {
                return;
            }
        }
        return;
    }
    let mut targets: Vec<usize> = branch
        .edges
        .iter()
        .filter(|(from, role, _)| *from == world && role_subsumes(branch, property, role))
        .map(|(_, _, to)| *to)
        .collect();
    let inv = inverse_role(property);
    for (from, role, to) in &branch.edges {
        if *to == world && role_subsumes(branch, &inv, role) {
            targets.push(*from);
        }
    }
    for target in targets {
        assert_label(branch, target, filler);
        if branch.clash {
            return;
        }
    }
}

pub(crate) fn inverse_role(role: &RoleExpr) -> RoleExpr {
    match role {
        RoleExpr::Atomic(id) => RoleExpr::Inverse(*id),
        RoleExpr::Inverse(id) => RoleExpr::Atomic(*id),
    }
}

pub(crate) fn apply_universal_on_edge(branch: &mut Branch<'_>, from: usize, to: usize) {
    let roles: Vec<RoleExpr> = branch
        .edges
        .iter()
        .filter(|(f, _, t)| *f == from && *t == to)
        .map(|(_, role, _)| role.clone())
        .collect();
    let mut universal: Vec<(RoleExpr, CeId)> = branch.worlds[from]
        .labels
        .iter()
        .filter_map(|&ce| match branch.dl.core().dl().ce(ce)? {
            ClassExpr::All { property, filler } => Some((property.clone(), *filler)),
            _ => None,
        })
        .collect();
    for (sub, property, filler) in &branch.universals {
        if world_satisfies_subject(branch, from, *sub) {
            universal.push((property.clone(), *filler));
        }
    }
    for role in roles {
        for (property, filler) in &universal {
            if role_subsumes(branch, property, &role) {
                assert_label(branch, to, *filler);
                if branch.clash {
                    return;
                }
            }
            if role_subsumes(branch, &inverse_role(property), &role) {
                assert_label(branch, from, *filler);
                if branch.clash {
                    return;
                }
            }
        }
    }
}

fn world_satisfies_subject(branch: &Branch<'_>, world: usize, sub: CeId) -> bool {
    if branch.worlds[world].labels.contains(&sub) {
        return true;
    }
    if clash::is_thing_ce(branch, sub) {
        return branch.worlds[world]
            .labels
            .iter()
            .any(|&label| clash::is_thing_ce(branch, label));
    }
    matches!(branch.dl.core().dl().ce(sub), Some(ClassExpr::Top))
}

fn apply_existential_clauses(
    branch: &mut Branch<'_>,
    world: usize,
    property: &RoleExpr,
    filler: CeId,
) {
    for (r, f, sup) in branch.existentials.clone() {
        if existential_clause_filler_matches(branch, f, filler)
            && role_subsumes(branch, &r, property)
        {
            assert_label(branch, world, sup);
            if branch.clash {
                return;
            }
        }
    }
}

fn existential_clause_filler_matches(
    branch: &Branch<'_>,
    clause_filler: CeId,
    expanded_filler: CeId,
) -> bool {
    if clause_filler == expanded_filler {
        return true;
    }
    matches!(
        branch.dl.core().dl().ce(clause_filler),
        Some(ClassExpr::Top)
    )
}

fn expand_disjunction(
    branch: &mut Branch<'_>,
    world: usize,
    ops: Vec<CeId>,
) -> Result<bool, crate::Error> {
    for alt in ops {
        let mut child = branch.clone();
        assert_label(&mut child, world, alt);
        match child.expand() {
            Ok(true) => {
                *branch = child;
                return Ok(true);
            }
            Ok(false) => {
                if child.cache.is_unsat(&child.worlds[world].labels) {
                    branch.cache.record_unsat(&child.worlds[world].labels);
                }
            }
            Err(crate::Error::ResourceLimit(limit)) => {
                return Err(crate::Error::ResourceLimit(limit));
            }
            Err(e) => return Err(e),
        }
    }
    Ok(false)
}

pub(crate) fn count_role_successors(
    branch: &Branch<'_>,
    world: usize,
    property: &RoleExpr,
    filler: Option<CeId>,
) -> usize {
    role_successor_worlds(branch, world, property, filler).len()
}

fn role_successor_worlds(
    branch: &Branch<'_>,
    world: usize,
    property: &RoleExpr,
    filler: Option<CeId>,
) -> Vec<usize> {
    let filler = effective_cardinality_filler(branch, filler);
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for (from, role, to) in &branch.edges {
        if *from == world
            && role_subsumes(branch, property, role)
            && filler.is_none_or(|f| branch.worlds[*to].labels.contains(&f))
            && seen.insert(*to)
        {
            out.push(*to);
        }
    }
    out
}

fn try_merge_role_successors(branch: &mut Branch<'_>, successors: &[usize]) -> bool {
    if successors.len() < 2 {
        return false;
    }
    let keep = successors[0];
    for &drop in &successors[1..] {
        branch.merge_worlds(keep, drop);
        if branch.clash {
            return false;
        }
    }
    true
}

/// Whether `sub_role` ⊑ `super_role` in the saturated role hierarchy.
pub(crate) fn role_subsumes(
    branch: &Branch<'_>,
    super_role: &RoleExpr,
    sub_role: &RoleExpr,
) -> bool {
    if role_equivalent(branch, super_role, sub_role) {
        return true;
    }
    if branch.role_chains.iter().any(|(chain, sup)| {
        chain.len() == 1
            && role_exprs_equal(&chain[0], sub_role)
            && (role_exprs_equal(sup, super_role) || role_subsumes(branch, super_role, sup))
    }) {
        return true;
    }
    match (super_role, sub_role) {
        (RoleExpr::Atomic(sup), RoleExpr::Atomic(sub)) => {
            if sup == sub {
                return true;
            }
            branch
                .role_hierarchy
                .get(sub)
                .is_some_and(|supers| supers.contains(sup))
        }
        (RoleExpr::Inverse(is), RoleExpr::Inverse(it)) => {
            role_subsumes(branch, &RoleExpr::Atomic(*it), &RoleExpr::Atomic(*is))
        }
        (RoleExpr::Inverse(is), RoleExpr::Atomic(sub)) => {
            role_subsumes(branch, &RoleExpr::Atomic(*sub), &RoleExpr::Atomic(*is))
        }
        (RoleExpr::Atomic(sup), RoleExpr::Inverse(it)) => {
            role_subsumes(branch, &RoleExpr::Inverse(*it), &RoleExpr::Atomic(*sup))
        }
    }
}

fn role_exprs_equal(left: &RoleExpr, right: &RoleExpr) -> bool {
    left == right
}

fn is_top_object_property(branch: &Branch<'_>, property: &RoleExpr) -> bool {
    const TOP: &str = "http://www.w3.org/2002/07/owl#topObjectProperty";
    let RoleExpr::Atomic(id) = property else {
        return false;
    };
    branch
        .dl
        .core()
        .entity(*id)
        .ok()
        .and_then(|record| branch.dl.core().resolve_iri(record.iri).ok())
        .is_some_and(|iri| iri == TOP)
}

fn role_equivalent(branch: &Branch<'_>, left: &RoleExpr, right: &RoleExpr) -> bool {
    if left == right {
        return true;
    }
    match (left, right) {
        (RoleExpr::Atomic(prop), RoleExpr::Inverse(inv))
        | (RoleExpr::Inverse(inv), RoleExpr::Atomic(prop)) => branch
            .role_inverses
            .get(prop)
            .is_some_and(|partner| *partner == *inv),
        _ => false,
    }
}
