use std::collections::BTreeSet;

use ontologos_core::{Ontology, OwlConstruct};

use crate::rules::{
    dl_diagnostics, el_diagnostics, ql_diagnostics, rl_diagnostics, satisfies_el, satisfies_ql,
    satisfies_rl,
};
use crate::scanner::scan_constructs;
use crate::{OwlProfile, ProfileReport, Result};

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
pub fn detect_profile(ontology: &Ontology) -> Result<ProfileReport> {
    let constructs = scan_constructs(ontology);

    if satisfies_ql(&constructs) {
        return Ok(ProfileReport {
            detected: Some(OwlProfile::Ql),
            diagnostics: ql_diagnostics(&constructs),
        });
    }

    let el_ok = satisfies_el(&constructs);
    let rl_ok = satisfies_rl(&constructs);

    if el_ok && rl_ok {
        let detected = if has_marker(&constructs, EL_MARKERS) {
            OwlProfile::El
        } else if has_marker(&constructs, RL_MARKERS) {
            OwlProfile::Rl
        } else {
            OwlProfile::El
        };
        return Ok(ProfileReport {
            detected: Some(detected),
            diagnostics: match detected {
                OwlProfile::El => el_diagnostics(&constructs),
                OwlProfile::Rl => rl_diagnostics(&constructs),
                _ => Vec::new(),
            },
        });
    }

    if el_ok {
        return Ok(ProfileReport {
            detected: Some(OwlProfile::El),
            diagnostics: el_diagnostics(&constructs),
        });
    }

    if rl_ok {
        return Ok(ProfileReport {
            detected: Some(OwlProfile::Rl),
            diagnostics: rl_diagnostics(&constructs),
        });
    }

    Ok(ProfileReport {
        detected: Some(OwlProfile::Dl),
        diagnostics: dl_diagnostics(&constructs),
    })
}

fn has_marker(constructs: &BTreeSet<OwlConstruct>, markers: &[OwlConstruct]) -> bool {
    constructs.iter().any(|c| markers.contains(c))
}
