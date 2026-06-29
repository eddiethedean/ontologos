//! Union / max-cardinality hyper clausification (HermiT atMost encoding).

use std::collections::HashSet;

use ontologos_core::{CeId, ClassExpr, DlAxiom, Ontology, RoleExpr};

use crate::hyperclause::{abbrev_role, concept_name, HyperAtom, HyperClause, Term};

const VAR_X: &str = "X";

pub(crate) struct CardinalityHyperContext {
    processed_ce_pairs: HashSet<(CeId, CeId)>,
}

impl CardinalityHyperContext {
    pub(crate) fn empty() -> Self {
        Self {
            processed_ce_pairs: HashSet::new(),
        }
    }

    pub(crate) fn handled_subclass(&self, sub: CeId, sup: CeId) -> bool {
        self.processed_ce_pairs.contains(&(sub, sup))
    }
}

pub(crate) fn clausify_cardinality_subclass_axioms(
    ontology: &Ontology,
    ctx: &mut CardinalityHyperContext,
    push_clause: &mut dyn FnMut(HyperClause),
) {
    let dl_axioms: Vec<DlAxiom> = ontology.dl().axioms().cloned().collect();
    for axiom in dl_axioms {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let _ = try_union_max_cardinality(ontology, ctx, sub, sup, push_clause);
    }
}

fn try_union_max_cardinality(
    ontology: &Ontology,
    ctx: &mut CardinalityHyperContext,
    sub: CeId,
    sup: CeId,
    push_clause: &mut dyn FnMut(HyperClause),
) -> bool {
    if ctx.processed_ce_pairs.contains(&(sub, sup)) {
        return true;
    }
    let Some(sub_name) = concept_name(ontology, sub) else {
        return false;
    };
    let Some(ClassExpr::Or(disjuncts)) = ontology.dl().ce(sup).cloned() else {
        return false;
    };

    let mut alt_name = None;
    let mut max = None;
    for d in disjuncts {
        match ontology.dl().ce(d) {
            Some(ClassExpr::Atomic(_)) => alt_name = concept_name(ontology, d),
            Some(ClassExpr::MaxCardinality {
                n,
                property: RoleExpr::Atomic(prop),
                filler,
            }) => {
                let filler_name = filler
                    .and_then(|f| concept_name(ontology, f))
                    .unwrap_or_else(|| "owl:Thing".into());
                max = Some((*n, *prop, filler_name));
            }
            _ => {}
        }
    }
    let Some(alt) = alt_name else {
        return false;
    };
    let Some((n, prop, filler_name)) = max else {
        return false;
    };

    let role = abbrev_role(ontology, prop);
    let witness_count = n + 1;
    let vars: Vec<String> = (1..=witness_count).map(|i| format!("Y{i}")).collect();

    let mut head = vec![HyperAtom::Concept {
        name: alt,
        term: Term::Var(VAR_X.into()),
    }];
    for i in 0..witness_count as usize {
        for j in i + 1..witness_count as usize {
            head.push(HyperAtom::AtMostAnnotated {
                n,
                role: role.clone(),
                concept: filler_name.clone(),
                term: Term::Var(VAR_X.into()),
                eq_left: Term::Var(vars[i].clone()),
                eq_right: Term::Var(vars[j].clone()),
            });
        }
    }

    let mut body = Vec::new();
    for pair in vars.windows(2) {
        body.push(HyperAtom::NodeLe {
            left: Term::Var(pair[0].clone()),
            right: Term::Var(pair[1].clone()),
        });
    }
    body.push(HyperAtom::Concept {
        name: sub_name,
        term: Term::Var(VAR_X.into()),
    });
    for var in &vars {
        body.push(HyperAtom::Concept {
            name: filler_name.clone(),
            term: Term::Var(var.clone()),
        });
    }
    for var in &vars {
        body.push(HyperAtom::Role {
            role: role.clone(),
            subject: Term::Var(VAR_X.into()),
            object: Term::Var(var.clone()),
        });
    }
    body.push(HyperAtom::NodeIDsAscendingOrEqual {
        vars: vars.iter().map(|v| Term::Var(v.clone())).collect(),
    });

    push_clause(HyperClause { head, body });
    ctx.processed_ce_pairs.insert((sub, sup));
    true
}
