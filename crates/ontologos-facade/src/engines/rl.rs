//! RL engine trait implementations.

use ontologos_bridge::has_bottom_chain_violation;
use ontologos_core::{ConsistencyResult, Reasoner};
use ontologos_el::ClassifyOutcome;
use ontologos_rl::{RlEngine, RlEngineAdapter};

use super::{ClassifyEngine, ConsistencyEngine, RoleQueryEngine};
use crate::error::{Error, Result};
use crate::lookup::index_sub_object_properties;

pub(crate) struct RlAdapter;

impl ClassifyEngine for RlAdapter {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
        Ok(ClassifyOutcome::Rl(
            RlEngineAdapter
                .saturate(reasoner)
                .map_err(|e| Error::El(e.into()))?,
        ))
    }
}

impl ConsistencyEngine for RlAdapter {
    fn check_consistency(&self, reasoner: &Reasoner) -> Result<ConsistencyResult> {
        let mut working = reasoner.ontology().clone();
        let report = RlEngine::new(1)
            .saturate(&mut working)
            .map_err(|e| Error::El(ontologos_el::Error::Message(format!("rl saturate: {e}"))))?;
        if !report.clashes.is_empty() || has_bottom_chain_violation(&working) {
            return Ok(ConsistencyResult::inconsistent());
        }
        let consistent = ontologos_abox::is_abox_consistent(&working).map_err(|e| {
            Error::El(ontologos_el::Error::Message(format!(
                "abox consistent: {e}"
            )))
        })?;
        Ok(if consistent {
            ConsistencyResult::consistent()
        } else {
            ConsistencyResult::inconsistent()
        })
    }
}

impl RoleQueryEngine for RlAdapter {
    fn sub_object_properties(
        &self,
        reasoner: &Reasoner,
        property: ontologos_core::EntityId,
        direct: bool,
    ) -> Result<std::collections::HashSet<ontologos_core::RoleExpr>> {
        Ok(index_sub_object_properties(
            reasoner.ontology(),
            property,
            direct,
        ))
    }
}
