//! Post-load ontology validation for malformed datatype definitions.

use ontologos_core::{Axiom, ClassExpr, DataExpr, DlAxiom, EntityId, Ontology};

use crate::Error;

/// Reject ontologies with inconsistent datatype definitions or invalid literals.
pub fn validate_loaded_ontology(ontology: &Ontology) -> Result<(), Error> {
    let store = ontology.dl();
    for axiom in store.axioms() {
        match axiom {
            DlAxiom::DatatypeDefinition { range, .. } => validate_data_expr(ontology, *range)?,
            DlAxiom::SubClassOf { sub, sup } => {
                for id in [*sub, *sup] {
                    if let Some(ce) = store.ce(id) {
                        validate_ce_data(ontology, ce)?;
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
            DlAxiom::ObjectPropertyAssertion {
                subject, object, ..
            } if is_blank_individual(ontology, *subject)
                || is_blank_individual(ontology, *object) =>
            {
                validate_blank_object_property_graph(ontology)?;
            }
            _ => {}
        }
    }
    validate_blank_object_property_graph(ontology)?;
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

fn validate_ce_data(ontology: &Ontology, ce: &ClassExpr) -> Result<(), Error> {
    let store = ontology.dl();
    match ce {
        ClassExpr::DataSome { range, .. } | ClassExpr::DataAll { range, .. } => {
            validate_data_expr(ontology, *range)?;
        }
        ClassExpr::DataHasValue { value, .. } => validate_data_expr(ontology, *value)?,
        ClassExpr::And(ops) | ClassExpr::Or(ops) => {
            for op in ops {
                if let Some(inner) = store.ce(*op) {
                    validate_ce_data(ontology, inner)?;
                }
            }
        }
        ClassExpr::Not(inner) => {
            if let Some(inner_ce) = store.ce(*inner) {
                validate_ce_data(ontology, inner_ce)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn validate_data_expr(ontology: &Ontology, de: ontologos_core::DeId) -> Result<(), Error> {
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
            let mut literal_dtypes = std::collections::HashSet::new();
            for &op in ops {
                if let Some(DataExpr::Literal { lexical, datatype }) = store.de(op) {
                    let dt = datatype_iri(ontology, *datatype);
                    validate_literal_lexical(&dt, lexical)?;
                    literal_dtypes.insert(dt);
                }
            }
            if matches!(expr, DataExpr::Or(_)) && literal_dtypes.len() > 1 {
                return Err(Error::Parse(
                    "data oneOf cannot mix literal datatypes".into(),
                ));
            }
        }
        DataExpr::Not(inner) => validate_data_expr(ontology, *inner)?,
        DataExpr::Facet { base, .. } => validate_data_expr(ontology, *base)?,
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
        {
            if is_blank_individual(ontology, *subject) && is_blank_individual(ontology, *object) {
                graph.entry(*subject).or_default().push(*object);
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let Axiom::ObjectPropertyAssertion {
            subject, object, ..
        } = axiom
        {
            if is_blank_individual(ontology, *subject) && is_blank_individual(ontology, *object) {
                graph.entry(*subject).or_default().push(*object);
            }
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
