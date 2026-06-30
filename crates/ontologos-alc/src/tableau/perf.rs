//! Optional wall-clock phase breakdown for ALC tableau classification.

use std::time::Instant;

use serde::Serialize;

/// Per-phase wall times for one tableau classify invocation (seconds).
#[derive(Debug, Default, Clone, Serialize)]
pub struct TableauPerfTimings {
    /// Per-class satisfiability checks.
    pub class_sat_s: f64,
    /// Pairwise named-class entailment probes.
    pub pairwise_entail_s: f64,
    /// End-to-end tableau time.
    pub total_s: f64,
}

/// Whether tableau perf tracing is enabled (`ONTOLOGOS_TABLEAU_PERF=1`).
#[must_use]
pub fn perf_enabled() -> bool {
    std::env::var("ONTOLOGOS_TABLEAU_PERF")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

/// RAII timer that accumulates into a phase field when dropped.
pub struct PhaseTimer<'a> {
    start: Instant,
    target: &'a mut f64,
}

impl<'a> PhaseTimer<'a> {
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
