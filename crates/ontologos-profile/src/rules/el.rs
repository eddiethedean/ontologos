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
    OwlConstruct::InverseObjectProperties,
    OwlConstruct::FunctionalObjectProperty,
    OwlConstruct::InverseFunctionalObjectProperty,
    OwlConstruct::SymmetricObjectProperty,
    OwlConstruct::ReflexiveObjectProperty,
    OwlConstruct::AsymmetricObjectProperty,
    OwlConstruct::IrreflexiveObjectProperty,
    OwlConstruct::DisjointObjectProperties,
    OwlConstruct::EquivalentObjectProperties,
];

/// Mapped constructs that the in-house EL completion engine cannot soundly classify.
///
/// Narrower than [`EL_FORBIDDEN`]: property characteristics and ABox shapes may appear in
/// hybrid corpora (e.g. Pizza) but are ignored by completion and do not block `--profile el`.
pub(crate) const EL_CLASSIFICATION_FORBIDDEN: &[OwlConstruct] = &[
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
    OwlConstruct::SwrlRule,
];

/// Returns true when every construct in `constructs` is allowed in OWL 2 EL.
pub fn satisfies_el(constructs: &BTreeSet<OwlConstruct>) -> bool {
    !constructs.iter().any(|c| EL_FORBIDDEN.contains(c))
}

/// Constructs present in `constructs` that fall outside OWL 2 EL.
pub fn el_forbidden_in(constructs: &BTreeSet<OwlConstruct>) -> BTreeSet<OwlConstruct> {
    constructs
        .iter()
        .filter(|c| EL_FORBIDDEN.contains(c))
        .cloned()
        .collect()
}

/// Constructs that block in-house EL classification when present in mapped axioms.
pub fn el_classification_forbidden_in(
    constructs: &BTreeSet<OwlConstruct>,
) -> BTreeSet<OwlConstruct> {
    constructs
        .iter()
        .filter(|c| EL_CLASSIFICATION_FORBIDDEN.contains(c))
        .cloned()
        .collect()
}

/// Profile diagnostics for constructs outside OWL 2 EL.
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
