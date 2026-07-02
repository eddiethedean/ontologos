//! Post-load ontology validation for malformed datatype definitions.

use ontologos_core::{Axiom, ClassExpr, DataExpr, DlAxiom, EntityId, Ontology};

use crate::Error;

/// Lightweight validation run after every successful load.
pub fn validate_loaded_ontology_light(ontology: &Ontology) -> Result<(), Error> {
    validate_ontology_datatypes(ontology, false)
}

/// Expensive blank-node graph validation for strict loads.
pub(crate) fn validate_loaded_ontology_strict_graph(ontology: &Ontology) -> Result<(), Error> {
    validate_ontology_datatypes(ontology, true)?;
    validate_blank_object_property_graph(ontology)
}

fn validate_ontology_datatypes(ontology: &Ontology, strict: bool) -> Result<(), Error> {
    let store = ontology.dl();
    for axiom in store.axioms() {
        match axiom {
            DlAxiom::DatatypeDefinition { range, .. } => {
                validate_data_expr(ontology, *range, strict)?;
            }
            DlAxiom::SubClassOf { sub, sup } => {
                for id in [*sub, *sup] {
                    if let Some(ce) = store.ce(id) {
                        validate_ce_data(ontology, ce, strict)?;
                    }
                }
            }
            DlAxiom::SameIndividual(ids) | DlAxiom::DifferentIndividuals(ids)
                if individuals_mix_named_and_blank(ontology, ids) =>
            {
                return Err(Error::Parse(
                    "same/different individuals cannot mix named and blank nodes".into(),
                ));
            }
            _ => {}
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        match axiom {
            Axiom::SameIndividual(ids) | Axiom::DifferentIndividuals(ids)
                if individuals_mix_named_and_blank(ontology, ids) =>
            {
                return Err(Error::Parse(
                    "same/different individuals cannot mix named and blank nodes".into(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

/// Full validation including blank-node assertion and graph cycle checks.
pub fn validate_loaded_ontology(ontology: &Ontology) -> Result<(), Error> {
    validate_loaded_ontology_light(ontology)?;
    validate_data_oneof_homogeneity(ontology)?;
    validate_blank_node_assertions(ontology)?;
    validate_blank_object_property_graph(ontology)
}

fn validate_data_oneof_homogeneity(ontology: &Ontology) -> Result<(), Error> {
    let store = ontology.dl();
    for axiom in store.axioms() {
        match axiom {
            DlAxiom::DatatypeDefinition { range, .. } => {
                validate_data_expr_oneof_homogeneity(ontology, *range)?;
            }
            DlAxiom::SubClassOf { sub, sup } => {
                for id in [*sub, *sup] {
                    if let Some(ce) = store.ce(id) {
                        validate_ce_oneof_homogeneity(ontology, ce)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_ce_oneof_homogeneity(ontology: &Ontology, ce: &ClassExpr) -> Result<(), Error> {
    let store = ontology.dl();
    match ce {
        ClassExpr::DataSome { range, .. } | ClassExpr::DataAll { range, .. } => {
            validate_data_expr_oneof_homogeneity(ontology, *range)?;
        }
        ClassExpr::DataHasValue { value, .. } => {
            validate_data_expr_oneof_homogeneity(ontology, *value)?;
        }
        ClassExpr::And(ops) | ClassExpr::Or(ops) => {
            for op in ops {
                if let Some(inner) = store.ce(*op) {
                    validate_ce_oneof_homogeneity(ontology, inner)?;
                }
            }
        }
        ClassExpr::Not(inner) => {
            if let Some(inner_ce) = store.ce(*inner) {
                validate_ce_oneof_homogeneity(ontology, inner_ce)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_data_expr_oneof_homogeneity(
    ontology: &Ontology,
    de: ontologos_core::DeId,
) -> Result<(), Error> {
    let store = ontology.dl();
    let Some(expr) = store.de(de) else {
        return Ok(());
    };
    match expr {
        DataExpr::Or(ops) | DataExpr::And(ops) => {
            let mut literal_datatypes = Vec::new();
            for &op in ops {
                if let Some(DataExpr::Literal { datatype, .. }) = store.de(op) {
                    literal_datatypes.push(*datatype);
                } else {
                    validate_data_expr_oneof_homogeneity(ontology, op)?;
                }
            }
            if literal_datatypes.len() == ops.len() && literal_datatypes.len() > 1 {
                let first = literal_datatypes[0];
                if literal_datatypes.iter().any(|dt| *dt != first) {
                    return Err(Error::Parse(
                        "DataOneOf literals must share the same datatype".into(),
                    ));
                }
            }
        }
        DataExpr::Not(inner) => validate_data_expr_oneof_homogeneity(ontology, *inner)?,
        DataExpr::Facet { base, .. } => validate_data_expr_oneof_homogeneity(ontology, *base)?,
        DataExpr::Literal { .. } | DataExpr::Datatype(_) | DataExpr::Top => {}
    }
    Ok(())
}

fn validate_blank_node_assertions(ontology: &Ontology) -> Result<(), Error> {
    for axiom in ontology.dl().axioms() {
        match axiom {
            DlAxiom::NegativeObjectPropertyAssertion {
                subject, object, ..
            } if is_blank_individual(ontology, *subject)
                || is_blank_individual(ontology, *object) =>
            {
                return Err(Error::Parse(
                    "negative object property assertions cannot use blank nodes".into(),
                ));
            }
            DlAxiom::NegativeDataPropertyAssertion { subject, .. }
                if is_blank_individual(ontology, *subject) =>
            {
                return Err(Error::Parse(
                    "negative data property assertions cannot use blank nodes".into(),
                ));
            }
            DlAxiom::DataPropertyAssertion { subject, .. }
                if is_blank_individual(ontology, *subject) =>
            {
                return Err(Error::Parse(
                    "data property assertions cannot use blank nodes".into(),
                ));
            }
            _ => {}
        }
    }
    Ok(())
}

fn validate_ce_data(ontology: &Ontology, ce: &ClassExpr, strict: bool) -> Result<(), Error> {
    let store = ontology.dl();
    match ce {
        ClassExpr::DataSome { range, .. } | ClassExpr::DataAll { range, .. } => {
            validate_data_expr(ontology, *range, strict)?;
        }
        ClassExpr::DataHasValue { value, .. } => validate_data_expr(ontology, *value, strict)?,
        ClassExpr::And(ops) | ClassExpr::Or(ops) => {
            for op in ops {
                if let Some(inner) = store.ce(*op) {
                    validate_ce_data(ontology, inner, strict)?;
                }
            }
        }
        ClassExpr::Not(inner) => {
            if let Some(inner_ce) = store.ce(*inner) {
                validate_ce_data(ontology, inner_ce, strict)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_data_expr(
    ontology: &Ontology,
    de: ontologos_core::DeId,
    _strict: bool,
) -> Result<(), Error> {
    let store = ontology.dl();
    let Some(expr) = store.de(de) else {
        return Ok(());
    };
    match expr {
        DataExpr::Literal { lexical, datatype } => {
            let dt = datatype_iri(ontology, *datatype);
            validate_literal_lexical(&dt, lexical)?;
        }
        DataExpr::Or(ops) | DataExpr::And(ops) => {
            for &op in ops {
                if let Some(DataExpr::Literal { lexical, datatype }) = store.de(op) {
                    let dt = datatype_iri(ontology, *datatype);
                    validate_literal_lexical(&dt, lexical)?;
                } else {
                    validate_data_expr(ontology, op, _strict)?;
                }
            }
        }
        DataExpr::Not(inner) => validate_data_expr(ontology, *inner, _strict)?,
        DataExpr::Facet { base, .. } => validate_data_expr(ontology, *base, _strict)?,
        DataExpr::Datatype(_) | DataExpr::Top => {}
    }
    Ok(())
}

fn datatype_iri(ontology: &Ontology, id: EntityId) -> String {
    ontology
        .entity(id)
        .ok()
        .and_then(|record| ontology.resolve_iri(record.iri).ok())
        .unwrap_or("")
        .to_string()
}

fn validate_literal_lexical(datatype_iri: &str, lexical: &str) -> Result<(), Error> {
    if (datatype_iri.contains("integer") || datatype_iri.ends_with("#int"))
        && lexical.parse::<i64>().is_err()
    {
        return Err(Error::Parse(format!(
            "invalid xsd:integer literal {lexical:?}"
        )));
    }
    if datatype_iri.contains("short") && lexical.parse::<i16>().is_err() {
        return Err(Error::Parse(format!(
            "invalid xsd:short literal {lexical:?}"
        )));
    }
    Ok(())
}

fn individuals_mix_named_and_blank(ontology: &Ontology, ids: &[EntityId]) -> bool {
    let mut named = false;
    let mut blank = false;
    for &id in ids {
        if is_blank_individual(ontology, id) {
            blank = true;
        } else {
            named = true;
        }
    }
    named && blank
}

fn is_blank_individual(ontology: &Ontology, id: EntityId) -> bool {
    ontology
        .entity(id)
        .ok()
        .and_then(|record| ontology.resolve_iri(record.iri).ok())
        .is_some_and(|iri| {
            iri.contains("#_")
                || iri.contains("anon")
                || iri.contains("/.genid-")
                || iri.contains("urn:ontologos:anon:")
        })
}

/// Reject cyclic blank-node chains in object property assertions.
fn validate_blank_object_property_graph(ontology: &Ontology) -> Result<(), Error> {
    use std::collections::{HashMap, HashSet};

    let mut graph: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::ObjectPropertyAssertion {
            subject, object, ..
        } = axiom
            && is_blank_individual(ontology, *subject)
            && is_blank_individual(ontology, *object)
        {
            graph.entry(*subject).or_default().push(*object);
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ObjectPropertyAssertion {
            subject, object, ..
        } = axiom
            && is_blank_individual(ontology, *subject)
            && is_blank_individual(ontology, *object)
        {
            graph.entry(*subject).or_default().push(*object);
        }
    }
    for &start in graph.keys() {
        let mut stack = vec![(start, HashSet::from([start]))];
        while let Some((node, path)) = stack.pop() {
            for &next in graph.get(&node).into_iter().flatten() {
                if next == start && path.len() > 1 {
                    return Err(Error::Parse(
                        "cyclic blank-node object property chain".into(),
                    ));
                }
                if path.contains(&next) {
                    return Err(Error::Parse(
                        "cyclic blank-node object property chain".into(),
                    ));
                }
                let mut next_path = path.clone();
                next_path.insert(next);
                stack.push((next, next_path));
            }
        }
    }
    Ok(())
}

/// Reject IRIs that would break OWL Functional Syntax interpolation in supplements.
pub(crate) fn validate_supplement_iri(iri: &str) -> Result<(), Error> {
    ontologos_core::validate_iri(iri).map_err(|e| Error::Parse(e.to_string()))?;
    if iri
        .bytes()
        .any(|b| matches!(b, b'>' | b')' | b'\n' | b'\r'))
    {
        return Err(Error::Parse(format!(
            "supplement IRI contains OWL functional metacharacters: {iri}"
        )));
    }
    Ok(())
}

/// Validate every IRI embedded in `<...>` tokens inside a supplement OFN body.
pub(crate) fn validate_supplement_ofn_body(body: &str) -> Result<(), Error> {
    let mut rest = body;
    while let Some(start) = rest.find('<') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('>') else {
            return Err(Error::Parse(
                "unterminated IRI in supplement OFN body".into(),
            ));
        };
        let iri = &after[..end];
        validate_supplement_iri(iri)?;
        rest = &after[end + 1..];
    }
    Ok(())
}
