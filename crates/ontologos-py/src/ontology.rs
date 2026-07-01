//! Python `Ontology` and `OntologyBuilder` bindings.

use std::sync::{Arc, Mutex};

use ontologos_core::{Limits, Ontology, OntologyBuilder};
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyString, PyType};

use crate::convert::py_err;
use crate::exceptions::{map_core_py_err, map_parser_py_err};

/// Shared in-memory ontology handle used by `Ontology` and `Reasoner`.
pub(crate) type SharedOntology = Arc<Mutex<Ontology>>;

/// In-memory ontology constructed from JSON or the builder API.
#[pyclass(name = "Ontology", unsendable)]
pub(crate) struct PyOntology {
    pub(crate) inner: SharedOntology,
}

impl PyOntology {
    pub(crate) fn from_owned(ontology: Ontology) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ontology)),
        }
    }
}

#[pymethods]
impl PyOntology {
    #[classmethod]
    fn from_json(_cls: &Bound<'_, PyType>, json: &str) -> PyResult<Self> {
        let inner = Ontology::from_json(json).map_err(map_core_py_err)?;
        Ok(Self::from_owned(inner))
    }

    #[classmethod]
    #[pyo3(signature = (json, *, max_json_bytes=None, max_entities=None, max_axioms=None, max_iri_len=None))]
    fn from_json_with_limits(
        _cls: &Bound<'_, PyType>,
        json: &str,
        max_json_bytes: Option<usize>,
        max_entities: Option<usize>,
        max_axioms: Option<usize>,
        max_iri_len: Option<usize>,
    ) -> PyResult<Self> {
        let mut limits = Limits::default();
        if let Some(n) = max_json_bytes {
            limits.max_json_bytes = n;
        }
        if let Some(n) = max_entities {
            limits.max_entities = n;
        }
        if let Some(n) = max_axioms {
            limits.max_axioms = n;
        }
        if let Some(n) = max_iri_len {
            limits.max_iri_len = n;
        }
        let inner = Ontology::from_json_with_limits(json, limits).map_err(map_core_py_err)?;
        Ok(Self::from_owned(inner))
    }

    #[classmethod]
    fn from_dict<'py>(
        _cls: &Bound<'_, PyType>,
        py: Python<'py>,
        data: &Bound<'py, PyDict>,
    ) -> PyResult<Self> {
        let json_mod = PyModule::import(py, "json")?;
        let json: String = json_mod.call_method1("dumps", (data,))?.extract()?;
        Self::from_json_with_limits(_cls, &json, None, None, None, None)
    }

    #[classmethod]
    fn load_in(_cls: &Bound<'_, PyType>, base: &str, path: &str) -> PyResult<Self> {
        let ontology = ontologos_parser::load_ontology_in(
            std::path::Path::new(base),
            std::path::Path::new(path),
        )
        .map_err(map_parser_py_err)?;
        Ok(Self::from_owned(ontology))
    }

    fn to_json(&self) -> PyResult<String> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| py_err(format!("ontology lock poisoned: {e}")))?;
        guard.to_json().map_err(py_err)
    }

    fn to_dict<'py>(&self, py: Python<'py>) -> PyResult<Bound<'py, PyDict>> {
        let json_mod = PyModule::import(py, "json")?;
        let json = self.to_json()?;
        let value = json_mod.call_method1("loads", (PyString::new(py, &json),))?;
        value.downcast_into().map_err(|e| {
            PyRuntimeError::new_err(format!("expected dict from ontology JSON, got {e}"))
        })
    }

    #[getter]
    fn axiom_count(&self) -> PyResult<usize> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| py_err(format!("ontology lock poisoned: {e}")))?;
        Ok(guard.axiom_count())
    }

    #[getter]
    fn entity_count(&self) -> PyResult<usize> {
        let guard = self
            .inner
            .lock()
            .map_err(|e| py_err(format!("ontology lock poisoned: {e}")))?;
        Ok(guard.entity_count())
    }
}

/// Fluent builder for constructing ontologies in memory.
#[pyclass(name = "OntologyBuilder", unsendable)]
pub(crate) struct PyOntologyBuilder {
    builder: OntologyBuilder,
}

#[pymethods]
impl PyOntologyBuilder {
    #[new]
    fn new() -> Self {
        Self {
            builder: OntologyBuilder::new(),
        }
    }

    #[pyo3(name = "add_class")]
    fn class(mut slf: PyRefMut<'_, Self>, iri: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .class(iri)
            .map_err(py_err)?;
        Ok(())
    }

    fn individual(mut slf: PyRefMut<'_, Self>, iri: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .individual(iri)
            .map_err(py_err)?;
        Ok(())
    }

    fn object_property(mut slf: PyRefMut<'_, Self>, iri: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .object_property(iri)
            .map_err(py_err)?;
        Ok(())
    }

    fn subclass_of(mut slf: PyRefMut<'_, Self>, subclass: &str, superclass: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .subclass_of(subclass, superclass)
            .map_err(py_err)?;
        Ok(())
    }

    fn subproperty_of(mut slf: PyRefMut<'_, Self>, sub: &str, sup: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .subproperty_of(sub, sup)
            .map_err(py_err)?;
        Ok(())
    }

    fn property_domain(mut slf: PyRefMut<'_, Self>, property: &str, domain: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .property_domain(property, domain)
            .map_err(py_err)?;
        Ok(())
    }

    fn property_range(mut slf: PyRefMut<'_, Self>, property: &str, range: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .property_range(property, range)
            .map_err(py_err)?;
        Ok(())
    }

    fn class_assertion(mut slf: PyRefMut<'_, Self>, individual: &str, class: &str) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .class_assertion(individual, class)
            .map_err(py_err)?;
        Ok(())
    }

    fn object_property_assertion(
        mut slf: PyRefMut<'_, Self>,
        subject: &str,
        property: &str,
        object: &str,
    ) -> PyResult<()> {
        slf.builder = std::mem::take(&mut slf.builder)
            .object_property_assertion(subject, property, object)
            .map_err(py_err)?;
        Ok(())
    }

    fn build(mut slf: PyRefMut<'_, Self>) -> PyResult<PyOntology> {
        let inner = std::mem::take(&mut slf.builder).build().map_err(py_err)?;
        Ok(PyOntology::from_owned(inner))
    }
}
