//! Result of a profile-routed classification run.

use ontologos_core::Taxonomy;
use ontologos_rl::rdfs::MaterializationReport as RdfsReport;
use ontologos_rl::MaterializationReport as RlReport;

/// Outcome of [`crate::classify`].
#[derive(Debug)]
pub enum ClassifyOutcome {
    /// OWL EL / DL / ALC / SWRL taxonomy from classification.
    Taxonomy(Taxonomy),
    /// RDFS materialization report.
    Rdfs(RdfsReport),
    /// OWL RL saturation report.
    Rl(RlReport),
}
