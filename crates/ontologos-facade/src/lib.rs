//! Unified reasoner facade — routes all OWL profiles without circular crate deps.

#![warn(missing_docs)]

use ontologos_core::{Profile, Reasoner, Taxonomy};
use ontologos_el::{classify_with_profile as el_classify, ClassifyOutcome, ElClassifier};
use ontologos_profile::{
    classify_hybrid, detect_profile, merge_taxonomies, subontology_with_axioms, OwlProfile,
};
use thiserror::Error;

/// Result type for facade operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Facade routing errors.
#[derive(Debug, Error)]
pub enum Error {
    /// EL engine error.
    #[error(transparent)]
    El(#[from] ontologos_el::Error),
    /// ALC engine error.
    #[error(transparent)]
    Alc(#[from] ontologos_alc::Error),
    /// DL engine error.
    #[error(transparent)]
    Dl(#[from] ontologos_dl::Error),
    /// SWRL engine error.
    #[error(transparent)]
    Swrl(#[from] ontologos_swrl::Error),
}

/// Classify using any supported profile (EL, RL, RDFS, ALC, DL, SWRL, Auto).
pub fn classify(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    match reasoner.profile() {
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
    }
}

fn classify_auto(reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
    let ontology = reasoner.ontology();
    let report = detect_profile(ontology)
        .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
    if report.detected == Some(OwlProfile::Dl) {
        return classify_hybrid_auto(ontology);
    }
    el_classify(reasoner).map_err(Error::El)
}

fn classify_hybrid_auto(ontology: &ontologos_core::Ontology) -> Result<ClassifyOutcome> {
    let hybrid = classify_hybrid(ontology)
        .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
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
        let view = subontology_with_axioms(ontology, &module.axiom_ids)
            .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
        let tax = match module.profile {
            OwlProfile::El | OwlProfile::Ql => ElClassifier::new().classify(&view)?,
            OwlProfile::Dl => ontologos_dl::classify(&view)?,
            OwlProfile::Rl => {
                let mut materialized = subontology_with_axioms(ontology, &module.axiom_ids)
                    .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
                ontologos_rl::RlEngine::new(1)
                    .saturate(&mut materialized)
                    .map_err(|e| {
                        Error::El(ontologos_el::Error::Profile(format!("rl saturate: {e}")))
                    })?;
                ElClassifier::new().classify(&materialized)?
            }
        };
        parts.push(tax);
    }
    Ok(ClassifyOutcome::Taxonomy(merge_taxonomies(parts)))
}

/// Check ontology consistency for the configured profile.
pub fn is_consistent(reasoner: &Reasoner) -> Result<bool> {
    match reasoner.profile() {
        Profile::Alc => Ok(ontologos_alc::is_consistent(reasoner.ontology())?),
        Profile::Dl | Profile::Swrl => Ok(ontologos_dl::is_consistent(reasoner.ontology())?),
        Profile::Auto => is_consistent_auto(reasoner),
        Profile::El => el_is_consistent(reasoner.ontology()),
        Profile::Rl | Profile::Rdfs => Ok(true),
    }
}

fn el_is_consistent(ontology: &ontologos_core::Ontology) -> Result<bool> {
    ontologos_el::ElClassifier::new()
        .classify(ontology)
        .map(|t| t.unsatisfiable.is_empty())
        .map_err(Error::El)
}

fn is_consistent_auto(reasoner: &Reasoner) -> Result<bool> {
    let report = detect_profile(reasoner.ontology())
        .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
    match report.detected {
        Some(OwlProfile::Dl) => Ok(ontologos_dl::is_consistent(reasoner.ontology())?),
        Some(OwlProfile::El) | Some(OwlProfile::Ql) => el_is_consistent(reasoner.ontology()),
        Some(OwlProfile::Rl) => Ok(true),
        None => Err(Error::El(ontologos_el::Error::Profile(
            "no profile detected".into(),
        ))),
    }
}

/// Extract taxonomy when the outcome is classification-shaped.
#[must_use]
pub fn taxonomy_from_outcome(outcome: &ClassifyOutcome) -> Option<&Taxonomy> {
    match outcome {
        ClassifyOutcome::Taxonomy(t) => Some(t),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner};
    use ontologos_el::ClassifyOutcome;

    fn el_ontology() -> Ontology {
        Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .class("http://example.org/B")
            .unwrap()
            .subclass_of("http://example.org/A", "http://example.org/B")
            .unwrap()
            .build()
            .unwrap()
    }

    fn el_chain_ontology() -> Ontology {
        Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .class("http://example.org/B")
            .unwrap()
            .class("http://example.org/C")
            .unwrap()
            .subclass_of("http://example.org/A", "http://example.org/B")
            .unwrap()
            .subclass_of("http://example.org/B", "http://example.org/C")
            .unwrap()
            .build()
            .unwrap()
    }

    fn unsatisfiable_el_ontology() -> Ontology {
        let mut ontology = Ontology::new();
        let a = ontology
            .entity_id("http://example.org/A", EntityKind::Class)
            .expect("A");
        let nothing = ontology
            .entity_id("http://www.w3.org/2002/07/owl#Nothing", EntityKind::Class)
            .expect("Nothing");
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: a,
                superclass: nothing,
            })
            .expect("A sub Nothing");
        ontology
    }

    fn el_reasoner() -> Reasoner {
        Reasoner::builder()
            .profile(Profile::El)
            .build(el_ontology())
            .unwrap()
    }

    #[test]
    fn classify_el_returns_taxonomy_with_subsumption() {
        let mut reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(el_chain_ontology())
            .unwrap();
        let outcome = super::classify(&mut reasoner).unwrap();
        let tax = super::taxonomy_from_outcome(&outcome).expect("EL should return Taxonomy");
        let a = reasoner
            .ontology()
            .lookup_entity("http://example.org/A")
            .unwrap();
        let c = reasoner
            .ontology()
            .lookup_entity("http://example.org/C")
            .unwrap();
        assert!(tax.is_subsumed(a, c));
    }

    #[test]
    fn classify_rdfs_returns_materialization_report() {
        let mut reasoner = Reasoner::builder()
            .profile(Profile::Rdfs)
            .build(el_ontology())
            .unwrap();
        let outcome = super::classify(&mut reasoner).unwrap();
        assert!(matches!(outcome, ClassifyOutcome::Rdfs(_)));
        assert!(super::taxonomy_from_outcome(&outcome).is_none());
    }

    #[test]
    fn classify_rl_returns_saturation_report() {
        let mut reasoner = Reasoner::builder()
            .profile(Profile::Rl)
            .build(el_ontology())
            .unwrap();
        let outcome = super::classify(&mut reasoner).unwrap();
        assert!(matches!(outcome, ClassifyOutcome::Rl(_)));
        assert!(super::taxonomy_from_outcome(&outcome).is_none());
    }

    #[test]
    fn classify_auto_routes_el_fixture_to_taxonomy() {
        let mut reasoner = Reasoner::builder()
            .profile(Profile::Auto)
            .build(el_chain_ontology())
            .unwrap();
        let outcome = super::classify(&mut reasoner).unwrap();
        let tax = super::taxonomy_from_outcome(&outcome).expect("auto on EL fixture");
        let a = reasoner
            .ontology()
            .lookup_entity("http://example.org/A")
            .unwrap();
        let c = reasoner
            .ontology()
            .lookup_entity("http://example.org/C")
            .unwrap();
        assert!(tax.is_subsumed(a, c));
    }

    #[test]
    fn classify_dl_returns_taxonomy_for_named_subsumption() {
        let mut reasoner = Reasoner::builder()
            .profile(Profile::Dl)
            .build(el_ontology())
            .unwrap();
        let outcome = super::classify(&mut reasoner).unwrap();
        let tax = super::taxonomy_from_outcome(&outcome).expect("DL should return Taxonomy");
        let a = reasoner
            .ontology()
            .lookup_entity("http://example.org/A")
            .unwrap();
        let b = reasoner
            .ontology()
            .lookup_entity("http://example.org/B")
            .unwrap();
        assert!(tax.is_subsumed(a, b));
    }

    #[test]
    fn taxonomy_from_outcome_none_for_rdfs() {
        let mut reasoner = Reasoner::builder()
            .profile(Profile::Rdfs)
            .build(el_ontology())
            .unwrap();
        let outcome = super::classify(&mut reasoner).unwrap();
        assert!(super::taxonomy_from_outcome(&outcome).is_none());
    }

    #[test]
    fn is_consistent_el_uses_el_classifier() {
        let reasoner = el_reasoner();
        assert!(super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_consistent_el_detects_unsatisfiable() {
        let reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(unsatisfiable_el_ontology())
            .unwrap();
        assert!(!super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_consistent_auto_routes_el_to_el_classifier() {
        let reasoner = Reasoner::builder()
            .profile(Profile::Auto)
            .build(el_ontology())
            .unwrap();
        assert!(super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_consistent_rl_returns_true_without_dl_tableau() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .build()
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::Rl)
            .build(ontology)
            .unwrap();
        assert!(super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_consistent_alc_uses_alc_engine() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .build()
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::Alc)
            .build(ontology)
            .unwrap();
        let err = super::is_consistent(&reasoner).expect_err("alc consistency on bare class");
        assert!(matches!(err, super::Error::Alc(_)));
    }

    #[test]
    fn classify_auto_hybrid_partitions_mixed_ontology() {
        let report = ontologos_profile::classify_hybrid(&el_chain_ontology()).expect("hybrid");
        assert!(!report.modules.is_empty());
        let mut reasoner = Reasoner::builder()
            .profile(Profile::Auto)
            .build(el_chain_ontology())
            .unwrap();
        let outcome = super::classify(&mut reasoner).expect("auto classify");
        assert!(super::taxonomy_from_outcome(&outcome).is_some());
    }
}
