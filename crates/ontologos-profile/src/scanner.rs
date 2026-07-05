use std::collections::BTreeSet;

use ontologos_core::{Ontology, OwlConstruct};

/// Collect OWL constructs present in a single axiom.
#[must_use]
pub fn axiom_constructs(axiom: &ontologos_core::Axiom) -> BTreeSet<OwlConstruct> {
    let mut constructs = BTreeSet::new();
    note_axiom_construct(axiom, &mut constructs);
    constructs
}

/// Collect OWL constructs used for profile **classification** (mapped TBox shapes).
pub fn scan_constructs(ontology: &Ontology) -> BTreeSet<OwlConstruct> {
    let mut constructs = if ontology.dirty().is_dirty() {
        scan_constructs_from_axioms(ontology)
    } else if let Some(meta) = ontology.parse_meta()
        && !meta.profile_constructs.is_empty()
    {
        meta.profile_constructs.clone()
    } else {
        scan_constructs_from_axioms(ontology)
    };

    // Freshly loaded ontologies are dirty but still carry parse-time profile tags
    // (e.g. datatype restrictions) that core axioms alone do not surface.
    if ontology.dirty().is_dirty()
        && let Some(meta) = ontology.parse_meta()
    {
        constructs.extend(meta.profile_constructs.iter().cloned());
    }

    constructs
}

fn scan_constructs_from_axioms(ontology: &Ontology) -> BTreeSet<OwlConstruct> {
    let mut constructs = BTreeSet::new();
    for (_, axiom) in ontology.axioms().iter() {
        note_axiom_construct(axiom, &mut constructs);
    }
    constructs.extend(scan_constructs_from_dl(ontology));
    constructs
}

/// Collect OWL constructs present in the DL axiom store.
#[must_use]
pub fn scan_constructs_from_dl(ontology: &Ontology) -> BTreeSet<OwlConstruct> {
    let mut constructs = BTreeSet::new();
    let store = ontology.dl();
    for axiom in store.axioms() {
        note_dl_axiom_construct(store, axiom, &mut constructs);
    }
    constructs
}

/// Profile constructs used by a single DL axiom (including nested class expressions).
#[must_use]
pub fn dl_axiom_constructs(
    store: &ontologos_core::DlStore,
    axiom: &ontologos_core::DlAxiom,
) -> BTreeSet<OwlConstruct> {
    let mut constructs = BTreeSet::new();
    note_dl_axiom_construct(store, axiom, &mut constructs);
    constructs
}

