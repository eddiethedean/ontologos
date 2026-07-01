//! Wall-clock budgets and capped parallelism for DL user paths.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::Error;

/// Whether WG corpus consistency shortcuts are enabled (`ONTOLOGOS_DL_WG_SHORTCUTS=1`).
#[must_use]
pub fn wg_shortcuts_enabled() -> bool {
    std::env::var("ONTOLOGOS_DL_WG_SHORTCUTS")
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
    config
        .map(Duration::from_secs)
        .or_else(|| {
            std::env::var("ONTOLOGOS_DL_BUDGET_SECS")
                .ok()
                .and_then(|s| s.parse().ok())
                .map(Duration::from_secs)
        })
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
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _permit = permit;
        let _ = tx.send(work());
    });
    match rx.recv_timeout(budget) {
        Ok(v) => Ok(v),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
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
