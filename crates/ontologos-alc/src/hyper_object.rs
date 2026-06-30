//! Object-property hyper clausification (HermiT normalization + transitivity encoding subset).

use std::collections::HashSet;

use ontologos_core::{Axiom, CeId, ClassExpr, DlAxiom, EntityId, Ontology, RoleExpr};

use crate::hyperclause::{
    abbrev_role, concept_name, entity_canonical_iri, HyperAtom, HyperClause, Term,
};

const VAR_X: &str = "X";

pub(crate) struct ObjectHyperContext {
    pub transitive: HashSet<EntityId>,
    processed_signatures: HashSet<(String, String, String, String)>,
    processed_ce_pairs: HashSet<(CeId, CeId)>,
}

impl ObjectHyperContext {
    pub(crate) fn empty() -> Self {
        Self {
            transitive: HashSet::new(),
            processed_signatures: HashSet::new(),
            processed_ce_pairs: HashSet::new(),
        }
    }

    pub(crate) fn new(ontology: &Ontology) -> Self {
        Self {
            transitive: Self::collect_transitive(ontology),
            processed_signatures: HashSet::new(),
            processed_ce_pairs: HashSet::new(),
        }
    }

    pub(crate) fn handled_subclass(&self, sub: CeId, sup: CeId) -> bool {
        self.processed_ce_pairs.contains(&(sub, sup))
    }

    fn collect_transitive(ontology: &Ontology) -> HashSet<EntityId> {
        let mut out = HashSet::new();
        for (_, axiom) in ontology.axioms().iter() {
            if let Axiom::TransitiveObjectProperty(p) = axiom {
                out.insert(*p);
            }
        }
        for axiom in ontology.dl().axioms() {
            if let DlAxiom::TransitiveObjectProperty(RoleExpr::Atomic(p)) = axiom {
                out.insert(*p);
            }
        }
        out
    }
}

pub(crate) fn clausify_object_subclass_axioms(
    ontology: &Ontology,
    ctx: &mut ObjectHyperContext,
    def_index: &mut u32,
    push: &mut dyn FnMut(HyperClause),
) {
    let dl_axioms: Vec<DlAxiom> = ontology.dl().axioms().cloned().collect();
    for axiom in dl_axioms {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        if try_object_subclass(ontology, ctx, sub, sup, def_index, push) {
            continue;
        }
        let _ = try_simple_some_superclass(ontology, ctx, sub, sup, push);
    }
    for (_, axiom) in ontology.axioms().iter() {
        let Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } = axiom
        else {
            continue;
        };
        let _ = try_simple_some_existential(ontology, *subclass, *property, *filler, push);
    }
}

