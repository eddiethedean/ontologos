//! OWL 2 DL clausification and NNF.

use ontologos_core::{Axiom, CeId, ClassExpr, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr};

use crate::clause::{Clause, ClauseSet};
use crate::Error;

/// Convert ontology axioms + DL store into clausal form.
pub fn clausify(ontology: &mut Ontology) -> Result<ClauseSet, Error> {
    let mut out = ClauseSet::new();
    let _ = ontology.dl_mut().intern_ce(ClassExpr::Top);
    let _ = ontology.dl_mut().intern_ce(ClassExpr::Bottom);
    let flat_axioms: Vec<Axiom> = ontology.axioms().iter().map(|(_, a)| a.clone()).collect();
    let dl_axioms: Vec<DlAxiom> = ontology.dl().axioms().cloned().collect();
    let class_ids: Vec<EntityId> = ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .map(|(id, _)| id)
        .collect();

    for axiom in &flat_axioms {
        match axiom {
            Axiom::SubClassOf {
                subclass,
                superclass,
            } => {
                let sub = atomic_ce(ontology, *subclass);
                let sup = atomic_ce(ontology, *superclass);
                out.push(Clause::Subsumption { sub, sup });
            }
            Axiom::SubClassOfExistential {
                subclass,
                property,
                filler,
            } => {
                let sub = atomic_ce(ontology, *subclass);
                let filler_ce = atomic_ce(ontology, *filler);
                let exists = ontology.dl_mut().intern_ce(ClassExpr::Some {
                    property: RoleExpr::Atomic(*property),
                    filler: filler_ce,
                });
                out.push(Clause::Subsumption { sub, sup: exists });
            }
            Axiom::EquivalentClasses(ids) if ids.len() >= 2 => {
                for w in ids.windows(2) {
                    let a = atomic_ce(ontology, w[0]);
                    let b = atomic_ce(ontology, w[1]);
                    out.push(Clause::Subsumption { sub: a, sup: b });
                    out.push(Clause::Subsumption { sub: b, sup: a });
                }
            }
            Axiom::DisjointClasses(ids) if ids.len() >= 2 => {
                for w in ids.windows(2) {
                    out.push(Clause::Disjoint {
                        left: atomic_ce(ontology, w[0]),
                        right: atomic_ce(ontology, w[1]),
                    });
                }
            }
            Axiom::SubObjectPropertyOf {
                sub_property,
                super_property,
            } => {
                out.push(Clause::RoleSubsumption {
                    sub: *sub_property,
                    sup: *super_property,
                });
            }
            Axiom::EquivalentObjectProperties(ids) if ids.len() >= 2 => {
                for w in ids.windows(2) {
                    out.push(Clause::RoleSubsumption {
                        sub: w[0],
                        sup: w[1],
                    });
                    out.push(Clause::RoleSubsumption {
                        sub: w[1],
                        sup: w[0],
                    });
                }
            }
            _ => {}
        }
    }

    for axiom in dl_axioms {
        match axiom {
            DlAxiom::SubClassOf { sub, sup } => {
                let sub_nnf = nnf(ontology, sub);
                if let Some(ClassExpr::And(ops)) = ontology.dl().ce(sup).cloned() {
                    for op in ops {
                        let op_nnf = nnf(ontology, op);
                        out.push(Clause::Subsumption {
                            sub: sub_nnf,
                            sup: op_nnf,
                        });
                    }
                } else {
                    let sup_nnf = nnf(ontology, sup);
                    out.push(Clause::Subsumption {
                        sub: sub_nnf,
                        sup: sup_nnf,
                    });
                }
            }
            DlAxiom::EquivalentClasses(ids) if ids.len() >= 2 => {
                for w in ids.windows(2) {
                    let a = nnf(ontology, w[0]);
                    let b = nnf(ontology, w[1]);
                    out.push(Clause::Subsumption { sub: a, sup: b });
                    out.push(Clause::Subsumption { sub: b, sup: a });
                }
            }
            DlAxiom::DisjointClasses(ids) if ids.len() >= 2 => {
                for w in ids.windows(2) {
                    out.push(Clause::Disjoint {
                        left: nnf(ontology, w[0]),
                        right: nnf(ontology, w[1]),
                    });
                }
            }
            DlAxiom::SubObjectPropertyChain {
                chain,
                super_property,
            } => {
                out.push(Clause::RoleChain {
                    chain: chain.clone(),
                    sup: super_property.clone(),
                });
            }
            DlAxiom::ClassAssertion { individual, class } => {
                let nom = ontology
                    .dl_mut()
                    .intern_ce(ClassExpr::OneOf(vec![individual]));
                out.push(Clause::NominalSubsumption {
                    sub: nom,
                    individual,
                });
                let ce = nnf(ontology, class);
                out.push(Clause::Subsumption { sub: nom, sup: ce });
            }
            DlAxiom::DisjointObjectProperties(ids) if ids.len() >= 2 => {
                for w in ids.windows(2) {
                    out.push(Clause::RoleDisjoint {
                        left: w[0],
                        right: w[1],
                    });
                }
            }
            DlAxiom::SubObjectPropertyOf { sub, sup } => match (&sub, &sup) {
                (RoleExpr::Atomic(sub_id), RoleExpr::Atomic(sup_id)) => {
                    out.push(Clause::RoleSubsumption {
                        sub: *sub_id,
                        sup: *sup_id,
                    });
                }
                _ => {
                    out.push(Clause::RoleChain {
                        chain: vec![sub.clone()],
                        sup: sup.clone(),
                    });
                }
            },
            DlAxiom::ObjectPropertyDomain { property, domain } => {
                clausify_domain_range(ontology, &mut out, property, domain, true);
            }
            DlAxiom::ObjectPropertyRange { property, range } => {
                clausify_domain_range(ontology, &mut out, property, range, false);
            }
            DlAxiom::ObjectPropertyAssertion {
                subject,
                property: RoleExpr::Atomic(prop),
                object,
            } => {
                emit_nominal_role_clause(ontology, &mut out, subject, prop, object);
            }
            DlAxiom::TransitiveObjectProperty(role) => {
                out.push(Clause::RoleChain {
                    chain: vec![role.clone(), role.clone()],
                    sup: role.clone(),
                });
            }
            DlAxiom::SymmetricObjectProperty(_) => {}
            DlAxiom::HasKey {
                class,
                object_properties,
                data_properties,
            } => {
                out.push(Clause::HasKey {
                    class: nnf(ontology, class),
                    object_properties: object_properties.clone(),
                    data_properties: data_properties.clone(),
                });
            }
            _ => {}
        }
    }

    for axiom in &flat_axioms {
        match axiom {
            Axiom::ObjectPropertyDomain { property, domain } => {
                let top = ontology.dl_mut().intern_ce(ClassExpr::Top);
                let dom = atomic_ce(ontology, *domain);
                out.push(Clause::Existential {
                    property: RoleExpr::Atomic(*property),
                    filler: top,
                    sup: dom,
                });
            }
            Axiom::ObjectPropertyRange { property, range } => {
                let top = ontology.dl_mut().intern_ce(ClassExpr::Top);
                let rng = atomic_ce(ontology, *range);
                out.push(Clause::Universal {
                    sub: top,
                    property: RoleExpr::Atomic(*property),
                    filler: rng,
                });
            }
            Axiom::ObjectPropertyAssertion {
                subject,
                property,
                object,
            } => {
                emit_nominal_role_clause(ontology, &mut out, *subject, *property, *object);
            }
            Axiom::TransitiveObjectProperty(prop) => {
                out.push(Clause::RoleChain {
                    chain: vec![RoleExpr::Atomic(*prop), RoleExpr::Atomic(*prop)],
                    sup: RoleExpr::Atomic(*prop),
                });
            }
            Axiom::AsymmetricObjectProperty(_) => {}
            _ => {}
        }
    }
    for id in class_ids {
        atomic_ce(ontology, id);
    }

    Ok(out)
}

