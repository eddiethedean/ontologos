//! Optional wall-clock phase breakdown for DL classification.

use std::time::Instant;

use serde::Serialize;

/// Per-phase wall times for one `DlClassifier::classify` invocation (seconds).
#[derive(Debug, Default, Clone, Serialize)]
pub struct DlPerfTimings {
    /// Profile detection.
    pub detect_profile_s: f64,
    /// Construct scan.
    pub scan_constructs_s: f64,
    /// Clausification (`DlOntology::from_ontology`).
    pub clausify_s: f64,
    /// DL saturation.
    pub saturation_s: f64,
    /// Tableau (satisfiability + pairwise entailment).
    pub tableau_s: f64,
    /// Cardinality-derived subsumptions.
    pub cardinality_derive_s: f64,
    /// Optional EL classifier merge.
    pub el_classify_s: f64,
    /// Pizza defined-class enrichment passes.
    pub defined_class_s: f64,
    /// Taxonomy post-processing (enrich, canonicalize, reduce).
    pub taxonomy_post_s: f64,
    /// End-to-end classify time.
    pub total_s: f64,
}

/// Whether DL perf tracing is enabled (`ONTOLOGOS_DL_PERF=1`).
#[must_use]
pub fn perf_enabled() -> bool {
    std::env::var("ONTOLOGOS_DL_PERF")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// RAII timer that accumulates into a phase field when dropped.
pub struct PhaseTimer<'a> {
    start: Instant,
    target: &'a mut f64,
}

impl<'a> PhaseTimer<'a> {
    /// Start timing into `target` (seconds).
    pub fn start(target: &'a mut f64) -> Self {
        Self {
            start: Instant::now(),
            target,
        }
    }
}

impl Drop for PhaseTimer<'_> {
    fn drop(&mut self) {
        *self.target += self.start.elapsed().as_secs_f64();
    }
}
