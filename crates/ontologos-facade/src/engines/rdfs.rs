//! RDFS engine trait implementations.

use ontologos_core::Reasoner;
use ontologos_el::ClassifyOutcome;
use ontologos_rdfs::RdfsEngineAdapter;

use super::{ClassifyEngine, ConsistencyEngine};
use crate::error::{Error, Result};

pub(crate) struct RdfsAdapter;

impl ClassifyEngine for RdfsAdapter {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
        Ok(ClassifyOutcome::Rdfs(
            RdfsEngineAdapter
                .materialize(reasoner)
                .map_err(|e| Error::El(e.into()))?,
        ))
    }
}

impl ConsistencyEngine for RdfsAdapter {
    fn is_consistent(&self, reasoner: &Reasoner) -> Result<bool> {
        RdfsEngineAdapter
            .is_consistent(reasoner.ontology())
            .map_err(|e| Error::El(e.into()))
    }
}
