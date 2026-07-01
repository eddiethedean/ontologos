//! SWRL engine trait implementations.

use ontologos_core::Reasoner;
use ontologos_dl::DlEngine;
use ontologos_el::ClassifyOutcome;
use ontologos_swrl::SwrlEngine;

use super::{ClassifyEngine, ConsistencyEngine, RoleQueryEngine};
use crate::error::{Error, Result};

pub(crate) struct SwrlAdapter;

impl ClassifyEngine for SwrlAdapter {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
        let (taxonomy, _report) = SwrlEngine
            .classify_with_swrl(reasoner.ontology())
            .map_err(Error::Swrl)?;
        Ok(ClassifyOutcome::Taxonomy(taxonomy))
    }
}

impl ConsistencyEngine for SwrlAdapter {
    fn is_consistent(&self, reasoner: &Reasoner) -> Result<bool> {
        SwrlEngine
            .is_consistent(reasoner.ontology())
            .map_err(Error::Swrl)
    }
}

impl RoleQueryEngine for SwrlAdapter {
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
