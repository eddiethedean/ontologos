//! Nominal / OneOf hyper clausification (HermiT structural subset).

use std::collections::{HashMap, HashSet};

use ontologos_core::{CeId, ClassExpr, DlAxiom, EntityId, Ontology, RoleExpr};

use crate::hyperclause::{
    abbrev_entity, concept_name, entity_canonical_iri, role_atom, HyperAtom, HyperClause, Term,
};

const VAR_X: &str = "X";

pub(crate) struct NominalHyperContext {
    processed_ce_pairs: HashSet<(CeId, CeId)>,
    processed_signatures: HashSet<String>,
    oneof_defs: HashMap<Vec<String>, String>,
    emitted_nom_facts: HashSet<Vec<String>>,
    emitted_nom_individuals: HashSet<String>,
}

impl NominalHyperContext {
    pub(crate) fn empty() -> Self {
        Self {
            processed_ce_pairs: HashSet::new(),
            processed_signatures: HashSet::new(),
            oneof_defs: HashMap::new(),
            emitted_nom_facts: HashSet::new(),
            emitted_nom_individuals: HashSet::new(),
        }
    }

    pub(crate) fn handled_subclass(&self, sub: CeId, sup: CeId) -> bool {
        self.processed_ce_pairs.contains(&(sub, sup))
    }
}

pub(crate) fn clausify_nominal_subclass_axioms(
    ontology: &Ontology,
    ctx: &mut NominalHyperContext,
    def_index: &mut u32,
    push_clause: &mut dyn FnMut(HyperClause),
    push_fact: &mut dyn FnMut(HyperAtom),
) {
    let dl_axioms: Vec<DlAxiom> = ontology.dl().axioms().cloned().collect();
    for axiom in dl_axioms {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        if try_has_value(ontology, ctx, sub, sup, def_index, push_clause, push_fact) {
            continue;
        }
        if try_object_restriction(ontology, ctx, sub, sup, def_index, push_clause, push_fact) {
            continue;
        }
        if try_all_restriction(ontology, ctx, sub, sup, def_index, push_clause, push_fact) {
            continue;
        }
    }
}

fn try_has_value(
    ontology: &Ontology,
    ctx: &mut NominalHyperContext,
    sub: CeId,
    sup: CeId,
    def_index: &mut u32,
    push_clause: &mut dyn FnMut(HyperClause),
    push_fact: &mut dyn FnMut(HyperAtom),
) -> bool {
    let Some(sub_name) = concept_name(ontology, sub) else {
        return false;
    };
    let Some(ClassExpr::HasValue {
        property,
        individual,
    }) = ontology.dl().ce(sup).cloned()
    else {
        return false;
    };
    let sig = format!(
        "hasvalue:{}:{}",
        entity_canonical_iri(ontology, entity_from_ce(ontology, sub).unwrap_or(EntityId(0))),
        entity_canonical_iri(ontology, individual)
    );
    if ctx.processed_signatures.contains(&sig) {
        ctx.processed_ce_pairs.insert((sub, sup));
        return true;
    }

    let oneof_key = vec![entity_canonical_iri(ontology, individual)];
    emit_nominal_facts(ontology, ctx, &oneof_key, std::slice::from_ref(&individual), push_fact);
    let role = role_atom(
        ontology,
        &property,
        Term::Var(VAR_X.into()),
        Term::Var("Y".into()),
    );
    push_clause(HyperClause {
        head: vec![role],
        body: vec![
            HyperAtom::Concept {
                name: sub_name,
                term: Term::Var(VAR_X.into()),
            },
            HyperAtom::Concept {
                name: nominal_fact_name(ontology, individual),
                term: Term::Var("Y".into()),
            },
        ],
    });
    ctx.processed_signatures.insert(sig);
    ctx.processed_ce_pairs.insert((sub, sup));
    let _ = def_index;
    true
}

