//! Coupled EL-style saturation feeding the tableau.

use ontologos_core::{CeId, ClassExpr, EntityId, Ontology, RoleExpr};

use ontologos_alc::{Clause, ClauseSet, DlOntology};
use crate::Error;

/// Facts produced by saturation pass.
#[derive(Debug, Default)]
pub struct SaturatedFacts {
    /// Additional subsumptions.
    pub subsumptions: Vec<(EntityId, EntityId)>,
    /// Existential subsumptions `∃r.C ⊑ D`.
    pub existentials: Vec<(RoleExpr, CeId, CeId)>,
}

/// Run lightweight saturation on existential/subsumption clauses.
pub fn saturate(ontology: &Ontology, clauses: &ClauseSet) -> Result<SaturatedFacts, Error> {
    let _dl = DlOntology::from_ontology(ontology)?;
    let mut facts = SaturatedFacts::default();
    let mut worklist: Vec<(CeId, CeId)> = Vec::new();

    for clause in clauses.clauses() {
        if let Clause::Subsumption { sub, sup } = clause {
            worklist.push((*sub, *sup));
        }
    }

    while let Some((sub, sup)) = worklist.pop() {
        if let (Some(a), Some(b)) = (
            as_atomic(ontology, sub),
            as_atomic(ontology, sup),
        ) {
            if facts.subsumptions.iter().all(|&(x, y)| x != a || y != b) {
                facts.subsumptions.push((a, b));
            }
        }
        propagate_intersection(ontology, sub, sup, &mut worklist);
    }

    Ok(facts)
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
            if properties_related(property, p2) {
                worklist.push((*filler, *f2));
            }
        }
    }
}

fn properties_related(a: &RoleExpr, b: &RoleExpr) -> bool {
    a == b
}
