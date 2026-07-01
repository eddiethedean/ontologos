//! RL profile engine adapter (DIP unit struct).

use ontologos_core::Reasoner;

use crate::report::MaterializationReport;

/// RL profile engine adapter (distinct from [`crate::RlEngine`] saturator).
#[derive(Debug, Default, Clone, Copy)]
pub struct RlEngineAdapter;

impl RlEngineAdapter {
    /// Saturate OWL RL inferences for a reasoner configured with RL profile.
    pub fn saturate(&self, reasoner: &mut Reasoner) -> crate::Result<MaterializationReport> {
        crate::materialize_reasoner(reasoner)
    }
}
