use ontologos_core::{Profile, Reasoner, Taxonomy};
use ontologos_el::{ClassifyOutcome, ElClassifier, classify_with_profile as el_classify};
use ontologos_profile::{
    OwlProfile, classify_hybrid, detect_profile, merge_taxonomies, subontology_with_axioms,
};

use crate::error::{Error, Result};

/// Classify using any supported profile (EL, RL, RDFS, ALC, DL, SWRL, Auto).
pub fn classify(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    let outcome = match reasoner.profile() {
        Profile::Alc => Ok(ClassifyOutcome::Taxonomy(ontologos_alc::classify(
            reasoner.ontology(),
        )?)),
        Profile::Dl => Ok(ClassifyOutcome::Taxonomy(ontologos_dl::classify(
            reasoner.ontology(),
        )?)),
        Profile::Swrl => Ok(ClassifyOutcome::Taxonomy(
            ontologos_swrl::classify_with_swrl(reasoner.ontology())?.0,
        )),
        Profile::Auto => classify_auto(reasoner),
        _ => el_classify(reasoner).map_err(Error::El),
    }?;
    if let Some(taxonomy) = taxonomy_from_outcome(&outcome) {
        reasoner.set_cached_taxonomy(taxonomy.clone());
    } else {
        reasoner.invalidate_classify_cache();
    }
    Ok(outcome)
}

fn classify_auto(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    let ontology = reasoner.ontology();
    let report = detect_profile(ontology).map_err(|e| Error::El(e.into()))?;
    if report.detected == Some(OwlProfile::Dl) {
        return classify_hybrid_auto(ontology);
    }
    el_classify(reasoner).map_err(Error::El)
}

fn classify_hybrid_auto(ontology: &ontologos_core::Ontology) -> Result<ClassifyOutcome> {
    let hybrid = classify_hybrid(ontology).map_err(|e| Error::El(e.into()))?;
    if hybrid.modules.len() <= 1 {
        let module = hybrid.modules.first();
        if module.is_some_and(|m| m.profile == OwlProfile::Dl) {
            return Ok(ClassifyOutcome::Taxonomy(ontologos_dl::classify(ontology)?));
        }
        return Ok(ClassifyOutcome::Taxonomy(
            ElClassifier::new().classify(ontology)?,
        ));
    }

    let mut parts = Vec::with_capacity(hybrid.modules.len());
    for module in &hybrid.modules {
        let mut view = subontology_with_axioms(ontology, &module.axiom_ids)
            .map_err(|e| Error::El(e.into()))?;
        let tax = match module.profile {
            OwlProfile::El | OwlProfile::Ql => ElClassifier::new().classify(&view)?,
            OwlProfile::Dl => ontologos_dl::classify(&view)?,
            OwlProfile::Rl => {
                ontologos_rl::RlEngine::new(1)
                    .saturate(&mut view)
                    .map_err(|e| {
                        Error::El(ontologos_el::Error::Message(format!("rl saturate: {e}")))
                    })?;
                ElClassifier::new().classify(&view)?
            }
        };
        parts.push(tax);
    }
    Ok(ClassifyOutcome::Taxonomy(merge_taxonomies(parts)))
}

/// Extract taxonomy when the outcome is classification-shaped.
#[must_use]
pub fn taxonomy_from_outcome(outcome: &ClassifyOutcome) -> Option<&Taxonomy> {
    match outcome {
        ClassifyOutcome::Taxonomy(t) => Some(t),
        _ => None,
    }
}
