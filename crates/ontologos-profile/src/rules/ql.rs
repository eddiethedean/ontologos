use std::collections::BTreeSet;

use crate::ProfileDiagnostic;
use ontologos_core::OwlConstruct;

const QL_FORBIDDEN: &[OwlConstruct] = &[
    OwlConstruct::ObjectComplementOf,
    OwlConstruct::ObjectUnionOf,
    OwlConstruct::ObjectOneOf,
    OwlConstruct::ObjectCardinality,
    OwlConstruct::ObjectHasValue,
    OwlConstruct::ObjectHasSelf,
    OwlConstruct::SubClassOfExistential,
    OwlConstruct::SubClassOfIntersection,
    OwlConstruct::ObjectSomeValuesFrom,
    OwlConstruct::ObjectIntersectionOf,
    OwlConstruct::SubObjectPropertyChain,
    OwlConstruct::DisjointUnion,
    OwlConstruct::HasKey,
    OwlConstruct::IndividualEquality,
    OwlConstruct::ClassAssertion,
    OwlConstruct::ObjectPropertyAssertion,
    OwlConstruct::DataPropertyAssertion,
    OwlConstruct::SwrlRule,
    OwlConstruct::InverseObjectProperties,
    OwlConstruct::EquivalentObjectProperties,
    OwlConstruct::SymmetricObjectProperty,
    OwlConstruct::AsymmetricObjectProperty,
    OwlConstruct::ReflexiveObjectProperty,
    OwlConstruct::IrreflexiveObjectProperty,
    OwlConstruct::FunctionalObjectProperty,
    OwlConstruct::InverseFunctionalObjectProperty,
    OwlConstruct::TransitiveObjectProperty,
];

pub fn satisfies_ql(constructs: &BTreeSet<OwlConstruct>) -> bool {
    !constructs.iter().any(|c| QL_FORBIDDEN.contains(c))
}

pub fn ql_diagnostics(constructs: &BTreeSet<OwlConstruct>) -> Vec<ProfileDiagnostic> {
    constructs
        .iter()
        .filter(|c| QL_FORBIDDEN.contains(c))
        .map(|c| ProfileDiagnostic {
            construct: format!("{c:?}"),
            message: "construct is outside OWL 2 QL".into(),
        })
        .collect()
}