fn try_simple_some_superclass(
    ontology: &Ontology,
    ctx: &mut ObjectHyperContext,
    sub: CeId,
    sup: CeId,
    push: &mut dyn FnMut(HyperClause),
) -> bool {
    if ctx.processed_ce_pairs.contains(&(sub, sup)) {
        return true;
    }
    let Some(sub_name) = concept_name(ontology, sub) else {
        return false;
    };
    let Some(ClassExpr::Some {
        property: RoleExpr::Atomic(prop),
        filler,
    }) = ontology.dl().ce(sup).cloned()
    else {
        return false;
    };
    let Some(filler_name) = concept_name(ontology, filler) else {
        return false;
    };
    let role = abbrev_role(ontology, prop);
    push(HyperClause {
        head: vec![HyperAtom::AtLeastObject {
            n: 1,
            role,
            concept: filler_name,
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![HyperAtom::Concept {
            name: sub_name,
            term: Term::Var(VAR_X.into()),
        }],
    });
    ctx.processed_ce_pairs.insert((sub, sup));
    true
}

fn try_simple_some_existential(
    ontology: &Ontology,
    subclass: EntityId,
    property: EntityId,
    filler: EntityId,
    push: &mut dyn FnMut(HyperClause),
) -> bool {
    let sub_name = abbrev_entity(ontology, subclass);
    let filler_name = abbrev_entity(ontology, filler);
    let role = abbrev_role(ontology, property);
    push(HyperClause {
        head: vec![HyperAtom::AtLeastObject {
            n: 1,
            role,
            concept: filler_name,
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![HyperAtom::Concept {
            name: sub_name,
            term: Term::Var(VAR_X.into()),
        }],
    });
    true
}

fn abbrev_entity(ontology: &Ontology, id: EntityId) -> String {
    crate::hyperclause::abbrev_entity(ontology, id)
}

fn try_object_subclass(
    ontology: &Ontology,
    ctx: &mut ObjectHyperContext,
    sub: CeId,
    sup: CeId,
    def_index: &mut u32,
    push: &mut dyn FnMut(HyperClause),
) -> bool {
    let Some(sub_expr) = ontology.dl().ce(sub).cloned() else {
        return false;
    };
    let ClassExpr::Some {
        property: RoleExpr::Atomic(prop),
        filler,
    } = sub_expr
    else {
        return false;
    };

    let (inner_prop, inner_filler) = match ontology.dl().ce(filler).cloned() {
        Some(ClassExpr::Some {
            property: RoleExpr::Atomic(ip),
            filler: ifiller,
        }) => (ip, ifiller),
        _ => return false,
    };
    let Some(sup_entity) = entity_from_ce(ontology, sup) else {
        return false;
    };
    let Some(filler_entity) = entity_from_ce(ontology, inner_filler) else {
        return false;
    };
    if !ctx.processed_signatures.insert((
        entity_canonical_iri(ontology, prop),
        entity_canonical_iri(ontology, inner_prop),
        entity_canonical_iri(ontology, filler_entity),
        entity_canonical_iri(ontology, sup_entity),
    )) {
        ctx.processed_ce_pairs.insert((sub, sup));
        return true;
    }
    ctx.processed_ce_pairs.insert((sub, sup));

    let def_name = {
        let def = fresh_def(def_index);
        if ctx.transitive.contains(&inner_prop) {
            emit_transitive_existential(ontology, inner_prop, inner_filler, &def, 0, push);
        } else {
            emit_existential_definition(ontology, inner_prop, inner_filler, &def, push);
        }
        def
    };

    let sup_name = concept_name(ontology, sup).expect("atomic sup");
    let role = abbrev_role(ontology, prop);
    push(HyperClause {
        head: vec![HyperAtom::Concept {
            name: sup_name,
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![
            HyperAtom::Role {
                role,
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            },
            HyperAtom::Concept {
                name: def_name,
                term: Term::Var("Y".into()),
            },
        ],
    });
    true
}

fn fresh_def(def_index: &mut u32) -> String {
    let name = format!("def:{}", *def_index);
    *def_index += 1;
    name
}

fn emit_transitive_existential(
    ontology: &Ontology,
    property: EntityId,
    filler: CeId,
    def_name: &str,
    automaton_id: u32,
    push: &mut dyn FnMut(HyperClause),
) {
    let role = abbrev_role(ontology, property);
    let filler_name = concept_name(ontology, filler).expect("atomic filler");
    let pre_final = format!("all:{automaton_id}_1");
    let terminal = format!("all:{automaton_id}_0");

    push(HyperClause {
        head: vec![HyperAtom::Concept {
            name: def_name.to_string(),
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![HyperAtom::Concept {
            name: pre_final.clone(),
            term: Term::Var(VAR_X.into()),
        }],
    });
    push(HyperClause {
        head: vec![HyperAtom::Concept {
            name: pre_final,
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![
            HyperAtom::Role {
                role: role.clone(),
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            },
            HyperAtom::Concept {
                name: terminal.clone(),
                term: Term::Var("Y".into()),
            },
        ],
    });
    push(HyperClause {
        head: vec![HyperAtom::Concept {
            name: terminal.clone(),
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![
            HyperAtom::Role {
                role,
                subject: Term::Var(VAR_X.into()),
                object: Term::Var("Y".into()),
            },
            HyperAtom::Concept {
                name: terminal.clone(),
                term: Term::Var("Y".into()),
            },
        ],
    });
    push(HyperClause {
        head: vec![HyperAtom::Concept {
            name: terminal,
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![HyperAtom::Concept {
            name: filler_name,
            term: Term::Var(VAR_X.into()),
        }],
    });
}

fn entity_from_ce(ontology: &Ontology, ce: CeId) -> Option<EntityId> {
    match ontology.dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn emit_existential_definition(
    ontology: &Ontology,
    property: EntityId,
    filler: CeId,
    def_name: &str,
    push: &mut dyn FnMut(HyperClause),
) {
    let role = abbrev_role(ontology, property);
    let filler_name = concept_name(ontology, filler).expect("atomic filler");
    push(HyperClause {
        head: vec![HyperAtom::AtLeastObject {
            n: 1,
            role,
            concept: filler_name,
            term: Term::Var(VAR_X.into()),
        }],
        body: vec![HyperAtom::Concept {
            name: def_name.to_string(),
            term: Term::Var(VAR_X.into()),
        }],
    });
}
