//! ABox class-assertion hyper clausification (HasSelf subset).

use std::collections::HashMap;

use ontologos_core::{CeId, ClassExpr, DlAxiom, EntityId, Ontology, RoleExpr};

use crate::hyperclause::{HyperAtom, HyperClause, Term, abbrev_entity, abbrev_role};

pub(crate) fn clausify_abox_class_assertions(
    ontology: &Ontology,
    def_index: &mut u32,
    push_clause: &mut dyn FnMut(HyperClause),
    push_fact: &mut dyn FnMut(HyperAtom),
) {
    let mut by_individual: HashMap<EntityId, Vec<CeId>> = HashMap::new();
    for axiom in ontology.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        by_individual.entry(*individual).or_default().push(*class);
    }

    for (individual, classes) in by_individual {
        for ce in &classes {
            if let Some(ClassExpr::Atomic(id)) = ontology.dl().ce(*ce) {
                push_fact(HyperAtom::Concept {
                    name: abbrev_entity(ontology, *id),
                    term: Term::Ind(abbrev_entity(ontology, individual)),
                });
            }
        }
        let _ = try_has_self_all_bottom(
            ontology,
            individual,
            &classes,
            def_index,
            push_clause,
            push_fact,
        );
        let _ = try_has_self_all_has_self_filler(
            ontology,
            individual,
            &classes,
            def_index,
            push_clause,
            push_fact,
        );
    }
}

fn try_has_self_all_bottom(
    ontology: &Ontology,
    individual: EntityId,
    classes: &[CeId],
    def_index: &mut u32,
    push_clause: &mut dyn FnMut(HyperClause),
    push_fact: &mut dyn FnMut(HyperAtom),
) -> bool {
    let mut has_self = None;
    let mut all_bottom = false;
    for ce in classes {
        let Some(expr) = ontology.dl().ce(*ce) else {
            return false;
        };
        match expr {
            ClassExpr::HasSelf(prop) => has_self = Some(*prop),
            ClassExpr::All {
                property: RoleExpr::Atomic(prop),
                filler,
            } if matches!(ontology.dl().ce(*filler), Some(ClassExpr::Bottom)) => {
                all_bottom = true;
                has_self.get_or_insert(*prop);
            }
            _ => {}
        }
    }
    let Some(prop) = has_self else {
        return false;
    };
    if !all_bottom {
        return false;
    }

    let def0 = fresh_def(def_index);
    let def1 = fresh_def(def_index);
    let ind = abbrev_entity(ontology, individual);
    let role = abbrev_role(ontology, prop);

    push_fact(HyperAtom::Concept {
        name: def0.clone(),
        term: Term::Ind(ind.clone()),
    });
    push_fact(HyperAtom::NotConcept {
        name: def1.clone(),
        term: Term::Ind(ind),
    });
    push_clause(HyperClause {
        head: vec![HyperAtom::Concept {
            name: def1.clone(),
            term: Term::Var("X".into()),
        }],
        body: vec![HyperAtom::Role {
            role: role.clone(),
            subject: Term::Var("X".into()),
            object: Term::Var("Y".into()),
        }],
    });
    push_clause(HyperClause {
        head: vec![HyperAtom::Role {
            role,
            subject: Term::Var("X".into()),
            object: Term::Var("X".into()),
        }],
        body: vec![HyperAtom::Concept {
            name: def0,
            term: Term::Var("X".into()),
        }],
    });
    true
}

fn try_has_self_all_has_self_filler(
    ontology: &Ontology,
    individual: EntityId,
    classes: &[CeId],
    def_index: &mut u32,
    push_clause: &mut dyn FnMut(HyperClause),
    push_fact: &mut dyn FnMut(HyperAtom),
) -> bool {
    let mut matched = None;
    for ce in classes {
        let Some(ClassExpr::All {
            property: RoleExpr::Atomic(prop),
            filler,
        }) = ontology.dl().ce(*ce).cloned()
        else {
            continue;
        };
        if matches!(
            ontology.dl().ce(filler),
            Some(ClassExpr::HasSelf(p)) if *p == prop
        ) {
            matched = Some(prop);
            break;
        }
    }
    let Some(prop) = matched else {
        return false;
    };

    let def0 = fresh_def(def_index);
    let def1 = fresh_def(def_index);
    let ind = abbrev_entity(ontology, individual);
    let role = abbrev_role(ontology, prop);

    push_fact(HyperAtom::Concept {
        name: def0.clone(),
        term: Term::Ind(ind),
    });
    push_clause(HyperClause {
        head: vec![HyperAtom::Concept {
            name: def1.clone(),
            term: Term::Var("Y".into()),
        }],
        body: vec![
            HyperAtom::Role {
                role: role.clone(),
                subject: Term::Var("X".into()),
                object: Term::Var("Y".into()),
            },
            HyperAtom::Concept {
                name: def0.clone(),
                term: Term::Var("X".into()),
            },
        ],
    });
    push_clause(HyperClause {
        head: vec![HyperAtom::Role {
            role,
            subject: Term::Var("X".into()),
            object: Term::Var("X".into()),
        }],
        body: vec![HyperAtom::Concept {
            name: def1,
            term: Term::Var("X".into()),
        }],
    });
    true
}

fn fresh_def(def_index: &mut u32) -> String {
    let name = format!("def:{}", *def_index);
    *def_index += 1;
    name
}