fn clausify_domain_range(
    ontology: &mut Ontology,
    out: &mut ClauseSet,
    property: EntityId,
    ce: CeId,
    is_domain: bool,
) {
    let store = ontology.dl();
    let Some(expr) = store.ce(ce).cloned() else {
        return;
    };
    let top = ontology.dl_mut().intern_ce(ClassExpr::Top);
    match expr {
        ClassExpr::OneOf(individuals) if individuals.len() == 1 => {
            let ind = individuals[0];
            let nom = ontology.dl_mut().intern_ce(ClassExpr::OneOf(vec![ind]));
            if is_domain {
                out.push(Clause::Existential {
                    property: RoleExpr::Atomic(property),
                    filler: top,
                    sup: nom,
                });
            } else {
                out.push(Clause::Universal {
                    sub: top,
                    property: RoleExpr::Atomic(property),
                    filler: nom,
                });
            }
        }
        ClassExpr::Atomic(class) => {
            let filler = atomic_ce(ontology, class);
            if is_domain {
                out.push(Clause::Existential {
                    property: RoleExpr::Atomic(property),
                    filler: top,
                    sup: filler,
                });
            } else {
                out.push(Clause::Universal {
                    sub: top,
                    property: RoleExpr::Atomic(property),
                    filler,
                });
            }
        }
        other => {
            let filler = ontology.dl_mut().intern_ce(other);
            if is_domain {
                out.push(Clause::Existential {
                    property: RoleExpr::Atomic(property),
                    filler: top,
                    sup: filler,
                });
            } else {
                out.push(Clause::Universal {
                    sub: top,
                    property: RoleExpr::Atomic(property),
                    filler,
                });
            }
        }
    }
}

