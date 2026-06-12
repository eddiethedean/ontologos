//! Python bindings for OntoLogos.
//!
//! v0.4: loads ontologies via `ontologos_parser::load_ontology`.
//! Pass `profile="rdfs"` or `profile="rl"` to run materialization via `classify()`.

use ontologos_core::{ParseMetaSummary, Profile, Reasoner};
use ontologos_parser::load_ontology;
use ontologos_rdfs::classify_reasoner;
use ontologos_rl::classify_reasoner as classify_rl_reasoner;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::PyDict;

/// Python wrapper around the OntoLogos reasoner.
#[pyclass(name = "Reasoner")]
struct PyReasoner {
    reasoner: Reasoner,
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

#[pymethods]
impl PyReasoner {
    #[new]
    #[pyo3(signature = (path, profile=None))]
    fn new(path: &str, profile: Option<&str>) -> PyResult<Self> {
        let ontology = load_ontology(std::path::Path::new(path))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let reasoner = Reasoner::builder()
            .profile(parse_profile(profile)?)
            .build(ontology)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { reasoner })
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

    fn classify(&mut self) -> PyResult<()> {
        match self.reasoner.profile() {
            Profile::Rdfs => classify_reasoner(&mut self.reasoner)
                .map_err(|e| PyRuntimeError::new_err(e.to_string())),
            Profile::Rl => classify_rl_reasoner(&mut self.reasoner)
                .map_err(|e| PyRuntimeError::new_err(e.to_string())),
            _ => self
                .reasoner
                .classify()
                .map_err(|e| PyRuntimeError::new_err(e.to_string())),
        }
    }
}

#[pymodule]
fn _ontologos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReasoner>()?;
    Ok(())
}
