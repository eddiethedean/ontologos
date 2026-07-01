//! EL engine trait implementations.

use ontologos_core::{ConsistencyResult, Profile, Reasoner};
use ontologos_el::{ClassifyOutcome, ElEngine};

use super::{ClassifyEngine, ConsistencyEngine, RoleQueryEngine};
use crate::error::{Error, Result};
use crate::lookup::index_sub_object_properties;

pub(crate) struct ElAdapter;

impl ClassifyEngine for ElAdapter {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
        if reasoner.profile() == Profile::Auto {
            ElEngine.classify_auto(reasoner).map_err(Error::El)
        } else {
            ElEngine.classify_with_profile(reasoner).map_err(Error::El)
        }
    }
}

impl ConsistencyEngine for ElAdapter {
    fn check_consistency(&self, reasoner: &Reasoner) -> Result<ConsistencyResult> {
        let consistent = ElEngine
            .is_consistent(reasoner.ontology())
            .map_err(Error::El)?;
        Ok(consistency_from_bool(consistent))
    }
}

fn consistency_from_bool(consistent: bool) -> ConsistencyResult {
    if consistent {
        ConsistencyResult::consistent()
    } else {
        ConsistencyResult::inconsistent()
    }
}

impl RoleQueryEngine for ElAdapter {
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
