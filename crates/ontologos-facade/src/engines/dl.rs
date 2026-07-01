//! DL engine trait implementations.

use ontologos_core::{ConsistencyResult, Reasoner};
use ontologos_dl::DlEngine;
use ontologos_el::ClassifyOutcome;

use super::{ClassifyEngine, ConsistencyEngine, RoleQueryEngine};
use crate::error::{Error, Result};

pub(crate) struct DlAdapter;

impl ClassifyEngine for DlAdapter {
    fn classify(&self, reasoner: &mut Reasoner) -> Result<ClassifyOutcome> {
        let taxonomy = if reasoner.profile() == ontologos_core::Profile::DlPreview {
            ontologos_dl::DlClassifier::new()
                .preview(true)
                .classify(reasoner.ontology())
                .map_err(Error::Dl)?
        } else {
            DlEngine.classify(reasoner.ontology()).map_err(Error::Dl)?
        };
        Ok(ClassifyOutcome::Taxonomy(taxonomy))
    }
}

impl ConsistencyEngine for DlAdapter {
    fn check_consistency(&self, reasoner: &Reasoner) -> Result<ConsistencyResult> {
        DlEngine
            .check_consistency(reasoner.ontology(), reasoner.config().budget_secs)
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
