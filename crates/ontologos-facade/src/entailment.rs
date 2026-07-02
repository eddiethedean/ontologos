use std::collections::HashSet;

use ontologos_core::{
    Axiom, ClassExpr, ConsistencyResult, DlAxiom, EngineKind, EntityId, Ontology, Profile,
    Reasoner, Taxonomy, uses_dl_entailment,
};

use crate::classify::{classify, taxonomy_from_outcome};
use crate::engines::{check_consistency as dispatch_check_consistency, resolve};
use crate::error::{EntailmentCheck, Error, Result};
use crate::lookup::{lookup_class, lookup_individual, lookup_object_property};

/// Check ontology consistency for the configured profile.
pub fn check_consistency(reasoner: &Reasoner) -> Result<ConsistencyResult> {
    let route = resolve(reasoner)?;
    let result = dispatch_check_consistency(route, reasoner)?;
    tracing::info!(
        complete = result.complete,
        consistent = result.consistent,
        profile = ?reasoner.profile(),
        "consistency check finished"
    );
    Ok(result)
}

/// Check ontology consistency (errors when the answer is incomplete).
pub fn is_consistent(reasoner: &Reasoner) -> Result<bool> {
    check_consistency(reasoner)?
        .into_bool()
        .map_err(Error::Core)
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
    let route = resolve(reasoner)?;
    if matches!(route.kind, EngineKind::Rl | EngineKind::Rdfs) {
        classify(reasoner)?;
        return Ok(named_class_subsumed(reasoner.ontology(), sub, sup));
    }
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

fn named_class_subsumed(ontology: &Ontology, subclass: EntityId, superclass: EntityId) -> bool {
    if ontology.direct_superclasses(subclass).contains(&superclass) {
        return true;
    }
    transitive_superclasses(ontology, subclass, superclass)
}

fn transitive_superclasses(ontology: &Ontology, subclass: EntityId, target: EntityId) -> bool {
    let mut stack: Vec<EntityId> = ontology.direct_superclasses(subclass).to_vec();
    let mut seen = HashSet::new();
    while let Some(current) = stack.pop() {
        if !seen.insert(current) {
            continue;
        }
        if current == target {
            return true;
        }
        stack.extend_from_slice(ontology.direct_superclasses(current));
    }
    false
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

    let route = resolve(reasoner)?;
    if uses_dl_entailment(route.kind) {
        return dl_entails_class_assertion(ontology, individual, class);
    }
    if matches!(route.kind, EngineKind::El) && needs_dl_class_assertion(ontology, individual, class)
    {
        return dl_entails_class_assertion(ontology, individual, class);
    }
    taxonomy_entails_class_assertion(reasoner, individual, class)
}

fn needs_dl_class_assertion(ontology: &Ontology, individual: EntityId, _class: EntityId) -> bool {
    ontology.dl().axioms().any(|axiom| {
        let DlAxiom::ClassAssertion {
            individual: ind,
            class: ce,
        } = axiom
        else {
            return false;
        };
        *ind == individual
            && ontology
                .dl()
                .ce(*ce)
                .is_some_and(|expr| !matches!(expr, ClassExpr::Atomic(_)))
    })
}

fn taxonomy_entails_class_assertion(
    reasoner: &mut Reasoner,
    individual: EntityId,
    class: EntityId,
) -> Result<bool> {
    if matches!(reasoner.profile(), Profile::Rl | Profile::Rdfs) {
        let mut working = reasoner.ontology().clone();
        if reasoner.profile() == Profile::Rl {
            ontologos_rl::abox::materialize_abox(&mut working)?;
        }
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
        if let Some(tax) = taxonomy
            && tax.is_subsumed(asserted, class)
        {
            return true;
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

    let route = resolve(reasoner)?;
    if uses_dl_entailment(route.kind) {
        return dl_entails_object_property_assertion(ontology, subject, property, object);
    }
    abox_entails_object_property_assertion(ontology, subject, property, object)
}

fn abox_entails_object_property_assertion(
    ontology: &ontologos_core::Ontology,
    subject: EntityId,
    property: EntityId,
    object: EntityId,
) -> Result<bool> {
    let mut working = ontology.clone();
    let values = ontologos_rl::abox::object_property_values(&mut working, subject, property)?;
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
    if let Some(cluster) = ontology.same_as(left)
        && cluster.contains(&right)
    {
        return true;
    }
    if let Some(cluster) = ontology.same_as(right)
        && cluster.contains(&left)
    {
        return true;
    }
    false
}
