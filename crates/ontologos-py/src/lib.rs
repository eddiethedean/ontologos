//! Python bindings for OntoLogos (v1.0).

mod convert;
mod exceptions;
mod ontology;
mod reasoner;

use exceptions::register_exceptions;

use ontology::{PyOntology, PyOntologyBuilder};
use reasoner::PyReasoner;

use pyo3::prelude::*;

#[pymodule]
fn _ontologos(m: &Bound<'_, PyModule>) -> PyResult<()> {
    register_exceptions(m)?;
    m.add_class::<PyReasoner>()?;
    m.add_class::<PyOntology>()?;
    m.add_class::<PyOntologyBuilder>()?;
    Ok(())
}
