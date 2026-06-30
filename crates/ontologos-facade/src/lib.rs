//! Unified reasoner facade — routes all OWL profiles without circular crate deps.

#![warn(missing_docs)]

use std::collections::HashSet;

use ontologos_core::{Axiom, EntityId, EntityKind, Profile, Reasoner, RoleExpr, Taxonomy};
use ontologos_el::{classify_with_profile as el_classify, ClassifyOutcome, ElClassifier};
use ontologos_profile::{
    classify_hybrid, detect_profile, merge_taxonomies, subontology_with_axioms, OwlProfile,
};
use serde::Serialize;
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
    /// ABox engine error.
    #[error(transparent)]
    Abox(#[from] ontologos_abox::Error),
}

/// Axiom-shaped entailment checks for [`is_entailed_axiom`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum EntailmentCheck {
    /// Named class subsumption `SubClassOf(sub, sup)`.
    SubClassOf {
        /// Subclass IRI.
        sub: String,
        /// Superclass IRI.
        sup: String,
    },
    /// `ClassAssertion(individual, class)` with named classes.
    ClassAssertion {
        /// Individual IRI.
        individual: String,
        /// Class IRI.
        class: String,
    },
    /// `ObjectPropertyAssertion(subject, property, object)`.
    ObjectPropertyAssertion {
        /// Subject individual IRI.
        subject: String,
        /// Object property IRI.
        property: String,
        /// Object individual IRI.
        object: String,
    },
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
        Profile::Rl => rl_is_consistent(reasoner.ontology()),
        Profile::Rdfs => rdfs_is_consistent(reasoner.ontology()),
    }
}

fn rl_is_consistent(ontology: &ontologos_core::Ontology) -> Result<bool> {
    let mut working = ontology.clone();
    let report = ontologos_rl::RlEngine::new(1)
        .saturate(&mut working)
        .map_err(|e| Error::El(ontologos_el::Error::Profile(format!("rl saturate: {e}"))))?;
    if !report.clashes.is_empty() || ontologos_bridge::has_bottom_chain_violation(&working) {
        return Ok(false);
    }
    ontologos_abox::is_abox_consistent(&working).map_err(|e| {
        Error::El(ontologos_el::Error::Profile(format!(
            "abox consistent: {e}"
        )))
    })
}

fn rdfs_is_consistent(ontology: &ontologos_core::Ontology) -> Result<bool> {
    let mut working = ontology.clone();
    let report = ontologos_rdfs::RdfsEngine::new()
        .materialize(&mut working)
        .map_err(|e| {
            Error::El(ontologos_el::Error::Profile(format!(
                "rdfs materialize: {e}"
            )))
        })?;
    Ok(report.clashes.is_empty())
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
        Some(OwlProfile::Rl) => rl_is_consistent(reasoner.ontology()),
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

/// Whether named class `sub_iri` is entailed to be subsumed by `sup_iri` after classification.
pub fn is_subsumption_entailed(
    reasoner: &mut Reasoner,
    sub_iri: &str,
    sup_iri: &str,
) -> Result<bool> {
    let outcome = classify(reasoner)?;
    let taxonomy = taxonomy_from_outcome(&outcome).ok_or_else(|| {
        Error::El(ontologos_el::Error::Profile(
            "profile did not produce a taxonomy".into(),
        ))
    })?;
    let ontology = reasoner.ontology();
    let sub = ontology.lookup_entity(sub_iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Profile(format!(
            "unknown class IRI: {sub_iri}"
        )))
    })?;
    let sup = ontology.lookup_entity(sup_iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Profile(format!(
            "unknown class IRI: {sup_iri}"
        )))
    })?;
    Ok(taxonomy.is_subsumed(sub, sup))
}

/// OWLReasoner-style class subsumption entailment (`isEntailed` for `SubClassOf`).
pub fn is_entailed(reasoner: &mut Reasoner, sub_iri: &str, sup_iri: &str) -> Result<bool> {
    is_entailed_axiom(
        reasoner,
        EntailmentCheck::SubClassOf {
            sub: sub_iri.to_owned(),
            sup: sup_iri.to_owned(),
        },
    )
}

