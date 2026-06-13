//! ABox materialization report.

/// Summary of ABox materialization.
#[derive(Debug, Clone, Default)]
pub struct AboxReport {
    /// `sameAs` equivalence clusters discovered.
    pub same_as_clusters: Vec<Vec<ontologos_core::EntityId>>,
    /// RL inferences added during saturation.
    pub rl_inferences: usize,
}
