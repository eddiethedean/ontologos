//! Python bindings for OntoLogos.
//!
//! v0.5: loads ontologies via `ontologos_parser::load_ontology`.
//! Pass `profile="rdfs"`, `"rl"`, `"el"`, or `"auto"` to run classification via `classify()`.

use ontologos_core::{ParseMetaSummary, Profile, Reasoner, ReasonerConfig};
use ontologos_el::{classify_with_profile, ClassifyOutcome};
use ontologos_parser::load_ontology;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Python wrapper around the OntoLogos reasoner.
#[pyclass(name = "Reasoner", unsendable)]
struct PyReasoner {
    reasoner: Reasoner,
    last_taxonomy: Option<ontologos_core::Taxonomy>,
}

fn parse_profile(profile: Option<&str>) -> PyResult<Profile> {
    match profile.unwrap_or("auto").to_ascii_lowercase().as_str() {
        "auto" => Ok(Profile::Auto),
        "rdfs" => Ok(Profile::Rdfs),
        "rl" => Ok(Profile::Rl),
        "el" => Ok(Profile::El),
        other => Err(PyRuntimeError::new_err(format!(
            "unsupported profile {other:?}; use auto, rdfs, rl, or el"
        ))),
    }
}

fn parse_meta_dict<'py>(
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

fn taxonomy_dict<'py>(
    py: Python<'py>,
    ontology: &ontologos_core::Ontology,
    taxonomy: &ontologos_core::Taxonomy,
) -> PyResult<Bound<'py, PyDict>> {
    let dict = PyDict::new(py);
    dict.set_item("subsumption_count", taxonomy.subsumption_count())?;

    let subs: Vec<(String, String)> = taxonomy
        .subsumptions
        .iter()
        .map(|&(sub, sup)| Ok((entity_iri(ontology, sub)?, entity_iri(ontology, sup)?)))
        .collect::<PyResult<Vec<_>>>()?;
    dict.set_item("subsumptions", subs)?;

    let equiv: Vec<Vec<String>> = taxonomy
        .equivalences
        .iter()
        .map(|cluster| {
            cluster
                .iter()
                .map(|&id| entity_iri(ontology, id))
                .collect::<PyResult<Vec<_>>>()
        })
        .collect::<PyResult<Vec<_>>>()?;
    dict.set_item("equivalences", equiv)?;

    let unsat: Vec<String> = taxonomy
        .unsatisfiable
        .iter()
        .map(|&id| entity_iri(ontology, id))
        .collect::<PyResult<Vec<_>>>()?;
    dict.set_item("unsatisfiable", unsat)?;
    Ok(dict)
}

fn entity_iri(
    ontology: &ontologos_core::Ontology,
    id: ontologos_core::EntityId,
) -> PyResult<String> {
    let record = ontology
        .entity(id)
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
    ontology
        .resolve_iri(record.iri)
        .map(|s| s.to_owned())
        .map_err(|e| PyRuntimeError::new_err(e.to_string()))
}

#[pymethods]
impl PyReasoner {
    #[new]
    #[pyo3(signature = (path, profile=None, incremental=false))]
    fn new(path: &str, profile: Option<&str>, incremental: bool) -> PyResult<Self> {
        let ontology = load_ontology(std::path::Path::new(path))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let reasoner = Reasoner::builder()
            .profile(parse_profile(profile)?)
            .config(ReasonerConfig {
                incremental,
                ..ReasonerConfig::default()
            })
            .build(ontology)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self {
            reasoner,
            last_taxonomy: None,
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
        let outcome = classify_with_profile(&mut self.reasoner)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
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
}

#[pymodule]
fn _ontologos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReasoner>()?;
    Ok(())
}