fn try_object_restriction(
    ontology: &Ontology,
    ctx: &mut NominalHyperContext,
    sub: CeId,
    sup: CeId,
    def_index: &mut u32,
    push_clause: &mut dyn FnMut(HyperClause),
    push_fact: &mut dyn FnMut(HyperAtom),
) -> bool {
    let Some(sub_name) = concept_name(ontology, sub) else {
        return false;
    };
    let Some(ClassExpr::Some { property, filler }) = ontology.dl().ce(sup).cloned() else {
        return false;
    };
    let Some(oneof) = oneof_from_filler(ontology, filler) else {
        return false;
    };
    let individuals = canonical_individuals(ontology, &oneof);
    if individuals.is_empty() {
        return false;
    }

    let signature = subclass_signature(ontology, sub, &property, &individuals, "some");
    if ctx.processed_signatures.contains(&signature) {
        ctx.processed_ce_pairs.insert((sub, sup));
        return true;
    }

    let oneof_key: Vec<String> = individuals
        .iter()
        .map(|id| entity_canonical_iri(ontology, *id))
        .collect();
    emit_nominal_facts(ontology, ctx, &oneof_key, &individuals, push_fact);

    if is_complement_oneof(ontology, filler) {
        let def = complement_def_name(ctx, &oneof_key, def_index);
        for ind in &individuals {
            push_fact(HyperAtom::Concept {
                name: def.clone(),
                term: individual_term(ontology, *ind),
            });
        }
        let role = role_atom(
            ontology,
            &property,
            Term::Var(VAR_X.into()),
            Term::Var("Y".into()),
        );
        let role_name = match &role {
            HyperAtom::Role { role, .. } => role.clone(),
            _ => unreachable!(),
        };
        push_clause(HyperClause {
            head: vec![HyperAtom::AtLeastObject {
                n: 1,
                role: role_name,
                concept: format!("not({def})"),
                term: Term::Var(VAR_X.into()),
            }],
            body: vec![HyperAtom::Concept {
                name: sub_name,
                term: Term::Var(VAR_X.into()),
            }],
        });
    } else {
        let mut head = Vec::new();
        let mut body = vec![HyperAtom::Concept {
            name: sub_name,
            term: Term::Var(VAR_X.into()),
        }];
        for (i, ind) in individuals.iter().enumerate() {
            let var = nominal_var(i);
            body.push(HyperAtom::Concept {
                name: nominal_fact_name(ontology, *ind),
                term: Term::Var(var.clone()),
            });
            head.push(role_atom(
                ontology,
                &property,
                Term::Var(VAR_X.into()),
                Term::Var(var),
            ));
        }
        push_clause(HyperClause { head, body });
    }

    ctx.processed_signatures.insert(signature);
    ctx.processed_ce_pairs.insert((sub, sup));
    true
}

fn try_all_restriction(
    ontology: &Ontology,
    ctx: &mut NominalHyperContext,
    sub: CeId,
    sup: CeId,
    def_index: &mut u32,
    push_clause: &mut dyn FnMut(HyperClause),
    push_fact: &mut dyn FnMut(HyperAtom),
) -> bool {
    let Some(sub_name) = concept_name(ontology, sub) else {
        return false;
    };
    let Some(ClassExpr::All { property, filler }) = ontology.dl().ce(sup).cloned() else {
        return false;
    };

    if is_complement_oneof(ontology, filler) {
        let Some(oneof) = oneof_from_filler(ontology, filler) else {
            return false;
        };
        let individuals = canonical_individuals(ontology, &oneof);
        let signature = subclass_signature(ontology, sub, &property, &individuals, "all-not");
        if ctx.processed_signatures.contains(&signature) {
            ctx.processed_ce_pairs.insert((sub, sup));
            return true;
        }
        let oneof_key: Vec<String> = individuals
            .iter()
            .map(|id| entity_canonical_iri(ontology, *id))
            .collect();
        emit_nominal_facts(ontology, ctx, &oneof_key, &individuals, push_fact);

        let mut body = vec![
            HyperAtom::Concept {
                name: sub_name,
                term: Term::Var(VAR_X.into()),
            },
            role_atom(
                ontology,
                &property,
                Term::Var(VAR_X.into()),
                Term::Var("Y".into()),
            ),
        ];

        if individuals.len() == 1 {
            body.push(HyperAtom::Concept {
                name: nominal_fact_name(ontology, individuals[0]),
                term: Term::Var("Y".into()),
            });
        } else {
            let def = complement_def_name(ctx, &oneof_key, def_index);
            for ind in &individuals {
                push_fact(HyperAtom::Concept {
                    name: def.clone(),
                    term: individual_term(ontology, *ind),
                });
            }
            body.push(HyperAtom::Concept {
                name: def,
                term: Term::Var("Y".into()),
            });
        }
        push_clause(HyperClause { head: vec![], body });
        ctx.processed_signatures.insert(signature);
        ctx.processed_ce_pairs.insert((sub, sup));
        return true;
    }

    let Some(oneof) = oneof_from_filler(ontology, filler) else {
        return false;
    };
    let individuals = canonical_individuals(ontology, &oneof);
    let signature = subclass_signature(ontology, sub, &property, &individuals, "all");
    if ctx.processed_signatures.contains(&signature) {
        ctx.processed_ce_pairs.insert((sub, sup));
        return true;
    }
    let oneof_key: Vec<String> = individuals
        .iter()
        .map(|id| entity_canonical_iri(ontology, *id))
        .collect();
    emit_nominal_facts(ontology, ctx, &oneof_key, &individuals, push_fact);

    let mut head = Vec::new();
    let mut body = vec![
        HyperAtom::Concept {
            name: sub_name,
            term: Term::Var(VAR_X.into()),
        },
        role_atom(
            ontology,
            &property,
            Term::Var(VAR_X.into()),
            Term::Var("Y".into()),
        ),
    ];
    for (i, ind) in individuals.iter().enumerate() {
        let zi = format!("Y{}", i + 1);
        body.push(HyperAtom::Concept {
            name: nominal_fact_name(ontology, *ind),
            term: Term::Var(zi.clone()),
        });
        head.push(HyperAtom::Equality {
            left: Term::Var("Y".into()),
            right: Term::Var(zi),
        });
    }
    push_clause(HyperClause { head, body });
    ctx.processed_signatures.insert(signature);
    ctx.processed_ce_pairs.insert((sub, sup));
    true
}