fn emit_nominal_role_clause(
    ontology: &mut Ontology,
    out: &mut ClauseSet,
    subject: EntityId,
    property: EntityId,
    object: EntityId,
) {
    let sub_nom = ontology.dl_mut().intern_ce(ClassExpr::OneOf(vec![subject]));
    let obj_nom = ontology.dl_mut().intern_ce(ClassExpr::OneOf(vec![object]));
    out.push(Clause::NominalSubsumption {
        sub: sub_nom,
        individual: subject,
    });
    out.push(Clause::Existential {
        property: RoleExpr::Atomic(property),
        filler: obj_nom,
        sup: sub_nom,
    });
}

fn atomic_ce(ontology: &mut Ontology, entity: EntityId) -> CeId {
    ontology.dl_mut().intern_ce(ClassExpr::Atomic(entity))
}

fn nnf(ontology: &mut Ontology, id: CeId) -> CeId {
    let store = ontology.dl();
    let Some(expr) = store.ce(id).cloned() else {
        return id;
    };
    match expr {
        ClassExpr::Not(inner) => nnf_negate(ontology, inner),
        _ => id,
    }
}

fn nnf_negate(ontology: &mut Ontology, id: CeId) -> CeId {
    let store = ontology.dl();
    let Some(expr) = store.ce(id).cloned() else {
        return ontology.dl_mut().intern_ce(ClassExpr::Not(id));
    };
    match expr {
        ClassExpr::Not(inner) => nnf(ontology, inner),
        ClassExpr::And(ops) => {
            let parts: Vec<CeId> = ops.iter().map(|&c| nnf_negate(ontology, c)).collect();
            ontology.dl_mut().intern_ce(ClassExpr::Or(parts))
        }
        ClassExpr::Or(ops) => {
            let parts: Vec<CeId> = ops.iter().map(|&c| nnf_negate(ontology, c)).collect();
            ontology.dl_mut().intern_ce(ClassExpr::And(parts))
        }
        ClassExpr::Some { property, filler } => {
            let property = property.clone();
            let filler = nnf_negate(ontology, filler);
            ontology
                .dl_mut()
                .intern_ce(ClassExpr::All { property, filler })
        }
        ClassExpr::All { property, filler } => {
            let property = property.clone();
            let filler = nnf_negate(ontology, filler);
            ontology
                .dl_mut()
                .intern_ce(ClassExpr::Some { property, filler })
        }
        ClassExpr::Top => ontology.dl_mut().intern_ce(ClassExpr::Bottom),
        ClassExpr::Bottom => ontology.dl_mut().intern_ce(ClassExpr::Top),
        ClassExpr::Atomic(a) => {
            let inner = ontology.dl_mut().intern_ce(ClassExpr::Atomic(a));
            ontology.dl_mut().intern_ce(ClassExpr::Not(inner))
        }
        other => {
            let inner = ontology.dl_mut().intern_ce(other);
            ontology.dl_mut().intern_ce(ClassExpr::Not(inner))
        }
    }
}

/// Negate a class expression into NNF (may intern new CEs in `ontology`).
pub(crate) fn negate_ce(ontology: &mut Ontology, id: CeId) -> CeId {
    nnf_negate(ontology, id)
}
