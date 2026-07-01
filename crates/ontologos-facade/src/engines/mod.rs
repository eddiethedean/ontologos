//! ISP engine operation traits (facade-internal).

use std::collections::HashSet;

use ontologos_core::{EntityId, Reasoner, RoleExpr};
use ontologos_el::ClassifyOutcome;

use crate::error::Result;

/// Classification / materialization engine.
pub(crate) trait ClassifyEngine {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome>;
}

/// Consistency checking engine.
pub(crate) trait ConsistencyEngine {
    fn check_consistency(
        &self,
        reasoner: &Reasoner,
    ) -> Result<ontologos_core::ConsistencyResult>;

    fn is_consistent(&self, reasoner: &Reasoner) -> Result<bool> {
        self.check_consistency(reasoner)?
            .into_bool()
            .map_err(crate::error::Error::Core)
    }
}

/// Sub-object-property query engine.
pub(crate) trait RoleQueryEngine {
    fn sub_object_properties(
        &self,
        reasoner: &Reasoner,
        property: EntityId,
        direct: bool,
    ) -> Result<HashSet<RoleExpr>>;
}

mod alc;
mod dl;
mod el;
mod hybrid;
mod rdfs;
mod registry;
mod rl;
mod swrl;

pub(crate) use registry::EngineRegistry;
