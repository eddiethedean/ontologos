//! Python bindings for OntoLogos (v0.9).

mod convert;
mod ontology;
mod reasoner;

use ontology::{PyOntology, PyOntologyBuilder};
use reasoner::PyReasoner;

use pyo3::prelude::*;

#[pymodule]
fn _ontologos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyReasoner>()?;
    m.add_class::<PyOntology>()?;
    m.add_class::<PyOntologyBuilder>()?;
    Ok(())
}
