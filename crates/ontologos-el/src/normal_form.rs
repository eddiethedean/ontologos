use std::collections::BTreeSet;

use ontologos_core::{Ontology, OwlConstruct};
use ontologos_profile::scanner::scan_constructs;
use ontologos_profile::{detect_profile, el_classification_forbidden_in};

use crate::Error;

/// Validate EL classification eligibility for SWRL forward chaining.
///
/// SWRL rules themselves are not part of the EL TBox and are ignored here.
pub fn validate_el_profile_for_swrl(ontology: &Ontology) -> crate::Result<()> {
    let mut forbidden = non_el_constructs(ontology);
    forbidden.remove(&OwlConstruct::SwrlRule);
    if forbidden.is_empty() {
        return Ok(());
    }
    let report = detect_profile(ontology)?;
    let detected = report
        .detected
        .ok_or_else(|| ontologos_profile::Error::Message("no profile detected".into()))?;
    Err(Error::NonElProfile { detected })
}

/// Validate that mapped TBox axioms are within OWL EL for classification.
///
/// Hybrid or DL-detected ontologies may still be classified with `--profile el`;
/// only mapped constructs outside EL cause failure.
pub fn validate_el_profile(ontology: &Ontology) -> crate::Result<()> {
    if non_el_constructs(ontology).is_empty() {
        return Ok(());
    }
    let report = detect_profile(ontology)?;
    let detected = report
        .detected
        .ok_or_else(|| ontologos_profile::Error::Message("no profile detected".into()))?;
    Err(Error::NonElProfile { detected })
}

/// Constructs that block EL classification when present in mapped axioms.
pub fn non_el_constructs(ontology: &Ontology) -> BTreeSet<OwlConstruct> {
    el_classification_forbidden_in(&scan_constructs(ontology))
}
