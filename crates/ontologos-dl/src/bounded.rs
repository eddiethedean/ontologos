//! Wall-clock budgets and capped parallelism for DL user paths.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::Error;

static DL_CANCEL: AtomicBool = AtomicBool::new(false);

/// Whether a bounded DL worker should cooperatively stop (set on budget timeout).
#[must_use]
pub fn dl_cancel_requested() -> bool {
    DL_CANCEL.load(Ordering::Relaxed)
}

/// Whether WG corpus consistency shortcuts are enabled.
///
/// On in unit tests (`cfg(test)`), or when `ONTOLOGOS_CONFORMANCE=1` (CI / conformance
/// harness). Production embedders should leave both unset.
#[must_use]
pub fn wg_shortcuts_enabled() -> bool {
    cfg!(test) || conformance_harness_enabled()
}

fn conformance_harness_enabled() -> bool {
    std::env::var("ONTOLOGOS_CONFORMANCE")
        .ok()
        .is_some_and(|v| v == "1" || v.eq_ignore_ascii_case("true"))
}

/// Max parallel unsat workers (`ONTOLOGOS_DL_MAX_WORKERS`, default 4).
#[must_use]
pub fn dl_max_workers() -> usize {
    std::env::var("ONTOLOGOS_DL_MAX_WORKERS")
        .ok()
        .and_then(|s| s.parse().ok())
        .filter(|&n| n > 0)
        .unwrap_or(4)
}

/// Resolve wall-clock budget from config or `ONTOLOGOS_DL_BUDGET_SECS`.
#[must_use]
pub fn resolve_budget_secs(config: Option<u64>) -> Option<Duration> {
    let from_env = || {
        std::env::var("ONTOLOGOS_DL_BUDGET_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&secs| secs > 0)
            .map(Duration::from_secs)
    };
    match config {
        Some(0) => None,
        Some(secs) => Some(Duration::from_secs(secs)),
        None => from_env(),
    }
}

/// Run `work` on a worker thread with an optional wall-clock budget.
pub fn run_bounded<T, F>(budget_secs: Option<u64>, work: F) -> crate::Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let Some(budget) = resolve_budget_secs(budget_secs) else {
        return Ok(work());
    };
    run_bounded_inner(budget, work)
}

fn run_bounded_inner<T, F>(budget: Duration, work: F) -> crate::Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let gate = dl_worker_gate();
    let permit = acquire_dl_worker_permit(&gate);
    let reclaimed = permit.reclaimed.clone();
    DL_CANCEL.store(false, Ordering::Release);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _permit = permit;
        let _ = tx.send(work());
    });
    match rx.recv_timeout(budget) {
        Ok(v) => Ok(v),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            DL_CANCEL.store(true, Ordering::Release);
            if !reclaimed.swap(true, Ordering::AcqRel) {
                release_dl_permit(&gate);
            }
            Err(Error::IncompleteReasoning(format!(
                "dl operation exceeded {}s budget",
                budget.as_secs()
            )))
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            if !reclaimed.swap(true, Ordering::AcqRel) {
                release_dl_permit(&gate);
            }
            Err(Error::IncompleteReasoning("dl worker disconnected".into()))
        }
    }
}

struct DlWorkerPermit {
    gate: Arc<(Mutex<usize>, Condvar)>,
    reclaimed: Arc<AtomicBool>,
}

impl Drop for DlWorkerPermit {
    fn drop(&mut self) {
        if !self.reclaimed.swap(true, Ordering::AcqRel) {
            release_dl_permit(&self.gate);
        }
    }
}

fn dl_worker_gate() -> Arc<(Mutex<usize>, Condvar)> {
    static GATE: std::sync::OnceLock<Arc<(Mutex<usize>, Condvar)>> = std::sync::OnceLock::new();
    GATE.get_or_init(|| {
        let max = dl_max_workers();
        Arc::new((Mutex::new(max), Condvar::new()))
    })
    .clone()
}

fn release_dl_permit(gate: &Arc<(Mutex<usize>, Condvar)>) {
    let (lock, cvar) = &**gate;
    let mut permits = lock.lock().expect("dl worker gate");
    *permits += 1;
    cvar.notify_one();
}

fn acquire_dl_worker_permit(gate: &Arc<(Mutex<usize>, Condvar)>) -> DlWorkerPermit {
    let (lock, cvar) = &**gate;
    let mut permits = lock.lock().expect("dl worker gate");
    while *permits == 0 {
        permits = cvar.wait(permits).expect("dl worker gate");
    }
    *permits -= 1;
    drop(permits);
    DlWorkerPermit {
        gate: gate.clone(),
        reclaimed: Arc::new(AtomicBool::new(false)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_secs_zero_means_unlimited() {
        assert_eq!(resolve_budget_secs(Some(0)), None);
    }
}
