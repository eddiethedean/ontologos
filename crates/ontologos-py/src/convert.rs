//! Shared conversions between Rust core types and Python dicts.

use ontologos_core::{Axiom, AxiomId, EntityId, EntityKind, Ontology, ParseMetaSummary, Taxonomy};
use ontologos_explain::{ProofGraph, ProofNode};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

pub(crate) fn py_err(message: impl ToString) -> PyErr {
    PyRuntimeError::new_err(message.to_string())
}

pub(crate) fn parse_profile(profile: Option<&str>) -> PyResult<ontologos_core::Profile> {
    use ontologos_core::Profile;
    match profile.unwrap_or("auto").to_ascii_lowercase().as_str() {
        "auto" => Ok(Profile::Auto),
        "rdfs" => Ok(Profile::Rdfs),
        "rl" => Ok(Profile::Rl),
        "el" => Ok(Profile::El),
        "alc" | "dl" => Ok(Profile::Dl),
        "dl-preview" | "dl_preview" => Ok(Profile::DlPreview),
        "swrl" => Ok(Profile::Swrl),
        other => Err(py_err(format!(
            "unsupported profile {other:?}; use auto, rdfs, rl, el, dl, or dl-preview"
        ))),
    }
}

pub(crate) fn entity_iri(ontology: &Ontology, id: EntityId) -> PyResult<String> {
    let record = ontology.entity(id).map_err(py_err)?;
    ontology
        .resolve_iri(record.iri)
        .map(|s| s.to_owned())
        .map_err(py_err)
}

pub(crate) fn parse_meta_dict<'py>(
    py: Python<'py>,
    summary: &ParseMetaSummary,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("warnings", &summary.warnings)?;
    dict.set_item("mapped_axiom_count", summary.mapped_axiom_count)?;
    dict.set_item("skipped_axiom_count", summary.skipped_axiom_count)?;
    dict.set_item("logical_axiom_count", summary.logical_axiom_count)?;
    Ok(dict)
}

fn optional_entity_pair(
    ontology: &Ontology,
    pair: Option<(EntityId, EntityId)>,
) -> PyResult<Option<(String, String)>> {
    match pair {
        Some((left, right)) => Ok(Some((
            entity_iri(ontology, left)?,
            entity_iri(ontology, right)?,
        ))),
        None => Ok(None),
    }
}

fn optional_entity_triple(
    ontology: &Ontology,
    triple: Option<(EntityId, EntityId, EntityId)>,
) -> PyResult<Option<(String, String, String)>> {
    match triple {
        Some((a, b, c)) => Ok(Some((
            entity_iri(ontology, a)?,
            entity_iri(ontology, b)?,
            entity_iri(ontology, c)?,
        ))),
        None => Ok(None),
    }
}

fn proof_node_dict<'py>(
    py: Python<'py>,
    ontology: &Ontology,
    node: &ProofNode,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("rule", &node.rule)?;
    dict.set_item(
        "premises",
        node.premises.iter().map(|id| id.0).collect::<Vec<_>>(),
    )?;
    if let Some(axiom_id) = node.conclusion_axiom {
        dict.set_item("conclusion_axiom", axiom_id.index())?;
    }
    if let Some(pair) = optional_entity_pair(ontology, node.conclusion_sub)? {
        dict.set_item("conclusion_sub", pair)?;
    }
    if let Some(triple) = optional_entity_triple(ontology, node.conclusion_existential)? {
        dict.set_item("conclusion_existential", triple)?;
    }
    if let Some(pair) = optional_entity_pair(ontology, node.conclusion_subproperty)? {
        dict.set_item("conclusion_subproperty", pair)?;
    }
    Ok(dict)
}

