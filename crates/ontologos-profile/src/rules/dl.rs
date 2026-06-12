use std::collections::{BTreeSet, HashSet};

use super::{el_diagnostics, rl_diagnostics, satisfies_el, satisfies_rl};
use crate::ProfileDiagnostic;
use ontologos_core::OwlConstruct;

pub fn dl_diagnostics(constructs: &BTreeSet<OwlConstruct>) -> Vec<ProfileDiagnostic> {
    let mut diagnostics = Vec::new();
    let mut seen = HashSet::new();

    if !satisfies_el(constructs) {
        for diag in el_diagnostics(constructs) {
            if seen.insert(diag.construct.clone()) {
                diagnostics.push(ProfileDiagnostic {
                    message: format!("{} (mapped axiom rules out OWL 2 EL)", diag.message),
                    ..diag
                });
            }
        }
    }

    if !satisfies_rl(constructs) {
        for diag in rl_diagnostics(constructs) {
            if seen.insert(diag.construct.clone()) {
                diagnostics.push(ProfileDiagnostic {
                    message: format!("{} (mapped axiom rules out OWL 2 RL)", diag.message),
                    ..diag
                });
            }
        }
    }

    for construct in constructs {
        if matches!(construct, OwlConstruct::Unknown | OwlConstruct::SwrlRule) {
            let key = format!("{construct:?}");
            if seen.insert(key.clone()) {
                diagnostics.push(ProfileDiagnostic {
                    construct: key,
                    message: "construct requires OWL 2 DL reasoning".into(),
                });
            }
        }
    }

    diagnostics
}

pub fn skipped_only_dl_diagnostic() -> ProfileDiagnostic {
    ProfileDiagnostic {
        construct: "SkippedAxioms".into(),
        message: "source contains constructs but no mapped TBox axioms; classified as DL".into(),
    }
}
