use std::collections::BTreeSet;

use crate::ProfileDiagnostic;
use ontologos_core::OwlConstruct;

pub fn dl_diagnostics(constructs: &BTreeSet<OwlConstruct>) -> Vec<ProfileDiagnostic> {
    constructs
        .iter()
        .filter(|c| matches!(c, OwlConstruct::Unknown | OwlConstruct::SwrlRule))
        .map(|c| ProfileDiagnostic {
            construct: format!("{c:?}"),
            message: "construct requires OWL 2 DL reasoning".into(),
        })
        .collect()
}
