//! Wall-clock budgets and capped parallelism for DL user paths.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::Error;

/// Monotonic id for each bounded DL operation (0 = no active op on this thread).
static NEXT_OP_ID: AtomicU64 = AtomicU64::new(1);
/// Op id that should cooperatively cancel (0 = none). Per-op, not global, so parallel
/// scans do not poison each other when one case times out.
static CANCELLED_OP: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static CURRENT_OP: Cell<u64> = const { Cell::new(0) };
}

/// Whether the current DL worker should cooperatively stop (budget timeout for *this* op).
#[must_use]
pub fn dl_cancel_requested() -> bool {
    let op = CURRENT_OP.with(Cell::get);
    op != 0 && CANCELLED_OP.load(Ordering::Acquire) == op
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
    let op_id = NEXT_OP_ID.fetch_add(1, Ordering::Relaxed);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _permit = permit;
        CURRENT_OP.with(|c| c.set(op_id));
        let result = work();
        CURRENT_OP.with(|c| c.set(0));
        let _ = tx.send(result);
    });
    match rx.recv_timeout(budget) {
        Ok(v) => Ok(v),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            CANCELLED_OP.store(op_id, Ordering::Release);
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
    use std::sync::Barrier;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn budget_secs_zero_means_unlimited() {
        assert_eq!(resolve_budget_secs(Some(0)), None);
    }

    #[test]
    fn timeout_cancel_does_not_poison_sibling_ops() {
        let barrier = Arc::new(Barrier::new(2));
        let barrier_slow = Arc::clone(&barrier);
        let barrier_fast = Arc::clone(&barrier);

        let slow = thread::spawn(move || {
            run_bounded(Some(1), move || {
                barrier_slow.wait();
                // Hold past the 1s budget so the parent marks this op cancelled.
                thread::sleep(Duration::from_millis(1500));
                dl_cancel_requested()
            })
        });

        let fast = thread::spawn(move || {
            barrier_fast.wait();
            // Sibling must not observe the slow op's cancel flag.
            run_bounded(Some(5), || {
                thread::sleep(Duration::from_millis(50));
                dl_cancel_requested()
            })
        });

        let slow_result = slow.join().expect("slow join");
        assert!(slow_result.is_err(), "slow op should time out");

        let fast_result = fast.join().expect("fast join").expect("fast op");
        assert!(
            !fast_result,
            "fast sibling must not see cancel from timed-out peer"
        );
    }
}
