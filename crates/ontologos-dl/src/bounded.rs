//! Wall-clock budgets and capped parallelism for DL user paths.

use std::cell::Cell;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::Duration;

use crate::Error;

/// Monotonic id for each bounded DL operation (0 = no active op on this thread).
static NEXT_OP_ID: AtomicU64 = AtomicU64::new(1);
/// Op id that should cooperatively cancel (0 = none). Per-op, not global, so parallel
/// scans do not poison each other when one case times out.
static CANCELLED_OP: AtomicU64 = AtomicU64::new(0);
/// Timed-out jobs still executing on pool workers; reject new work when at capacity.
static ORPHANED_WORKERS: AtomicUsize = AtomicUsize::new(0);

thread_local! {
    static CURRENT_OP: Cell<u64> = const { Cell::new(0) };
}

type PoolJob = Box<dyn FnOnce() + Send>;

struct DlThreadPool {
    submit_tx: std::sync::mpsc::Sender<PoolJob>,
}

/// Whether the current DL worker should cooperatively stop (budget timeout for *this* op).
#[must_use]
pub fn dl_cancel_requested() -> bool {
    let op = CURRENT_OP.with(Cell::get);
    op != 0 && CANCELLED_OP.load(Ordering::Acquire) == op
}

/// Whether WG corpus consistency shortcuts are enabled.
///
/// Enabled in unit tests (`cfg(test)`), or in debug builds when `ONTOLOGOS_WG_SHORTCUTS=1`.
/// Production release builds never enable shortcuts via environment variables.
#[must_use]
pub fn wg_shortcuts_enabled() -> bool {
    if cfg!(test) {
        return true;
    }
    #[cfg(debug_assertions)]
    {
        wg_shortcuts_env_enabled()
    }
    #[cfg(not(debug_assertions))]
    {
        false
    }
}

fn wg_shortcuts_env_enabled() -> bool {
    std::env::var("ONTOLOGOS_WG_SHORTCUTS")
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
pub fn resolve_budget_secs(config: Option<u64>) -> crate::Result<Option<Duration>> {
    let from_env = || {
        std::env::var("ONTOLOGOS_DL_BUDGET_SECS")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&secs| secs > 0)
            .map(Duration::from_secs)
    };
    match config {
        Some(0) => Err(Error::Message(
            "budget_secs must be > 0 when set; use None for unlimited".into(),
        )),
        Some(secs) => Ok(Some(Duration::from_secs(secs))),
        None => Ok(from_env()),
    }
}

/// Run `work` on a worker thread with an optional wall-clock budget.
pub fn run_bounded<T, F>(budget_secs: Option<u64>, work: F) -> crate::Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let Some(budget) = resolve_budget_secs(budget_secs)? else {
        return Ok(work());
    };
    run_bounded_inner(budget, work)
}

fn run_bounded_inner<T, F>(budget: Duration, work: F) -> crate::Result<T>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let max_workers = dl_max_workers();
    if ORPHANED_WORKERS.load(Ordering::Acquire) >= max_workers {
        return Err(Error::IncompleteReasoning(format!(
            "dl worker pool saturated ({max_workers} timed-out workers still running)"
        )));
    }

    let gate = dl_worker_gate();
    let permit = acquire_dl_worker_permit(&gate);
    let reclaimed = permit.reclaimed.clone();
    let op_id = NEXT_OP_ID.fetch_add(1, Ordering::Relaxed);
    let orphaned = Arc::new(AtomicBool::new(false));
    let orphaned_flag = Arc::clone(&orphaned);
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let job: PoolJob = Box::new(move || {
        CURRENT_OP.with(|c| c.set(op_id));
        let result = work();
        CURRENT_OP.with(|c| c.set(0));
        if orphaned_flag.load(Ordering::Acquire) {
            ORPHANED_WORKERS.fetch_sub(1, Ordering::AcqRel);
        }
        let _ = tx.send(result);
    });
    dl_thread_pool()
        .submit_tx
        .send(job)
        .map_err(|_| Error::IncompleteReasoning("dl worker pool shut down".into()))?;
    match rx.recv_timeout(budget) {
        Ok(v) => Ok(v),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
            CANCELLED_OP.store(op_id, Ordering::Release);
            orphaned.store(true, Ordering::Release);
            ORPHANED_WORKERS.fetch_add(1, Ordering::AcqRel);
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

fn dl_thread_pool() -> &'static DlThreadPool {
    static POOL: std::sync::OnceLock<DlThreadPool> = std::sync::OnceLock::new();
    POOL.get_or_init(|| {
        let (submit_tx, submit_rx) = std::sync::mpsc::channel::<PoolJob>();
        let shared_rx = Arc::new(Mutex::new(submit_rx));
        let max = dl_max_workers();
        for worker in 0..max {
            let rx = Arc::clone(&shared_rx);
            std::thread::Builder::new()
                .name(format!("ontologos-dl-{worker}"))
                .spawn(move || {
                    loop {
                        let job = match rx.lock().expect("dl pool rx").recv() {
                            Ok(job) => job,
                            Err(_) => break,
                        };
                        job();
                    }
                })
                .expect("spawn dl worker");
        }
        DlThreadPool { submit_tx }
    })
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
    fn budget_secs_zero_is_rejected() {
        let err = resolve_budget_secs(Some(0)).unwrap_err();
        assert!(err.to_string().contains("budget_secs"));
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
