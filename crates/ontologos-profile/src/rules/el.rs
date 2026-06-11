use std::collections::BTreeSet;

use crate::ProfileDiagnostic;
use ontologos_core::OwlConstruct;

pub(crate) const EL_FORBIDDEN: &[OwlConstruct] = &[
    OwlConstruct::ObjectAllValuesFrom,
    OwlConstruct::ObjectComplementOf,
    OwlConstruct::ObjectUnionOf,
    OwlConstruct::ObjectOneOf,
    OwlConstruct::ObjectCardinality,
    OwlConstruct::ObjectHasValue,
    OwlConstruct::ObjectHasSelf,
    OwlConstruct::SubObjectPropertyChain,
    OwlConstruct::DisjointUnion,
    OwlConstruct::HasKey,
    OwlConstruct::IndividualEquality,
    OwlConstruct::ClassAssertion,
    OwlConstruct::ObjectPropertyAssertion,
    OwlConstruct::DataPropertyAssertion,
    OwlConstruct::SwrlRule,
];

pub fn satisfies_el(constructs: &BTreeSet<OwlConstruct>) -> bool {
    !constructs.iter().any(|c| EL_FORBIDDEN.contains(c))
}

pub fn el_diagnostics(constructs: &BTreeSet<OwlConstruct>) -> Vec<ProfileDiagnostic> {
    constructs
        .iter()
        .filter(|c| EL_FORBIDDEN.contains(c))
        .map(|c| ProfileDiagnostic {
            construct: format!("{c:?}"),
            message: "construct is outside OWL 2 EL".into(),
        })
        .collect()
}
