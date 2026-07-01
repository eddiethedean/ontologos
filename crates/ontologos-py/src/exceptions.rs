//! Typed Python exceptions for OntoLogos errors.

use ontologos_core::Error as CoreError;
use ontologos_facade::Error as FacadeError;
use pyo3::exceptions::PyException;
use pyo3::prelude::*;

pyo3::create_exception!(_ontologos, ParseError, PyException);
pyo3::create_exception!(_ontologos, ResourceLimitError, PyException);
pyo3::create_exception!(_ontologos, IncompleteReasoningError, PyException);

pub(crate) fn register_exceptions(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("ParseError", m.py().get_type::<ParseError>())?;
    m.add("ResourceLimitError", m.py().get_type::<ResourceLimitError>())?;
    m.add(
        "IncompleteReasoningError",
        m.py().get_type::<IncompleteReasoningError>(),
    )?;
    Ok(())
}

pub(crate) fn map_core_py_err(error: CoreError) -> PyErr {
    match &error {
        CoreError::Parse(_) | CoreError::Serialization(_) => ParseError::new_err(error.to_string()),
        _ => PyException::new_err(error.to_string()),
    }
}

pub(crate) fn map_facade_py_err(error: FacadeError) -> PyErr {
    match error {
        FacadeError::Alc(e) if matches!(e, ontologos_alc::Error::ResourceLimit(_)) => {
            ResourceLimitError::new_err(e.to_string())
        }
        FacadeError::Dl(e) => match e {
            ontologos_dl::Error::IncompleteReasoning(msg) => {
                IncompleteReasoningError::new_err(msg)
            }
            ontologos_dl::Error::Alc(ontologos_alc::Error::ResourceLimit(inner)) => {
                ResourceLimitError::new_err(inner.to_string())
            }
            other => PyException::new_err(other.to_string()),
        },
        FacadeError::Core(e) => map_core_py_err(e),
        other => PyException::new_err(other.to_string()),
    }
}
