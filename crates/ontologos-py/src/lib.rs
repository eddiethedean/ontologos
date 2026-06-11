//! Python bindings for Ontologos.

use ontologos_core::{Ontology, Reasoner};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

/// Python wrapper around the Ontologos reasoner.
#[pyclass(name = "Reasoner")]
struct PyReasoner {
    reasoner: Reasoner,
}

#[pymethods]
impl PyReasoner {
    #[new]
    fn new(path: &str) -> PyResult<Self> {
        let ontology =
            Ontology::from_file(path).map_err(|e| PyRuntimeError::new_err(e.to_string()))?;
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
fn ontologos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReasoner>()?;
    Ok(())
}
