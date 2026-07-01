use ontologos_core::{EntityId, EntityKind, Reasoner, RoleExpr};

use crate::engines::{resolve, sub_object_properties as dispatch_sub_object_properties};
use crate::error::{Error, Result};

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
    values.iter().map(|id| entity_iri(ontology, *id)).collect()
}

/// OWL API `getSubObjectProperties` for a named property IRI.
pub fn get_sub_object_properties(
    reasoner: &Reasoner,
    property_iri: &str,
    direct: bool,
) -> Result<Vec<String>> {
    let ontology = reasoner.ontology();
    let property = lookup_object_property(ontology, property_iri)?;
    let route = resolve(reasoner)?;
    let roles = dispatch_sub_object_properties(route, reasoner, property, direct)?;

    let mut out: Vec<String> = roles
        .iter()
        .filter_map(|expr| role_expr_iri(ontology, expr).ok())
        .collect();
    out.sort();
    out.dedup();
    Ok(out)
}

pub(crate) fn lookup_class(ontology: &ontologos_core::Ontology, iri: &str) -> Result<EntityId> {
    let id = ontology.lookup_entity(iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Message(format!(
            "unknown class IRI: {iri}"
        )))
    })?;
    if ontology.entity(id).ok().map(|r| r.kind) != Some(EntityKind::Class) {
        return Err(Error::El(ontologos_el::Error::Message(format!(
            "expected class IRI: {iri}"
        ))));
    }
    Ok(id)
}

pub(crate) fn lookup_individual(
    ontology: &ontologos_core::Ontology,
    iri: &str,
) -> Result<EntityId> {
    let id = ontology.lookup_entity(iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Message(format!(
            "unknown individual IRI: {iri}"
        )))
    })?;
    if ontology.entity(id).ok().map(|r| r.kind) != Some(EntityKind::Individual) {
        return Err(Error::El(ontologos_el::Error::Message(format!(
            "expected individual IRI: {iri}"
        ))));
    }
    Ok(id)
}

pub(crate) fn lookup_object_property(
    ontology: &ontologos_core::Ontology,
    iri: &str,
) -> Result<EntityId> {
    let id = ontology.lookup_entity(iri).ok_or_else(|| {
        Error::El(ontologos_el::Error::Message(format!(
            "unknown object property IRI: {iri}"
        )))
    })?;
    if ontology.entity(id).ok().map(|r| r.kind) != Some(EntityKind::ObjectProperty) {
        return Err(Error::El(ontologos_el::Error::Message(format!(
            "expected object property IRI: {iri}"
        ))));
    }
    Ok(id)
}

pub(crate) fn entity_iri(ontology: &ontologos_core::Ontology, id: EntityId) -> Result<String> {
    let record = ontology.entity(id).map_err(|e| Error::El(e.into()))?;
    ontology
        .resolve_iri(record.iri)
        .map(|s| s.to_owned())
        .map_err(|e| Error::El(e.into()))
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

pub(crate) fn index_sub_object_properties(
    ontology: &ontologos_core::Ontology,
    property: EntityId,
    direct: bool,
) -> std::collections::HashSet<RoleExpr> {
    let mut out = std::collections::HashSet::new();
    if direct {
        for &sub in ontology.direct_subproperties(property) {
            out.insert(RoleExpr::Atomic(sub));
        }
        return out;
    }
    let mut frontier = ontology.direct_subproperties(property).to_vec();
    let mut seen = std::collections::HashSet::new();
    while let Some(prop) = frontier.pop() {
        if seen.insert(prop) {
            out.insert(RoleExpr::Atomic(prop));
            frontier.extend_from_slice(ontology.direct_subproperties(prop));
        }
    }
    out
}
