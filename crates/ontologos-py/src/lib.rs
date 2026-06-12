//! Python bindings for OntoLogos.
//!
//! v0.4: loads ontologies via `ontologos_parser::load_ontology`.
//! Pass `profile="rdfs"` or `profile="rl"` to run materialization via `classify()`.

use ontologos_core::{Profile, Reasoner};
use ontologos_parser::load_ontology;
use ontologos_rdfs::classify_reasoner;
use ontologos_rl::classify_reasoner as classify_rl_reasoner;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

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