/// General `isEntailed` for common axiom types (`SubClassOf`, `ClassAssertion`, `ObjectPropertyAssertion`).
pub fn is_entailed_axiom(reasoner: &mut Reasoner, check: EntailmentCheck) -> Result<bool> {
    match check {
        EntailmentCheck::SubClassOf { sub, sup } => is_subsumption_entailed(reasoner, &sub, &sup),
        EntailmentCheck::ClassAssertion { individual, class } => {
            is_class_assertion_entailed(reasoner, &individual, &class)
        }
        EntailmentCheck::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } => is_object_property_assertion_entailed(reasoner, &subject, &property, &object),
    }
}

fn is_class_assertion_entailed(
    reasoner: &mut Reasoner,
    individual_iri: &str,
    class_iri: &str,
) -> Result<bool> {
    let ontology = reasoner.ontology();
    let individual = lookup_individual(ontology, individual_iri)?;
    let class = lookup_class(ontology, class_iri)?;

    match reasoner.profile() {
        Profile::Alc | Profile::Dl | Profile::Swrl => {
            dl_entails_class_assertion(ontology, individual, class)
        }
        Profile::Auto => {
            let report = detect_profile(ontology)
                .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
            if report.detected == Some(OwlProfile::Dl) {
                dl_entails_class_assertion(ontology, individual, class)
            } else {
                taxonomy_entails_class_assertion(reasoner, individual, class)
            }
        }
        _ => taxonomy_entails_class_assertion(reasoner, individual, class),
    }
}

fn taxonomy_entails_class_assertion(
    reasoner: &mut Reasoner,
    individual: EntityId,
    class: EntityId,
) -> Result<bool> {
    if reasoner.profile() == Profile::Rl {
        let mut working = reasoner.ontology().clone();
        ontologos_abox::materialize_abox(&mut working)?;
        return Ok(individual_entails_named_class(&working, individual, class, None));
    }

    let outcome = classify(reasoner)?;
    let taxonomy = taxonomy_from_outcome(&outcome).ok_or_else(|| {
        Error::El(ontologos_el::Error::Profile(
            "profile did not produce a taxonomy for ClassAssertion entailment".into(),
        ))
    })?;
    Ok(individual_entails_named_class(
        reasoner.ontology(),
        individual,
        class,
        Some(taxonomy),
    ))
}

fn individual_entails_named_class(
    ontology: &ontologos_core::Ontology,
    individual: EntityId,
    class: EntityId,
    taxonomy: Option<&Taxonomy>,
) -> bool {
    for &asserted in ontology.classes_of(individual) {
        if asserted == class {
            return true;
        }
        if let Some(tax) = taxonomy {
            if tax.is_subsumed(asserted, class) {
                return true;
            }
        }
    }
    false
}

fn dl_entails_class_assertion(
    ontology: &ontologos_core::Ontology,
    individual: EntityId,
    class: EntityId,
) -> Result<bool> {
    let store = ontology.dl();
    let class_ce = store
        .expressions()
        .find_map(|(id, expr)| match expr {
            ontologos_core::ClassExpr::Atomic(entity) if *entity == class => Some(id),
            _ => None,
        })
        .ok_or_else(|| {
            Error::El(ontologos_el::Error::Profile(format!(
                "no atomic class expression for {class:?}"
            )))
        })?;
    ontologos_dl::entails_class_assertion(ontology, individual, class_ce).map_err(Error::Dl)
}

fn is_object_property_assertion_entailed(
    reasoner: &mut Reasoner,
    subject_iri: &str,
    property_iri: &str,
    object_iri: &str,
) -> Result<bool> {
    let ontology = reasoner.ontology();
    let subject = lookup_individual(ontology, subject_iri)?;
    let property = lookup_object_property(ontology, property_iri)?;
    let object = lookup_individual(ontology, object_iri)?;

    match reasoner.profile() {
        Profile::Alc | Profile::Dl | Profile::Swrl => {
            dl_entails_object_property_assertion(ontology, subject, property, object)
        }
        Profile::Auto => {
            let report = detect_profile(ontology)
                .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
            if report.detected == Some(OwlProfile::Dl) {
                dl_entails_object_property_assertion(ontology, subject, property, object)
            } else {
                abox_entails_object_property_assertion(ontology, subject, property, object)
            }
        }
        _ => abox_entails_object_property_assertion(ontology, subject, property, object),
    }
}

