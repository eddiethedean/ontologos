use std::collections::BTreeSet;

use ontologos_core::{Ontology, OwlConstruct};
use ontologos_profile::{detect_profile, OwlProfile};

use crate::Error;

/// Validate that mapped TBox axioms are within OWL EL for classification.
///
/// Hybrid or DL-detected ontologies may still be classified with `--profile el`;
/// only mapped constructs outside EL cause failure.
pub fn validate_el_profile(ontology: &Ontology) -> crate::Result<()> {
    if non_el_constructs(ontology).is_empty() {
        return Ok(());
    }
    let report = detect_profile(ontology).map_err(|e| Error::Profile(e.to_string()))?;
    Err(Error::NonElProfile {
        detected: report.detected.unwrap_or(OwlProfile::Dl),
    })
}

/// Constructs that block EL classification when present in mapped axioms.
pub fn non_el_constructs(ontology: &Ontology) -> BTreeSet<OwlConstruct> {
    let mut constructs = BTreeSet::new();
    use ontologos_profile::scanner::scan_constructs;
    scan_constructs(ontology)
        .into_iter()
        .filter(|c| is_non_el_construct(c.clone()))
        .for_each(|c| {
            constructs.insert(c);
        });
    constructs
}

fn is_non_el_construct(c: OwlConstruct) -> bool {
    matches!(
        c,
        OwlConstruct::ObjectAllValuesFrom
            | OwlConstruct::ObjectComplementOf
            | OwlConstruct::ObjectUnionOf
            | OwlConstruct::ObjectOneOf
            | OwlConstruct::ObjectCardinality
            | OwlConstruct::ObjectHasValue
            | OwlConstruct::ObjectHasSelf
            | OwlConstruct::SubObjectPropertyChain
            | OwlConstruct::DisjointUnion
            | OwlConstruct::HasKey
            | OwlConstruct::SwrlRule
    )
}
