//! Python `Reasoner` bindings.

use ontologos_core::{Axiom, ParseMetaSummary, Reasoner, ReasonerConfig};
use ontologos_el::ClassifyOutcome;
use ontologos_explain::explain_with_profile;
use ontologos_parser::load_ontology;
use pyo3::prelude::*;
use pyo3::types::{PyAny, PyDict};

use crate::convert::{
    find_subclass_axiom_id, parse_meta_dict, parse_profile, proof_graph_dict, py_err,
    resolve_class, resolve_individual, resolve_object_property, taxonomy_dict,
};
use crate::ontology::PyOntology;

/// Python wrapper around the OntoLogos reasoner.
#[pyclass(name = "Reasoner", unsendable)]
pub(crate) struct PyReasoner {
    pub(crate) reasoner: Reasoner,
    pub(crate) last_taxonomy: Option<ontologos_core::Taxonomy>,
    pub(crate) dl_preview: bool,
}

fn build_reasoner(
    ontology: ontologos_core::Ontology,
    profile: Option<&str>,
    incremental: bool,
) -> PyResult<Reasoner> {
    Reasoner::builder()
        .profile(parse_profile(profile)?)
        .config(ReasonerConfig {
            incremental,
            ..ReasonerConfig::default()
        })
        .build(ontology)
        .map_err(py_err)
}

#[pymethods]
impl PyReasoner {
    #[new]
    #[pyo3(signature = (path=None, ontology=None, profile=None, incremental=false))]
    fn new(
        path: Option<&str>,
        ontology: Option<&PyOntology>,
        profile: Option<&str>,
        incremental: bool,
    ) -> PyResult<Self> {
        let has_path = path.is_some();
        let has_ontology = ontology.is_some();
        if has_path == has_ontology {
            return Err(py_err(
                "Reasoner requires exactly one of `path` or `ontology`",
            ));
        }

        let core_ontology = if let Some(path) = path {
            load_ontology(std::path::Path::new(path)).map_err(py_err)?
        } else {
            ontology.expect("ontology checked above").inner.clone()
        };

        let dl_preview = matches!(
            profile.map(str::to_ascii_lowercase).as_deref(),
            Some("dl-preview") | Some("dl_preview")
        );
        let reasoner = build_reasoner(core_ontology, profile, incremental)?;
        Ok(Self {
            reasoner,
            last_taxonomy: None,
            dl_preview,
        })
    }

    /// Parse metadata from the loaded ontology (warnings and axiom counts).
    #[getter]
    fn parse_meta(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let summary = self
            .reasoner
            .ontology()
            .parse_meta()
            .map(ParseMetaSummary::from)
            .unwrap_or_default();
        Ok(parse_meta_dict(py, &summary)?.into())
    }

    /// Taxonomy from the last EL classification (`None` for RDFS/RL runs).
    #[getter]
    fn taxonomy(&self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let Some(ref taxonomy) = self.last_taxonomy else {
            return Ok(py.None());
        };
        Ok(taxonomy_dict(py, self.reasoner.ontology(), taxonomy)?.into())
    }

    fn classify(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let outcome = match self.reasoner.profile() {
            ontologos_core::Profile::Dl if self.dl_preview => {
                ClassifyOutcome::Taxonomy(
                    ontologos_dl::DlClassifier::new()
                        .preview(true)
                        .classify(self.reasoner.ontology())
                        .map_err(py_err)?,
                )
            }
            _ => ontologos_facade::classify(&mut self.reasoner).map_err(map_facade_py_err)?,
        };
        match outcome {
            ClassifyOutcome::Taxonomy(taxonomy) => {
                let dict = taxonomy_dict(py, self.reasoner.ontology(), &taxonomy)?;
                self.last_taxonomy = Some(taxonomy);
                Ok(dict.into())
            }
            ClassifyOutcome::Rdfs(report) => {
                self.last_taxonomy = None;
                let dict = PyDict::new(py);
                dict.set_item("initial_axiom_count", report.initial_axiom_count)?;
                dict.set_item("final_axiom_count", report.final_axiom_count)?;
                dict.set_item("inferred_axioms", report.inferred_total())?;
                Ok(dict.into())
            }
            ClassifyOutcome::Rl(report) => {
                self.last_taxonomy = None;
                let dict = PyDict::new(py);
                dict.set_item("initial_axiom_count", report.initial_axiom_count)?;
                dict.set_item("final_axiom_count", report.final_axiom_count)?;
                dict.set_item("inferred_axioms", report.inferred_total())?;
                Ok(dict.into())
            }
        }
    }

    fn explain(&mut self, py: Python<'_>) -> PyResult<Py<PyAny>> {
        let graph = explain_with_profile(&mut self.reasoner).map_err(py_err)?;
        let summary = self
            .reasoner
            .ontology()
            .parse_meta()
            .map(ParseMetaSummary::from)
            .unwrap_or_default();
        Ok(proof_graph_dict(py, self.reasoner.ontology(), &graph, Some(&summary))?.into())
    }

