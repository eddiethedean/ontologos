use std::collections::BTreeSet;

use ontologos_core::{Ontology, OwlConstruct};

/// Collect OWL constructs used for profile **classification** (mapped TBox shapes).
pub fn scan_constructs(ontology: &Ontology) -> BTreeSet<OwlConstruct> {
    if let Some(meta) = ontology.parse_meta() {
        if !meta.profile_constructs.is_empty() {
            return meta.profile_constructs.clone();
        }
    }

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
    }
}