pub(crate) fn proof_graph_dict<'py>(
    py: Python<'py>,
    ontology: &Ontology,
    graph: &ProofGraph,
    parse_meta: Option<&ParseMetaSummary>,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("node_count", graph.node_count())?;
    let nodes: Vec<Bound<'py, PyDict>> = graph
        .nodes
        .iter()
        .map(|node| proof_node_dict(py, ontology, node))
        .collect::<PyResult<Vec<_>>>()?;
    dict.set_item("nodes", nodes)?;
    if let Some(summary) = parse_meta {
        dict.set_item("parse_meta", parse_meta_dict(py, summary)?)?;
    }
    Ok(dict)
}

pub(crate) fn resolve_class(ontology: &mut Ontology, iri: &str) -> PyResult<EntityId> {
    ontology.entity_id(iri, EntityKind::Class).map_err(py_err)
}

pub(crate) fn resolve_individual(ontology: &mut Ontology, iri: &str) -> PyResult<EntityId> {
    ontology
        .entity_id(iri, EntityKind::Individual)
        .map_err(py_err)
}

pub(crate) fn resolve_object_property(ontology: &mut Ontology, iri: &str) -> PyResult<EntityId> {
    ontology
        .entity_id(iri, EntityKind::ObjectProperty)
        .map_err(py_err)
}

pub(crate) fn find_subclass_axiom_id(
    ontology: &Ontology,
    subclass_iri: &str,
    superclass_iri: &str,
) -> PyResult<Option<AxiomId>> {
    let subclass = ontology
        .try_lookup_entity(subclass_iri)
        .map_err(py_err)?
        .ok_or_else(|| py_err(format!("unknown class IRI: {subclass_iri}")))?;
    let superclass = ontology
        .try_lookup_entity(superclass_iri)
        .map_err(py_err)?
        .ok_or_else(|| py_err(format!("unknown class IRI: {superclass_iri}")))?;

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

pub(crate) fn json_to_py(py: Python<'_>, value: &serde_json::Value) -> PyResult<Py<PyAny>> {
    let json_mod = PyModule::import(py, "json")?;
    let s = serde_json::to_string(value).map_err(py_err)?;
    Ok(json_mod.call_method1("loads", (s,))?.unbind())
}

pub(crate) fn json_to_pydict<'py>(
    py: Python<'py>,
    value: &serde_json::Value,
) -> PyResult<Bound<'py, PyDict>> {
    json_to_py(py, value)?
        .into_bound(py)
        .downcast_into()
        .map_err(|e| py_err(e.to_string()))
}

pub(crate) fn parse_meta_from_ontology(ontology: &Ontology) -> Option<ParseMetaSummary> {
    ontology.parse_meta().map(ParseMetaSummary::from)
}

pub(crate) fn taxonomy_classify_dict<'py>(
    py: Python<'py>,
    ontology: &Ontology,
    taxonomy: &Taxonomy,
) -> PyResult<Bound<'py, PyDict>> {
    let parse_meta = parse_meta_from_ontology(ontology);
    let json = ontologos_facade::taxonomy_json(
        "classified",
        taxonomy,
        ontology,
        parse_meta.as_ref(),
    )
    .map_err(py_err)?;
    let value = serde_json::to_value(&json).map_err(py_err)?;
    json_to_pydict(py, &value)
}

pub(crate) fn rdfs_classify_dict<'py>(
    py: Python<'py>,
    ontology: &Ontology,
    report: &ontologos_rl::rdfs::MaterializationReport,
) -> PyResult<Bound<'py, PyDict>> {
    let parse_meta = parse_meta_from_ontology(ontology);
    let json =
        ontologos_facade::rdfs_materialization_json("classified", report, parse_meta.as_ref());
    let value = serde_json::to_value(&json).map_err(py_err)?;
    json_to_pydict(py, &value)
}

pub(crate) fn rl_classify_dict<'py>(
    py: Python<'py>,
    ontology: &Ontology,
    report: &ontologos_rl::MaterializationReport,
) -> PyResult<Bound<'py, PyDict>> {
    let parse_meta = parse_meta_from_ontology(ontology);
    let json = ontologos_facade::rl_materialization_json("classified", report, parse_meta.as_ref());
    let value = serde_json::to_value(&json).map_err(py_err)?;
    json_to_pydict(py, &value)
}
