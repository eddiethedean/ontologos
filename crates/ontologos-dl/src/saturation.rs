//! Coupled EL-style saturation feeding the tableau.

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

    for clause in clauses.clauses() {
        match clause {
            Clause::Subsumption { sub, sup } => worklist.push((*sub, *sup)),
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

    while let Some((sub, sup)) = worklist.pop() {
        if let (Some(a), Some(b)) = (as_atomic(ontology, sub), as_atomic(ontology, sup)) {
            push_subsumption(&mut facts.subsumptions, a, b);
        }
        propagate_intersection(ontology, sub, sup, &mut worklist, roles);
    }

    for (chain, sup) in roles.chains() {
        if let [RoleExpr::Atomic(r)] = chain.as_slice() {
            push_role_subsumption(&mut facts.role_subsumptions, *r, *sup);
        }
    }

    Ok(facts)
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
