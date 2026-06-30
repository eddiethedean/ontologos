//! Cardinality-aware subsumption derivation for classification.

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, DlAxiom, EntityId, Ontology, RoleExpr};

/// Derive atomic subsumptions from cardinality/universal patterns in equivalences.
pub fn derive_cardinality_subsumptions(ontology: &Ontology) -> Vec<(EntityId, EntityId)> {
    let disjoint = disjoint_pairs(ontology);
    let subclass_unions = subclass_union_map(ontology);
    let equivalences = equivalence_map(ontology);
    let role_hierarchy = role_subsumption_map(ontology);
    let mut out = Vec::new();

    for (&sub, def) in &equivalences {
        let ClassExpr::And(ops) = def else {
            continue;
        };
        let parts = decompose_and(ops, ontology);

        // Ian QNR: ≥n₁ r₁.C₁ ⊓ ≥n₂ r₂.C₂ with r₁,r₂ ⊑ s and disjoint Cᵢ  =>  ⊑ ≥(n₁+n₂) s
        if parts.min_qualified.len() >= 2 {
            let subroles: Vec<RoleExpr> = parts
                .min_qualified
                .iter()
                .map(|(_, p, _)| p.clone())
                .collect();
            for super_role in common_super_roles(&role_hierarchy, &subroles) {
                let entries: Vec<(u32, RoleExpr, CeId)> = parts
                    .min_qualified
                    .iter()
                    .filter(|(_, p, _)| role_subsumes(&role_hierarchy, &super_role, p))
                    .cloned()
                    .collect();
                if entries.len() < 2 {
                    continue;
                }
                let all_disjoint = entries.iter().all(|(_, _, f1)| {
                    entries.iter().all(|(_, _, f2)| {
                        f1 == f2 || fillers_disjoint(ontology, *f1, *f2, &disjoint)
                    })
                });
                if !all_disjoint {
                    continue;
                }
                let sum: u32 = entries.iter().map(|(n, _, _)| *n).sum();
                if let Some(sup) =
                    find_named_min_card(&equivalences, super_role.clone(), sum, None, ontology)
                {
                    push_sub(&mut out, sub, sup);
                }
            }
        }

        if let Some((n, prop)) = parts.min_unqualified.as_ref() {
            let some_entities: Vec<EntityId> = parts
                .some_restrictions
                .iter()
                .filter(|(p, _)| p == prop)
                .filter_map(|(_, filler)| atomic_entity(ontology, *filler))
                .collect();
            if some_entities.len() >= 2
                && some_entities.iter().any(|a| {
                    some_entities
                        .iter()
                        .any(|b| a != b && disjoint.contains(&(*a, *b)))
                })
            {
                if let Some(sup) =
                    find_named_min_card(&equivalences, prop.clone(), n + 1, None, ontology)
                {
                    push_sub(&mut out, sub, sup);
                }
            }
        }

        // complex2: MaxCard(n,r) ⊓ ∃r.C ⊓ ∃r.D with disjoint C,D  =>  ⊑ MaxCard(1,r,C) ⊓ MaxCard(1,r,D)
        if let Some((_, prop)) = parts.max_unqualified.as_ref() {
            let some_entities: Vec<EntityId> = parts
                .some_restrictions
                .iter()
                .filter(|(p, _)| p == prop)
                .filter_map(|(_, filler)| atomic_entity(ontology, *filler))
                .collect();
            if some_entities.len() >= 2 {
                if let (Some(c), Some(d)) = (some_entities.first(), some_entities.get(1)) {
                    if disjoint.contains(&(*c, *d)) || disjoint.contains(&(*d, *c)) {
                        if let Some(sup) =
                            find_named_max_pair(&equivalences, prop.clone(), *c, *d, ontology)
                        {
                            push_sub(&mut out, sub, sup);
                        }
                    }
                }
            }
        }

        // complex3: ∀r.A ⊓ MinCard(n,r) ⊓ MaxCard(m,r,C) with A ⊑ C ⊔ D  =>  ⊑ MinCard(n-m,r,D)
        if let (Some((min_n, prop)), Some((max_m, max_prop, max_filler))) =
            (parts.min_unqualified.as_ref(), parts.max_qualified.first())
        {
            if prop == max_prop && *min_n > *max_m {
                if let Some((_, all_filler)) = parts.all_restriction {
                    if let Some(all_entity) = atomic_entity(ontology, all_filler) {
                        for &(union_sub, union_a, union_b) in &subclass_unions {
                            if union_sub != all_entity {
                                continue;
                            }
                            for alt in [union_a, union_b] {
                                if alt == *max_filler {
                                    continue;
                                }
                                if let Some(alt_entity) = atomic_entity(ontology, alt) {
                                    if let Some(sup) = find_named_min_card(
                                        &equivalences,
                                        prop.clone(),
                                        min_n - max_m,
                                        Some(alt_entity),
                                        ontology,
                                    ) {
                                        push_sub(&mut out, sub, sup);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    for axiom in ontology.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(sub_e)) = ontology.dl().ce(*sub) else {
            continue;
        };
        let Some(ClassExpr::And(ops)) = ontology.dl().ce(*sup) else {
            continue;
        };
        let mut exact_by_prop: HashMap<RoleExpr, Vec<(u32, CeId)>> = HashMap::new();
        for &op in ops {
            let Some(ClassExpr::ExactCardinality {
                n,
                property,
                filler: Some(f),
            }) = ontology.dl().ce(op)
            else {
                continue;
            };
            exact_by_prop
                .entry(property.clone())
                .or_default()
                .push((*n, *f));
        }
        for (property, entries) in exact_by_prop {
            if entries.len() < 2 {
                continue;
            }
            let Some(base) = cardinality_filler_base(ontology, entries[0].1) else {
                continue;
            };
            if !entries
                .iter()
                .all(|(_, f)| cardinality_filler_base(ontology, *f) == Some(base))
            {
                continue;
            }
            let sum: u32 = entries.iter().map(|(n, _)| *n).sum();
            if let Some(sup_e) = find_named_exact_card(&equivalences, property, sum, base, ontology)
            {
                push_sub(&mut out, *sub_e, sup_e);
            }
        }
    }

    out
}

#[derive(Default)]
struct AndParts {
    min_unqualified: Option<(u32, RoleExpr)>,
    min_qualified: Vec<(u32, RoleExpr, CeId)>,
    max_unqualified: Option<(u32, RoleExpr)>,
    max_qualified: Vec<(u32, RoleExpr, CeId)>,
    some_restrictions: Vec<(RoleExpr, CeId)>,
    all_restriction: Option<(RoleExpr, CeId)>,
}

fn decompose_and(ops: &[CeId], ontology: &Ontology) -> AndParts {
    let mut parts = AndParts::default();
    let store = ontology.dl();
    for &op in ops {
        let Some(expr) = store.ce(op) else {
            continue;
        };
        match expr {
            ClassExpr::MinCardinality {
                n,
                property,
                filler: None,
            } => parts.min_unqualified = Some((*n, property.clone())),
            ClassExpr::MinCardinality {
                n,
                property,
                filler: Some(f),
            } => parts.min_qualified.push((*n, property.clone(), *f)),
            ClassExpr::MaxCardinality {
                n,
                property,
                filler: None,
            } => parts.max_unqualified = Some((*n, property.clone())),
            ClassExpr::MaxCardinality {
                n,
                property,
                filler: Some(f),
            } => parts.max_qualified.push((*n, property.clone(), *f)),
            ClassExpr::Some { property, filler } => {
                parts.some_restrictions.push((property.clone(), *filler));
            }
            ClassExpr::All { property, filler } => {
                parts.all_restriction = Some((property.clone(), *filler));
            }
            ClassExpr::And(inner) => merge_parts(&mut parts, decompose_and(inner, ontology)),
            _ => {}
        }
    }
    parts
}

fn merge_parts(target: &mut AndParts, nested: AndParts) {
    if target.min_unqualified.is_none() {
        target.min_unqualified = nested.min_unqualified;
    }
    target.min_qualified.extend(nested.min_qualified);
    if target.max_unqualified.is_none() {
        target.max_unqualified = nested.max_unqualified;
    }
    target.max_qualified.extend(nested.max_qualified);
    target.some_restrictions.extend(nested.some_restrictions);
    if target.all_restriction.is_none() {
        target.all_restriction = nested.all_restriction;
    }
}

fn equivalence_map(ontology: &Ontology) -> HashMap<EntityId, ClassExpr> {
    let store = ontology.dl();
    let mut out = HashMap::new();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        if ids.len() < 2 {
            continue;
        }
        let Some(ClassExpr::Atomic(entity)) = store.ce(ids[0]) else {
            continue;
        };
        let Some(def) = store.ce(ids[1]).cloned() else {
            continue;
        };
        out.insert(*entity, def);
    }
    out
}

fn disjoint_pairs(ontology: &Ontology) -> HashSet<(EntityId, EntityId)> {
    let store = ontology.dl();
    let mut out = HashSet::new();
    for axiom in store.axioms() {
        let DlAxiom::DisjointClasses(ids) = axiom else {
            continue;
        };
        for w in ids.windows(2) {
            if let (Some(ClassExpr::Atomic(a)), Some(ClassExpr::Atomic(b))) =
                (store.ce(w[0]), store.ce(w[1]))
            {
                out.insert((*a, *b));
                out.insert((*b, *a));
            }
        }
    }
    out
}

fn subclass_union_map(ontology: &Ontology) -> Vec<(EntityId, CeId, CeId)> {
    let store = ontology.dl();
    let mut out = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let (Some(ClassExpr::Atomic(sub_e)), Some(ClassExpr::Or(ops))) =
            (store.ce(*sub), store.ce(*sup))
        else {
            continue;
        };
        if ops.len() == 2 {
            out.push((*sub_e, ops[0], ops[1]));
        }
    }
    out
}

fn atomic_entity(ontology: &Ontology, ce: CeId) -> Option<EntityId> {
    match ontology.dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn find_named_min_card(
    equivalences: &HashMap<EntityId, ClassExpr>,
    property: RoleExpr,
    n: u32,
    filler: Option<EntityId>,
    ontology: &Ontology,
) -> Option<EntityId> {
    equivalences.iter().find_map(|(&entity, def)| {
        ce_matches_min_card(ontology, def, &property, n, filler).then_some(entity)
    })
}

fn ce_matches_min_card(
    ontology: &Ontology,
    ce: &ClassExpr,
    property: &RoleExpr,
    n: u32,
    filler: Option<EntityId>,
) -> bool {
    match ce {
        ClassExpr::MinCardinality {
            n: card_n,
            property: prop,
            filler: f,
        } if *card_n == n && prop == property => match (filler, f) {
            (None, None) => true,
            (Some(e), Some(f_ce)) => atomic_entity(ontology, *f_ce) == Some(e),
            _ => false,
        },
        ClassExpr::And(ops) => ops.iter().any(|&id| {
            ontology
                .dl()
                .ce(id)
                .is_some_and(|e| ce_matches_min_card(ontology, e, property, n, filler))
        }),
        _ => false,
    }
}

fn find_named_max_pair(
    equivalences: &HashMap<EntityId, ClassExpr>,
    property: RoleExpr,
    c: EntityId,
    d: EntityId,
    ontology: &Ontology,
) -> Option<EntityId> {
    equivalences.iter().find_map(|(&entity, def)| {
        let ClassExpr::And(ops) = def else {
            return None;
        };
        let mut has_c = false;
        let mut has_d = false;
        for &op in ops {
            let Some(ClassExpr::MaxCardinality {
                n: 1,
                property: prop,
                filler: Some(f),
            }) = ontology.dl().ce(op)
            else {
                continue;
            };
            if *prop != property {
                continue;
            }
            match atomic_entity(ontology, *f) {
                Some(e) if e == c => has_c = true,
                Some(e) if e == d => has_d = true,
                _ => {}
            }
        }
        (has_c && has_d).then_some(entity)
    })
}

fn push_sub(out: &mut Vec<(EntityId, EntityId)>, sub: EntityId, sup: EntityId) {
    if sub != sup && !out.iter().any(|&(a, b)| a == sub && b == sup) {
        out.push((sub, sup));
    }
}

fn role_subsumption_map(ontology: &Ontology) -> HashMap<EntityId, HashSet<EntityId>> {
    let mut hierarchy: HashMap<EntityId, HashSet<EntityId>> = HashMap::new();
    for axiom in ontology.dl().axioms() {
        let DlAxiom::SubObjectPropertyOf { sub, sup } = axiom else {
            continue;
        };
        let (RoleExpr::Atomic(sub_id), RoleExpr::Atomic(sup_id)) = (sub, sup) else {
            continue;
        };
        hierarchy.entry(*sub_id).or_default().insert(*sup_id);
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::SubObjectPropertyOf {
            sub_property,
            super_property,
        } = axiom
        {
            hierarchy
                .entry(*sub_property)
                .or_default()
                .insert(*super_property);
        }
    }
    let mut changed = true;
    while changed {
        changed = false;
        let pairs: Vec<(EntityId, EntityId)> = hierarchy
            .iter()
            .flat_map(|(&a, ss)| ss.iter().map(move |&b| (a, b)))
            .collect();
        for (a, b) in pairs {
            if let Some(bb) = hierarchy.get(&b).cloned() {
                for c in bb {
                    if hierarchy.entry(a).or_default().insert(c) {
                        changed = true;
                    }
                }
            }
        }
    }
    hierarchy
}

fn role_subsumes(
    hierarchy: &HashMap<EntityId, HashSet<EntityId>>,
    super_role: &RoleExpr,
    sub_role: &RoleExpr,
) -> bool {
    match (super_role, sub_role) {
        (RoleExpr::Atomic(sup), RoleExpr::Atomic(sub)) => {
            if sup == sub {
                return true;
            }
            hierarchy
                .get(sub)
                .is_some_and(|supers| supers.contains(sup))
        }
        _ => super_role == sub_role,
    }
}

fn common_super_roles(
    hierarchy: &HashMap<EntityId, HashSet<EntityId>>,
    subroles: &[RoleExpr],
) -> Vec<RoleExpr> {
    let atomic_subs: Vec<EntityId> = subroles
        .iter()
        .filter_map(|r| match r {
            RoleExpr::Atomic(id) => Some(*id),
            _ => None,
        })
        .collect();
    if atomic_subs.len() < 2 {
        return Vec::new();
    }
    let mut candidates: HashSet<EntityId> =
        hierarchy.get(&atomic_subs[0]).cloned().unwrap_or_default();
    candidates.insert(atomic_subs[0]);
    for sub in &atomic_subs[1..] {
        let mut reachable: HashSet<EntityId> = hierarchy.get(sub).cloned().unwrap_or_default();
        reachable.insert(*sub);
        candidates = candidates.intersection(&reachable).copied().collect();
    }
    candidates.into_iter().map(RoleExpr::Atomic).collect()
}

fn fillers_disjoint(
    ontology: &Ontology,
    left: CeId,
    right: CeId,
    disjoint: &HashSet<(EntityId, EntityId)>,
) -> bool {
    if left == right {
        return false;
    }
    if let (Some(a), Some(b)) = (
        atomic_entity(ontology, left),
        atomic_entity(ontology, right),
    ) {
        if disjoint.contains(&(a, b)) {
            return true;
        }
    }
    if let Some(ClassExpr::Not(inner)) = ontology.dl().ce(left) {
        if *inner == right {
            return true;
        }
        if atomic_entity(ontology, *inner) == atomic_entity(ontology, right) {
            return true;
        }
    }
    if let Some(ClassExpr::Not(inner)) = ontology.dl().ce(right) {
        if *inner == left {
            return true;
        }
        if atomic_entity(ontology, *inner) == atomic_entity(ontology, left) {
            return true;
        }
    }
    false
}

fn cardinality_filler_base(ontology: &Ontology, filler: CeId) -> Option<EntityId> {
    match ontology.dl().ce(filler)? {
        ClassExpr::Atomic(id) => Some(*id),
        ClassExpr::And(ops) => ops.iter().find_map(|&op| atomic_entity(ontology, op)),
        _ => None,
    }
}

fn find_named_exact_card(
    equivalences: &HashMap<EntityId, ClassExpr>,
    property: RoleExpr,
    n: u32,
    filler: EntityId,
    ontology: &Ontology,
) -> Option<EntityId> {
    equivalences.iter().find_map(|(&entity, def)| {
        ce_contains_exact_card(ontology, def, &property, n, filler).then_some(entity)
    })
}

fn ce_contains_exact_card(
    ontology: &Ontology,
    ce: &ClassExpr,
    property: &RoleExpr,
    n: u32,
    filler: EntityId,
) -> bool {
    match ce {
        ClassExpr::ExactCardinality {
            n: card_n,
            property: prop,
            filler: Some(f),
        } if *card_n == n && prop == property && atomic_entity(ontology, *f) == Some(filler) => {
            true
        }
        ClassExpr::And(ops) => ops.iter().any(|&id| {
            ontology
                .dl()
                .ce(id)
                .is_some_and(|e| ce_contains_exact_card(ontology, e, property, n, filler))
        }),
        _ => false,
    }
}

#[cfg(test)]
mod flower_tests {
    use super::*;
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    #[test]
    fn classification_subclass_bug_derives_c2_sub_c4() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testclassificationsubclassbug.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        let c2 = ont.lookup_entity("file:/c/test.owl#c2").expect("c2");
        let c4 = ont.lookup_entity("file:/c/test.owl#c4").expect("c4");
        let derived = derive_cardinality_subsumptions(&ont);
        assert!(
            derived.iter().any(|&(s, t)| s == c2 && t == c4),
            "expected c2 ⊑ c4 in {derived:?}"
        );
    }

    #[test]
    fn ian_qnr_derives_a_sub_b() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../benchmarks/data/hermit/axioms/hermit_reasoner_reasonertest_testianqnrtest.ofn",
        );
        let ont = load_ontology(&path).expect("load");
        let a = ont.lookup_entity("file:/c/test.owl#A").expect("A");
        let b = ont.lookup_entity("file:/c/test.owl#B").expect("B");
        let eq = equivalence_map(&ont);
        let def = eq.get(&a).expect("A equivalence");
        let ClassExpr::And(ops) = def else {
            panic!("A def not And: {def:?}");
        };
        let parts = decompose_and(ops, &ont);
        assert_eq!(parts.min_qualified.len(), 2);
        let derived = derive_cardinality_subsumptions(&ont);
        assert!(
            derived.iter().any(|&(s, t)| s == a && t == b),
            "expected A ⊑ B in {derived:?}"
        );
    }
}