fn abox_entails_object_property_assertion(
    ontology: &ontologos_core::Ontology,
    subject: EntityId,
    property: EntityId,
    object: EntityId,
) -> Result<bool> {
    let mut working = ontology.clone();
    let values = ontologos_abox::object_property_values(&mut working, subject, property)?;
    Ok(values
        .iter()
        .any(|&candidate| individuals_entailed_same(ontology, candidate, object)))
}

fn dl_entails_object_property_assertion(
    ontology: &ontologos_core::Ontology,
    subject: EntityId,
    property: EntityId,
    object: EntityId,
) -> Result<bool> {
    let mut test = ontology.clone();
    test.add_axiom(Axiom::NegativeObjectPropertyAssertion {
        subject,
        property,
        object,
    })
    .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
    Ok(!ontologos_dl::is_consistent(&test)?)
}

fn individuals_entailed_same(
    ontology: &ontologos_core::Ontology,
    left: EntityId,
    right: EntityId,
) -> bool {
    if left == right {
        return true;
    }
    if let Some(cluster) = ontology.same_as(left) {
        if cluster.contains(&right) {
            return true;
        }
    }
    if let Some(cluster) = ontology.same_as(right) {
        if cluster.contains(&left) {
            return true;
        }
    }
    false
}

/// OWL API `getObjectPropertyValues` for named individuals and properties.
pub fn get_object_property_values(
    reasoner: &Reasoner,
    subject_iri: &str,
    property_iri: &str,
) -> Result<Vec<String>> {
    let ontology = reasoner.ontology();
    let subject = lookup_individual(ontology, subject_iri)?;
    let property = lookup_object_property(ontology, property_iri)?;
    let mut working = ontology.clone();
    let values = ontologos_abox::object_property_values(&mut working, subject, property)?;
    values
        .iter()
        .map(|id| entity_iri(ontology, *id))
        .collect()
}

/// OWL API `getSubObjectProperties` for a named property IRI.
pub fn get_sub_object_properties(
    reasoner: &Reasoner,
    property_iri: &str,
    direct: bool,
) -> Result<Vec<String>> {
    let ontology = reasoner.ontology();
    let property = lookup_object_property(ontology, property_iri)?;
    let role = RoleExpr::Atomic(property);

    let roles = match reasoner.profile() {
        Profile::Alc | Profile::Dl | Profile::Swrl => {
            ontologos_dl::sub_object_property_expressions(ontology, &role, direct)?
        }
        Profile::Auto => {
            let report = detect_profile(ontology)
                .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
            if report.detected == Some(OwlProfile::Dl) {
                ontologos_dl::sub_object_property_expressions(ontology, &role, direct)?
            } else {
                index_sub_object_properties(ontology, property, direct)
            }
        }
        _ => index_sub_object_properties(ontology, property, direct),
    };

    let mut out: Vec<String> = roles
        .iter()
        .filter_map(|expr| role_expr_iri(ontology, expr).ok())
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

fn index_sub_object_properties(
    ontology: &ontologos_core::Ontology,
    property: EntityId,
    direct: bool,
) -> HashSet<RoleExpr> {
    let mut out = HashSet::new();
    if direct {
        for &sub in ontology.direct_subproperties(property) {
            out.insert(RoleExpr::Atomic(sub));
        }
        return out;
    }
    let mut frontier = ontology.direct_subproperties(property).to_vec();
    let mut seen = HashSet::new();
    while let Some(prop) = frontier.pop() {
        if seen.insert(prop) {
            out.insert(RoleExpr::Atomic(prop));
            frontier.extend_from_slice(ontology.direct_subproperties(prop));
        }
    }
    out
}

fn lookup_class(ontology: &ontologos_core::Ontology, iri: &str) -> Result<EntityId> {
    let id = ontology.lookup_entity(iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Profile(format!("unknown class IRI: {iri}")))
    })?;
    if ontology.entity(id).ok().map(|r| r.kind) != Some(EntityKind::Class) {
        return Err(Error::El(ontologos_el::Error::Profile(format!(
            "expected class IRI: {iri}"
        ))));
    }
    Ok(id)
}