fn note_dl_axiom_construct(
    store: &ontologos_core::DlStore,
    axiom: &ontologos_core::DlAxiom,
    constructs: &mut BTreeSet<OwlConstruct>,
) {
    use ontologos_core::DlAxiom;
    match axiom {
        DlAxiom::SubClassOf { sub, sup } => {
            constructs.insert(OwlConstruct::SubClassOfNamed);
            note_ce_construct(store, *sub, constructs);
            note_ce_construct(store, *sup, constructs);
        }
        DlAxiom::EquivalentClasses(ids) => {
            constructs.insert(OwlConstruct::EquivalentClasses);
            for id in ids {
                note_ce_construct(store, *id, constructs);
            }
        }
        DlAxiom::DisjointClasses(ids) => {
            constructs.insert(OwlConstruct::DisjointClasses);
            for id in ids {
                note_ce_construct(store, *id, constructs);
            }
        }
        DlAxiom::ObjectPropertyDomain { domain, .. } => {
            constructs.insert(OwlConstruct::ObjectPropertyDomain);
            note_ce_construct(store, *domain, constructs);
        }
        DlAxiom::ObjectPropertyRange { range, .. } => {
            constructs.insert(OwlConstruct::ObjectPropertyRange);
            note_ce_construct(store, *range, constructs);
        }
        DlAxiom::SubObjectPropertyChain { .. } => {
            constructs.insert(OwlConstruct::SubObjectPropertyChain);
        }
        DlAxiom::SubObjectPropertyOf { sub, sup } => {
            constructs.insert(OwlConstruct::SubObjectPropertyOf);
            note_role_construct(sub, constructs);
            note_role_construct(sup, constructs);
        }
        DlAxiom::HasKey { class, .. } => {
            constructs.insert(OwlConstruct::HasKey);
            note_ce_construct(store, *class, constructs);
        }
        DlAxiom::ClassAssertion { class, .. } => {
            constructs.insert(OwlConstruct::ClassAssertion);
            note_ce_construct(store, *class, constructs);
        }
        DlAxiom::DataPropertyDomain { domain, .. } => {
            constructs.insert(OwlConstruct::Datatype);
            note_ce_construct(store, *domain, constructs);
        }
        DlAxiom::DataPropertyRange { .. } => {
            constructs.insert(OwlConstruct::Datatype);
        }
        DlAxiom::SubDataPropertyOf { .. } => {
            constructs.insert(OwlConstruct::DataPropertyAxiom);
        }
        DlAxiom::DataPropertyAssertion { .. } => {
            constructs.insert(OwlConstruct::DataPropertyAssertion);
        }
        DlAxiom::NegativeObjectPropertyAssertion { .. } => {}
        DlAxiom::NegativeDataPropertyAssertion { .. } => {}
        DlAxiom::SameIndividual(_) | DlAxiom::DifferentIndividuals(_) => {
            constructs.insert(OwlConstruct::IndividualEquality);
        }
        DlAxiom::ObjectPropertyAssertion { property, .. } => {
            constructs.insert(OwlConstruct::ObjectPropertyAssertion);
            note_role_construct(property, constructs);
        }
        DlAxiom::DatatypeDefinition { .. } => {
            constructs.insert(OwlConstruct::Datatype);
        }
        DlAxiom::FunctionalDataProperty(_) => {
            constructs.insert(OwlConstruct::DataPropertyAxiom);
        }
        DlAxiom::EquivalentDataProperties(_) => {
            constructs.insert(OwlConstruct::DataPropertyAxiom);
        }
        DlAxiom::DisjointDataProperties(_) => {
            constructs.insert(OwlConstruct::DataPropertyAxiom);
        }
        DlAxiom::DisjointObjectProperties(_) => {
            constructs.insert(OwlConstruct::DisjointObjectProperties);
        }
        DlAxiom::TransitiveObjectProperty(property) => {
            constructs.insert(OwlConstruct::TransitiveObjectProperty);
            note_role_construct(property, constructs);
        }
        DlAxiom::SymmetricObjectProperty(property) => {
            constructs.insert(OwlConstruct::SymmetricObjectProperty);
            note_role_construct(property, constructs);
        }
        DlAxiom::SwrlRule => {
            constructs.insert(OwlConstruct::SwrlRule);
        }
        DlAxiom::InverseFunctionalObjectProperty(_) => {
            constructs.insert(OwlConstruct::InverseFunctionalObjectProperty);
        }
        DlAxiom::IrreflexiveObjectProperty(_) => {
            constructs.insert(OwlConstruct::IrreflexiveObjectProperty);
        }
    }
}

fn note_ce_construct(
    store: &ontologos_core::DlStore,
    ce: ontologos_core::CeId,
    constructs: &mut BTreeSet<OwlConstruct>,
) {
    use ontologos_core::ClassExpr;
    let Some(expr) = store.ce(ce) else {
        return;
    };
    match expr {
        ClassExpr::Top | ClassExpr::Bottom | ClassExpr::Atomic(_) => {}
        ClassExpr::Not(inner) => {
            constructs.insert(OwlConstruct::ObjectComplementOf);
            note_ce_construct(store, *inner, constructs);
        }
        ClassExpr::And(ids) => {
            constructs.insert(OwlConstruct::ObjectIntersectionOf);
            for id in ids {
                note_ce_construct(store, *id, constructs);
            }
        }
        ClassExpr::Or(ids) => {
            constructs.insert(OwlConstruct::ObjectUnionOf);
            for id in ids {
                note_ce_construct(store, *id, constructs);
            }
        }
        ClassExpr::OneOf(_) => {
            constructs.insert(OwlConstruct::ObjectOneOf);
        }
        ClassExpr::Some { property, filler } => {
            constructs.insert(OwlConstruct::ObjectSomeValuesFrom);
            note_role_construct(property, constructs);
            note_ce_construct(store, *filler, constructs);
        }
        ClassExpr::All { property, filler } => {
            constructs.insert(OwlConstruct::ObjectAllValuesFrom);
            note_role_construct(property, constructs);
            note_ce_construct(store, *filler, constructs);
        }
        ClassExpr::HasValue { property, .. } => {
            constructs.insert(OwlConstruct::ObjectHasValue);
            note_role_construct(property, constructs);
        }
        ClassExpr::HasSelf(_) => {
            constructs.insert(OwlConstruct::ObjectHasSelf);
        }
        ClassExpr::MinCardinality { .. }
        | ClassExpr::MaxCardinality { .. }
        | ClassExpr::ExactCardinality { .. } => {
            constructs.insert(OwlConstruct::ObjectCardinality);
        }
        ClassExpr::DataAll { .. }
        | ClassExpr::DataSome { .. }
        | ClassExpr::DataHasValue { .. }
        | ClassExpr::DataMinCardinality { .. }
        | ClassExpr::DataMaxCardinality { .. }
        | ClassExpr::DataExactCardinality { .. } => {
            constructs.insert(OwlConstruct::Datatype);
        }
    }
}