    fn add_subclass_of(&mut self, subclass: &str, superclass: &str) -> PyResult<()> {
        let ontology = self.reasoner.ontology_mut();
        let sub = resolve_class(ontology, subclass)?;
        let sup = resolve_class(ontology, superclass)?;
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: sub,
                superclass: sup,
            })
            .map_err(py_err)?;
        Ok(())
    }

    fn remove_subclass_of(&mut self, subclass: &str, superclass: &str) -> PyResult<()> {
        let id = find_subclass_axiom_id(self.reasoner.ontology(), subclass, superclass)?
            .ok_or_else(|| py_err(format!("no SubClassOf axiom for {subclass} ⊑ {superclass}")))?;
        self.reasoner
            .ontology_mut()
            .remove_axiom(id)
            .map_err(py_err)?;
        Ok(())
    }

    fn add_axiom_json(&mut self, py: Python<'_>, axiom: &Bound<'_, PyAny>) -> PyResult<()> {
        let json_mod = PyModule::import(py, "json")?;
        let axiom_json: String = json_mod.call_method1("dumps", (axiom,))?.extract()?;
        let snapshot: serde_json::Value =
            serde_json::from_str(&axiom_json).map_err(|e| py_err(e.to_string()))?;

        let ontology = self.reasoner.ontology_mut();
        apply_snapshot_axiom(ontology, &snapshot)?;
        Ok(())
    }
}

fn apply_snapshot_axiom(
    ontology: &mut ontologos_core::Ontology,
    value: &serde_json::Value,
) -> PyResult<()> {
    if let Some(inner) = value.get("SubClassOf") {
        let subclass = inner
            .get("subclass")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubClassOf missing subclass"))?;
        let superclass = inner
            .get("superclass")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubClassOf missing superclass"))?;
        let sub = resolve_class(ontology, subclass)?;
        let sup = resolve_class(ontology, superclass)?;
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: sub,
                superclass: sup,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("SubObjectPropertyOf") {
        let sub = inner
            .get("sub_property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubObjectPropertyOf missing sub_property"))?;
        let sup = inner
            .get("super_property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("SubObjectPropertyOf missing super_property"))?;
        let sub_id = resolve_object_property(ontology, sub)?;
        let sup_id = resolve_object_property(ontology, sup)?;
        ontology
            .add_axiom(Axiom::SubObjectPropertyOf {
                sub_property: sub_id,
                super_property: sup_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyDomain") {
        let property = inner
            .get("property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyDomain missing property"))?;
        let domain = inner
            .get("domain")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyDomain missing domain"))?;
        let property_id = resolve_object_property(ontology, property)?;
        let domain_id = resolve_class(ontology, domain)?;
        ontology
            .add_axiom(Axiom::ObjectPropertyDomain {
                property: property_id,
                domain: domain_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyRange") {
        let property = inner
            .get("property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyRange missing property"))?;
        let range = inner
            .get("range")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyRange missing range"))?;
        let property_id = resolve_object_property(ontology, property)?;
        let range_id = resolve_class(ontology, range)?;
        ontology
            .add_axiom(Axiom::ObjectPropertyRange {
                property: property_id,
                range: range_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ClassAssertion") {
        let individual = inner
            .get("individual")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ClassAssertion missing individual"))?;
        let class = inner
            .get("class")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ClassAssertion missing class"))?;
        let individual_id = resolve_individual(ontology, individual)?;
        let class_id = resolve_class(ontology, class)?;
        ontology
            .add_axiom(Axiom::ClassAssertion {
                individual: individual_id,
                class: class_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    if let Some(inner) = value.get("ObjectPropertyAssertion") {
        let subject = inner
            .get("subject")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyAssertion missing subject"))?;
        let property = inner
            .get("property")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyAssertion missing property"))?;
        let object = inner
            .get("object")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| py_err("ObjectPropertyAssertion missing object"))?;
        let subject_id = resolve_individual(ontology, subject)?;
        let property_id = resolve_object_property(ontology, property)?;
        let object_id = resolve_individual(ontology, object)?;
        ontology
            .add_axiom(Axiom::ObjectPropertyAssertion {
                subject: subject_id,
                property: property_id,
                object: object_id,
            })
            .map_err(py_err)?;
        return Ok(());
    }

    Err(py_err(
        "unsupported axiom JSON; use format v2 axiom objects (e.g. {\"SubClassOf\": {...}})",
    ))
}

fn map_facade_py_err(error: ontologos_facade::Error) -> PyErr {
    match error {
        ontologos_facade::Error::El(e) => py_err(e.to_string()),
        ontologos_facade::Error::Alc(e) => py_err(e.to_string()),
        ontologos_facade::Error::Dl(e) => py_err(e.to_string()),
        ontologos_facade::Error::Swrl(e) => py_err(e.to_string()),
    }
}
