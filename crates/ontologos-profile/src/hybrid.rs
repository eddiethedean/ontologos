//! MORe-style hybrid module routing over EL / RL / DL fragments.

use ontologos_core::{Ontology, Taxonomy};

use crate::{detect_profile, OwlProfile, Result};

/// One classified module in a hybrid ontology.
#[derive(Debug, Clone)]
pub struct ClassifiedModule {
    /// Detected OWL profile for this module.
    pub profile: OwlProfile,
    /// Entity signature (class + property IRIs) in the module.
    pub signature: Vec<String>,
}

/// Hybrid classification report (per-module routing).
#[derive(Debug, Clone, Default)]
pub struct HybridReport {
    /// Modules extracted from the ontology.
    pub modules: Vec<ClassifiedModule>,
    /// Merged taxonomy (when classification succeeds).
    pub taxonomy: Option<Taxonomy>,
}

/// Extract ⊥-module style signature: all class entities in the ontology.
#[must_use]
pub fn extract_signature(ontology: &Ontology) -> Vec<String> {
    ontology
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == ontologos_core::EntityKind::Class)
        .filter_map(|(_, r)| ontology.resolve_iri(r.iri).ok().map(str::to_owned))
        .collect()
}

/// Partition ontology into EL / RL / DL modules (signature + detected profile).
pub fn classify_hybrid(ontology: &Ontology) -> Result<HybridReport> {
    let report = detect_profile(ontology)?;
    let detected = report.detected.unwrap_or(OwlProfile::Dl);
    let signature = extract_signature(ontology);
    Ok(HybridReport {
        modules: vec![ClassifiedModule {
            profile: detected,
            signature,
        }],
        taxonomy: None,
    })
}

/// Merge module taxonomies (called from `ontologos-dl` after per-engine classify).
#[must_use]
pub fn merge_taxonomies(mut parts: Vec<Taxonomy>) -> Taxonomy {
    let mut subsumptions = Vec::new();
    for t in &mut parts {
        subsumptions.append(&mut t.subsumptions);
    }
    subsumptions.sort_unstable_by_key(|(a, b)| (a.0, b.0));
    subsumptions.dedup();
    Taxonomy {
        subsumptions,
        equivalences: Vec::new(),
        unsatisfiable: parts.into_iter().flat_map(|t| t.unsatisfiable).collect(),
    }
}

/// Route module to engine name for conformance / CLI.
#[must_use]
pub fn engine_for_profile(profile: OwlProfile) -> &'static str {
    match profile {
        OwlProfile::El | OwlProfile::Ql => "el",
        OwlProfile::Rl => "rl",
        OwlProfile::Dl => "dl",
    }
}
