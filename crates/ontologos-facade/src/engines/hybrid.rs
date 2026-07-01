//! Hybrid multi-module classification orchestration.

use ontologos_core::Ontology;
use ontologos_el::ElClassifier;
use ontologos_profile::{OwlProfile, classify_hybrid, merge_taxonomies, subontology_with_axioms};

use crate::error::{Error, Result};
use crate::outcome::ClassifyOutcome;

/// Classify a hybrid DL ontology by per-module engine dispatch.
pub(crate) fn classify_hybrid_modules(ontology: &Ontology) -> Result<ClassifyOutcome> {
    let hybrid = classify_hybrid(ontology).map_err(|e| Error::El(e.into()))?;
    debug_assert!(
        hybrid.modules.len() > 1,
        "Hybrid route requires multiple modules"
    );

    let mut parts = Vec::with_capacity(hybrid.modules.len());
    for module in &hybrid.modules {
        let mut view = subontology_with_axioms(ontology, &module.axiom_ids)
            .map_err(|e| Error::El(e.into()))?;
        let tax = match module.profile {
            OwlProfile::El => ElClassifier::new().classify(&view).map_err(Error::El)?,
            OwlProfile::Ql | OwlProfile::Dl => {
                ontologos_dl::classify(&view).map_err(Error::Dl)?
            }
            OwlProfile::Rl => {
                ontologos_rl::RlEngine::new(1)
                    .saturate(&mut view)
                    .map_err(|e| {
                        Error::El(ontologos_el::Error::Message(format!("rl saturate: {e}")))
                    })?;
                ElClassifier::new().classify(&view).map_err(Error::El)?
            }
        };
        parts.push(tax);
    }
    Ok(ClassifyOutcome::Taxonomy(merge_taxonomies(parts)))
}
