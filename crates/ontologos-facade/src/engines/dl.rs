//! DL engine trait implementations.

use ontologos_core::Reasoner;
use ontologos_dl::DlEngine;
use ontologos_el::ClassifyOutcome;

use super::{ClassifyEngine, ConsistencyEngine, RoleQueryEngine};
use crate::error::{Error, Result};

pub(crate) struct DlAdapter;

impl ClassifyEngine for DlAdapter {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
        Ok(ClassifyOutcome::Taxonomy(
            DlEngine.classify(reasoner.ontology()).map_err(Error::Dl)?,
        ))
    }
}

impl ConsistencyEngine for DlAdapter {
    fn is_consistent(&self, reasoner: &Reasoner) -> Result<bool> {
        DlEngine
            .is_consistent(reasoner.ontology())
            .map_err(Error::Dl)
    }
}

impl RoleQueryEngine for DlAdapter {
    fn sub_object_properties(
        &self,
        reasoner: &Reasoner,
        property: ontologos_core::EntityId,
        direct: bool,
    ) -> Result<std::collections::HashSet<ontologos_core::RoleExpr>> {
        let role = ontologos_core::RoleExpr::Atomic(property);
        DlEngine
            .sub_object_properties(reasoner.ontology(), &role, direct)
            .map_err(Error::Dl)
    }
}
