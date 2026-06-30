//! Optional wall-clock phase breakdown for reasonable bridge materialization.

use std::time::Instant;

use serde::Serialize;

/// Per-phase wall times for one materialize invocation (seconds).
#[derive(Debug, Default, Clone, Serialize)]
pub struct BridgePerfTimings {
    pub core_to_triples_s: f64,
    pub reasonable_reason_s: f64,
    pub merge_s: f64,
    pub postprocess_s: f64,
    pub total_s: f64,
}

/// Whether bridge perf tracing is enabled (`ONTOLOGOS_BRIDGE_PERF=1`).
#[must_use]
pub fn perf_enabled() -> bool {
    std::env::var("ONTOLOGOS_BRIDGE_PERF")
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