fn note_role_construct(
    property: &ontologos_core::RoleExpr,
    constructs: &mut BTreeSet<OwlConstruct>,
) {
    if matches!(property, ontologos_core::RoleExpr::Inverse(_)) {
        constructs.insert(OwlConstruct::InverseObjectProperties);
    }
}

/// Full construct set from parse-time scanning (includes skipped/unmapped shapes).
pub fn source_constructs(ontology: &Ontology) -> BTreeSet<OwlConstruct> {
    ontology
        .parse_meta()
        .map(|meta| meta.constructs.clone())
        .unwrap_or_default()
}

fn note_axiom_construct(axiom: &ontologos_core::Axiom, constructs: &mut BTreeSet<OwlConstruct>) {
    use ontologos_core::Axiom;
    match axiom {
        Axiom::SubClassOf { .. } => {
            constructs.insert(OwlConstruct::SubClassOfNamed);
        }
        Axiom::SubClassOfExistential { .. } => {
            constructs.insert(OwlConstruct::SubClassOfExistential);
            constructs.insert(OwlConstruct::ObjectSomeValuesFrom);
        }
        Axiom::EquivalentClasses(_) => {
            constructs.insert(OwlConstruct::EquivalentClasses);
        }
        Axiom::DisjointClasses(_) => {
            constructs.insert(OwlConstruct::DisjointClasses);
        }
        Axiom::ObjectPropertyDomain { .. } => {
            constructs.insert(OwlConstruct::ObjectPropertyDomain);
        }
        Axiom::ObjectPropertyRange { .. } => {
            constructs.insert(OwlConstruct::ObjectPropertyRange);
        }
        Axiom::SubObjectPropertyOf { .. } => {
            constructs.insert(OwlConstruct::SubObjectPropertyOf);
        }
        Axiom::InverseObjectProperties { .. } => {
            constructs.insert(OwlConstruct::InverseObjectProperties);
        }
        Axiom::TransitiveObjectProperty(_) => {
            constructs.insert(OwlConstruct::TransitiveObjectProperty);
        }
        Axiom::SymmetricObjectProperty(_) => {
            constructs.insert(OwlConstruct::SymmetricObjectProperty);
        }
        Axiom::ReflexiveObjectProperty(_) => {
            constructs.insert(OwlConstruct::ReflexiveObjectProperty);
        }
        Axiom::FunctionalObjectProperty(_) => {
            constructs.insert(OwlConstruct::FunctionalObjectProperty);
        }
        Axiom::InverseFunctionalObjectProperty(_) => {
            constructs.insert(OwlConstruct::InverseFunctionalObjectProperty);
        }
        Axiom::IrreflexiveObjectProperty(_) => {
            constructs.insert(OwlConstruct::IrreflexiveObjectProperty);
        }
        Axiom::AsymmetricObjectProperty(_) => {
            constructs.insert(OwlConstruct::AsymmetricObjectProperty);
        }
        Axiom::EquivalentObjectProperties(_) => {
            constructs.insert(OwlConstruct::EquivalentObjectProperties);
        }
        Axiom::ClassAssertion { .. } => {
            constructs.insert(OwlConstruct::ClassAssertion);
        }
        Axiom::ObjectPropertyAssertion { .. } => {
            constructs.insert(OwlConstruct::ObjectPropertyAssertion);
        }
        Axiom::DataPropertyAssertion { .. } => {
            constructs.insert(OwlConstruct::DataPropertyAssertion);
        }
        Axiom::NegativeObjectPropertyAssertion { .. } => {}
        Axiom::NegativeDataPropertyAssertion { .. } => {}
        Axiom::SameIndividual(_) | Axiom::DifferentIndividuals(_) => {
            constructs.insert(OwlConstruct::IndividualEquality);
        }
    }
}
