use ontologos_core::{Axiom, EntityId, Profile, Reasoner, Taxonomy};
use ontologos_profile::{OwlProfile, detect_profile};

use crate::classify::{classify, taxonomy_from_outcome};
use crate::error::{EntailmentCheck, Error, Result};
use crate::lookup::{lookup_class, lookup_individual, lookup_object_property};

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
        .map_err(|e| Error::El(ontologos_el::Error::Message(format!("rl saturate: {e}"))))?;
    if !report.clashes.is_empty() || ontologos_bridge::has_bottom_chain_violation(&working) {
        return Ok(false);
    }
    ontologos_abox::is_abox_consistent(&working).map_err(|e| {
        Error::El(ontologos_el::Error::Message(format!(
            "abox consistent: {e}"
        )))
    })
}

fn rdfs_is_consistent(ontology: &ontologos_core::Ontology) -> Result<bool> {
    let mut working = ontology.clone();
    let report = ontologos_rdfs::RdfsEngine::new()
        .materialize(&mut working)
        .map_err(|e| {
            Error::El(ontologos_el::Error::Message(format!(
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
    let report = detect_profile(reasoner.ontology()).map_err(|e| Error::El(e.into()))?;
    match report.detected {
        Some(OwlProfile::Dl) => Ok(ontologos_dl::is_consistent(reasoner.ontology())?),
        Some(OwlProfile::El) | Some(OwlProfile::Ql) => el_is_consistent(reasoner.ontology()),
        Some(OwlProfile::Rl) => rl_is_consistent(reasoner.ontology()),
        None => Err(Error::El(ontologos_el::Error::Message(
            "no profile detected".into(),
        ))),
    }
}

/// Whether named class `sub_iri` is entailed to be subsumed by `sup_iri` after classification.
pub fn is_subsumption_entailed(
    reasoner: &mut Reasoner,
    sub_iri: &str,
    sup_iri: &str,
) -> Result<bool> {
    let ontology = reasoner.ontology();
    let sub = ontology.lookup_entity(sub_iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Message(format!(
            "unknown class IRI: {sub_iri}"
        )))
    })?;
    let sup = ontology.lookup_entity(sup_iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Message(format!(
            "unknown class IRI: {sup_iri}"
        )))
    })?;
    if let Some(taxonomy) = reasoner.cached_taxonomy() {
        return Ok(taxonomy.is_subsumed(sub, sup));
    }
    let outcome = classify(reasoner)?;
    let taxonomy = taxonomy_from_outcome(&outcome).ok_or_else(|| {
        Error::El(ontologos_el::Error::Message(
            "profile did not produce a taxonomy".into(),
        ))
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
            let report = detect_profile(ontology).map_err(|e| Error::El(e.into()))?;
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
        return Ok(individual_entails_named_class(
            &working, individual, class, None,
        ));
    }

    if let Some(taxonomy) = reasoner.cached_taxonomy() {
        return Ok(individual_entails_named_class(
            reasoner.ontology(),
            individual,
            class,
            Some(taxonomy),
        ));
    }

    let outcome = classify(reasoner)?;
    let taxonomy = taxonomy_from_outcome(&outcome).ok_or_else(|| {
        Error::El(ontologos_el::Error::Message(
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
            Error::El(ontologos_el::Error::Message(format!(
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
            let report = detect_profile(ontology).map_err(|e| Error::El(e.into()))?;
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
    .map_err(|e| Error::El(e.into()))?;
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
