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
    fn is_consistent(&self, reasoner: &Reasoner) -> Result<bool>;
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
