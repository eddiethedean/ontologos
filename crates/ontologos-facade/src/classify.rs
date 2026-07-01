use ontologos_core::{Reasoner, Taxonomy};
use ontologos_el::ClassifyOutcome;

use crate::engines::EngineRegistry;
use crate::error::Result;

/// Classify using any supported profile (EL, RL, RDFS, ALC, DL, SWRL, Auto).
#[tracing::instrument(skip(reasoner), fields(profile = ?reasoner.profile()))]
pub fn classify(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    let route = EngineRegistry::resolve(reasoner)?;
    tracing::debug!(engine = ?route.kind, "resolved classify route");
    let outcome = EngineRegistry::classify(route, reasoner)?;
    if let Some(taxonomy) = taxonomy_from_outcome(&outcome) {
        reasoner.set_cached_taxonomy(taxonomy.clone());
    } else {
        reasoner.invalidate_classify_cache();
    }
    Ok(outcome)
}

/// Extract taxonomy when the outcome is classification-shaped.
#[must_use]
pub fn taxonomy_from_outcome(outcome: &ClassifyOutcome) -> Option<&Taxonomy> {
    match outcome {
        ClassifyOutcome::Taxonomy(t) => Some(t),
        _ => None,
    }
}