fn lookup_individual(ontology: &ontologos_core::Ontology, iri: &str) -> Result<EntityId> {
    let id = ontology.lookup_entity(iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Profile(format!("unknown individual IRI: {iri}")))
    })?;
    if ontology.entity(id).ok().map(|r| r.kind) != Some(EntityKind::Individual) {
        return Err(Error::El(ontologos_el::Error::Profile(format!(
            "expected individual IRI: {iri}"
        ))));
    }
    Ok(id)
}

fn lookup_object_property(ontology: &ontologos_core::Ontology, iri: &str) -> Result<EntityId> {
    let id = ontology.lookup_entity(iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Profile(format!(
            "unknown object property IRI: {iri}"
        )))
    })?;
    if ontology.entity(id).ok().map(|r| r.kind) != Some(EntityKind::ObjectProperty) {
        return Err(Error::El(ontologos_el::Error::Profile(format!(
            "expected object property IRI: {iri}"
        ))));
    }
    Ok(id)
}

fn entity_iri(ontology: &ontologos_core::Ontology, id: EntityId) -> Result<String> {
    let record = ontology
        .entity(id)
        .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))?;
    ontology
        .resolve_iri(record.iri)
        .map(|s| s.to_owned())
        .map_err(|e| Error::El(ontologos_el::Error::Profile(e.to_string())))
}

fn role_expr_iri(ontology: &ontologos_core::Ontology, role: &RoleExpr) -> Result<String> {
    match role {
        RoleExpr::Atomic(id) => entity_iri(ontology, *id),
        RoleExpr::Inverse(id) => {
            let inner = entity_iri(ontology, *id)?;
            Ok(format!("inverse({inner})"))
        }
    }
}

