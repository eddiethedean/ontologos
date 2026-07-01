//! ALC engine trait implementations.

use ontologos_alc::AlcEngine;
use ontologos_core::{ConsistencyResult, Reasoner};
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
    fn check_consistency(&self, reasoner: &Reasoner) -> Result<ConsistencyResult> {
        match AlcEngine.is_consistent(reasoner.ontology()) {
            Ok(consistent) => Ok(if consistent {
                ConsistencyResult::consistent()
            } else {
                ConsistencyResult::inconsistent()
            }),
            Err(ontologos_alc::Error::ResourceLimit(_)) => Ok(ConsistencyResult::incomplete()),
            Err(e) => Err(Error::Alc(e)),
        }
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
