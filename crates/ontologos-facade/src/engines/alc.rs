//! ALC engine trait implementations.

use ontologos_alc::AlcEngine;
use ontologos_core::Reasoner;
use ontologos_dl::DlEngine;
use ontologos_el::ClassifyOutcome;

use super::{ClassifyEngine, ConsistencyEngine, RoleQueryEngine};
use crate::error::{Error, Result};

pub(crate) struct AlcAdapter;

impl ClassifyEngine for AlcAdapter {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
        Ok(ClassifyOutcome::Taxonomy(
            AlcEngine
                .classify(reasoner.ontology())
                .map_err(Error::Alc)?,
        ))
    }
}

impl ConsistencyEngine for AlcAdapter {
    fn is_consistent(&self, reasoner: &Reasoner) -> Result<bool> {
        AlcEngine
            .is_consistent(reasoner.ontology())
            .map_err(Error::Alc)
    }
}

impl RoleQueryEngine for AlcAdapter {
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
