//! Hybrid multi-module classification orchestration.

use ontologos_core::Ontology;
use ontologos_profile::{OwlProfile, classify_hybrid, merge_taxonomies, subontology_with_axioms};

use crate::error::{Error, Result};
use crate::outcome::ClassifyOutcome;

/// Classify each EL/RL/DL module and merge taxonomies.
pub(crate) fn classify_hybrid_modules(ontology: &Ontology) -> Result<ClassifyOutcome> {
    let report = classify_hybrid(ontology).map_err(|e| Error::El(e.into()))?;
    if report.modules.len() <= 1 {
        return Ok(ClassifyOutcome::Taxonomy(
            ontologos_dl::classify(ontology).map_err(Error::Dl)?,
        ));
    }

    let mut taxonomies = Vec::with_capacity(report.modules.len());
    for module in &report.modules {
        let sub = subontology_with_axioms(ontology, &module.axiom_ids, module.include_dl_store)
            .map_err(|e| Error::El(e.into()))?;
        let taxonomy = match module.profile {
            OwlProfile::El | OwlProfile::Ql => ontologos_el::ElClassifier::new()
                .classify(&sub)
                .map_err(Error::El)?,
            OwlProfile::Rl => {
                let mut working = sub.clone();
                ontologos_rl::RlEngine::new(1)
                    .saturate(&mut working)
                    .map_err(Error::Rl)?;
                ontologos_el::ElClassifier::new()
                    .classify(&working)
                    .map_err(Error::El)?
            }
            OwlProfile::Dl => ontologos_dl::classify(&sub).map_err(Error::Dl)?,
        };
        taxonomies.push(taxonomy);
    }

    Ok(ClassifyOutcome::Taxonomy(merge_taxonomies(taxonomies)))
}