/// Query handle over a classified ontology (call after [`classify`]).
pub fn query_engine<'a>(
    ontology: &'a ontologos_core::Ontology,
    taxonomy: &'a Taxonomy,
) -> ontologos_query::QueryEngine<'a> {
    ontologos_query::QueryEngine::new(ontology, taxonomy)
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
    fn is_consistent_rl_saturates_without_dl_tableau() {
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
    fn is_consistent_rl_detects_disjoint_clash() {
        let mut ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .class("http://example.org/B")
            .unwrap()
            .class("http://example.org/D")
            .unwrap()
            .individual("http://example.org/x")
            .unwrap()
            .class_assertion("http://example.org/x", "http://example.org/B")
            .unwrap()
            .class_assertion("http://example.org/x", "http://example.org/D")
            .unwrap()
            .build()
            .unwrap();
        let a = ontology.lookup_entity("http://example.org/A").unwrap();
        let b = ontology.lookup_entity("http://example.org/B").unwrap();
        let d = ontology.lookup_entity("http://example.org/D").unwrap();
        ontology
            .add_axiom(Axiom::EquivalentClasses(vec![a, b]))
            .unwrap();
        ontology
            .add_axiom(Axiom::DisjointClasses(vec![a, d]))
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::Rl)
            .build(ontology)
            .unwrap();
        assert!(!super::is_consistent(&reasoner).unwrap());
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
        assert!(super::is_consistent(&reasoner).unwrap());
    }

    #[test]
    fn is_subsumption_entailed_after_classify() {
        let ontology = el_chain_ontology();
        let mut reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(ontology)
            .unwrap();
        assert!(super::is_subsumption_entailed(
            &mut reasoner,
            "http://example.org/A",
            "http://example.org/C"
        )
        .unwrap());
        assert!(!super::is_subsumption_entailed(
            &mut reasoner,
            "http://example.org/C",
            "http://example.org/A"
        )
        .unwrap());
        assert!(super::is_entailed(
            &mut reasoner,
            "http://example.org/A",
            "http://example.org/C"
        )
        .unwrap());
    }

    #[test]
    fn query_engine_direct_subclasses() {
        let ontology = el_chain_ontology();
        let mut reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(ontology)
            .unwrap();
        let outcome = super::classify(&mut reasoner).unwrap();
        let tax = super::taxonomy_from_outcome(&outcome).expect("taxonomy");
        let q = super::query_engine(reasoner.ontology(), tax);
        let a = reasoner
            .ontology()
            .lookup_entity("http://example.org/A")
            .unwrap();
        let c = reasoner
            .ontology()
            .lookup_entity("http://example.org/C")
            .unwrap();
        assert!(q.is_subsumed(a, c).unwrap());
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

    #[test]
    fn is_entailed_class_assertion_via_subsumption() {
        let ontology = Ontology::builder()
            .class("http://example.org/A")
            .unwrap()
            .class("http://example.org/B")
            .unwrap()
            .individual("http://example.org/x")
            .unwrap()
            .subclass_of("http://example.org/A", "http://example.org/B")
            .unwrap()
            .class_assertion("http://example.org/x", "http://example.org/A")
            .unwrap()
            .build()
            .unwrap();
        let mut reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(ontology)
            .unwrap();
        assert!(super::is_entailed_axiom(
            &mut reasoner,
            super::EntailmentCheck::ClassAssertion {
                individual: "http://example.org/x".into(),
                class: "http://example.org/B".into(),
            }
        )
        .unwrap());
    }

    #[test]
    fn is_entailed_object_property_assertion_after_rl_materialization() {
        let ontology = Ontology::builder()
            .individual("http://example.org/c")
            .unwrap()
            .individual("http://example.org/d")
            .unwrap()
            .object_property("http://example.org/r")
            .unwrap()
            .object_property_assertion(
                "http://example.org/c",
                "http://example.org/r",
                "http://example.org/d",
            )
            .unwrap()
            .build()
            .unwrap();
        let mut reasoner = Reasoner::builder()
            .profile(Profile::Rl)
            .build(ontology)
            .unwrap();
        assert!(super::is_entailed_axiom(
            &mut reasoner,
            super::EntailmentCheck::ObjectPropertyAssertion {
                subject: "http://example.org/c".into(),
                property: "http://example.org/r".into(),
                object: "http://example.org/d".into(),
            }
        )
        .unwrap());
    }

    #[test]
    fn get_object_property_values_returns_fillers() {
        let ontology = Ontology::builder()
            .individual("http://example.org/c")
            .unwrap()
            .individual("http://example.org/d")
            .unwrap()
            .object_property("http://example.org/r")
            .unwrap()
            .object_property_assertion(
                "http://example.org/c",
                "http://example.org/r",
                "http://example.org/d",
            )
            .unwrap()
            .build()
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::Rl)
            .build(ontology)
            .unwrap();
        let values = super::get_object_property_values(
            &reasoner,
            "http://example.org/c",
            "http://example.org/r",
        )
        .unwrap();
        assert_eq!(values, vec!["http://example.org/d"]);
    }

    #[test]
    fn get_sub_object_properties_uses_asserted_hierarchy_for_el() {
        let ontology = Ontology::builder()
            .object_property("http://example.org/p")
            .unwrap()
            .object_property("http://example.org/q")
            .unwrap()
            .subproperty_of("http://example.org/q", "http://example.org/p")
            .unwrap()
            .build()
            .unwrap();
        let reasoner = Reasoner::builder()
            .profile(Profile::El)
            .build(ontology)
            .unwrap();
        let direct = super::get_sub_object_properties(&reasoner, "http://example.org/p", true)
            .unwrap();
        assert_eq!(direct, vec!["http://example.org/q"]);
        let all = super::get_sub_object_properties(&reasoner, "http://example.org/p", false)
            .unwrap();
        assert_eq!(all, vec!["http://example.org/q"]);
    }
}