fn subclass_signature(
    ontology: &Ontology,
    sub: CeId,
    property: &RoleExpr,
    individuals: &[EntityId],
    mode: &str,
) -> String {
    let sub_iri = entity_from_ce(ontology, sub)
        .map(|id| entity_canonical_iri(ontology, id))
        .unwrap_or_default();
    let prop_iri = match property {
        RoleExpr::Atomic(id) => entity_canonical_iri(ontology, *id),
        RoleExpr::Inverse(id) => format!("inv:{}", entity_canonical_iri(ontology, *id)),
    };
    let oneof_key = individuals
        .iter()
        .map(|id| entity_canonical_iri(ontology, *id))
        .collect::<Vec<_>>()
        .join("|");
    format!("{sub_iri}:{prop_iri}:{oneof_key}:{mode}")
}

fn oneof_from_filler(ontology: &Ontology, filler: CeId) -> Option<Vec<EntityId>> {
    match ontology.dl().ce(filler)? {
        ClassExpr::OneOf(v) => Some(v.clone()),
        ClassExpr::Not(inner) => match ontology.dl().ce(*inner)? {
            ClassExpr::OneOf(v) => Some(v.clone()),
            _ => None,
        },
        _ => None,
    }
}

fn is_complement_oneof(ontology: &Ontology, filler: CeId) -> bool {
    matches!(
        ontology.dl().ce(filler),
        Some(ClassExpr::Not(inner)) if matches!(
            ontology.dl().ce(*inner),
            Some(ClassExpr::OneOf(_))
        )
    )
}

fn canonical_individuals(ontology: &Ontology, individuals: &[EntityId]) -> Vec<EntityId> {
    let mut out: Vec<(String, EntityId)> = individuals
        .iter()
        .map(|id| (entity_canonical_iri(ontology, *id), *id))
        .collect();
    out.sort_by(|a, b| a.0.cmp(&b.0));
    out.dedup_by(|a, b| a.0 == b.0);
    out.into_iter().map(|(_, id)| id).collect()
}

fn emit_nominal_facts(
    ontology: &Ontology,
    ctx: &mut NominalHyperContext,
    oneof_key: &[String],
    individuals: &[EntityId],
    push_fact: &mut dyn FnMut(HyperAtom),
) {
    if !ctx.emitted_nom_facts.insert(oneof_key.to_vec()) {
        return;
    }
    for ind in individuals {
        let iri = entity_canonical_iri(ontology, *ind);
        if !ctx.emitted_nom_individuals.insert(iri) {
            continue;
        }
        push_fact(HyperAtom::Concept {
            name: nominal_fact_name(ontology, *ind),
            term: individual_term(ontology, *ind),
        });
    }
}

fn complement_def_name(
    ctx: &mut NominalHyperContext,
    oneof_key: &[String],
    def_index: &mut u32,
) -> String {
    if let Some(existing) = ctx.oneof_defs.get(oneof_key) {
        return existing.clone();
    }
    let name = format!("def:{}", *def_index);
    *def_index += 1;
    ctx.oneof_defs.insert(oneof_key.to_vec(), name.clone());
    name
}

fn entity_from_ce(ontology: &Ontology, ce: CeId) -> Option<EntityId> {
    match ontology.dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn nominal_fact_name(ontology: &Ontology, ind: EntityId) -> String {
    let abbrev = abbrev_entity(ontology, ind);
    let local = abbrev.strip_prefix(':').unwrap_or(&abbrev);
    format!("nom:{local}")
}

fn individual_term(ontology: &Ontology, ind: EntityId) -> Term {
    Term::Ind(abbrev_entity(ontology, ind))
}

fn nominal_var(index: usize) -> String {
    if index == 0 {
        "Y".into()
    } else {
        format!("Y{}", index)
    }
}
