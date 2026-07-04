//! Shared conversions between Rust core types and JSON values.

use ontologos_core::{
    Axiom, AxiomId, EntityId, EntityKind, Ontology, ParseMetaSummary, Profile, Taxonomy,
};
use ontologos_explain::{ProofGraph, ProofNode};
use serde_json::{Value, json};

use crate::error::{JsError, Result};

pub fn parse_profile(profile: Option<&str>) -> Result<Profile> {
    match profile.unwrap_or("auto").to_ascii_lowercase().as_str() {
        "auto" => Ok(Profile::Auto),
        "rdfs" => Ok(Profile::Rdfs),
        "rl" => Ok(Profile::Rl),
        "el" => Ok(Profile::El),
        "alc" | "dl" => Ok(Profile::Dl),
        "dl-preview" | "dl_preview" => Ok(Profile::DlPreview),
        "swrl" => Ok(Profile::Swrl),
        other => Err(JsError::Other(format!(
            "unsupported profile {other:?}; use auto, rdfs, rl, el, dl, or dl-preview"
        ))),
    }
}

pub fn entity_iri(ontology: &Ontology, id: EntityId) -> Result<String> {
    let record = ontology
        .entity(id)
        .map_err(|e| JsError::Other(e.to_string()))?;
    ontology
        .resolve_iri(record.iri)
        .map(|s| s.to_owned())
        .map_err(|e| JsError::Other(e.to_string()))
}

pub fn parse_meta_from_ontology(ontology: &Ontology) -> Option<ParseMetaSummary> {
    ontology.parse_meta().map(ParseMetaSummary::from)
}

pub fn parse_meta_value(summary: &ParseMetaSummary) -> Value {
    json!({
        "warnings": summary.warnings,
        "mapped_axiom_count": summary.mapped_axiom_count,
        "skipped_axiom_count": summary.skipped_axiom_count,
        "logical_axiom_count": summary.logical_axiom_count,
    })
}

fn optional_entity_pair(
    ontology: &Ontology,
    pair: Option<(EntityId, EntityId)>,
) -> Result<Option<[String; 2]>> {
    match pair {
        Some((left, right)) => Ok(Some([
            entity_iri(ontology, left)?,
            entity_iri(ontology, right)?,
        ])),
        None => Ok(None),
    }
}

fn optional_entity_triple(
    ontology: &Ontology,
    triple: Option<(EntityId, EntityId, EntityId)>,
) -> Result<Option<[String; 3]>> {
    match triple {
        Some((a, b, c)) => Ok(Some([
            entity_iri(ontology, a)?,
            entity_iri(ontology, b)?,
            entity_iri(ontology, c)?,
        ])),
        None => Ok(None),
    }
}

fn proof_node_value(ontology: &Ontology, node: &ProofNode) -> Result<Value> {
    let mut obj = json!({
        "rule": node.rule,
        "premises": node.premises.iter().map(|id| id.0.to_string()).collect::<Vec<_>>(),
    });
    if let Some(axiom_id) = node.conclusion_axiom {
        obj["conclusion_axiom"] = json!(axiom_id.index().to_string());
    }
    if let Some(pair) = optional_entity_pair(ontology, node.conclusion_sub)? {
        obj["conclusion_sub"] = json!(pair);
    }
    if let Some(triple) = optional_entity_triple(ontology, node.conclusion_existential)? {
        obj["conclusion_existential"] = json!(triple);
    }
    if let Some(pair) = optional_entity_pair(ontology, node.conclusion_subproperty)? {
        obj["conclusion_subproperty"] = json!(pair);
    }
    Ok(obj)
}

pub fn proof_graph_value(
    ontology: &Ontology,
    graph: &ProofGraph,
    parse_meta: Option<&ParseMetaSummary>,
) -> Result<Value> {
    let nodes: Vec<Value> = graph
        .nodes
        .iter()
        .map(|node| proof_node_value(ontology, node))
        .collect::<Result<Vec<_>>>()?;
    let mut obj = json!({
        "node_count": graph.node_count(),
        "nodes": nodes,
    });
    if let Some(summary) = parse_meta {
        obj["parse_meta"] = parse_meta_value(summary);
    }
    Ok(obj)
}

pub fn resolve_class(ontology: &mut Ontology, iri: &str) -> Result<EntityId> {
    ontology
        .entity_id(iri, EntityKind::Class)
        .map_err(JsError::from)
}

pub fn resolve_individual(ontology: &mut Ontology, iri: &str) -> Result<EntityId> {
    ontology
        .entity_id(iri, EntityKind::Individual)
        .map_err(JsError::from)
}

pub fn resolve_object_property(ontology: &mut Ontology, iri: &str) -> Result<EntityId> {
    ontology
        .entity_id(iri, EntityKind::ObjectProperty)
        .map_err(JsError::from)
}

pub fn find_subclass_axiom_id(
    ontology: &Ontology,
    subclass_iri: &str,
    superclass_iri: &str,
) -> Result<Option<AxiomId>> {
    let subclass = ontology
        .try_lookup_entity(subclass_iri)
        .map_err(|e| JsError::Other(e.to_string()))?
        .ok_or_else(|| JsError::Other(format!("unknown class IRI: {subclass_iri}")))?;
    let superclass = ontology
        .try_lookup_entity(superclass_iri)
        .map_err(|e| JsError::Other(e.to_string()))?
        .ok_or_else(|| JsError::Other(format!("unknown class IRI: {superclass_iri}")))?;

    for (id, axiom) in ontology.axioms().iter_asserted() {
        if let Axiom::SubClassOf {
            subclass: sub,
            superclass: sup,
        } = axiom
            && *sub == subclass
            && *sup == superclass
        {
            return Ok(Some(id));
        }
    }
    Ok(None)
}

pub fn taxonomy_classify_value(ontology: &Ontology, taxonomy: &Taxonomy) -> Result<Value> {
    let parse_meta = parse_meta_from_ontology(ontology);
    let json =
        ontologos_facade::taxonomy_json("classified", taxonomy, ontology, parse_meta.as_ref())
            .map_err(JsError::from)?;
    serde_json::to_value(&json).map_err(|e| JsError::Other(e.to_string()))
}

pub fn rdfs_classify_value(
    ontology: &Ontology,
    report: &ontologos_rl::rdfs::MaterializationReport,
) -> Result<Value> {
    let parse_meta = parse_meta_from_ontology(ontology);
    let json =
        ontologos_facade::rdfs_materialization_json("classified", report, parse_meta.as_ref());
    serde_json::to_value(&json).map_err(|e| JsError::Other(e.to_string()))
}

pub fn rl_classify_value(
    ontology: &Ontology,
    report: &ontologos_rl::MaterializationReport,
) -> Result<Value> {
    let parse_meta = parse_meta_from_ontology(ontology);
    let json = ontologos_facade::rl_materialization_json("classified", report, parse_meta.as_ref());
    serde_json::to_value(&json).map_err(|e| JsError::Other(e.to_string()))
}

/// Convert a usize count to u32 for JS bindings, rejecting overflow.
pub fn usize_to_u32(count: usize) -> Result<u32> {
    u32::try_from(count)
        .map_err(|_| JsError::ResourceLimit(format!("count {count} exceeds maximum u32 value")))
}
