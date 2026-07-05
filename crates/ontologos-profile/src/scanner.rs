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
    constructs
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
