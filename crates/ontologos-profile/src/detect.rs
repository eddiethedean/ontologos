use std::collections::BTreeSet;

use ontologos_core::{Ontology, OwlConstruct};

use crate::rules::{
    dl_diagnostics, el_diagnostics, ql_diagnostics, rl_diagnostics, satisfies_el, satisfies_ql,
    satisfies_rl, source_only_diagnostics,
};
use crate::scanner::{scan_constructs, source_constructs};
use crate::{OwlProfile, ProfileDiagnostic, ProfileReport, Result};

const EL_MARKERS: &[OwlConstruct] = &[
    OwlConstruct::SubClassOfExistential,
    OwlConstruct::ObjectSomeValuesFrom,
    OwlConstruct::SubClassOfIntersection,
    OwlConstruct::ObjectIntersectionOf,
];

const RL_MARKERS: &[OwlConstruct] = &[
    OwlConstruct::SymmetricObjectProperty,
    OwlConstruct::ReflexiveObjectProperty,
    OwlConstruct::TransitiveObjectProperty,
    OwlConstruct::AsymmetricObjectProperty,
    OwlConstruct::IrreflexiveObjectProperty,
];

/// Detect the most specific OWL profile supported by the ontology.
///
/// Classification uses mapped TBox constructs (`profile_constructs`). Diagnostics also
/// flag constructs observed in the full parse that fall outside the detected profile.
pub fn detect_profile(ontology: &Ontology) -> Result<ProfileReport> {
    let mapped = scan_constructs(ontology);
    let source = source_constructs(ontology);

    let detected = classify_profile(&mapped);
    let diagnostics = merge_diagnostics(detected, &mapped, &source);

    Ok(ProfileReport {
        detected: Some(detected),
        diagnostics,
    })
}

fn classify_profile(mapped: &BTreeSet<OwlConstruct>) -> OwlProfile {
    if satisfies_ql(mapped) {
        return OwlProfile::Ql;
    }

    let el_ok = satisfies_el(mapped);
    let rl_ok = satisfies_rl(mapped);

    if el_ok && rl_ok {
        if has_marker(mapped, EL_MARKERS) {
            return OwlProfile::El;
        }
        if has_marker(mapped, RL_MARKERS) {
            return OwlProfile::Rl;
        }
        return OwlProfile::El;
    }

    if el_ok {
        return OwlProfile::El;
    }

    if rl_ok {
        return OwlProfile::Rl;
    }

    OwlProfile::Dl
}

fn merge_diagnostics(
    detected: OwlProfile,
    mapped: &BTreeSet<OwlConstruct>,
    source: &BTreeSet<OwlConstruct>,
) -> Vec<ProfileDiagnostic> {
    let mut diagnostics = match detected {
        OwlProfile::Ql => ql_diagnostics(mapped),
        OwlProfile::El => el_diagnostics(mapped),
        OwlProfile::Rl => rl_diagnostics(mapped),
        OwlProfile::Dl => dl_diagnostics(mapped),
    };

    diagnostics.extend(source_only_diagnostics(detected, source, mapped));
    diagnostics
}

fn has_marker(constructs: &BTreeSet<OwlConstruct>, markers: &[OwlConstruct]) -> bool {
    constructs.iter().any(|c| markers.contains(c))
}
