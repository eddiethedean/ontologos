use ontologos_core::OwlConstruct;

use crate::{OwlProfile, ProfileDiagnostic};

use super::el::EL_FORBIDDEN;
use super::ql::QL_FORBIDDEN;
use super::rl::RL_FORBIDDEN;

/// Constructs that do not affect profile classification or source diagnostics.
const NEUTRAL: &[OwlConstruct] = &[
    OwlConstruct::Annotation,
    OwlConstruct::Import,
    OwlConstruct::Unknown,
];

/// Forbidden constructs for the given detected profile.
#[must_use]
pub fn forbidden_for_profile(profile: OwlProfile) -> &'static [OwlConstruct] {
    match profile {
        OwlProfile::Ql => QL_FORBIDDEN,
        OwlProfile::El => EL_FORBIDDEN,
        OwlProfile::Rl => RL_FORBIDDEN,
        OwlProfile::Dl => &[],
    }
}

/// Diagnostics for constructs observed in the full parse but not reflected in mapped profile inputs.
pub fn source_only_diagnostics(
    detected: OwlProfile,
    source: &std::collections::BTreeSet<OwlConstruct>,
    mapped: &std::collections::BTreeSet<OwlConstruct>,
) -> Vec<ProfileDiagnostic> {
    let forbidden = forbidden_for_profile(detected);
    source
        .iter()
        .filter(|c| !mapped.contains(c))
        .filter(|c| !NEUTRAL.contains(c))
        .filter(|c| forbidden.contains(c))
        .map(|c| ProfileDiagnostic {
            construct: format!("{c:?}"),
            message: format!(
                "construct observed in source but outside detected {detected:?} profile (not mapped to core)"
            ),
        })
        .collect()
}
