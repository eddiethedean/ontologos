//! Coupled EL-style saturation feeding the tableau.

use std::collections::HashSet;

use ontologos_core::{CeId, ClassExpr, EntityId, Ontology, RoleExpr};

use ontologos_alc::{Clause, ClauseSet, DlOntology};

use crate::ria::RoleHierarchy;
use crate::Error;

/// Facts produced by saturation pass.
#[derive(Debug, Default, Clone)]
pub struct SaturatedFacts {
    /// Additional subsumptions.
    pub subsumptions: Vec<(EntityId, EntityId)>,
    /// Existential subsumptions `∃r.C ⊑ D`.
    pub existentials: Vec<(RoleExpr, CeId, CeId)>,
    /// Saturated role subsumptions.
    pub role_subsumptions: Vec<(EntityId, EntityId)>,
}

/// Run lightweight saturation on existential/subsumption clauses.
pub fn saturate(
    ontology: &Ontology,
    clauses: &ClauseSet,
    roles: &RoleHierarchy,
) -> Result<SaturatedFacts, Error> {
    let _dl = DlOntology::from_ontology(ontology)?;
    let mut facts = SaturatedFacts::default();
    let mut worklist: Vec<(CeId, CeId)> = Vec::new();
    let mut all_pairs: Vec<(CeId, CeId)> = Vec::new();
    let mut seen_pairs: HashSet<(CeId, CeId)> = HashSet::new();

    for clause in clauses.clauses() {
        match clause {
            Clause::Subsumption { sub, sup } => {
                worklist.push((*sub, *sup));
                all_pairs.push((*sub, *sup));
            }
            Clause::Existential {
                property,
                filler,
                sup,
            } => facts.existentials.push((property.clone(), *filler, *sup)),
            Clause::RoleSubsumption { sub, sup } => {
                push_role_subsumption(&mut facts.role_subsumptions, *sub, *sup);
            }
            _ => {}
        }
    }

    for (chain, sup) in roles.chains() {
        if let [RoleExpr::Atomic(r)] = chain.as_slice() {
            if let RoleExpr::Atomic(s) = sup {
                push_role_subsumption(&mut facts.role_subsumptions, *r, *s);
            }
        }
    }

    while let Some((sub, sup)) = worklist.pop() {
        if !seen_pairs.insert((sub, sup)) {
            continue;
        }
        if let (Some(a), Some(b)) = (as_atomic(ontology, sub), as_atomic(ontology, sup)) {
            push_subsumption(&mut facts.subsumptions, a, b);
        }
        propagate_intersection(ontology, sub, sup, &mut worklist, roles);
        propagate_existential_role(ontology, sub, sup, &mut worklist, &facts.role_subsumptions);
        propagate_through(ontology, sub, sup, &mut worklist, &all_pairs);
    }

    Ok(facts)
}

fn propagate_through(
    _ontology: &Ontology,
    sub: CeId,
    sup: CeId,
    worklist: &mut Vec<(CeId, CeId)>,
    all_pairs: &[(CeId, CeId)],
) {
    for &(mid, top) in all_pairs {
        if mid == sup && mid != top && sub != top {
            worklist.push((sub, top));
        }
    }
}

fn propagate_existential_role(
    ontology: &Ontology,
    sub: CeId,
    sup: CeId,
    worklist: &mut Vec<(CeId, CeId)>,
    role_subs: &[(EntityId, EntityId)],
) {
    let store = ontology.dl();
    let Some(ClassExpr::Some {
        property: RoleExpr::Atomic(r),
        filler,
    }) = store.ce(sup).cloned()
    else {
        return;
    };
    for &(r_sub, r_sup) in role_subs {
        if r != r_sub {
            continue;
        }
        if let Some(exists_sup) = find_some_ce(ontology, RoleExpr::Atomic(r_sup), filler) {
            worklist.push((sub, exists_sup));
        }
    }
}

fn find_some_ce(ontology: &Ontology, property: RoleExpr, filler: CeId) -> Option<CeId> {
    ontology
        .dl()
        .expressions()
        .find_map(|(id, expr)| match expr {
            ClassExpr::Some {
                property: p,
                filler: f,
            } if *p == property && *f == filler => Some(id),
            _ => None,
        })
}

fn push_subsumption(out: &mut Vec<(EntityId, EntityId)>, sub: EntityId, sup: EntityId) {
    if out.iter().all(|&(x, y)| x != sub || y != sup) {
        out.push((sub, sup));
    }
}

fn push_role_subsumption(out: &mut Vec<(EntityId, EntityId)>, sub: EntityId, sup: EntityId) {
    if sub != sup && out.iter().all(|&(x, y)| x != sub || y != sup) {
        out.push((sub, sup));
    }
}

fn as_atomic(ontology: &Ontology, ce: CeId) -> Option<EntityId> {
    match ontology.dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn propagate_intersection(
    ontology: &Ontology,
    sub: CeId,
    sup: CeId,
    worklist: &mut Vec<(CeId, CeId)>,
    roles: &RoleHierarchy,
) {
    let store = ontology.dl();
    if let Some(ClassExpr::And(ops)) = store.ce(sup) {
        for op in ops {
            worklist.push((sub, *op));
        }
    }
    if let Some(ClassExpr::Some { property, filler }) = store.ce(sub) {
        if let Some(ClassExpr::All {
            property: p2,
            filler: f2,
        }) = store.ce(sup)
        {
            if properties_related(property, p2, roles) {
                worklist.push((*filler, *f2));
            }
        }
    }
    if let Some(ClassExpr::Some {
        property: RoleExpr::Atomic(r),
        filler,
    }) = store.ce(sub).cloned()
    {
        if let Some(ClassExpr::Some {
            property: RoleExpr::Atomic(r2),
            filler: f2,
        }) = store.ce(sup).cloned()
        {
            if filler == f2 && roles.is_subrole(r, r2) {
                worklist.push((sub, sup));
            }
        }
    }
}

fn properties_related(a: &RoleExpr, b: &RoleExpr, roles: &RoleHierarchy) -> bool {
    match (a, b) {
        (RoleExpr::Atomic(sa), RoleExpr::Atomic(sb)) => {
            if sa == sb {
                return true;
            }
            roles.is_subrole(*sa, *sb) || roles.is_subrole(*sb, *sa)
        }
        (RoleExpr::Inverse(ia), RoleExpr::Inverse(ib)) => ia == ib,
        _ => a == b,
    }
}
