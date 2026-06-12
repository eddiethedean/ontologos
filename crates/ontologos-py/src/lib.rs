//! Python bindings for OntoLogos.
//!
//! v0.3: alpha placeholder — loads ontologies via `ontologos_parser::load_ontology`;
//! `Reasoner::classify()` returns not-implemented until v0.5 (except `Profile::Rdfs` when enabled).

use ontologos_core::Reasoner;
use ontologos_parser::load_ontology;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Python wrapper around the OntoLogos reasoner.
#[pyclass(name = "Reasoner")]
struct PyReasoner {
    reasoner: Reasoner,
}

#[pymethods]
impl PyReasoner {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let ontology = load_ontology(std::path::Path::new(path))
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        let reasoner = Reasoner::builder()
            .build(ontology)
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
        Ok(Self { reasoner })
    }

    fn classify(&self) -> PyResult<()> {
        self.reasoner
            .classify()
            .map_err(|e| PyRuntimeError::new_err(e.to_string()))
    }
}

#[pymodule]
fn _ontologos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReasoner>()?;
    Ok(())
}
