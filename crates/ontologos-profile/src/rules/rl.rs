use std::collections::BTreeSet;

use crate::ProfileDiagnostic;
use ontologos_core::OwlConstruct;

pub(crate) const RL_FORBIDDEN: &[OwlConstruct] = &[
    OwlConstruct::ObjectComplementOf,
    OwlConstruct::ObjectOneOf,
    OwlConstruct::ObjectHasValue,
    OwlConstruct::ObjectHasSelf,
    OwlConstruct::ObjectCardinality,
    OwlConstruct::DisjointUnion,
    OwlConstruct::HasKey,
    OwlConstruct::DataPropertyAssertion,
    OwlConstruct::SwrlRule,
    OwlConstruct::ObjectAllValuesFrom,
    OwlConstruct::ObjectUnionOf,
    OwlConstruct::SubClassOfExistential,
    OwlConstruct::ObjectSomeValuesFrom,
    OwlConstruct::SubClassOfIntersection,
    OwlConstruct::ObjectIntersectionOf,
];

pub fn satisfies_rl(constructs: &BTreeSet<OwlConstruct>) -> bool {
    !constructs.iter().any(|c| RL_FORBIDDEN.contains(c))
}

pub fn rl_diagnostics(constructs: &BTreeSet<OwlConstruct>) -> Vec<ProfileDiagnostic> {
    constructs
        .iter()
        .filter(|c| RL_FORBIDDEN.contains(c))
        .map(|c| ProfileDiagnostic {
            construct: format!("{c:?}"),
            message: "construct is outside OWL 2 RL".into(),
        })
        .collect()
}
