//! Tableau expansion rules (∧, ∨, ∃, ∀, ¬).

use ontologos_core::{CeId, ClassExpr, RoleExpr};

use super::block;
use super::clash::{assert_label, assert_negation};
use super::Branch;

/// Process one queued class expression in `world`.
pub fn process(branch: &mut Branch<'_>, world: usize, ce: CeId) {
    if branch.clash || block::is_blocked(branch, world) {
        return;
    }
    let Some(expr) = branch.dl.core().dl().ce(ce).cloned() else {
        return;
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
                    return;
                }
            }
        }
        ClassExpr::Or(ops) => {
            if !expand_disjunction(branch, world, ops) {
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
                    assert_label(branch, world, id);
                }
            }
        }
        ClassExpr::HasValue { individual, .. } => {
            let _ = individual;
        }
        ClassExpr::HasSelf(_) => {}
        ClassExpr::MinCardinality { n: 0, .. } => {}
        ClassExpr::MinCardinality { n, filler, .. } => {
            if let Some(f) = filler {
                for _ in 0..n {
                    assert_label(branch, world, f);
                }
            }
        }
        ClassExpr::MaxCardinality { n: 0, .. } => branch.clash = true,
        ClassExpr::MaxCardinality { .. } => {}
        ClassExpr::ExactCardinality { n, filler, .. } => {
            if let Some(f) = filler {
                for _ in 0..n {
                    assert_label(branch, world, f);
                }
            }
        }
    }
}

fn expand_existential(branch: &mut Branch<'_>, world: usize, property: RoleExpr, filler: CeId) {
    let new_world = branch.worlds.len();
    branch.worlds.push(super::World::default());
    branch.edges.push((world, property, new_world));
    assert_label(branch, new_world, filler);
    apply_universal_on_edge(branch, world, new_world);
}

fn expand_universal(branch: &mut Branch<'_>, world: usize, property: &RoleExpr, filler: CeId) {
    let targets: Vec<usize> = branch
        .edges
        .iter()
        .filter(|(from, role, _)| *from == world && roles_related(branch, role, property))
        .map(|(_, _, to)| *to)
        .collect();
    for target in targets {
        assert_label(branch, target, filler);
        if branch.clash {
            return;
        }
    }
}

fn apply_universal_on_edge(branch: &mut Branch<'_>, from: usize, to: usize) {
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
            if roles_related(branch, &role, property) {
                assert_label(branch, to, *filler);
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
    matches!(branch.dl.core().dl().ce(sub), Some(ClassExpr::Top))
}

fn apply_existential_clauses(
    branch: &mut Branch<'_>,
    world: usize,
    property: &RoleExpr,
    filler: CeId,
) {
    for (r, f, sup) in branch.existentials.clone() {
        if filler == f && roles_related(branch, &r, property) {
            assert_label(branch, world, sup);
            if branch.clash {
                return;
            }
        }
    }
}

fn expand_disjunction(branch: &mut Branch<'_>, world: usize, ops: Vec<CeId>) -> bool {
    for alt in ops {
        let mut child = branch.clone();
        assert_label(&mut child, world, alt);
        if child.expand() {
            *branch = child;
            return true;
        }
        if child.cache.is_unsat(&child.worlds[world].labels) {
            branch.cache.record_unsat(&child.worlds[world].labels);
        }
    }
    false
}

pub(crate) fn roles_related(branch: &Branch<'_>, a: &RoleExpr, b: &RoleExpr) -> bool {
    match (a, b) {
        (RoleExpr::Atomic(sa), RoleExpr::Atomic(sb)) => {
            if sa == sb {
                return true;
            }
            branch
                .role_hierarchy
                .get(sa)
                .is_some_and(|supers| supers.contains(sb))
        }
        (RoleExpr::Inverse(ia), RoleExpr::Inverse(ib)) => ia == ib,
        _ => a == b,
    }
}
