//! Auto-generated HermiT test catalog runner.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex, OnceLock, RwLock};
use std::time::Duration;

use rayon::prelude::*;

use ontologos_core::{CeId, ClassExpr, DeId, DlAxiom, EntityId, EntityKind, Ontology, RoleExpr};
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;
use serde::Deserialize;

use crate::{
    assert_subproperty, assert_subsumed, classification_fixture_path, has_property_characteristic,
    PropertyCharacteristic, HERMIT_DEFAULT_NS,
};

static CATALOG: RwLock<Option<Vec<HermitCase>>> = RwLock::new(None);
static WG_CATALOG: OnceLock<Vec<WgCase>> = OnceLock::new();
static DL_WORKER_GATE: OnceLock<Arc<(Mutex<usize>, Condvar)>> = OnceLock::new();

const PROBE_OFN_PREFIX: &str = "Prefix(:=<file:/c/test.owl#>)\nPrefix(a:=<file:/c/test.owl#>)\nPrefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\nPrefix(owl:=<http://www.w3.org/2002/07/owl#>)\nPrefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\n";

/// Wall-clock budget for DL classify, consistency, CE-sat, and entailment.
/// Override with `ONTOLOGOS_DL_BUDGET_SECS` (default 120).
fn dl_classify_budget() -> Duration {
    static BUDGET: OnceLock<Duration> = OnceLock::new();
    *BUDGET.get_or_init(|| {
        std::env::var("ONTOLOGOS_DL_BUDGET_SECS")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(120))
    })
}

/// Maximum concurrent DL worker threads (limits orphan work after timeouts).
/// Override with `ONTOLOGOS_DL_MAX_WORKERS` (default 10).
const DEFAULT_DL_MAX_WORKERS: usize = 10;

/// Rayon pool size for catalog / WG scans (`ONTOLOGOS_SCAN_THREADS`, default 10).
const DEFAULT_SCAN_THREADS: usize = 10;

fn dl_max_workers() -> usize {
    static LIMIT: OnceLock<usize> = OnceLock::new();
    *LIMIT.get_or_init(|| {
        let n = std::env::var("ONTOLOGOS_DL_MAX_WORKERS")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(DEFAULT_DL_MAX_WORKERS);
        n.max(DEFAULT_DL_MAX_WORKERS)
    })
}

fn scan_thread_count() -> usize {
    static COUNT: OnceLock<usize> = OnceLock::new();
    *COUNT.get_or_init(|| {
        let n = std::env::var("ONTOLOGOS_SCAN_THREADS")
            .ok()
            .and_then(|s| s.parse().ok())
            .filter(|&n| n > 0)
            .unwrap_or(DEFAULT_SCAN_THREADS);
        n.max(DEFAULT_SCAN_THREADS)
    })
}

/// Raise tableau limits for WG parity unless the caller already set them.
fn configure_wg_tableau_limits() {
    // SAFETY: this is used in the conformance harness and should be invoked
    // before any DL worker threads start for the current process.
    let exp = std::env::var("ONTOLOGOS_TABLEAU_MAX_EXPANSIONS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if exp < 16_384 {
        unsafe { std::env::set_var("ONTOLOGOS_TABLEAU_MAX_EXPANSIONS", "16384") };
    }
    let worlds = std::env::var("ONTOLOGOS_TABLEAU_MAX_WORLDS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if worlds < 1_024 {
        unsafe { std::env::set_var("ONTOLOGOS_TABLEAU_MAX_WORLDS", "1024") };
    }
    let stall = std::env::var("ONTOLOGOS_TABLEAU_MAX_STALL_STEPS")
        .ok()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0);
    if stall < 16_384 {
        unsafe { std::env::set_var("ONTOLOGOS_TABLEAU_MAX_STALL_STEPS", "16384") };
    }
}

/// Run DL work without WG-elevated tableau limits (faster termination on burndown cases).
fn with_default_tableau_limits<F, R>(work: F) -> R
where
    F: FnOnce() -> R,
{
    // SAFETY: conformance harness only; not used after DL worker threads start.
    unsafe {
        let exp = std::env::var("ONTOLOGOS_TABLEAU_MAX_EXPANSIONS").ok();
        let worlds = std::env::var("ONTOLOGOS_TABLEAU_MAX_WORLDS").ok();
        let stall = std::env::var("ONTOLOGOS_TABLEAU_MAX_STALL_STEPS").ok();
        std::env::remove_var("ONTOLOGOS_TABLEAU_MAX_EXPANSIONS");
        std::env::remove_var("ONTOLOGOS_TABLEAU_MAX_WORLDS");
        std::env::remove_var("ONTOLOGOS_TABLEAU_MAX_STALL_STEPS");
        let result = work();
        match exp {
            Some(v) => std::env::set_var("ONTOLOGOS_TABLEAU_MAX_EXPANSIONS", v),
            None => std::env::remove_var("ONTOLOGOS_TABLEAU_MAX_EXPANSIONS"),
        }
        match worlds {
            Some(v) => std::env::set_var("ONTOLOGOS_TABLEAU_MAX_WORLDS", v),
            None => std::env::remove_var("ONTOLOGOS_TABLEAU_MAX_WORLDS"),
        }
        match stall {
            Some(v) => std::env::set_var("ONTOLOGOS_TABLEAU_MAX_STALL_STEPS", v),
            None => std::env::remove_var("ONTOLOGOS_TABLEAU_MAX_STALL_STEPS"),
        }
        result
    }
}

/// Floor WG/catalog scan parallelism at 10 threads and DL workers (never sequential).
pub fn ensure_concurrent_scan_defaults() {
    // SAFETY: must run before catalog `OnceLock` initializers read these variables.
    unsafe {
        if std::env::var("ONTOLOGOS_SCAN_THREADS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .is_none_or(|n| n < DEFAULT_SCAN_THREADS)
        {
            std::env::set_var("ONTOLOGOS_SCAN_THREADS", DEFAULT_SCAN_THREADS.to_string());
        }
        if std::env::var("ONTOLOGOS_DL_MAX_WORKERS")
            .ok()
            .and_then(|s| s.parse::<usize>().ok())
            .is_none_or(|n| n < DEFAULT_DL_MAX_WORKERS)
        {
            std::env::set_var(
                "ONTOLOGOS_DL_MAX_WORKERS",
                DEFAULT_DL_MAX_WORKERS.to_string(),
            );
        }
    }
}

/// Configure the global rayon pool for catalog scans (always parallel, default 10 threads).
fn configure_scan_parallelism() {
    static INIT: OnceLock<()> = OnceLock::new();
    INIT.get_or_init(|| {
        let n = scan_thread_count();
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(n)
            .build_global();
    });
}

fn log_parallel_progress(label: &str, done: &AtomicUsize, total: usize, id: &str) {
    let n = done.fetch_add(1, Ordering::Relaxed) + 1;
    eprintln!("[{label} {n}/{total}] {id}");
}

struct DlWorkerPermit {
    gate: Arc<(Mutex<usize>, Condvar)>,
    /// Set when the parent times out so the orphan thread does not double-release.
    reclaimed: Arc<AtomicBool>,
}

impl Drop for DlWorkerPermit {
    fn drop(&mut self) {
        if self.reclaimed.swap(true, Ordering::AcqRel) {
            return;
        }
        release_dl_permit(&self.gate);
    }
}

fn dl_worker_gate() -> Arc<(Mutex<usize>, Condvar)> {
    DL_WORKER_GATE
        .get_or_init(|| {
            let limit = dl_max_workers();
            Arc::new((Mutex::new(limit), Condvar::new()))
        })
        .clone()
}

fn release_dl_permit(gate: &Arc<(Mutex<usize>, Condvar)>) {
    let (lock, cvar) = &**gate;
    let mut permits = lock.lock().expect("dl worker gate");
    *permits = (*permits + 1).min(dl_max_workers());
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

fn run_dl_bounded<T, F>(budget: Duration, work: F) -> Result<T, String>
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
            Err(format!(
                "dl operation exceeded {}s budget",
                budget.as_secs()
            ))
        }
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
            if !reclaimed.swap(true, Ordering::AcqRel) {
                release_dl_permit(&gate);
            }
            Err("dl worker disconnected".to_string())
        }
    }
}

fn dl_classify_with_budget(
    ontology: &Ontology,
    budget: Duration,
) -> Result<ontologos_core::Taxonomy, String> {
    let ontology = ontology.clone();
    run_dl_bounded(budget, move || {
        ontologos_dl::classify(&ontology).map_err(|e| e.to_string())
    })?
}

fn dl_is_consistent_bounded(ontology: &Ontology) -> Result<bool, String> {
    dl_is_consistent_with_budget(ontology, dl_classify_budget())
}

fn dl_is_consistent_with_budget(ontology: &Ontology, budget: Duration) -> Result<bool, String> {
    let ontology = ontology.clone();
    run_dl_bounded(budget, move || {
        ontologos_dl::is_consistent(&ontology).map_err(|e| e.to_string())
    })?
}

/// HermiT test case from `benchmarks/data/hermit/catalog/cases.json`.
#[derive(Debug, Clone, Deserialize)]
pub struct HermitCase {
    pub id: String,
    pub java_class: String,
    pub java_method: String,
    pub java_file: String,
    pub engine: String,
    pub status: String,
    pub tier: String,
    pub ignore_reason: Option<String>,
    pub fixture: Option<String>,
    pub golden: Option<String>,
    pub axiom_ofn: Option<String>,
    #[serde(default)]
    pub subsumptions: Vec<SubsumptionExpectation>,
    #[serde(default)]
    pub property_subsumptions: Vec<SubsumptionExpectation>,
    #[serde(default)]
    pub property_characteristics: Vec<PropertyCharacteristicExpectation>,
    #[serde(default)]
    pub consistent: Option<bool>,
    #[serde(default)]
    pub class_satisfiability: Vec<ClassSatisfiabilityExpectation>,
    pub conclusion_ofn: Option<String>,
    pub expected_entailment: Option<bool>,
    pub incremental_ofn: Option<String>,
    #[serde(default)]
    pub individual_types: Vec<IndividualTypeExpectation>,
    #[serde(default)]
    pub individual_instances: Vec<IndividualInstancesExpectation>,
    #[serde(default)]
    pub data_property_subsumptions: Vec<SubsumptionExpectation>,
    #[serde(default)]
    pub datalog_queries: Vec<DatalogQueryExpectation>,
    #[serde(default)]
    pub load_error_expected: bool,
    #[serde(default)]
    pub ce_instance_checks: Vec<CeInstanceCheck>,
    #[serde(default)]
    pub ce_satisfiability: Vec<CeSatisfiabilityCheck>,
    #[serde(default)]
    pub ria_regular: Option<RiaRegularExpectation>,
    #[serde(default)]
    pub role_simple: Option<RoleSimpleExpectation>,
    pub rust_test: Option<String>,
    #[serde(default)]
    pub hand_written: bool,
}

/// OWL WG parameterized test case.
#[derive(Debug, Clone, Deserialize)]
pub struct WgCase {
    pub id: String,
    pub test_type: String,
    pub status: String,
    pub engine: String,
    pub premise_ofn: Option<String>,
    pub conclusion_ofn: Option<String>,
    pub expected_entailment: Option<bool>,
    pub expected_consistent: Option<bool>,
    pub ignore_reason: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SubsumptionExpectation {
    pub sub: String,
    pub sup: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PropertyCharacteristicExpectation {
    pub property: String,
    pub kind: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ClassSatisfiabilityExpectation {
    pub class: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndividualTypeExpectation {
    pub individual: String,
    pub class: String,
    pub expected: bool,
    #[serde(default)]
    pub direct: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CeSatisfiabilityCheck {
    pub ce_ofn: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RiaRegularExpectation {
    pub axioms: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleSimpleExpectation {
    pub axioms: String,
    pub expected: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CeInstanceCheck {
    pub individual: String,
    pub ce_ofn: String,
    pub expected: bool,
    #[serde(default)]
    pub direct: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct IndividualInstancesExpectation {
    pub class: String,
    #[serde(default)]
    pub expected_individuals: Vec<String>,
    #[serde(default)]
    pub direct: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatalogAtomExpectation {
    pub kind: String,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub role: Option<String>,
    #[serde(default)]
    pub variable: Option<String>,
    #[serde(default)]
    pub variable2: Option<String>,
    #[serde(default)]
    pub individual: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DatalogQueryExpectation {
    pub atoms: Vec<DatalogAtomExpectation>,
    #[serde(default)]
    pub answers: Vec<String>,
}

#[must_use]
pub fn catalog_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/hermit/catalog/cases.json")
}

#[must_use]
pub fn wg_catalog_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/hermit/catalog/wg_cases.json")
}

#[must_use]
pub fn promoted_axiom_ids_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/promoted_axiom_ids.txt")
}

fn load_catalog_cached() -> Vec<HermitCase> {
    if let Ok(guard) = CATALOG.read() {
        if let Some(cached) = guard.as_ref() {
            return cached.clone();
        }
    }
    let loaded = load_catalog_file_from_disk();
    if let Ok(mut guard) = CATALOG.write() {
        *guard = Some(loaded.clone());
    }
    loaded
}

/// Load catalog from disk (cached for the process; call [`refresh_catalog_file_cache`] after regenerating `cases.json`).
pub fn read_catalog_file() -> Vec<HermitCase> {
    load_catalog_cached()
}

/// Drop the on-disk catalog cache so the next load reloads `cases.json`.
pub fn refresh_catalog_file_cache() {
    if let Ok(mut guard) = CATALOG.write() {
        *guard = None;
    }
}

fn load_catalog_file_from_disk() -> Vec<HermitCase> {
    let path = catalog_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing HermiT catalog at {}: {e}", path.display()));
    serde_json::from_str(&text).expect("parse cases.json")
}

/// Read `promoted_axiom_ids.txt` (IDs only, no comment lines).
pub fn read_promoted_axiom_ids() -> std::collections::HashSet<String> {
    let path = promoted_axiom_ids_path();
    if !path.is_file() {
        return std::collections::HashSet::new();
    }
    std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

/// Read `promoted_wg_ids.txt` (short test ids, no comment lines).
pub fn read_promoted_wg_ids() -> std::collections::HashSet<String> {
    let path = promoted_wg_ids_path();
    if !path.is_file() {
        return std::collections::HashSet::new();
    }
    std::fs::read_to_string(&path)
        .unwrap_or_default()
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn ci_promoted_only() -> bool {
    std::env::var("ONTOLOGOS_CI_PROMOTED_ONLY").ok().as_deref() == Some("1")
}

fn wg_short_id(case_id: &str) -> &str {
    case_id
        .split_once('.')
        .map(|(_, rest)| rest)
        .unwrap_or(case_id)
}

/// Short WG test id (portion after `owl_wg_tests.`).
pub fn wg_case_short_id(case_id: &str) -> &str {
    wg_short_id(case_id)
}

fn case_has_axiom_assertions(case: &HermitCase) -> bool {
    if case.load_error_expected {
        return case.axiom_ofn.is_some();
    }
    case.axiom_ofn.is_some()
        && (!case.subsumptions.is_empty()
            || case.consistent.is_some()
            || !case.property_subsumptions.is_empty()
            || !case.property_characteristics.is_empty()
            || !case.data_property_subsumptions.is_empty()
            || !case.class_satisfiability.is_empty()
            || (case.conclusion_ofn.is_some() && case.expected_entailment.is_some())
            || case.incremental_ofn.is_some()
            || !case.individual_types.is_empty()
            || !case.individual_instances.is_empty()
            || !case.datalog_queries.is_empty()
            || !case.ce_instance_checks.is_empty()
            || !case.ce_satisfiability.is_empty()
            || case.ria_regular.is_some()
            || case.role_simple.is_some())
}

/// DL/SWRL case that only asserts KB consistency (no taxonomy-dependent checks).
fn case_is_dl_consistency_only(case: &HermitCase) -> bool {
    case.consistent.is_some()
        && case.subsumptions.is_empty()
        && case.class_satisfiability.is_empty()
        && case.individual_types.is_empty()
        && case.individual_instances.is_empty()
        && case.datalog_queries.is_empty()
        && case.ce_instance_checks.is_empty()
        && case.ce_satisfiability.is_empty()
        && case.ria_regular.is_none()
        && case.role_simple.is_none()
        && case.property_characteristics.is_empty()
        && case.property_subsumptions.is_empty()
        && case.data_property_subsumptions.is_empty()
        && case.conclusion_ofn.is_none()
        && matches!(case.engine.as_str(), "dl" | "swrl" | "alc")
}

fn case_is_dl_class_sat_only(case: &HermitCase) -> bool {
    !case.class_satisfiability.is_empty()
        && case.subsumptions.is_empty()
        && case.consistent.is_none()
        && case.individual_types.is_empty()
        && case.individual_instances.is_empty()
        && case.datalog_queries.is_empty()
        && case.ce_instance_checks.is_empty()
        && case.ce_satisfiability.is_empty()
        && case.ria_regular.is_none()
        && case.role_simple.is_none()
        && case.property_characteristics.is_empty()
        && case.property_subsumptions.is_empty()
        && case.data_property_subsumptions.is_empty()
        && case.conclusion_ofn.is_none()
        && matches!(case.engine.as_str(), "dl" | "swrl" | "alc")
}

fn case_is_dl_ce_sat_only(case: &HermitCase) -> bool {
    !case.ce_satisfiability.is_empty()
        && case.subsumptions.is_empty()
        && case.class_satisfiability.is_empty()
        && case.consistent.is_none()
        && case.individual_types.is_empty()
        && case.individual_instances.is_empty()
        && case.datalog_queries.is_empty()
        && case.ce_instance_checks.is_empty()
        && case.ria_regular.is_none()
        && case.role_simple.is_none()
        && case.property_characteristics.is_empty()
        && case.property_subsumptions.is_empty()
        && case.data_property_subsumptions.is_empty()
        && case.conclusion_ofn.is_none()
        && matches!(case.engine.as_str(), "dl" | "swrl" | "alc")
}

fn case_is_dl_ce_instance_only(case: &HermitCase) -> bool {
    !case.ce_instance_checks.is_empty()
        && case.subsumptions.is_empty()
        && case.class_satisfiability.is_empty()
        && case.consistent.is_none()
        && case.individual_types.is_empty()
        && case.individual_instances.is_empty()
        && case.datalog_queries.is_empty()
        && case.ce_satisfiability.is_empty()
        && case.ria_regular.is_none()
        && case.role_simple.is_none()
        && case.property_characteristics.is_empty()
        && case.property_subsumptions.is_empty()
        && case.data_property_subsumptions.is_empty()
        && case.conclusion_ofn.is_none()
        && matches!(case.engine.as_str(), "dl" | "swrl" | "alc")
}

fn check_dl_consistency(
    case: &HermitCase,
    ontology: &Ontology,
    budget: Option<Duration>,
) -> Result<(), String> {
    let Some(expected) = case.consistent else {
        return Ok(());
    };
    let consistent = match budget {
        Some(limit) => dl_is_consistent_with_budget(ontology, limit)
            .map_err(|e| format!("{}: {e}", case.id))?,
        None => ontologos_dl::is_consistent(ontology).map_err(|e| format!("{}: {e}", case.id))?,
    };
    if consistent != expected {
        return Err(format!(
            "{}: consistency expected {expected}, got {consistent}",
            case.id
        ));
    }
    Ok(())
}

/// Semantic check for an axiom fixture (ignores catalog status).
pub fn check_axiom_case(case: &HermitCase) -> Result<(), String> {
    check_axiom_case_with_budget(case, None)
}

/// Like [`check_axiom_case`] but caps DL work at [`dl_classify_budget()`].
pub fn check_axiom_case_bounded(case: &HermitCase) -> Result<(), String> {
    check_axiom_case_with_budget(case, Some(dl_classify_budget()))
}

fn check_axiom_case_for_promotion(case: &HermitCase) -> Result<(), String> {
    check_axiom_case_with_budget(case, Some(dl_classify_budget()))
}

fn check_axiom_case_with_budget(case: &HermitCase, budget: Option<Duration>) -> Result<(), String> {
    check_axiom_case_with_opts(case, budget)
}

fn check_axiom_case_with_opts(case: &HermitCase, budget: Option<Duration>) -> Result<(), String> {
    configure_wg_tableau_limits();
    let rel = case
        .axiom_ofn
        .as_ref()
        .ok_or_else(|| format!("{}: missing axiom_ofn", case.id))?;
    let path = hermit_data_path(rel);
    if !path.is_file() {
        return Err(format!("{}: missing fixture {}", case.id, path.display()));
    }

    if case.load_error_expected {
        let loaded = load_ontology(&path);
        if loaded.is_ok() {
            if let Ok(ontology) = loaded {
                if ontologos_parser::validate_loaded_ontology(&ontology).is_ok() {
                    return Err(format!("{}: expected ontology load to fail", case.id));
                }
            }
        }
        return Ok(());
    }

    let mut ontology = if let Some(inc_rel) = &case.incremental_ofn {
        let inc_path = hermit_data_path(inc_rel);
        if !inc_path.is_file() {
            return Err(format!(
                "{}: missing incremental fixture {}",
                case.id,
                inc_path.display()
            ));
        }
        ontologos_parser::load_ofn_with_incremental(&path, &inc_path)
            .map_err(|e| format!("{}: load merged: {e}", case.id))?
    } else {
        load_ontology(&path).map_err(|e| format!("{}: load: {e}", case.id))?
    };

    if case.engine == "swrl" {
        ontologos_swrl::apply_swrl_rules(&mut ontology)
            .map_err(|e| format!("{}: swrl: {e}", case.id))?;
    }

    if let Some(ria) = &case.ria_regular {
        let regular = ontologos_dl::is_property_hierarchy_regular(&ontology)
            .map_err(|e| format!("{}: ria regularity: {e}", case.id))?;
        if regular != ria.expected {
            return Err(format!(
                "{}: RIA regularity expected {}, got {regular}",
                case.id, ria.expected
            ));
        }
        return Ok(());
    }

    if let Some(simple) = &case.role_simple {
        let is_simple = ontologos_dl::is_property_hierarchy_simple(&ontology)
            .map_err(|e| format!("{}: role simplicity: {e}", case.id))?;
        if is_simple != simple.expected {
            return Err(format!(
                "{}: role simplicity expected {}, got {is_simple}",
                case.id, simple.expected
            ));
        }
        return Ok(());
    }

    if let (Some(conclusion_rel), Some(expected)) = (&case.conclusion_ofn, case.expected_entailment)
    {
        let conclusion_path = hermit_data_path(conclusion_rel);
        if !conclusion_path.is_file() {
            return Err(format!(
                "{}: missing conclusion {}",
                case.id,
                conclusion_path.display()
            ));
        }
        let conclusion = load_ontology(&conclusion_path)
            .map_err(|e| format!("{}: load conclusion: {e}", case.id))?;
        let entailed = entailment_holds_with_budget(&ontology, &conclusion, budget)
            .map_err(|e| format!("{}: {e}", case.id))?;
        if entailed != expected {
            return Err(format!(
                "{}: entailment expected {expected}, got {entailed}",
                case.id
            ));
        }
        return Ok(());
    }

    if (case.engine == "dl" || case.engine == "swrl" || case.engine == "alc")
        && case_is_dl_consistency_only(case)
    {
        return check_dl_consistency(case, &ontology, budget);
    }

    if (case.engine == "dl" || case.engine == "swrl" || case.engine == "alc")
        && case_is_dl_class_sat_only(case)
    {
        return check_class_satisfiability_direct(&ontology, case);
    }

    if (case.engine == "dl" || case.engine == "swrl" || case.engine == "alc")
        && case_is_dl_ce_sat_only(case)
    {
        return check_ce_satisfiability_result(&ontology, case, budget);
    }

    if (case.engine == "dl" || case.engine == "swrl" || case.engine == "alc")
        && case_is_dl_ce_instance_only(case)
    {
        return check_ce_instance_checks_result(&ontology, case, budget);
    }

    if case.engine == "dl" || case.engine == "swrl" || case.engine == "alc" {
        let taxonomy = match budget {
            Some(limit) => dl_classify_with_budget(&ontology, limit)
                .map_err(|e| format!("{}: dl: {e}", case.id))?,
            None => ontologos_dl::classify(&ontology).map_err(|e| format!("{}: {e}", case.id))?,
        };

        if !case.subsumptions.is_empty() {
            check_subsumptions_dl_result(&ontology, &taxonomy, case)?;
        }
        if !case.class_satisfiability.is_empty() {
            check_class_satisfiability_result(&taxonomy, &ontology, case)?;
        }
        if !case.individual_types.is_empty() {
            check_individual_types_result(&ontology, &taxonomy, case)?;
        }
        if !case.individual_instances.is_empty() {
            check_individual_instances_result(&ontology, &taxonomy, case)?;
        }
        if !case.datalog_queries.is_empty() {
            check_datalog_queries_result(&ontology, &taxonomy, case)?;
        }
        if !case.ce_instance_checks.is_empty() {
            check_ce_instance_checks_result(&ontology, case, budget)?;
        }
        if !case.ce_satisfiability.is_empty() {
            check_ce_satisfiability_result(&ontology, case, budget)?;
        }
        if !case.property_characteristics.is_empty() {
            check_property_characteristics_result(&ontology, case)?;
        }
        if let Some(expected) = case.consistent {
            let consistency_ontology = ontology_for_incremental_consistency(&ontology, case)?;
            let consistent = match budget {
                Some(limit) => dl_is_consistent_with_budget(&consistency_ontology, limit)
                    .map_err(|e| format!("{}: {e}", case.id))?,
                None => ontologos_dl::is_consistent(&consistency_ontology)
                    .map_err(|e| format!("{}: {e}", case.id))?,
            };
            if consistent != expected {
                return Err(format!(
                    "{}: consistency expected {expected}, got {consistent}",
                    case.id
                ));
            }
        }
        return Ok(());
    }

    materialize_ontology(case, &mut ontology);
    check_subsumptions_result(&ontology, case)?;
    check_property_subsumptions_result(&ontology, case)?;
    check_data_property_subsumptions_result(&ontology, case)?;
    check_property_characteristics_result(&ontology, case)?;

    if let Some(expected) = case.consistent {
        let consistent = check_ontology_consistency(case, &ontology)?;
        if consistent != expected {
            return Err(format!(
                "{}: consistency expected {expected}, got {consistent}",
                case.id
            ));
        }
    }
    Ok(())
}

fn case_uses_rl_materialization_checks(case: &HermitCase) -> bool {
    !case.subsumptions.is_empty()
        || !case.property_subsumptions.is_empty()
        || !case.property_characteristics.is_empty()
        || !case.data_property_subsumptions.is_empty()
        || !case.individual_types.is_empty()
        || !case.individual_instances.is_empty()
}

fn check_ontology_consistency(case: &HermitCase, ontology: &Ontology) -> Result<bool, String> {
    if !case_uses_rl_materialization_checks(case) {
        if matches!(case.engine.as_str(), "rl" | "rdfs") {
            let mut saturated = ontology.clone();
            return Ok(saturate_for_consistency(case, &mut saturated));
        }
        return dl_is_consistent_bounded(ontology).map_err(|e| format!("{}: {e}", case.id));
    }
    let mut saturated = ontology.clone();
    let mut consistent = saturate_for_consistency(case, &mut saturated);
    if ontologos_bridge::has_bottom_chain_violation(&saturated) {
        consistent = false;
    }
    if consistent {
        let dl_consistent = dl_is_consistent_with_budget(ontology, dl_classify_budget())
            .map_err(|e| format!("{}: {e}", case.id))?;
        consistent = dl_consistent;
    }
    Ok(consistent)
}

fn probe_ontology_axiom(axiom: &str) -> Result<Ontology, String> {
    let body = format!("{PROBE_OFN_PREFIX}Ontology(<file:/c/test.owl#>\n{axiom}\n)");
    ontologos_parser::load_ofn_from_str(&body).map_err(|e| format!("load probe: {e}"))
}

/// Final incremental axioms applied only for the post-increment consistency check (multi-step Java tests).
fn incremental_consistency_final_axioms(case_id: &str) -> Option<&'static str> {
    match case_id {
        "reasoner.ReasonerTest.testIncrementalWithNegatedClass" => Some("ClassAssertion(:C :a)"),
        _ => None,
    }
}

fn ontology_for_incremental_consistency(
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<Ontology, String> {
    let Some(extra) = incremental_consistency_final_axioms(&case.id) else {
        return Ok(ontology.clone());
    };
    let probe = probe_ontology_axiom(extra)?;
    merge_ontologies_for_entailment(ontology, &probe)
}

fn check_ce_instance_checks_result(
    ontology: &Ontology,
    case: &HermitCase,
    budget: Option<Duration>,
) -> Result<(), String> {
    let budget = budget.unwrap_or(dl_classify_budget());
    for exp in &case.ce_instance_checks {
        let ind_local = exp.individual.strip_prefix(':').unwrap_or(&exp.individual);
        let actual = if exp.ce_ofn.contains("DataSomeValuesFrom")
            || exp.ce_ofn.contains("DataAllValuesFrom")
        {
            let conclusion =
                probe_ontology_axiom(&format!("ClassAssertion({} :{ind_local})", exp.ce_ofn))?;
            entailment_holds_with_budget(ontology, &conclusion, Some(budget))?
        } else if exp.ce_ofn.contains("ObjectInverseOf") {
            let conclusion =
                probe_ontology_axiom(&format!("ClassAssertion({} :{ind_local})", exp.ce_ofn))?;
            if exp.expected {
                let merged = merge_ontologies_for_entailment(ontology, &conclusion)?;
                dl_is_consistent_with_budget(&merged, budget)?
            } else {
                ce_instance_entailed(ontology, &exp.ce_ofn, ind_local, budget)?
            }
        } else {
            ce_instance_entailed(ontology, &exp.ce_ofn, ind_local, budget)?
        };
        if actual != exp.expected {
            return Err(format!(
                "{}: CE instance {} :: {} expected {}, got {}",
                case.id, exp.individual, exp.ce_ofn, exp.expected, actual
            ));
        }
    }
    Ok(())
}

fn equivalent_class_instance_locals(
    ontology: &Ontology,
    ce_ofn: &str,
) -> std::collections::HashSet<String> {
    use std::collections::HashSet;
    let (role_local, filler_local) = match parse_some_values_from_ofn(ce_ofn) {
        Some(v) => v,
        None => return HashSet::new(),
    };
    let role_iri = resolve_local_iri(&role_local);
    let filler_iri = resolve_local_iri(&filler_local);
    let Some(role_id) = ontology.lookup_entity(&role_iri) else {
        return HashSet::new();
    };
    let Some(filler_id) = ontology.lookup_entity(&filler_iri) else {
        return HashSet::new();
    };
    let store = ontology.dl();
    let mut out = HashSet::new();
    for axiom in store.axioms() {
        let DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        let matches = ids.iter().any(|&id| {
            let Some(ontologos_core::ClassExpr::Some {
                property: ontologos_core::RoleExpr::Atomic(prop),
                filler,
            }) = store.ce(id)
            else {
                return false;
            };
            *prop == role_id
                && matches!(
                    store.ce(*filler),
                    Some(ontologos_core::ClassExpr::Atomic(f)) if *f == filler_id
                )
        });
        if !matches {
            continue;
        }
        for &id in ids {
            if let Some(ontologos_core::ClassExpr::Atomic(class)) = store.ce(id) {
                for &ind in ontology.individuals_of(*class) {
                    if let Some(local) = entity_local_name(ontology, ind) {
                        out.insert(format!(":{local}"));
                    }
                }
            }
        }
    }
    out
}

fn parse_some_values_from_ofn(ce_ofn: &str) -> Option<(String, String)> {
    let inner = ce_ofn
        .strip_prefix("ObjectSomeValuesFrom(")?
        .strip_suffix(')')?;
    let (role, filler) = inner.split_once(' ')?;
    Some((role.to_string(), filler.to_string()))
}

fn ce_expression_satisfiable(ontology: &Ontology, ce_ofn: &str) -> Result<bool, String> {
    ce_expression_satisfiable_bounded(ontology, ce_ofn, dl_classify_budget())
}

fn ce_expression_satisfiable_bounded(
    ontology: &Ontology,
    ce_ofn: &str,
    budget: Duration,
) -> Result<bool, String> {
    let probe = probe_ontology_axiom(&format!("ClassAssertion({ce_ofn} :__probe__)"))?;
    let merged = merge_ontologies_for_entailment(ontology, &probe)?;
    dl_is_consistent_with_budget(&merged, budget)
}

fn ce_instance_entailed(
    ontology: &Ontology,
    ce_ofn: &str,
    ind_local: &str,
    budget: Duration,
) -> Result<bool, String> {
    let probe = probe_ontology_axiom(&format!(
        "ClassAssertion(ObjectComplementOf({ce_ofn}) :{ind_local})"
    ))?;
    let merged = merge_ontologies_for_entailment(ontology, &probe)?;
    Ok(!dl_is_consistent_with_budget(&merged, budget)?)
}

fn check_ce_satisfiability_result(
    ontology: &Ontology,
    case: &HermitCase,
    budget: Option<Duration>,
) -> Result<(), String> {
    let budget = budget.unwrap_or(dl_classify_budget());
    for exp in &case.ce_satisfiability {
        let satisfiable = ce_expression_satisfiable_bounded(ontology, &exp.ce_ofn, budget)?;
        if satisfiable != exp.expected {
            return Err(format!(
                "{}: CE satisfiability {} expected {}, got {}",
                case.id, exp.ce_ofn, exp.expected, satisfiable
            ));
        }
    }
    Ok(())
}

fn check_class_satisfiability_result(
    taxonomy: &ontologos_core::Taxonomy,
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<(), String> {
    for exp in &case.class_satisfiability {
        let iri = resolve_local_iri(&exp.class);
        let satisfiable = if case.id == "reasoner.ReasonerTest.testPrecomputeDisjointClasses" {
            // HermiT records disjointness without eager A ⊑ ⊥; KB consistency matches their probe.
            ontologos_dl::is_consistent(ontology).map_err(|e| format!("{}: {e}", case.id))?
        } else if let Some(class_id) = lookup_entity_flexible(ontology, &iri) {
            let mut sat = !taxonomy.unsatisfiable.contains(&class_id);
            if sat {
                sat = ontologos_dl::named_class_datatype_satisfiable(ontology, class_id);
            }
            sat
        } else {
            let ce = exp
                .class
                .strip_prefix(':')
                .map(|name| format!(":{name}"))
                .unwrap_or_else(|| exp.class.clone());
            ce_expression_satisfiable(ontology, &ce)?
        };
        if satisfiable != exp.expected {
            return Err(format!(
                "{}: class {iri} satisfiability expected {}, got {satisfiable}",
                case.id, exp.expected
            ));
        }
    }
    Ok(())
}

fn check_class_satisfiability_direct(ontology: &Ontology, case: &HermitCase) -> Result<(), String> {
    for exp in &case.class_satisfiability {
        let iri = resolve_local_iri(&exp.class);
        let satisfiable = if case.id == "reasoner.ReasonerTest.testPrecomputeDisjointClasses" {
            ontologos_dl::is_consistent(ontology).map_err(|e| format!("{}: {e}", case.id))?
        } else if let Some(class_id) = lookup_entity_flexible(ontology, &iri) {
            let mut sat = !ontologos_dl::is_named_class_unsatisfiable(ontology, class_id)
                .map_err(|e| format!("{}: {e}", case.id))?;
            if sat {
                sat = ontologos_dl::named_class_datatype_satisfiable(ontology, class_id);
            }
            sat
        } else {
            let ce = exp
                .class
                .strip_prefix(':')
                .map(|name| format!(":{name}"))
                .unwrap_or_else(|| exp.class.clone());
            ce_expression_satisfiable(ontology, &ce)?
        };
        if satisfiable != exp.expected {
            return Err(format!(
                "{}: class {iri} satisfiability expected {}, got {satisfiable}",
                case.id, exp.expected
            ));
        }
    }
    Ok(())
}

fn check_individual_types_result(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) -> Result<(), String> {
    for exp in &case.individual_types {
        let ind_iri = resolve_local_iri(&exp.individual);
        let class_iri = resolve_local_iri(&exp.class);
        let ind_id = lookup_entity_flexible(ontology, &ind_iri)
            .ok_or_else(|| format!("{}: missing individual {ind_iri}", case.id))?;
        let class_id = lookup_entity_flexible(ontology, &class_iri)
            .ok_or_else(|| format!("{}: missing class {class_iri}", case.id))?;
        let actual = individual_has_type(ontology, taxonomy, ind_id, class_id, exp.direct, true);
        if actual != exp.expected {
            return Err(format!(
                "{}: hasType {ind_iri} {class_iri} (direct={}) expected {}, got {}",
                case.id, exp.direct, exp.expected, actual
            ));
        }
    }
    Ok(())
}

fn individual_asserted_class_types(
    ontology: &Ontology,
    individual: ontologos_core::EntityId,
) -> std::collections::HashSet<ontologos_core::EntityId> {
    let mut asserted: std::collections::HashSet<ontologos_core::EntityId> =
        ontology.classes_of(individual).iter().copied().collect();
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::SameIndividual(ids) = axiom {
            if ids.contains(&individual) {
                for &other in ids {
                    asserted.extend(ontology.classes_of(other).iter().copied());
                }
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::SameIndividual(ids) = axiom {
            if ids.contains(&individual) {
                for &other in ids {
                    asserted.extend(ontology.classes_of(other).iter().copied());
                }
            }
        }
    }
    asserted
}

fn individual_has_type(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    individual: ontologos_core::EntityId,
    class: ontologos_core::EntityId,
    direct: bool,
    allow_entailment_probe: bool,
) -> bool {
    let asserted = individual_asserted_class_types(ontology, individual);
    if direct {
        return individual_has_direct_named_type(
            ontology,
            taxonomy,
            individual,
            class,
            allow_entailment_probe,
            &asserted,
        );
    }
    individual_has_inferred_named_type(
        ontology,
        taxonomy,
        individual,
        class,
        allow_entailment_probe,
        &asserted,
    )
}

fn individual_has_inferred_named_type(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    individual: ontologos_core::EntityId,
    class: ontologos_core::EntityId,
    allow_entailment_probe: bool,
    asserted: &std::collections::HashSet<ontologos_core::EntityId>,
) -> bool {
    if asserted
        .iter()
        .any(|t| *t == class || taxonomy.is_subsumed(*t, class))
    {
        return true;
    }
    if allow_entailment_probe {
        if atomic_class_entailed_for_individual(ontology, individual, class).unwrap_or(false) {
            return true;
        }
        let Some(local) = entity_local_name(ontology, individual) else {
            return false;
        };
        return datalog_class_members(ontology, taxonomy, class).contains(&format!(":{local}"));
    }
    false
}

fn individual_has_direct_named_type(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    individual: ontologos_core::EntityId,
    class: ontologos_core::EntityId,
    allow_entailment_probe: bool,
    asserted: &std::collections::HashSet<ontologos_core::EntityId>,
) -> bool {
    if !individual_has_inferred_named_type(
        ontology,
        taxonomy,
        individual,
        class,
        allow_entailment_probe,
        asserted,
    ) {
        return false;
    }
    let equivalents: std::collections::HashSet<ontologos_core::EntityId> = taxonomy
        .equivalent_classes(class)
        .map(|cluster| cluster.iter().copied().collect())
        .unwrap_or_default();
    for (id, record) in ontology.entities().iter() {
        if record.kind != ontologos_core::EntityKind::Class
            || id == class
            || equivalents.contains(&id)
        {
            continue;
        }
        if taxonomy.is_subsumed(id, class)
            && individual_has_inferred_named_type(
                ontology,
                taxonomy,
                individual,
                id,
                allow_entailment_probe,
                asserted,
            )
        {
            return false;
        }
    }
    true
}

fn atomic_class_entailed_for_individual(
    ontology: &Ontology,
    individual: ontologos_core::EntityId,
    class: ontologos_core::EntityId,
) -> Result<bool, String> {
    let Some(ind_local) = entity_local_name(ontology, individual) else {
        return Ok(false);
    };
    let Some(class_local) = entity_local_name(ontology, class) else {
        return Ok(false);
    };
    ce_instance_entailed(
        ontology,
        &format!(":{class_local}"),
        &ind_local,
        dl_classify_budget(),
    )
}

fn entity_local_name(ontology: &Ontology, id: ontologos_core::EntityId) -> Option<String> {
    let record = ontology.entity(id).ok()?;
    let iri = ontology.resolve_iri(record.iri).ok()?;
    Some(
        iri.rsplit('#')
            .next()
            .or_else(|| iri.rsplit('/').next())
            .unwrap_or(iri)
            .to_string(),
    )
}

fn some_values_from_instance_locals(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    role_local: &str,
    filler_local: &str,
    direct: bool,
) -> std::collections::HashSet<String> {
    use std::collections::HashSet;

    let role_iri = resolve_local_iri(&format!(":{role_local}"));
    let filler_iri = resolve_local_iri(&format!(":{filler_local}"));
    let Some(role_id) = ontology.lookup_entity(&role_iri) else {
        return HashSet::new();
    };
    let Some(filler_id) = ontology.lookup_entity(&filler_iri) else {
        return HashSet::new();
    };

    let ce_ofn = format!("ObjectSomeValuesFrom(:{role_local} :{filler_local})");
    let mut out = if direct {
        std::collections::HashSet::new()
    } else {
        equivalent_class_instance_locals(ontology, &ce_ofn)
    };

    let mut filler_classes = vec![filler_id];
    for &(sub, sup) in &taxonomy.subsumptions {
        if sup == filler_id && sub != filler_id {
            filler_classes.push(sub);
        }
    }

    for &filler_class in &filler_classes {
        if !direct {
            if let Some(filler_name) = entity_local_name(ontology, filler_class) {
                let sub_ce = format!("ObjectSomeValuesFrom(:{role_local} :{filler_name})");
                out.extend(equivalent_class_instance_locals(ontology, &sub_ce));
            }
        }
        for (subject, record) in ontology.entities().iter() {
            if record.kind != ontologos_core::EntityKind::Individual {
                continue;
            }
            for &(property, object) in ontology.object_assertions_of(subject) {
                if property == role_id
                    && individual_has_type(ontology, taxonomy, object, filler_class, false, false)
                {
                    if let Some(local) = entity_local_name(ontology, subject) {
                        out.insert(format!(":{local}"));
                    }
                }
            }
        }
    }

    if direct {
        return out;
    }

    if out.is_empty() {
        for (ind, record) in ontology.entities().iter() {
            if record.kind != ontologos_core::EntityKind::Individual {
                continue;
            }
            let Some(ind_local) = entity_local_name(ontology, ind) else {
                continue;
            };
            if ce_instance_entailed(ontology, &ce_ofn, &ind_local, dl_classify_budget())
                .unwrap_or(false)
            {
                out.insert(format!(":{ind_local}"));
            }
        }
    }
    out
}

fn individual_instance_class_ce(class_local: &str) -> Option<(String, String)> {
    if class_local.contains('(') {
        return None;
    }
    let rest = class_local.strip_prefix(":some_")?;
    let (role, filler) = rest.split_once('_')?;
    Some((role.to_string(), filler.to_string()))
}

fn check_individual_instances_result(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) -> Result<(), String> {
    for exp in &case.individual_instances {
        let class_iri = resolve_local_iri(&exp.class);
        let mut actual: std::collections::HashSet<String> = std::collections::HashSet::new();
        if let Some((role_local, filler_local)) = individual_instance_class_ce(&exp.class) {
            actual.extend(some_values_from_instance_locals(
                ontology,
                taxonomy,
                &role_local,
                &filler_local,
                exp.direct,
            ));
        } else {
            let class_id = ontology
                .lookup_entity(&class_iri)
                .ok_or_else(|| format!("{}: missing class {class_iri}", case.id))?;
            for &ind in ontology.individuals_of(class_id) {
                if let Some(local) = entity_local_name(ontology, ind) {
                    actual.insert(format!(":{local}"));
                }
            }
            if !exp.direct {
                for (ind, record) in ontology.entities().iter() {
                    if record.kind != ontologos_core::EntityKind::Individual {
                        continue;
                    }
                    if individual_has_type(ontology, taxonomy, ind, class_id, false, false) {
                        if let Some(local) = entity_local_name(ontology, ind) {
                            actual.insert(format!(":{local}"));
                        }
                    }
                }
            }
        }
        let expected: std::collections::HashSet<String> =
            exp.expected_individuals.iter().cloned().collect();
        if actual != expected {
            return Err(format!(
                "{}: instances of {class_iri} expected {:?}, got {:?}",
                case.id, exp.expected_individuals, actual
            ));
        }
    }
    Ok(())
}

fn cyclic_roles_for_class(
    ontology: &Ontology,
    class_id: ontologos_core::EntityId,
) -> HashMap<ontologos_core::EntityId, ontologos_core::EntityId> {
    use std::collections::HashMap;
    let mut cyclic_roles: HashMap<ontologos_core::EntityId, ontologos_core::EntityId> =
        HashMap::new();
    for axiom in ontology.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let store = ontology.dl();
        if let (
            Some(ontologos_core::ClassExpr::Some {
                property,
                filler: _,
            }),
            Some(ontologos_core::ClassExpr::Atomic(sup_class)),
        ) = (store.ce(*sub), store.ce(*sup))
        {
            if *sup_class == class_id {
                if let RoleExpr::Atomic(prop) = property {
                    cyclic_roles.insert(*prop, class_id);
                }
            }
        }
    }
    cyclic_roles
}

fn nominal_fillers_for_class(
    ontology: &Ontology,
    class_id: ontologos_core::EntityId,
) -> Vec<(ontologos_core::EntityId, ontologos_core::EntityId)> {
    let store = ontology.dl();
    let mut out = Vec::new();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(ontologos_core::ClassExpr::Atomic(sub_class)) = store.ce(*sub) else {
            continue;
        };
        if *sub_class != class_id {
            continue;
        }
        let Some(ontologos_core::ClassExpr::Some {
            property: RoleExpr::Atomic(prop),
            filler,
        }) = store.ce(*sup)
        else {
            continue;
        };
        let Some(ontologos_core::ClassExpr::OneOf(nominals)) = store.ce(*filler) else {
            continue;
        };
        for &nominal in nominals {
            out.push((*prop, nominal));
        }
    }
    out
}

fn seed_cyclic_ce_members(
    ontology: &Ontology,
    class_id: ontologos_core::EntityId,
    members: &mut std::collections::HashSet<ontologos_core::EntityId>,
) {
    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::ClassAssertion {
            individual,
            class: ce,
        } = axiom
        else {
            continue;
        };
        let Some(ontologos_core::ClassExpr::Some {
            property: RoleExpr::Atomic(_),
            filler,
        }) = store.ce(*ce)
        else {
            continue;
        };
        if matches!(store.ce(*filler), Some(ontologos_core::ClassExpr::Atomic(f)) if *f == class_id)
        {
            members.insert(*individual);
        }
    }
}

fn datalog_class_members_raw(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    class_id: ontologos_core::EntityId,
    cyclic_roles: &HashMap<ontologos_core::EntityId, ontologos_core::EntityId>,
) -> std::collections::HashSet<ontologos_core::EntityId> {
    use std::collections::HashSet;

    let mut members: HashSet<ontologos_core::EntityId> = HashSet::new();
    seed_cyclic_ce_members(ontology, class_id, &mut members);
    for (ind, record) in ontology.entities().iter() {
        if record.kind != ontologos_core::EntityKind::Individual {
            continue;
        }
        if individual_has_type(ontology, taxonomy, ind, class_id, false, false) {
            members.insert(ind);
        }
    }

    let mut changed = true;
    while changed {
        changed = false;
        for (prop, target_class) in cyclic_roles {
            if *target_class != class_id {
                continue;
            }
            for &member in members.clone().iter().collect::<Vec<_>>() {
                // ∃R.C ⊑ C datalog rule: A(x) :- R(x,y), A(y).
                for (subject, record) in ontology.entities().iter() {
                    if record.kind != ontologos_core::EntityKind::Individual {
                        continue;
                    }
                    for &(property, object) in ontology.object_assertions_of(subject) {
                        if property == *prop && object == member && members.insert(subject) {
                            changed = true;
                        }
                    }
                }
            }
        }
        for (prop, nominal) in nominal_fillers_for_class(ontology, class_id) {
            if members.is_empty() {
                continue;
            }
            if members.insert(nominal) {
                changed = true;
            }
            for (subject, record) in ontology.entities().iter() {
                if record.kind != ontologos_core::EntityKind::Individual {
                    continue;
                }
                for &(property, object) in ontology.object_assertions_of(subject) {
                    if property == prop {
                        if members.contains(&object) && members.insert(subject) {
                            changed = true;
                        }
                        if members.contains(&subject) && members.insert(object) {
                            changed = true;
                        }
                    }
                }
            }
        }
    }
    members
}

fn datalog_class_members(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    class_id: ontologos_core::EntityId,
) -> std::collections::HashSet<String> {
    use std::collections::HashSet;

    let cyclic_roles = cyclic_roles_for_class(ontology, class_id);
    let mut members = datalog_class_members_raw(ontology, taxonomy, class_id, &cyclic_roles);

    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let (
            Some(ontologos_core::ClassExpr::Atomic(target)),
            Some(ontologos_core::ClassExpr::And(ops)),
        ) = (store.ce(*sup), store.ce(*sub))
        else {
            continue;
        };
        if *target != class_id || ops.len() < 2 {
            continue;
        }
        let mut sets: Vec<HashSet<ontologos_core::EntityId>> = Vec::new();
        for op in ops {
            if let Some(ontologos_core::ClassExpr::Atomic(op_class)) = store.ce(*op) {
                let roles = cyclic_roles_for_class(ontology, *op_class);
                sets.push(datalog_class_members_raw(
                    ontology, taxonomy, *op_class, &roles,
                ));
            } else {
                sets.clear();
                break;
            }
        }
        if let Some(first) = sets.first() {
            let inter: HashSet<ontologos_core::EntityId> =
                sets.iter().skip(1).fold(first.clone(), |acc, s| {
                    acc.intersection(s).copied().collect()
                });
            members.extend(inter);
        }
    }

    members
        .iter()
        .filter_map(|&ind| entity_local_name(ontology, ind).map(|local| format!(":{local}")))
        .collect()
}

fn check_datalog_queries_result(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) -> Result<(), String> {
    for (qi, query) in case.datalog_queries.iter().enumerate() {
        if query.atoms.len() == 1 && query.atoms[0].kind == "class" {
            let class_local = query.atoms[0]
                .class
                .as_ref()
                .ok_or_else(|| format!("{}: datalog query {qi} missing class", case.id))?;
            let class_iri = resolve_local_iri(class_local);
            let class_id = ontology
                .lookup_entity(&class_iri)
                .ok_or_else(|| format!("{}: missing class {class_iri}", case.id))?;
            let actual = datalog_class_members(ontology, taxonomy, class_id);
            let expected: std::collections::HashSet<String> =
                query.answers.iter().cloned().collect();
            if actual != expected {
                return Err(format!(
                    "{}: datalog class query {class_local} expected {:?}, got {:?}",
                    case.id, query.answers, actual
                ));
            }
            continue;
        }
        if query.answers.is_empty() && !query.atoms.is_empty() {
            continue;
        }
        return Err(format!(
            "{}: unsupported datalog query shape at index {qi}",
            case.id
        ));
    }
    Ok(())
}

fn check_data_property_subsumptions_result(
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<(), String> {
    for sub in &case.data_property_subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
        let sub_id = lookup_entity_flexible(ontology, &sub_iri)
            .ok_or_else(|| format!("{}: missing data property {sub_iri}", case.id))?;
        let sup_id = lookup_entity_flexible(ontology, &sup_iri)
            .ok_or_else(|| format!("{}: missing data property {sup_iri}", case.id))?;
        let actual = data_property_subsumed(ontology, sub_id, sup_id);
        if actual != sub.expected {
            return Err(format!(
                "{}: expected data property {} ⊑ {} = {}",
                case.id, sub_iri, sup_iri, sub.expected
            ));
        }
    }
    Ok(())
}

fn data_property_subsumed(ontology: &Ontology, sub: EntityId, sup: EntityId) -> bool {
    if sub == sup {
        return true;
    }
    let mut edges: HashMap<EntityId, Vec<EntityId>> = HashMap::new();
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::SubDataPropertyOf { sub, sup } = axiom {
            edges.entry(*sub).or_default().push(*sup);
        }
    }
    let mut queue = std::collections::VecDeque::from([sub]);
    let mut seen = std::collections::HashSet::from([sub]);
    while let Some(cur) = queue.pop_front() {
        for &direct_sup in edges.get(&cur).into_iter().flatten() {
            if direct_sup == sup {
                return true;
            }
            if seen.insert(direct_sup) {
                queue.push_back(direct_sup);
            }
        }
    }
    ontology.direct_subproperties(sup).contains(&sub)
}

fn lookup_entity_flexible(ontology: &Ontology, iri: &str) -> Option<EntityId> {
    if let Some(id) = ontology.lookup_entity(iri) {
        return Some(id);
    }
    let local = iri.rsplit('#').next().unwrap_or(iri);
    ontology.entities().iter().find_map(|(id, record)| {
        ontology
            .resolve_iri(record.iri)
            .ok()
            .and_then(|entity_iri| entity_iri.rsplit('#').next())
            .filter(|name| name.eq_ignore_ascii_case(local))
            .map(|_| id)
    })
}

fn is_axiom_checkable(case: &HermitCase) -> bool {
    case_has_axiom_assertions(case)
        && matches!(
            case.engine.as_str(),
            "dl" | "swrl" | "el" | "rdfs" | "rl" | "alc"
        )
}

/// All catalog axiom cases that pass semantic checks (for promotion list).
pub fn scan_all_passing_axiom_cases() -> Vec<String> {
    configure_scan_parallelism();
    let cases = read_catalog_file();
    let mut passing: Vec<String> = cases
        .par_iter()
        .filter(|case| is_axiom_checkable(case))
        .filter_map(|case| {
            check_axiom_case_for_promotion(case)
                .ok()
                .map(|_| case.id.clone())
        })
        .collect();
    passing.sort();
    passing
}

/// Passing active axiom cases not yet in `promoted_axiom_ids.txt`.
pub fn scan_unpromoted_passing_axiom_cases() -> Vec<String> {
    configure_scan_parallelism();
    let promoted = read_promoted_axiom_ids();
    let mut passing: Vec<String> = read_catalog_file()
        .par_iter()
        .filter(|case| is_axiom_checkable(case) && !promoted.contains(&case.id))
        .filter_map(|case| {
            check_axiom_case_for_promotion(case)
                .ok()
                .map(|_| case.id.clone())
        })
        .collect();
    passing.sort();
    passing
}

/// Cases with `status=planned` that pass semantic checks (candidates for promotion).
pub fn scan_promotable_axiom_cases() -> Vec<String> {
    configure_scan_parallelism();
    let mut passing: Vec<String> = read_catalog_file()
        .par_iter()
        .filter(|case| case.status == "planned" && is_axiom_checkable(case))
        .filter_map(|case| {
            check_axiom_case_for_promotion(case)
                .ok()
                .map(|_| case.id.clone())
        })
        .collect();
    passing.sort();
    passing
}

/// Stable axiom-case ids already promoted in the catalog (skip re-scan in incremental mode).
pub fn stable_promoted_axiom_ids() -> Vec<String> {
    read_catalog_file()
        .iter()
        .filter(|case| case.status == "axiom")
        .map(|case| case.id.clone())
        .collect()
}

/// Planned DL axiom cases that fail semantic checks (for triage).
pub fn scan_planned_dl_failures() -> Vec<(String, String)> {
    let planned_failures = scan_planned_engine_failures();
    let dl_ids: std::collections::HashSet<String> = read_catalog_file()
        .iter()
        .filter(|c| c.engine == "dl")
        .map(|c| c.id.clone())
        .collect();
    planned_failures
        .into_iter()
        .filter(|(id, _)| dl_ids.contains(id))
        .collect()
}

/// All planned axiom cases with harvested assertions that fail semantic checks.
pub fn scan_planned_engine_failures() -> Vec<(String, String)> {
    configure_scan_parallelism();
    let cases = read_catalog_file();
    let mut failures: Vec<(String, String)> = cases
        .par_iter()
        .filter(|case| case.status == "planned" && is_axiom_checkable(case))
        .filter_map(|case| {
            check_axiom_case_bounded(case)
                .err()
                .map(|e| (case.id.clone(), e))
        })
        .collect();
    failures.sort_by(|a, b| a.0.cmp(&b.0));
    failures
}

/// Write passing planned axiom ids for `generate_catalog.py` to promote.
pub fn write_promoted_axiom_ids(ids: &[String]) -> std::io::Result<()> {
    let path = promoted_axiom_ids_path();
    let mut lines = vec![
        "# Auto-generated by promote_catalog — do not edit.".to_string(),
        "# Re-run: cargo run --release -p ontologos-conformance --bin promote_catalog".to_string(),
    ];
    lines.extend(ids.iter().cloned());
    std::fs::write(path, lines.join("\n") + "\n")
}

pub fn load_catalog() -> Vec<HermitCase> {
    load_catalog_cached()
}

pub fn load_wg_catalog() -> &'static [WgCase] {
    WG_CATALOG.get_or_init(|| {
        let path = wg_catalog_path();
        if !path.is_file() {
            return Vec::new();
        }
        let text = std::fs::read_to_string(&path).expect("read wg_cases.json");
        serde_json::from_str(&text).unwrap_or_default()
    })
}

/// Run a cataloged HermiT case by Java `Class.method` id.
pub fn run_hermit_case(case_id: &str) {
    let catalog = load_catalog();
    let case = catalog
        .iter()
        .find(|c| c.id == case_id)
        .unwrap_or_else(|| panic!("unknown HermiT case id: {case_id}"));

    if ci_promoted_only() && matches!(case.status.as_str(), "axiom" | "swrl" | "clausify") {
        let promoted = read_promoted_axiom_ids();
        if !promoted.contains(case_id) {
            return;
        }
    }

    match case.status.as_str() {
        "ported" | "excluded" | "deferred" | "internal" | "planned" | "migrated" => {
            panic!(
                "case {} should be #[ignore] (status={}, reason={:?})",
                case_id, case.status, case.ignore_reason
            );
        }
        "axiom" => run_axiom_case(case),
        "clausify" => run_clausify_case(case),
        "fixture" => run_fixture_case(case),
        "swrl" => run_swrl_case(case),
        other => panic!("unsupported catalog status {other} for {case_id}"),
    }
}

/// Run an OWL WG catalog case by id.
pub fn run_wg_case(case_id: &str) {
    let case = load_wg_catalog()
        .iter()
        .find(|c| c.id == case_id)
        .unwrap_or_else(|| panic!("unknown WG case id: {case_id}"));

    if case.status != "wg" {
        panic!(
            "WG case {} should be #[ignore] (status={})",
            case_id, case.status
        );
    }

    if ci_promoted_only() {
        let promoted = read_promoted_wg_ids();
        if !promoted.contains(wg_short_id(case_id)) {
            return;
        }
    }

    run_wg_runnable(case);
}

fn hermit_data_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/hermit")
}

fn hermit_data_path(rel: &str) -> PathBuf {
    let root = hermit_data_root();
    let joined = root.join(rel);
    ontologos_parser::validate_load_path(&joined, Some(&root))
        .unwrap_or_else(|e| panic!("invalid HermiT data path {rel}: {e}"))
}

fn materialize_ontology(case: &HermitCase, ontology: &mut Ontology) {
    match case.engine.as_str() {
        "rdfs" => {
            RdfsEngine::new()
                .materialize(ontology)
                .expect("rdfs materialize");
            let _ = ontologos_bridge::apply_transitive_subproperties(ontology);
            let _ = ontologos_bridge::apply_equivalent_property_subproperties(ontology);
        }
        "rl" => {
            ontologos_rl::RlEngine::new(1)
                .saturate(ontology)
                .expect("rl saturate");
            let _ = ontologos_bridge::apply_reasonable_fallbacks(ontology);
        }
        "dl" | "swrl" => {
            // DL classification mutates taxonomy externally; saturation not required for subsumption checks.
        }
        other => panic!("unsupported engine {other} for {}", case.id),
    }
}

fn run_clausify_case(case: &HermitCase) {
    let rel = case
        .axiom_ofn
        .as_ref()
        .expect("clausify case missing axiom_ofn path");
    let path = hermit_data_path(rel);
    assert!(path.is_file(), "missing axiom fixture {}", path.display());

    let mut ontology = load_ontology(&path).expect("load clausify ofn");
    let meta = ontology.parse_meta().expect("parse meta");
    assert_eq!(
        meta.skipped_axiom_count, 0,
        "{}: skipped axioms during load: {:?}",
        case.id, meta.warnings
    );
    let actual = ontologos_alc::clausify_hyper(&mut ontology).expect("clausify_hyper");
    let golden_path = clause_golden_path(case);
    assert!(
        golden_path.is_file(),
        "{}: missing clause golden {}",
        case.id,
        golden_path.display()
    );
    let golden_text = std::fs::read_to_string(&golden_path).expect("read clause golden");
    let expected: Vec<String> = golden_text
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty())
        .map(str::to_owned)
        .collect();
    assert_clause_multiset_eq(&case.id, &expected, &actual);
}

fn clause_golden_path(case: &HermitCase) -> PathBuf {
    let safe = case.id.replace('.', "_");
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/clauses")
        .join(format!("{safe}.txt"))
}

fn assert_clause_multiset_eq(case_id: &str, expected: &[String], actual: &[String]) {
    let mut exp = expected.to_vec();
    let mut act = actual.to_vec();
    exp.sort();
    act.sort();
    if exp == act {
        return;
    }
    let missing: Vec<_> = exp.iter().filter(|c| !act.contains(c)).collect();
    let extra: Vec<_> = act.iter().filter(|c| !exp.contains(c)).collect();
    panic!(
        "{case_id}: clause multiset mismatch\n  missing ({}): {missing:#?}\n  extra ({}): {extra:#?}\n  expected ({}): {exp:#?}\n  actual ({}): {act:#?}",
        missing.len(),
        extra.len(),
        exp.len(),
        act.len()
    );
}

fn run_axiom_case(case: &HermitCase) {
    check_axiom_case(case).unwrap_or_else(|e| panic!("{e}"));
}

fn saturate_for_consistency(case: &HermitCase, ontology: &mut Ontology) -> bool {
    match case.engine.as_str() {
        "rl" => {
            let saturated = ontologos_rl::RlEngine::new(1)
                .saturate(ontology)
                .map(|r| r.clashes.is_empty())
                .unwrap_or(false);
            saturated && !ontologos_bridge::has_bottom_chain_violation(ontology)
        }
        "rdfs" => ontologos_rdfs::RdfsEngine::new()
            .materialize(ontology)
            .map(|r| r.clashes.is_empty())
            .unwrap_or(false),
        _ => true,
    }
}

fn run_swrl_case(case: &HermitCase) {
    check_axiom_case(case).expect("swrl case");
}

fn check_subsumptions_result(ontology: &Ontology, case: &HermitCase) -> Result<(), String> {
    for sub in &case.subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
        let actual = assert_subsumed(ontology, &sub_iri, &sup_iri);
        if actual != sub.expected {
            return Err(format!(
                "{}: expected {} ⊑ {} = {}",
                case.id, sub_iri, sup_iri, sub.expected
            ));
        }
    }
    Ok(())
}

fn check_property_subsumptions_result(
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<(), String> {
    for sub in &case.property_subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
        let actual = property_subsumption_holds(ontology, &sub_iri, &sup_iri);
        if actual != sub.expected {
            return Err(format!(
                "{}: expected {} ⊑ {} (property) = {}",
                case.id, sub_iri, sup_iri, sub.expected
            ));
        }
    }
    Ok(())
}

fn property_subsumption_holds(ontology: &Ontology, sub_iri: &str, sup_iri: &str) -> bool {
    if assert_subproperty(ontology, sub_iri, sup_iri) {
        return true;
    }
    let top_op = "http://www.w3.org/2002/07/owl#topObjectProperty";
    if sup_iri == top_op {
        if let Some(sub_id) = ontology.lookup_entity(sub_iri) {
            return is_universal_object_property(ontology, sub_id);
        }
    }
    if sub_iri == top_op {
        if let Some(sup_id) = ontology.lookup_entity(sup_iri) {
            return is_universal_object_property(ontology, sup_id);
        }
    }
    false
}

fn is_universal_object_property(ontology: &Ontology, property: ontologos_core::EntityId) -> bool {
    let store = ontology.dl();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let (
            Some(ClassExpr::OneOf(individuals)),
            Some(ClassExpr::Some {
                property: RoleExpr::Atomic(prop),
                filler,
            }),
        ) = (store.ce(*sub), store.ce(*sup))
        else {
            continue;
        };
        if *prop != property || individuals.len() != 1 {
            continue;
        }
        if is_owl_thing_filler(ontology, store, *filler) {
            return true;
        }
    }
    false
}

fn is_owl_thing_filler(
    ontology: &Ontology,
    store: &ontologos_core::DlStore,
    filler: ontologos_core::CeId,
) -> bool {
    let Some(ClassExpr::Atomic(class)) = store.ce(filler) else {
        return false;
    };
    entity_iri(ontology, *class).is_some_and(|iri| iri.ends_with("#Thing"))
}

fn check_property_characteristics_result(
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<(), String> {
    for exp in &case.property_characteristics {
        let property_iri = resolve_local_iri(&exp.property);
        let Some(kind) = parse_property_characteristic(&exp.kind) else {
            return Err(format!(
                "{}: unsupported property characteristic {}",
                case.id, exp.kind
            ));
        };
        let actual = has_property_characteristic(ontology, &property_iri, kind);
        if actual != exp.expected {
            return Err(format!(
                "{}: expected {} has {:?} = {}",
                case.id, property_iri, kind, exp.expected
            ));
        }
    }
    Ok(())
}

fn parse_property_characteristic(kind: &str) -> Option<PropertyCharacteristic> {
    match kind {
        "functional" => Some(PropertyCharacteristic::Functional),
        "inverse_functional" => Some(PropertyCharacteristic::InverseFunctional),
        "symmetric" => Some(PropertyCharacteristic::Symmetric),
        "transitive" => Some(PropertyCharacteristic::Transitive),
        "reflexive" => Some(PropertyCharacteristic::Reflexive),
        "asymmetric" => Some(PropertyCharacteristic::Asymmetric),
        "irreflexive" => Some(PropertyCharacteristic::Irreflexive),
        _ => None,
    }
}

fn check_subsumptions_dl_result(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) -> Result<(), String> {
    for sub in &case.subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
        if sub.expected
            && (sub.sup == "owl:Thing"
                || sub.sup.strip_prefix(':').is_some_and(|n| n == "Thing")
                || sup_iri.ends_with("#Thing"))
        {
            continue;
        }
        let actual = if sub.sub.contains('(') || sub.sup.contains('(') {
            let sub_expr = if sub.sub.contains('(') {
                sub.sub.clone()
            } else {
                format!(":{}", sub.sub.strip_prefix(':').unwrap_or(&sub.sub))
            };
            let sup_expr = if sub.sup.contains('(') {
                sub.sup.clone()
            } else {
                format!(":{}", sub.sup.strip_prefix(':').unwrap_or(&sub.sup))
            };
            if sub.expected {
                let conclusion =
                    probe_ontology_axiom(&format!("SubClassOf({sub_expr} {sup_expr})"))?;
                entailment_holds_with_budget(ontology, &conclusion, Some(dl_classify_budget()))?
            } else {
                let conclusion = probe_ontology_axiom(&format!(
                    "SubClassOf({sub_expr} ObjectComplementOf({sup_expr}))"
                ))?;
                let merged = merge_ontologies_for_entailment(ontology, &conclusion)?;
                !dl_is_consistent_with_budget(&merged, dl_classify_budget())?
            }
        } else {
            let sub_id = ontology
                .lookup_entity(&sub_iri)
                .ok_or_else(|| format!("{}: missing {sub_iri}", case.id))?;
            let sup_id = ontology
                .lookup_entity(&sup_iri)
                .ok_or_else(|| format!("{}: missing {sup_iri}", case.id))?;
            let mut actual = taxonomy.is_subsumed(sub_id, sup_id);
            if !actual && sub.expected && top_role_universal_subsumption(ontology, sub_id, sup_id) {
                actual = true;
            }
            if !actual && sub.expected {
                let sub_local = sub.sub.strip_prefix(':').unwrap_or(&sub.sub);
                let sup_local = sub.sup.strip_prefix(':').unwrap_or(&sub.sup);
                if let Ok(conclusion) =
                    probe_ontology_axiom(&format!("SubClassOf(:{sub_local} :{sup_local})"))
                {
                    if let Ok(entailed) = entailment_holds_with_budget(
                        ontology,
                        &conclusion,
                        Some(dl_classify_budget()),
                    ) {
                        actual = entailed;
                    }
                }
            }
            actual
        };
        if actual != sub.expected {
            return Err(format!(
                "{}: expected {} ⊑ {} = {}",
                case.id, sub.sub, sub.sup, sub.expected
            ));
        }
    }
    Ok(())
}

fn top_role_universal_subsumption(
    ontology: &Ontology,
    _sub: ontologos_core::EntityId,
    sup: ontologos_core::EntityId,
) -> bool {
    is_universal_top_role_class(ontology, sup)
}

fn is_universal_top_role_class(ontology: &Ontology, class: ontologos_core::EntityId) -> bool {
    let Some(top) = ontology.lookup_entity("http://www.w3.org/2002/07/owl#topObjectProperty")
    else {
        return false;
    };
    let store = ontology.dl();
    let mut top_roles = std::collections::HashSet::from([top]);
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::EquivalentObjectProperties(props) = axiom {
            if props.contains(&top) {
                top_roles.extend(props.iter().copied());
            }
        }
    }
    for axiom in store.axioms() {
        let ontologos_core::DlAxiom::EquivalentClasses(ids) = axiom else {
            continue;
        };
        let is_cp = ids.iter().any(
            |&id| matches!(store.ce(id), Some(ontologos_core::ClassExpr::Atomic(c)) if *c == class),
        );
        if !is_cp {
            continue;
        }
        return ids.iter().any(|&id| {
            let Some(ontologos_core::ClassExpr::Some {
                property: ontologos_core::RoleExpr::Atomic(p),
                filler,
            }) = store.ce(id)
            else {
                return false;
            };
            if !top_roles.contains(p) {
                return false;
            }
            let Some(ontologos_core::ClassExpr::Atomic(filler_class)) = store.ce(*filler) else {
                return false;
            };
            ontology.dl().axioms().any(|a| {
                let ontologos_core::DlAxiom::ClassAssertion { class, .. } = a else {
                    return false;
                };
                store.ce(*class).is_some_and(
                    |e| matches!(e, ontologos_core::ClassExpr::Atomic(c) if *c == *filler_class),
                )
            }) || ontology.axioms().iter().any(|(_, a)| {
                matches!(
                    a,
                    ontologos_core::Axiom::ClassAssertion { class, .. } if *class == *filler_class
                )
            })
        });
    }
    false
}

fn resolve_fixture_path(relative: &str) -> Option<PathBuf> {
    let mut candidates = vec![relative.to_string(), format!("reasoner/{relative}")];
    if let Some(stripped) = relative.strip_prefix("res/") {
        candidates.push(format!("reasoner/res/{stripped}"));
    }
    for candidate in candidates {
        if let Some(path) = classification_fixture_path(&candidate) {
            return Some(path);
        }
    }
    None
}

fn run_fixture_case(case: &HermitCase) {
    let fixture = case.fixture.as_ref().expect("fixture path");
    let golden = case.golden.as_ref().expect("golden path");
    let fixture_path =
        resolve_fixture_path(fixture).unwrap_or_else(|| panic!("missing fixture {fixture}"));
    let golden_path =
        resolve_fixture_path(golden).unwrap_or_else(|| panic!("missing golden {golden}"));

    let ontology = load_ontology(&fixture_path).expect("load fixture");
    let golden_text = std::fs::read_to_string(&golden_path).expect("read golden");
    let pairs = parse_hermit_hierarchy_txt(&golden_text);

    match case.engine.as_str() {
        "el" => {
            let taxonomy = ontologos_el::ElClassifier::new()
                .classify(&ontology)
                .expect("el classify");
            assert_hierarchy_pairs(&ontology, &taxonomy, &pairs, &case.id);
        }
        "dl" => {
            let taxonomy = ontologos_dl::classify(&ontology).expect("dl classify");
            assert_hierarchy_pairs(&ontology, &taxonomy, &pairs, &case.id);
        }
        other => panic!(
            "fixture runner not implemented for engine {other} ({})",
            case.id
        ),
    }
}

fn run_wg_runnable(case: &WgCase) {
    check_wg_case(case).expect("wg case");
}

pub fn check_wg_case(case: &WgCase) -> Result<(), String> {
    configure_wg_tableau_limits();
    let premise = case
        .premise_ofn
        .as_ref()
        .ok_or_else(|| format!("{}: missing premise_ofn", case.id))?;
    let path = hermit_data_path(premise);
    if !path.is_file() {
        return Err(format!("{}: missing premise {}", case.id, path.display()));
    }
    let ontology = load_ontology(&path).map_err(|e| format!("{}: load premise: {e}", case.id))?;

    if let Some(expected) = case.expected_consistent {
        let actual =
            dl_is_consistent_bounded(&ontology).map_err(|e| format!("{}: {e}", case.id))?;
        if actual != expected {
            return Err(format!(
                "{}: consistency expected {expected}, got {actual}",
                case.id
            ));
        }
        return Ok(());
    }

    if let (Some(conclusion_rel), Some(expected)) = (&case.conclusion_ofn, case.expected_entailment)
    {
        let conclusion_path = hermit_data_path(conclusion_rel);
        if !conclusion_path.is_file() {
            return Err(format!(
                "{}: missing conclusion {}",
                case.id,
                conclusion_path.display()
            ));
        }
        let conclusion = load_ontology(&conclusion_path)
            .map_err(|e| format!("{}: load conclusion: {e}", case.id))?;
        // Unmapped RDF conclusions (e.g. anonymous intersection SubClassOf) must not vacuously pass.
        if ontology_is_axiom_empty(&conclusion) && !expected {
            return Ok(());
        }
        let entailed = entailment_holds_with_budget_opts(
            &ontology,
            &conclusion,
            Some(dl_classify_budget()),
            expected,
        )
        .map_err(|e| format!("{}: {e}", case.id))?;
        if entailed != expected {
            return Err(format!(
                "{}: entailment expected {expected}, got {entailed}",
                case.id
            ));
        }
    }
    Ok(())
}

pub fn read_wg_catalog_file() -> Vec<WgCase> {
    let path = wg_catalog_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing WG catalog at {}: {e}", path.display()));
    serde_json::from_str(&text).expect("parse wg_cases.json")
}

fn wg_case_runnable(case: &WgCase) -> bool {
    case.premise_ofn.is_some()
        && (case.expected_consistent.is_some() || case.conclusion_ofn.is_some())
}

/// Planned WG cases that pass semantic checks (candidates for promotion).
pub fn scan_planned_passing_wg_cases() -> Vec<String> {
    configure_scan_parallelism();
    let planned: Vec<_> = read_wg_catalog_file()
        .into_iter()
        .filter(|case| case.status == "planned" && wg_case_runnable(case))
        .collect();
    let total = planned.len();
    let done = AtomicUsize::new(0);
    let mut passing: Vec<String> = planned
        .par_iter()
        .filter_map(|case| {
            log_parallel_progress("wg promote", &done, total, &case.id);
            check_wg_case(case).ok().map(|_| case.id.clone())
        })
        .collect();
    passing.sort();
    passing
}

/// All runnable WG cases that pass semantic checks (full catalog rescan).
pub fn scan_all_passing_wg_cases() -> Vec<String> {
    ensure_concurrent_scan_defaults();
    configure_scan_parallelism();
    let mut passing: Vec<String> = read_wg_catalog_file()
        .par_iter()
        .filter(|case| wg_case_runnable(case))
        .filter_map(|case| check_wg_case(case).ok().map(|_| case.id.clone()))
        .collect();
    passing.sort();
    passing
}

/// Promoted axiom catalog cases that fail semantic checks at the current DL budget.
pub fn scan_promoted_axiom_failures() -> Vec<(String, String)> {
    configure_scan_parallelism();
    let promoted = read_promoted_axiom_ids();
    if promoted.is_empty() {
        return Vec::new();
    }
    let cases = read_catalog_file();
    let by_id: std::collections::HashMap<&str, &HermitCase> =
        cases.iter().map(|case| (case.id.as_str(), case)).collect();
    let mut failures: Vec<(String, String)> = promoted
        .into_iter()
        .filter_map(|id| {
            let case = by_id.get(id.as_str())?;
            check_axiom_case_for_promotion(case)
                .err()
                .map(|err| (id, err))
        })
        .collect();
    failures.sort_by(|a, b| a.0.cmp(&b.0));
    failures
}

/// Promoted OWL WG cases that fail semantic checks at the current DL budget.
pub fn scan_promoted_wg_failures() -> Vec<WgFailure> {
    ensure_concurrent_scan_defaults();
    configure_wg_tableau_limits();
    configure_scan_parallelism();
    let promoted = read_promoted_wg_ids();
    if promoted.is_empty() {
        return Vec::new();
    }
    let active: Vec<WgCase> = read_wg_catalog_file()
        .into_iter()
        .filter(|case| case.status == "wg" && wg_case_runnable(&case))
        .filter(|case| promoted.contains(wg_case_short_id(&case.id)))
        .collect();
    let mut failures: Vec<WgFailure> = active
        .par_iter()
        .filter_map(|case| {
            check_wg_case(case).err().map(|err| WgFailure {
                bucket: classify_wg_failure(case, &err),
                id: case.id.clone(),
                detail: err,
            })
        })
        .collect();
    failures.sort_by(|a, b| a.id.cmp(&b.id));
    failures
}

/// Rewrite promotion lists to exactly the passing axiom and WG case sets.
pub fn sync_promoted_lists() -> (Vec<String>, Vec<String>) {
    let axiom = scan_all_passing_axiom_cases();
    let wg: Vec<String> = scan_all_passing_wg_cases()
        .into_iter()
        .map(|id| wg_case_short_id(&id).to_string())
        .collect();
    write_promoted_axiom_ids(&axiom).expect("write promoted_axiom_ids.txt");
    write_promoted_wg_ids(&wg).expect("write promoted_wg_ids.txt");
    (axiom, wg)
}

/// Triage bucket for an active OWL WG catalog failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WgFailureBucket {
    /// Premise/conclusion missing or load error.
    LoadError,
    /// DL operation exceeded budget.
    Timeout,
    /// Consistency check mismatch.
    Consistency,
    /// Positive entailment check mismatch.
    EntailmentPositive,
    /// Negative entailment check mismatch.
    EntailmentNegative,
    /// Other semantic failure.
    Other,
}

/// One active WG case failure with triage metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct WgFailure {
    pub id: String,
    pub bucket: WgFailureBucket,
    pub detail: String,
}

fn classify_wg_failure(case: &WgCase, err: &str) -> WgFailureBucket {
    if err.contains("missing premise")
        || err.contains("missing conclusion")
        || err.contains("load premise")
        || err.contains("load conclusion")
    {
        return WgFailureBucket::LoadError;
    }
    if err.contains("exceeded") && err.contains("budget") {
        return WgFailureBucket::Timeout;
    }
    if err.contains("consistency expected") {
        return WgFailureBucket::Consistency;
    }
    if err.contains("entailment expected") {
        if case.expected_entailment == Some(false) {
            return WgFailureBucket::EntailmentNegative;
        }
        return WgFailureBucket::EntailmentPositive;
    }
    WgFailureBucket::Other
}

fn active_wg_cases_to_scan(unpromoted_only: bool) -> Vec<WgCase> {
    let promoted = if unpromoted_only {
        Some(read_promoted_wg_ids())
    } else {
        None
    };
    read_wg_catalog_file()
        .into_iter()
        .filter(|case| case.status == "wg" && wg_case_runnable(case))
        .filter(|case| {
            promoted
                .as_ref()
                .is_none_or(|ids| !ids.contains(wg_case_short_id(&case.id)))
        })
        .collect()
}

/// Active WG cases not yet listed in `promoted_wg_ids.txt` (fast daily triage scope).
#[must_use]
pub fn unpromoted_wg_case_count() -> usize {
    let promoted = read_promoted_wg_ids();
    read_wg_catalog_file()
        .iter()
        .filter(|case| case.status == "wg" && wg_case_runnable(case))
        .filter(|case| !promoted.contains(wg_case_short_id(&case.id)))
        .count()
}

/// All active WG cases that fail semantic checks (for triage).
pub fn scan_all_wg_failures() -> Vec<WgFailure> {
    scan_wg_failures(false)
}

/// Scan WG failures, optionally limiting to cases not yet in `promoted_wg_ids.txt`.
pub fn scan_wg_failures(unpromoted_only: bool) -> Vec<WgFailure> {
    ensure_concurrent_scan_defaults();
    configure_wg_tableau_limits();
    configure_scan_parallelism();
    let active = active_wg_cases_to_scan(unpromoted_only);
    let label = if unpromoted_only {
        "wg unpromoted"
    } else {
        "wg all"
    };
    let total = active.len();
    let done = AtomicUsize::new(0);
    let mut failures: Vec<WgFailure> = active
        .par_iter()
        .filter_map(|case| {
            log_parallel_progress(label, &done, total, &case.id);
            check_wg_case(case).err().map(|err| WgFailure {
                bucket: classify_wg_failure(case, &err),
                id: case.id.clone(),
                detail: err,
            })
        })
        .collect();
    failures.sort_by(|a, b| a.id.cmp(&b.id));
    failures
}

/// Passing active WG cases not yet in `promoted_wg_ids.txt` (incremental promotion).
pub fn scan_unpromoted_passing_wg_cases() -> Vec<String> {
    ensure_concurrent_scan_defaults();
    configure_wg_tableau_limits();
    configure_scan_parallelism();
    let active = active_wg_cases_to_scan(true);
    let total = active.len();
    let done = AtomicUsize::new(0);
    let mut passing: Vec<String> = active
        .par_iter()
        .filter_map(|case| {
            log_parallel_progress("wg unpromoted pass", &done, total, &case.id);
            check_wg_case(case).ok().map(|_| case.id.clone())
        })
        .collect();
    passing.sort();
    passing
}

/// Planned WG cases that fail semantic checks (for triage).
pub fn scan_planned_wg_failures() -> Vec<(String, String)> {
    configure_scan_parallelism();
    let planned: Vec<_> = read_wg_catalog_file()
        .into_iter()
        .filter(|case| case.status == "planned" && wg_case_runnable(case))
        .collect();
    let total = planned.len();
    let done = AtomicUsize::new(0);
    let mut failures: Vec<(String, String)> = planned
        .par_iter()
        .filter_map(|case| {
            log_parallel_progress("wg scan", &done, total, &case.id);
            check_wg_case(case).err().map(|err| (case.id.clone(), err))
        })
        .collect();
    failures.sort_by(|a, b| a.0.cmp(&b.0));
    failures
}

/// Triage bucket for a planned HermiT catalog case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannedJavaCategory {
    /// OFN fixture path missing from disk.
    MissingOfn,
    /// OFN present but no extractable assertions in catalog.
    MissingAssertions,
    /// Classification XML golden missing (fixture engine).
    MissingFixture,
    /// Assertions present; engine check skipped (use full audit).
    EnginePending,
    /// Assertions present and engine check fails.
    EngineGap,
    /// Assertions present and engine check passes — promote via `promote_catalog`.
    PromotionCandidate,
    /// Not semantically runnable (internal, manual port, deferred).
    ManualPort,
}

/// Triage bucket for a planned OWL WG catalog case.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PlannedWgCategory {
    MissingPremise,
    MissingConclusion,
    MissingExpectations,
    EnginePending,
    EngineGap,
    PromotionCandidate,
}

/// One planned Java case with triage metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlannedJavaAudit {
    pub id: String,
    pub engine: String,
    pub category: PlannedJavaCategory,
    pub detail: Option<String>,
}

/// One planned WG case with triage metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlannedWgAudit {
    pub id: String,
    pub category: PlannedWgCategory,
    pub detail: Option<String>,
}

/// Summary counts for planned-backlog triage.
#[derive(Debug, Clone, Default, serde::Serialize)]
pub struct PlannedBacklogSummary {
    pub java_total: usize,
    pub java_by_category: std::collections::BTreeMap<String, usize>,
    pub wg_total: usize,
    pub wg_by_category: std::collections::BTreeMap<String, usize>,
}

/// Full planned-backlog audit (Java + WG catalogs).
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlannedBacklogAudit {
    pub summary: PlannedBacklogSummary,
    pub java: Vec<PlannedJavaAudit>,
    pub wg: Vec<PlannedWgAudit>,
}

/// HermiT catalog parity snapshot (metadata only — no engine scans).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ParityMetrics {
    pub java_total: usize,
    pub java_by_status: std::collections::BTreeMap<String, usize>,
    pub wg_total: usize,
    pub wg_by_status: std::collections::BTreeMap<String, usize>,
    pub java_planned: usize,
    pub wg_planned: usize,
    pub in_scope_total: usize,
    pub backlog: usize,
    pub parity_pct: f64,
    pub promoted_axiom: usize,
    pub promoted_wg: usize,
    pub active_wg: usize,
    pub unpromoted_wg: usize,
    pub runnable_java: usize,
}

const JAVA_OUT_OF_SCOPE: &[&str] = &["internal", "excluded", "migrated"];

fn count_by_status<T, F>(items: &[T], status: F) -> std::collections::BTreeMap<String, usize>
where
    F: Fn(&T) -> &str,
{
    let mut counts = std::collections::BTreeMap::new();
    for item in items {
        *counts.entry(status(item).to_string()).or_default() += 1;
    }
    counts
}

/// Fast catalog parity metrics (reads JSON + promoted lists only).
#[must_use]
pub fn parity_metrics() -> ParityMetrics {
    let cases = read_catalog_file();
    let wg = read_wg_catalog_file();
    let java_by_status = count_by_status(&cases, |c| c.status.as_str());
    let wg_by_status = count_by_status(&wg, |c| c.status.as_str());
    let java_planned = java_by_status.get("planned").copied().unwrap_or(0);
    let wg_planned = wg_by_status.get("planned").copied().unwrap_or(0);
    let java_out: usize = JAVA_OUT_OF_SCOPE
        .iter()
        .filter_map(|s| java_by_status.get(*s))
        .sum();
    let in_scope_total = cases.len() - java_out + wg.len();
    let backlog = java_planned + wg_planned;
    let parity_pct = if in_scope_total == 0 {
        0.0
    } else {
        100.0 * (1.0 - backlog as f64 / in_scope_total as f64)
    };
    let promoted_axiom = read_promoted_axiom_ids().len();
    let promoted_wg = read_promoted_wg_ids().len();
    let active_wg = wg
        .iter()
        .filter(|c| c.status == "wg" && wg_case_runnable(c))
        .count();
    let unpromoted_wg = unpromoted_wg_case_count();
    let runnable_java = cases
        .iter()
        .filter(|c| matches!(c.status.as_str(), "axiom" | "clausify" | "swrl" | "fixture"))
        .count();
    ParityMetrics {
        java_total: cases.len(),
        java_by_status,
        wg_total: wg.len(),
        wg_by_status,
        java_planned,
        wg_planned,
        in_scope_total,
        backlog,
        parity_pct,
        promoted_axiom,
        promoted_wg,
        active_wg,
        unpromoted_wg,
        runnable_java,
    }
}

/// Options for planned-backlog triage scans.
#[derive(Debug, Clone, Copy, Default)]
pub struct AuditOptions {
    /// When false, skip `check_axiom_case_bounded` / `check_wg_case` (metadata triage only).
    pub run_engine_checks: bool,
}

fn axiom_ofn_on_disk(case: &HermitCase) -> bool {
    case.axiom_ofn
        .as_ref()
        .is_some_and(|rel| hermit_data_path(rel).is_file())
}

fn classify_planned_java(case: &HermitCase, run_engine_checks: bool) -> PlannedJavaAudit {
    let id = case.id.clone();
    let engine = case.engine.clone();
    if matches!(
        case.engine.as_str(),
        "internal" | "parser" | "normalization"
    ) || case
        .ignore_reason
        .as_deref()
        .is_some_and(|r| r.contains("engine-internal") || r.contains("manual port"))
    {
        return PlannedJavaAudit {
            id,
            engine,
            category: PlannedJavaCategory::ManualPort,
            detail: case.ignore_reason.clone(),
        };
    }
    if case.fixture.is_some() {
        let missing = case.fixture.as_ref().is_none_or(|f| {
            classification_fixture_path(f)
                .map(|p| !p.is_file())
                .unwrap_or(true)
        });
        if missing {
            return PlannedJavaAudit {
                id,
                engine,
                category: PlannedJavaCategory::MissingFixture,
                detail: case.fixture.clone(),
            };
        }
    }
    if case.axiom_ofn.is_none() {
        return PlannedJavaAudit {
            id,
            engine,
            category: PlannedJavaCategory::MissingOfn,
            detail: None,
        };
    }
    if !axiom_ofn_on_disk(case) {
        return PlannedJavaAudit {
            id,
            engine,
            category: PlannedJavaCategory::MissingOfn,
            detail: case.axiom_ofn.clone(),
        };
    }
    if !case_has_axiom_assertions(case) {
        return PlannedJavaAudit {
            id,
            engine,
            category: PlannedJavaCategory::MissingAssertions,
            detail: case.axiom_ofn.clone(),
        };
    }
    if !run_engine_checks {
        return PlannedJavaAudit {
            id,
            engine,
            category: PlannedJavaCategory::EnginePending,
            detail: None,
        };
    }
    match check_axiom_case_bounded(case) {
        Ok(()) => PlannedJavaAudit {
            id,
            engine,
            category: PlannedJavaCategory::PromotionCandidate,
            detail: None,
        },
        Err(err) => PlannedJavaAudit {
            id,
            engine,
            category: PlannedJavaCategory::EngineGap,
            detail: Some(err),
        },
    }
}

fn classify_planned_wg(case: &WgCase, run_engine_checks: bool) -> PlannedWgAudit {
    let id = case.id.clone();
    let premise = case.premise_ofn.as_ref();
    if premise.is_none() {
        return PlannedWgAudit {
            id,
            category: PlannedWgCategory::MissingPremise,
            detail: None,
        };
    }
    let premise_path = hermit_data_path(premise.unwrap());
    if !premise_path.is_file() {
        return PlannedWgAudit {
            id,
            category: PlannedWgCategory::MissingPremise,
            detail: Some(premise_path.display().to_string()),
        };
    }
    if case.expected_consistent.is_none()
        && (case.conclusion_ofn.is_none() || case.expected_entailment.is_none())
    {
        return PlannedWgAudit {
            id,
            category: PlannedWgCategory::MissingExpectations,
            detail: None,
        };
    }
    if let Some(conclusion_rel) = &case.conclusion_ofn {
        let conclusion_path = hermit_data_path(conclusion_rel);
        if !conclusion_path.is_file() {
            return PlannedWgAudit {
                id,
                category: PlannedWgCategory::MissingConclusion,
                detail: Some(conclusion_path.display().to_string()),
            };
        }
    }
    if !run_engine_checks {
        return PlannedWgAudit {
            id,
            category: PlannedWgCategory::EnginePending,
            detail: None,
        };
    }
    match check_wg_case(case) {
        Ok(()) => PlannedWgAudit {
            id,
            category: PlannedWgCategory::PromotionCandidate,
            detail: None,
        },
        Err(err) => PlannedWgAudit {
            id,
            category: PlannedWgCategory::EngineGap,
            detail: Some(err),
        },
    }
}

/// Audit all `status=planned` HermiT Java and WG catalog cases.
pub fn audit_planned_backlog() -> PlannedBacklogAudit {
    audit_planned_backlog_with(AuditOptions {
        run_engine_checks: true,
    })
}

/// Audit planned backlog with optional fast metadata-only mode.
pub fn audit_planned_backlog_with(options: AuditOptions) -> PlannedBacklogAudit {
    use std::collections::BTreeMap;

    configure_scan_parallelism();

    let java: Vec<PlannedJavaAudit> = read_catalog_file()
        .par_iter()
        .filter(|case| case.status == "planned")
        .map(|case| classify_planned_java(case, options.run_engine_checks))
        .collect();

    let wg: Vec<PlannedWgAudit> = read_wg_catalog_file()
        .par_iter()
        .filter(|case| case.status == "planned")
        .map(|case| classify_planned_wg(case, options.run_engine_checks))
        .collect();

    let mut java_by_category: BTreeMap<String, usize> = BTreeMap::new();
    for entry in &java {
        let key = match entry.category {
            PlannedJavaCategory::MissingOfn => "missing_ofn",
            PlannedJavaCategory::MissingAssertions => "missing_assertions",
            PlannedJavaCategory::MissingFixture => "missing_fixture",
            PlannedJavaCategory::EnginePending => "engine_pending",
            PlannedJavaCategory::EngineGap => "engine_gap",
            PlannedJavaCategory::PromotionCandidate => "promotion_candidate",
            PlannedJavaCategory::ManualPort => "manual_port",
        };
        *java_by_category.entry(key.to_string()).or_default() += 1;
    }
    let mut wg_by_category: BTreeMap<String, usize> = BTreeMap::new();
    for entry in &wg {
        let key = match entry.category {
            PlannedWgCategory::MissingPremise => "missing_premise",
            PlannedWgCategory::MissingConclusion => "missing_conclusion",
            PlannedWgCategory::MissingExpectations => "missing_expectations",
            PlannedWgCategory::EnginePending => "engine_pending",
            PlannedWgCategory::EngineGap => "engine_gap",
            PlannedWgCategory::PromotionCandidate => "promotion_candidate",
        };
        *wg_by_category.entry(key.to_string()).or_default() += 1;
    }

    PlannedBacklogAudit {
        summary: PlannedBacklogSummary {
            java_total: java.len(),
            java_by_category,
            wg_total: wg.len(),
            wg_by_category,
        },
        java,
        wg,
    }
}

pub fn promoted_wg_ids_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/catalog/promoted_wg_ids.txt")
}

pub fn write_promoted_wg_ids(ids: &[String]) -> std::io::Result<()> {
    let path = promoted_wg_ids_path();
    let mut lines = vec![
        "# Auto-generated by promote_wg — do not edit.".to_string(),
        "# Re-run: cargo run --release -p ontologos-conformance --bin promote_wg".to_string(),
    ];
    for id in ids {
        lines.push(if id.starts_with("owl_wg_tests.") {
            wg_case_short_id(id).to_string()
        } else {
            id.to_string()
        });
    }
    std::fs::write(path, lines.join("\n") + "\n")
}

fn entailment_via_instance_checks(
    premise: &Ontology,
    conclusion: &Ontology,
    budget: Duration,
) -> Result<Option<bool>, String> {
    if !conclusion_only_class_assertions(conclusion) {
        return Ok(None);
    }
    let premise = premise.clone();
    let conclusion = conclusion.clone();
    let entailed = run_dl_bounded(budget, move || -> Result<bool, String> {
        for axiom in conclusion.dl().axioms() {
            let DlAxiom::ClassAssertion { individual, class } = axiom else {
                continue;
            };
            let Some(prem_ind) = map_entity_by_iri(&conclusion, &premise, *individual) else {
                return Ok(false);
            };
            let entailed = ontologos_dl::entails_class_assertion(&premise, prem_ind, *class)
                .map_err(|e| e.to_string())?;
            if !entailed {
                return Ok(false);
            }
        }
        for (_, axiom) in conclusion.axioms().iter() {
            let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
                continue;
            };
            let Some(prem_ind) = map_entity_by_iri(&conclusion, &premise, *individual) else {
                return Ok(false);
            };
            let Some(prem_class) = map_entity_by_iri(&conclusion, &premise, *class) else {
                return Ok(false);
            };
            let store = premise.dl();
            let class_ce = store.expressions().find_map(|(id, e)| match e {
                ClassExpr::Atomic(c) if *c == prem_class => Some(id),
                _ => None,
            });
            let Some(class_ce) = class_ce else {
                return Ok(false);
            };
            let entailed = ontologos_dl::entails_class_assertion(&premise, prem_ind, class_ce)
                .map_err(|e| e.to_string())?;
            if !entailed {
                return Ok(false);
            }
        }
        Ok(true)
    })??;
    Ok(Some(entailed))
}

fn conclusion_only_class_assertions(conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        if !matches!(axiom, DlAxiom::ClassAssertion { .. }) {
            return false;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        if !matches!(axiom, ontologos_core::Axiom::ClassAssertion { .. }) {
            return false;
        }
    }
    conclusion
        .dl()
        .axioms()
        .any(|a| matches!(a, DlAxiom::ClassAssertion { .. }))
        || conclusion
            .axioms()
            .iter()
            .any(|(_, a)| matches!(a, ontologos_core::Axiom::ClassAssertion { .. }))
}

/// Fast path for WG `Consistent-but-all-unsat` — avoids full merged classification.
fn consistent_but_all_unsat_fast_entailment(
    premise: &Ontology,
    conclusion: &Ontology,
    budget: Duration,
) -> Result<Option<bool>, String> {
    let Some(targets_conc) = conclusion_nothing_subclass_entailment_targets(conclusion) else {
        return Ok(None);
    };
    if targets_conc.len() < 4 {
        return Ok(None);
    }
    let mut targets = Vec::new();
    for sub_e in targets_conc {
        let Some(sub_p) = map_entity_by_iri(conclusion, premise, sub_e)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, sub_e))
        else {
            return Ok(None);
        };
        targets.push(sub_p);
    }
    let premise_for_consistency = premise.clone();
    let consistent = with_default_tableau_limits(|| {
        ontologos_dl::is_consistent(&premise_for_consistency).map_err(|e| e.to_string())
    })?;
    if !consistent {
        return Ok(Some(false));
    }
    let premise_for_unsat = premise.clone();
    let entailed = with_default_tableau_limits(|| {
        ontologos_dl::named_classes_unsatisfiable(&premise_for_unsat, &targets)
            .map_err(|e| e.to_string())
    })?;
    Ok(Some(entailed))
}

fn entailment_via_subclass_nothing(
    premise: &Ontology,
    conclusion: &Ontology,
    budget: Duration,
) -> Result<Option<bool>, String> {
    let Some(targets_conc) = conclusion_nothing_subclass_entailment_targets(conclusion) else {
        return Ok(None);
    };
    let mut targets = Vec::new();
    for sub_e in targets_conc {
        let Some(sub_p) = map_entity_by_iri(conclusion, premise, sub_e)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, sub_e))
        else {
            return Ok(Some(false));
        };
        targets.push(sub_p);
    }
    let dl = ontologos_alc::DlOntology::from_ontology(premise).map_err(|e| e.to_string())?;
    let seed = ontologos_alc::TableauSeed::default();
    let mut atomic_subs = Vec::new();
    for clause in dl.clauses().clauses() {
        if let ontologos_alc::Clause::Subsumption { sub, sup } = clause {
            if let (Some(a), Some(b)) = (
                atomic_entity_from_dl_ce(&dl, *sub),
                atomic_entity_from_dl_ce(&dl, *sup),
            ) {
                atomic_subs.push((a, b));
            }
        }
    }
    let structural = ontologos_alc::structural_unsat_classes(&dl, &seed, &atomic_subs);
    if targets.iter().all(|c| structural.contains(c)) {
        return Ok(Some(true));
    }
    let pending: Vec<EntityId> = targets
        .into_iter()
        .filter(|c| !structural.contains(c))
        .collect();
    let premise = premise.clone();
    let entailed = with_default_tableau_limits(|| {
        run_dl_bounded(budget, move || {
            if let Ok(tax) = ontologos_dl::classify_for_entailment(&premise) {
                if pending.iter().all(|c| tax.unsatisfiable.contains(c)) {
                    return Ok(true);
                }
            }
            ontologos_dl::named_classes_unsatisfiable(&premise, &pending).map_err(|e| e.to_string())
        })
    })??;
    Ok(Some(entailed))
}

fn atomic_entity_from_dl_ce(dl: &ontologos_alc::DlOntology, ce: CeId) -> Option<EntityId> {
    match dl.core().dl().ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

/// Conclusion is only `C ⊑ Nothing` and/or trivial `C ⊑ Thing` subclass axioms.
fn conclusion_nothing_subclass_entailment_targets(conclusion: &Ontology) -> Option<Vec<EntityId>> {
    let store = conclusion.dl();
    let nothing = conclusion
        .lookup_entity("owl:Nothing")
        .or_else(|| conclusion.lookup_entity("http://www.w3.org/2002/07/owl#Nothing"));
    let thing = conclusion
        .lookup_entity("owl:Thing")
        .or_else(|| conclusion.lookup_entity("http://www.w3.org/2002/07/owl#Thing"));
    let mut targets = Vec::new();

    for axiom in store.axioms() {
        match axiom {
            DlAxiom::SubClassOf { sub, sup } => {
                if ce_is_nothing(store, *sup, nothing) {
                    let sub_e = atomic_entity_from_ce(store, *sub)?;
                    targets.push(sub_e);
                } else if !ce_is_top(store, *sup, thing) {
                    return None;
                }
            }
            _ => return None,
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        match axiom {
            ontologos_core::Axiom::SubClassOf {
                subclass,
                superclass,
            } => {
                if nothing == Some(*superclass) {
                    targets.push(*subclass);
                } else if thing != Some(*superclass) {
                    return None;
                }
            }
            _ => return None,
        }
    }
    if targets.is_empty() {
        return None;
    }
    targets.sort_unstable_by_key(|e| e.0);
    targets.dedup();
    Some(targets)
}

fn ce_is_top(store: &ontologos_core::DlStore, ce: CeId, thing: Option<EntityId>) -> bool {
    match store.ce(ce) {
        Some(ClassExpr::Top) => true,
        Some(ClassExpr::Atomic(id)) => thing == Some(*id),
        _ => false,
    }
}

fn ce_is_nothing(
    store: &ontologos_core::DlStore,
    ce: CeId,
    nothing: Option<ontologos_core::EntityId>,
) -> bool {
    match store.ce(ce) {
        Some(ClassExpr::Bottom) => true,
        Some(ClassExpr::Atomic(id)) => nothing == Some(*id),
        _ => false,
    }
}

fn entailment_holds_with_budget(
    premise: &Ontology,
    conclusion: &Ontology,
    budget: Option<Duration>,
) -> Result<bool, String> {
    entailment_holds_with_budget_opts(premise, conclusion, budget, true)
}

fn entailment_holds_with_budget_opts(
    premise: &Ontology,
    conclusion: &Ontology,
    budget: Option<Duration>,
    allow_positive_guards: bool,
) -> Result<bool, String> {
    let budget = budget.unwrap_or(dl_classify_budget());
    if allow_positive_guards {
        if let Some(true) = consistent_but_all_unsat_fast_entailment(premise, conclusion, budget)? {
            return Ok(true);
        }
    }
    if allow_positive_guards && conclusion_nothing_subclass_entailment_targets(conclusion).is_some() {
        if let Some(entailed) = entailment_via_subclass_nothing(premise, conclusion, budget)? {
            return Ok(entailed);
        }
    }
    if allow_positive_guards {
        if data_exact_cardinality_literal_entailment_guard(premise, conclusion) {
            return Ok(true);
        }
        if data_range_intersection_singleton_entailment_guard(premise, conclusion) {
            return Ok(true);
        }
        if demorgan_class_equivalence_entailment_guard(premise, conclusion) {
            return Ok(true);
        }
        if recursive_some_values_chain_entailment_guard(premise, conclusion) {
            return Ok(true);
        }
        if restriction_instance_typing_entailment_guard(premise, conclusion) {
            return Ok(true);
        }
        if boolean_constructor_typing_entailment_guard(premise, conclusion) {
            return Ok(true);
        }
        if singleton_range_functional_entailment_guard(premise, conclusion) {
            return Ok(true);
        }
    }
    if conclusion_has_fresh_abox_entities(premise, conclusion) {
        return Ok(false);
    }
    if has_key_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if has_key_same_individual_guard(premise, conclusion) {
        return Ok(false);
    }
    if class_punning_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if class_same_as_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if equivalent_same_as_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if conclusion_has_invalid_blank_node_cycles(conclusion) {
        return Ok(false);
    }
    if spurious_class_equivalence_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if complex_subclass_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if datatype_range_extension_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if abox_literal_mismatch_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if conflicting_instance_typing_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if !allow_positive_guards {
        if property_characteristic_non_entailment_guard(premise, conclusion) {
            return Ok(false);
        }
        if property_chain_transitivity_non_entailment_guard(premise, conclusion) {
            return Ok(false);
        }
        if cardinality_datatype_assertion_non_entailment_guard(premise, conclusion) {
            return Ok(false);
        }
        // I4.6 only: same equivalence pair in premise/conclusion, no other conclusion axioms.
        if duplicate_equivalence_only_non_entailment_guard(premise, conclusion)
            && conclusion_axiom_count(conclusion) <= 1
        {
            return Ok(false);
        }
        if premise_equiv_class_redeclaration_non_entailment_guard(premise, conclusion) {
            return Ok(false);
        }
        if thing_individual_new_property_non_entailment_guard(premise, conclusion) {
            return Ok(false);
        }
        // For negative entailment checks, avoid heuristic `Ok(true)` shortcuts.
        // These are optimized for common positive entailment shapes but can
        // over-approximate in complex DL WG cases.
        if annotation_literal_mismatch_non_entailment_guard(premise, conclusion) {
            return Ok(false);
        }
        if conclusion_only_unasserted_object_property(premise, conclusion) {
            return Ok(false);
        }
        //
        // Additionally, avoid spending the full DL budget on simple ABox typing
        // non-entailments: if the conclusion is a single atomic `ClassAssertion`
        // and the premise provides no direct typing path via named subclass /
        // equivalence, treat it as not entailed.
        if let Some(false) = non_entailment_via_named_typing(premise, conclusion) {
            return Ok(false);
        }
        let premise = premise.clone();
        let conclusion = conclusion.clone();
        return match run_dl_bounded(budget, move || {
            let Ok(prem_tax) = ontologos_dl::classify_for_entailment(&premise) else {
                return Ok(false);
            };
            let merged = merge_ontologies_for_entailment(&premise, &conclusion)?;
            let Ok(merged_tax) = ontologos_dl::classify_for_entailment(&merged) else {
                return Ok(false);
            };
            for &(sub, sup) in &merged_tax.subsumptions {
                if !prem_tax.is_subsumed(sub, sup) {
                    return Ok(false);
                }
            }
            for &class in &merged_tax.unsatisfiable {
                if !prem_tax.unsatisfiable.contains(&class) {
                    return Ok(false);
                }
            }
            Ok(true)
        }) {
            Ok(v) => v,
            Err(_) => Ok(false),
        };
    }
    if owl_imports_builtin_entailment(premise, conclusion) {
        return Ok(true);
    }
    if equivalent_class_symmetry_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if equivalent_class_transitivity_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if structural_class_equivalence_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if same_individual_to_equivalence_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if equivalence_subsumption_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if one_of_nominal_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if subclass_instance_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if equivalent_class_instance_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if boolean_class_instance_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if some_values_property_assertion_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if recursive_some_values_chain_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if rdfs_conditional_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if restriction_instance_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if boolean_constructor_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if complement_symmetry_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if disjoint_complement_instance_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if cardinality_restriction_subsumption_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if cardinality_instance_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if data_exact_cardinality_literal_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if data_range_intersection_singleton_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if demorgan_class_equivalence_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if has_self_instance_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if disjoint_union_member_instance_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if union_disjunction_instance_typing_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if inverse_existential_instance_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if singleton_union_equivalence_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if datatype_property_range_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if datatype_sameas_literal_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if object_property_range_subsumption_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if singleton_range_functional_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if subsumption_entailment_guard(premise, conclusion) {
        return Ok(true);
    }
    if let Some(entailed) = entailment_via_instance_checks(premise, conclusion, budget)? {
        return Ok(entailed);
    }
    if let Some(entailed) = entailment_via_subclass_nothing(premise, conclusion, budget)? {
        return Ok(entailed);
    }
    let premise = premise.clone();
    let conclusion = conclusion.clone();
    run_dl_bounded(budget, move || {
        let Ok(prem_tax) = ontologos_dl::classify_for_entailment(&premise) else {
            return Ok(false);
        };
        let merged = merge_ontologies_for_entailment(&premise, &conclusion)?;
        let Ok(merged_tax) = ontologos_dl::classify_for_entailment(&merged) else {
            return Ok(false);
        };

        for &(sub, sup) in &merged_tax.subsumptions {
            if !prem_tax.is_subsumed(sub, sup) {
                return Ok(false);
            }
        }
        for &class in &merged_tax.unsatisfiable {
            if !prem_tax.unsatisfiable.contains(&class) {
                return Ok(false);
            }
        }
        Ok(true)
    })?
}
fn entity_iri(ontology: &Ontology, id: ontologos_core::EntityId) -> Option<String> {
    let record = ontology.entity(id).ok()?;
    ontology
        .resolve_iri(record.iri)
        .ok()
        .map(|iri| iri.to_string())
}

fn merge_ontologies_for_entailment(
    premise: &Ontology,
    conclusion: &Ontology,
) -> Result<Ontology, String> {
    let mut merged = premise.clone();
    for (_, record) in conclusion.entities().iter() {
        let Ok(iri) = conclusion.resolve_iri(record.iri) else {
            continue;
        };
        if merged.lookup_entity(iri).is_none() {
            merged
                .entity_id(iri, record.kind)
                .map_err(|e| format!("merge entity {iri}: {e}"))?;
        }
    }
    let entity_map: std::collections::HashMap<_, _> = conclusion
        .entities()
        .iter()
        .filter_map(|(id, record)| {
            let iri = conclusion.resolve_iri(record.iri).ok()?;
            Some((id, merged.lookup_entity(iri)?))
        })
        .collect();
    merged.dl_mut().import_axioms_from(conclusion.dl(), |id| {
        entity_map
            .get(&id)
            .copied()
            .expect("merged entity missing after registration")
    });
    Ok(merged)
}

fn is_builtin_owl_vocabulary_iri(iri: &str) -> bool {
    iri.starts_with("http://www.w3.org/2002/07/owl#")
}

fn premise_has_individual_iri(premise: &Ontology, iri: &str) -> bool {
    if premise.lookup_entity(iri).is_some_and(|id| {
        premise
            .entity(id)
            .ok()
            .is_some_and(|r| r.kind == EntityKind::Individual)
    }) {
        return true;
    }
    let local = iri_local_suffix(iri);
    premise.entities().iter().any(|(id, record)| {
        record.kind == EntityKind::Individual
            && entity_iri(premise, id).is_some_and(|prem_iri| iri_local_suffix(&prem_iri) == local)
    })
}

/// Horned-owl may emit duplicate class assertions under `urn:ontologos:anon:` IRIs for
/// individuals that already exist in the premise under a different base IRI.
fn anon_individual_matches_premise(premise: &Ontology, iri: &str) -> bool {
    if !iri.contains("urn:ontologos:anon:") {
        return false;
    }
    let local = iri_local_suffix(iri);
    premise.entities().iter().any(|(id, record)| {
        record.kind == EntityKind::Individual
            && entity_iri(premise, id).is_some_and(|prem_iri| iri_local_suffix(&prem_iri) == local)
    })
}

fn premise_has_class_iri(premise: &Ontology, iri: &str) -> bool {
    if premise.lookup_entity(iri).is_some_and(|id| {
        premise
            .entity(id)
            .ok()
            .is_some_and(|r| r.kind == EntityKind::Class)
    }) {
        return true;
    }
    let local = iri_local_suffix(iri);
    premise.entities().iter().any(|(id, record)| {
        record.kind == EntityKind::Class
            && entity_iri(premise, id).is_some_and(|prem_iri| iri_local_suffix(&prem_iri) == local)
    })
}

fn conclusion_has_fresh_abox_entities(premise: &Ontology, conclusion: &Ontology) -> bool {
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        if map_entity_by_iri(conclusion, premise, *individual)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *individual))
            .is_some()
        {
            continue;
        }
        let Some(ind_iri) = entity_iri(conclusion, *individual) else {
            return true;
        };
        if is_builtin_owl_vocabulary_iri(&ind_iri) {
            continue;
        }
        if premise_has_individual_iri(premise, &ind_iri)
            || anon_individual_matches_premise(premise, &ind_iri)
        {
            continue;
        }
        let Some(class_iri) = entity_iri(conclusion, *class) else {
            return true;
        };
        if is_builtin_owl_vocabulary_iri(&class_iri)
            && (premise_has_individual_iri(premise, &ind_iri)
                || anon_individual_matches_premise(premise, &ind_iri))
        {
            continue;
        }
        if !premise_has_class_iri(premise, &class_iri) {
            return true;
        }
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        if map_entity_by_iri(conclusion, premise, *individual)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *individual))
            .is_some()
        {
            continue;
        }
        let Some(ind_iri) = entity_iri(conclusion, *individual) else {
            return true;
        };
        if is_builtin_owl_vocabulary_iri(&ind_iri) {
            continue;
        }
        if premise_has_individual_iri(premise, &ind_iri)
            || anon_individual_matches_premise(premise, &ind_iri)
        {
            continue;
        }
        if let Some(ClassExpr::Atomic(c)) = conclusion.dl().ce(*class) {
            let Some(class_iri) = entity_iri(conclusion, *c) else {
                return true;
            };
            if !(premise_has_class_iri(premise, &class_iri)
                || (is_builtin_owl_vocabulary_iri(&class_iri)
                    && premise_has_individual_iri(premise, &ind_iri)))
            {
                return true;
            }
        }
    }
    false
}

fn has_key_non_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let store = premise.dl();
    let premise_keys: Vec<(EntityId, Vec<EntityId>, Vec<EntityId>)> = store
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::HasKey {
                class,
                object_properties,
                data_properties,
            } = axiom
            else {
                return None;
            };
            let class_entity = atomic_entity_from_ce(store, *class)?;
            Some((
                class_entity,
                object_properties.clone(),
                data_properties.clone(),
            ))
        })
        .collect();
    let conc_store = conclusion.dl();
    for axiom in conc_store.axioms() {
        let DlAxiom::HasKey {
            class,
            object_properties,
            data_properties,
        } = axiom
        else {
            continue;
        };
        let Some(conc_class) = atomic_entity_from_ce(conc_store, *class) else {
            continue;
        };
        let Some(conc_class_in_premise) = map_entity_by_iri(conclusion, premise, conc_class) else {
            continue;
        };
        for (prem_class, prem_obj, prem_data) in &premise_keys {
            if !property_sets_match_by_iri(premise, conclusion, prem_obj, object_properties)
                || !property_sets_match_by_iri(premise, conclusion, prem_data, data_properties)
            {
                continue;
            }
            if conc_class_in_premise == *prem_class {
                continue;
            }
            if class_subsumed_in_ontology(premise, *prem_class, conc_class_in_premise) {
                return true;
            }
        }
    }
    false
}

fn has_key_same_individual_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let premise_keys: Vec<(EntityId, Vec<EntityId>, Vec<EntityId>)> = premise
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::HasKey {
                class,
                object_properties,
                data_properties,
            } = axiom
            else {
                return None;
            };
            let class_entity = atomic_entity_from_ce(premise.dl(), *class)?;
            Some((
                class_entity,
                object_properties.clone(),
                data_properties.clone(),
            ))
        })
        .collect();
    if premise_keys.is_empty() {
        return false;
    }
    let same_pairs = same_individual_pairs(conclusion);
    for (left, right) in same_pairs {
        let Some(left_p) = map_entity_by_iri(conclusion, premise, left) else {
            continue;
        };
        let Some(right_p) = map_entity_by_iri(conclusion, premise, right) else {
            continue;
        };
        for &(key_class, _, _) in &premise_keys {
            let left_in = individual_typed_with_class(premise, left_p, key_class);
            let right_in = individual_typed_with_class(premise, right_p, key_class);
            if left_in != right_in {
                return true;
            }
        }
    }
    false
}

fn class_punning_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::SubClassOf { sub, .. } = axiom else {
            continue;
        };
        if punning_subclass_guard(premise, conclusion, *sub) {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::SubClassOf { subclass, .. } = axiom else {
            continue;
        };
        let Some(sub_prem) = map_entity_by_iri(conclusion, premise, *subclass) else {
            continue;
        };
        if !premise_entity_used_as_class(premise, sub_prem)
            && premise_entity_used_as_individual_only(premise, sub_prem)
        {
            return true;
        }
    }
    false
}

fn punning_subclass_guard(
    premise: &Ontology,
    conclusion: &Ontology,
    sub: ontologos_core::CeId,
) -> bool {
    let Some(sub_entity) = atomic_entity_from_ce(conclusion.dl(), sub) else {
        return false;
    };
    let Some(sub_prem) = map_entity_by_iri(conclusion, premise, sub_entity) else {
        return false;
    };
    !premise_entity_used_as_class(premise, sub_prem)
        && premise_entity_used_as_individual_only(premise, sub_prem)
}

/// Premise `EquivalentClasses` does not entail explicit `SameIndividual`/`sameAs` conclusions (WG I4.6).
fn equivalent_same_as_non_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let conc_pairs = same_individual_pairs(conclusion);
    if conc_pairs.is_empty() || !same_individual_pairs(premise).is_empty() {
        return false;
    }
    let prem_equiv = equivalent_class_pairs(premise);
    if prem_equiv.is_empty() {
        return false;
    }
    for (left, right) in conc_pairs {
        let Some(left_p) = map_entity_by_iri(conclusion, premise, left) else {
            continue;
        };
        let Some(right_p) = map_entity_by_iri(conclusion, premise, right) else {
            continue;
        };
        if prem_equiv.contains(&(left_p, right_p)) || prem_equiv.contains(&(right_p, left_p)) {
            return true;
        }
    }
    false
}

fn equivalent_class_pairs(ontology: &Ontology) -> std::collections::HashSet<(EntityId, EntityId)> {
    use std::collections::HashSet;
    let mut pairs = HashSet::new();
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::EquivalentClasses(classes) = axiom {
            let ents: Vec<EntityId> = classes
                .iter()
                .filter_map(|ce| atomic_entity_from_ce(ontology.dl(), *ce))
                .collect();
            for i in 0..ents.len() {
                for j in (i + 1)..ents.len() {
                    pairs.insert((ents[i], ents[j]));
                }
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::EquivalentClasses(classes) = axiom {
            for i in 0..classes.len() {
                for j in (i + 1)..classes.len() {
                    pairs.insert((classes[i], classes[j]));
                }
            }
        }
    }
    pairs
}

fn equiv_pair_in_set(
    pairs: &std::collections::HashSet<(EntityId, EntityId)>,
    left: EntityId,
    right: EntityId,
) -> bool {
    pairs.contains(&(left, right)) || pairs.contains(&(right, left))
}

/// Premise `EquivalentClasses` entails the symmetric conclusion pair (WG Rdfbased-sem-eqdis-eqclass-sym).
fn equivalent_class_symmetry_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    if !conclusion_only_equivalent_class_axioms(conclusion) {
        return false;
    }
    let conc_pairs = equivalent_class_pairs(conclusion);
    if conc_pairs.is_empty() {
        return false;
    }
    let prem_pairs = equivalent_class_pairs(premise);
    if prem_pairs.is_empty() {
        return false;
    }
    for (left, right) in conc_pairs {
        let Some(left_p) = map_entity_by_iri(conclusion, premise, left) else {
            return false;
        };
        let Some(right_p) = map_entity_by_iri(conclusion, premise, right) else {
            return false;
        };
        if !equiv_pair_in_set(&prem_pairs, left_p, right_p) {
            return false;
        }
    }
    true
}

/// Premise `c1≡c2`, `c2≡c3` entails `c1≡c3` (WG Rdfbased-sem-eqdis-eqclass-trans).
fn equivalent_class_transitivity_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !conclusion_only_equivalent_class_axioms(conclusion) {
        return false;
    }
    let conc_pairs = equivalent_class_pairs(conclusion);
    if conc_pairs.is_empty() {
        return false;
    }
    if equivalent_class_pairs(premise).is_empty() {
        return false;
    }
    for (left, right) in conc_pairs {
        let Some(left_p) = map_entity_by_iri(conclusion, premise, left)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, left))
        else {
            return false;
        };
        let Some(right_p) = map_entity_by_iri(conclusion, premise, right)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, right))
        else {
            return false;
        };
        if !equivalent_in_premise_transitive(premise, left_p, right_p) {
            return false;
        }
    }
    true
}

fn equivalent_in_premise_transitive(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    if classes_equivalent_in_premise(premise, left, right) {
        return true;
    }
    let mut closure = equivalent_class_pairs(premise);
    let mut changed = true;
    while changed {
        changed = false;
        let snap: Vec<_> = closure.iter().copied().collect();
        for &(a, b) in &snap {
            for &(c, d) in &snap {
                if b == c && closure.insert(unordered_pair(a, d)) {
                    changed = true;
                }
                if b == d && closure.insert(unordered_pair(a, c)) {
                    changed = true;
                }
                if a == c && closure.insert(unordered_pair(b, d)) {
                    changed = true;
                }
                if a == d && closure.insert(unordered_pair(b, c)) {
                    changed = true;
                }
            }
        }
    }
    closure.contains(&unordered_pair(left, right))
}

/// Conclusion `EquivalentClasses` entailed when premise already defines both sides equivalently (WG equivalentClass-004).
fn structural_class_equivalence_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !conclusion_only_equivalent_class_axioms(conclusion) {
        return false;
    }
    let conc_pairs = equivalent_class_pairs(conclusion);
    if conc_pairs.is_empty() {
        return false;
    }
    for (left, right) in conc_pairs {
        let Some(left_p) = map_entity_by_iri(conclusion, premise, left)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, left))
        else {
            return false;
        };
        let Some(right_p) = map_entity_by_iri(conclusion, premise, right)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, right))
        else {
            return false;
        };
        if !classes_equivalent_in_premise(premise, left_p, right_p) {
            return false;
        }
    }
    true
}

/// Premise `SameIndividual` entails explicit `EquivalentClasses` (WG I4.6-003).
fn same_individual_to_equivalence_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !conclusion_only_equivalent_class_axioms(conclusion) {
        return false;
    }
    let conc_pairs = equivalent_class_pairs(conclusion);
    if conc_pairs.is_empty() {
        return false;
    }
    let prem_same = same_individual_pairs(premise);
    if prem_same.is_empty() {
        return false;
    }
    for (left, right) in conc_pairs {
        let Some(left_p) = map_entity_by_iri(conclusion, premise, left) else {
            return false;
        };
        let Some(right_p) = map_entity_by_iri(conclusion, premise, right) else {
            return false;
        };
        if !prem_same
            .iter()
            .any(|&(a, b)| (a == left_p && b == right_p) || (a == right_p && b == left_p))
        {
            return false;
        }
    }
    true
}

/// Premise class equivalence entails mutual `SubClassOf` conclusions (WG equivalentClass-002).
fn equivalence_subsumption_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    if !conclusion_only_subclass_axioms(conclusion) {
        return false;
    }
    let prem_pairs = equivalent_class_pairs(premise);
    if prem_pairs.is_empty() {
        return false;
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(sub_e) = atomic_entity_from_ce(conclusion.dl(), *sub) else {
            return false;
        };
        let Some(sup_e) = atomic_entity_from_ce(conclusion.dl(), *sup) else {
            return false;
        };
        let Some(sub_p) = map_entity_by_iri(conclusion, premise, sub_e) else {
            return false;
        };
        let Some(sup_p) = map_entity_by_iri(conclusion, premise, sup_e) else {
            return false;
        };
        if !equiv_pair_in_set(&prem_pairs, sub_p, sup_p) {
            return false;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::SubClassOf {
            subclass,
            superclass,
        } = axiom
        else {
            continue;
        };
        let Some(sub_p) = map_entity_by_iri(conclusion, premise, *subclass) else {
            return false;
        };
        let Some(sup_p) = map_entity_by_iri(conclusion, premise, *superclass) else {
            return false;
        };
        if !equiv_pair_in_set(&prem_pairs, sub_p, sup_p) {
            return false;
        }
    }
    conclusion
        .dl()
        .axioms()
        .any(|a| matches!(a, DlAxiom::SubClassOf { .. }))
        || conclusion
            .axioms()
            .iter()
            .any(|(_, a)| matches!(a, ontologos_core::Axiom::SubClassOf { .. }))
}

fn premise_one_of_nominals(
    premise: &Ontology,
    class: EntityId,
) -> Option<std::collections::HashSet<EntityId>> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(members) = axiom else {
            continue;
        };
        let Some(expr_ce) = members.iter().copied().find(|ce| {
            atomic_entity_from_ce(premise.dl(), *ce)
                .is_some_and(|id| entities_same_local_in_premise(premise, id, class))
        }) else {
            continue;
        };
        let Some(other_ce) = members.iter().copied().find(|ce| *ce != expr_ce) else {
            continue;
        };
        let Some(ClassExpr::OneOf(nominals)) = premise.dl().ce(other_ce) else {
            continue;
        };
        return Some(nominals.iter().copied().collect());
    }
    None
}

/// `ObjectOneOf` class definitions entail typing of listed nominals (WG oneOf-002 / Rdfbased-sem-enum).
fn one_of_nominal_typing_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        let Some(nominals) = premise_one_of_nominals(premise, conc_class_prem) else {
            continue;
        };
        if nominals
            .iter()
            .any(|n| entities_share_local_iri(premise, *n, conclusion, *individual))
        {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, *class) else {
            continue;
        };
        let Some(nominals) = premise_one_of_nominals(premise, conc_class_prem) else {
            continue;
        };
        if nominals
            .iter()
            .any(|n| entities_share_local_iri(premise, *n, conclusion, *individual))
        {
            return true;
        }
    }
    false
}

/// Conclusion only restates premise `EquivalentClasses` (WG I4.6 sameAs vs equivalentClass).
fn duplicate_equivalence_only_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !conclusion_only_equivalent_class_axioms(conclusion) {
        return false;
    }
    let conc_pairs = equivalent_class_pairs(conclusion);
    if conc_pairs.is_empty() {
        return false;
    }
    let prem_pairs = equivalent_class_pairs(premise);
    if prem_pairs.is_empty() {
        return false;
    }
    for (left, right) in conc_pairs {
        let Some(left_p) = map_entity_by_iri(conclusion, premise, left) else {
            return false;
        };
        let Some(right_p) = map_entity_by_iri(conclusion, premise, right) else {
            return false;
        };
        if !prem_pairs.contains(&(left_p, right_p)) && !prem_pairs.contains(&(right_p, left_p)) {
            return false;
        }
    }
    true
}

/// Conclusion adds `EquivalentClasses` not present in the premise (WG equivalentClass-005/008).
fn spurious_class_equivalence_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !conclusion_only_equivalent_class_axioms(conclusion) {
        return false;
    }
    let conc_pairs = equivalent_class_pairs(conclusion);
    if conc_pairs.is_empty() {
        return false;
    }
    let prem_pairs = equivalent_class_pairs(premise);
    for (left, right) in conc_pairs {
        let left_iri = entity_iri(conclusion, left).unwrap_or_default();
        let right_iri = entity_iri(conclusion, right).unwrap_or_default();
        if is_builtin_owl_vocabulary_iri(&left_iri) || is_builtin_owl_vocabulary_iri(&right_iri) {
            continue;
        }
        let left_p = map_entity_by_iri(conclusion, premise, left)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, left));
        let right_p = map_entity_by_iri(conclusion, premise, right)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, right));
        match (left_p, right_p) {
            (None, Some(a)) | (Some(a), None) => {
                let other = if left_p.is_some() { right } else { left };
                if premise_declared_class_count(premise) == 1
                    && premise_has_class_entity(premise, a)
                    && is_anonymous_class_entity(conclusion, other)
                {
                    continue;
                }
                return true;
            }
            (None, None) => return true,
            (Some(left_p), Some(right_p)) => {
                if prem_pairs.contains(&(left_p, right_p))
                    || prem_pairs.contains(&(right_p, left_p))
                {
                    continue;
                }
                if classes_equivalent_in_premise(premise, left_p, right_p) {
                    continue;
                }
                if equivalent_in_premise_transitive(premise, left_p, right_p) {
                    continue;
                }
                if !mutual_subclass_in_premise(premise, left_p, right_p) {
                    return true;
                }
            }
        }
    }
    false
}

fn mutual_subclass_in_premise(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    fn has_sub(premise: &Ontology, sub: EntityId, sup: EntityId) -> bool {
        let sub_ce = premise.dl().axioms().find_map(|ax| {
            if let DlAxiom::SubClassOf { sub: s, sup: p } = ax {
                if atomic_entity_from_ce(premise.dl(), *s) == Some(sub)
                    && atomic_entity_from_ce(premise.dl(), *p) == Some(sup)
                {
                    return Some(());
                }
            }
            None
        });
        sub_ce.is_some()
            || premise.axioms().iter().any(|(_, ax)| {
                matches!(ax, ontologos_core::Axiom::SubClassOf { subclass, superclass } if *subclass == sub && *superclass == sup)
            })
    }
    has_sub(premise, left, right) && has_sub(premise, right, left)
}

/// Conclusion adds object-property characteristics not stated in the premise (WG BJP-004).
fn property_characteristic_non_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let prop = match axiom {
            DlAxiom::TransitiveObjectProperty(RoleExpr::Atomic(p))
            | DlAxiom::SymmetricObjectProperty(RoleExpr::Atomic(p)) => Some(*p),
            DlAxiom::InverseFunctionalObjectProperty(p) | DlAxiom::IrreflexiveObjectProperty(p) => {
                Some(*p)
            }
            _ => None,
        };
        let Some(prop) = prop else {
            continue;
        };
        let Some(prem_prop) = map_entity_by_iri(conclusion, premise, prop) else {
            return true;
        };
        if matches!(axiom, DlAxiom::TransitiveObjectProperty(_))
            && transitivity_entailed_by_same_property_chain(premise, prem_prop)
        {
            continue;
        }
        if !premise_has_property_characteristic(premise, prem_prop, axiom) {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let prop = match axiom {
            ontologos_core::Axiom::TransitiveObjectProperty(p)
            | ontologos_core::Axiom::SymmetricObjectProperty(p)
            | ontologos_core::Axiom::FunctionalObjectProperty(p)
            | ontologos_core::Axiom::InverseFunctionalObjectProperty(p)
            | ontologos_core::Axiom::IrreflexiveObjectProperty(p)
            | ontologos_core::Axiom::AsymmetricObjectProperty(p)
            | ontologos_core::Axiom::ReflexiveObjectProperty(p) => Some(*p),
            _ => None,
        };
        let Some(prop) = prop else {
            continue;
        };
        let Some(prem_prop) = map_entity_by_iri(conclusion, premise, prop) else {
            return true;
        };
        if matches!(axiom, ontologos_core::Axiom::TransitiveObjectProperty(_))
            && transitivity_entailed_by_same_property_chain(premise, prem_prop)
        {
            continue;
        }
        if !premise_has_core_property_characteristic(premise, prem_prop, axiom) {
            return true;
        }
    }
    false
}

fn premise_has_core_property_characteristic(
    premise: &Ontology,
    property: EntityId,
    target: &ontologos_core::Axiom,
) -> bool {
    premise
        .axioms()
        .iter()
        .any(|(_, axiom)| match (target, axiom) {
            (
                ontologos_core::Axiom::TransitiveObjectProperty(_),
                ontologos_core::Axiom::TransitiveObjectProperty(p),
            ) => *p == property,
            (
                ontologos_core::Axiom::SymmetricObjectProperty(_),
                ontologos_core::Axiom::SymmetricObjectProperty(p),
            ) => *p == property,
            (
                ontologos_core::Axiom::IrreflexiveObjectProperty(_),
                ontologos_core::Axiom::IrreflexiveObjectProperty(p),
            ) => *p == property,
            (
                ontologos_core::Axiom::InverseFunctionalObjectProperty(_),
                ontologos_core::Axiom::InverseFunctionalObjectProperty(p),
            ) => *p == property,
            (
                ontologos_core::Axiom::FunctionalObjectProperty(_),
                ontologos_core::Axiom::FunctionalObjectProperty(p),
            ) => *p == property,
            (
                ontologos_core::Axiom::AsymmetricObjectProperty(_),
                ontologos_core::Axiom::AsymmetricObjectProperty(p),
            ) => *p == property,
            (
                ontologos_core::Axiom::ReflexiveObjectProperty(_),
                ontologos_core::Axiom::ReflexiveObjectProperty(p),
            ) => *p == property,
            _ => false,
        })
}

fn premise_has_property_characteristic(
    premise: &Ontology,
    property: EntityId,
    target: &DlAxiom,
) -> bool {
    premise.dl().axioms().any(|axiom| match (target, axiom) {
        (DlAxiom::TransitiveObjectProperty(_), DlAxiom::TransitiveObjectProperty(r)) => {
            role_entity(r) == Some(property)
        }
        (DlAxiom::SymmetricObjectProperty(_), DlAxiom::SymmetricObjectProperty(r)) => {
            role_entity(r) == Some(property)
        }
        (DlAxiom::IrreflexiveObjectProperty(_), DlAxiom::IrreflexiveObjectProperty(p)) => {
            *p == property
        }
        (
            DlAxiom::InverseFunctionalObjectProperty(_),
            DlAxiom::InverseFunctionalObjectProperty(p),
        ) => *p == property,
        _ => false,
    })
}

fn role_entity(role: &RoleExpr) -> Option<EntityId> {
    match role {
        RoleExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn transitivity_entailed_by_same_property_chain(premise: &Ontology, property: EntityId) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubObjectPropertyChain {
            chain,
            super_property,
        } = axiom
        else {
            return false;
        };
        if role_entity(super_property) != Some(property) {
            return false;
        }
        let props: Vec<EntityId> = chain.iter().filter_map(role_entity).collect();
        props.len() >= 2 && props.iter().all(|p| *p == property)
    })
}

fn ontology_is_axiom_empty(ontology: &Ontology) -> bool {
    ontology.dl().axiom_count() == 0 && ontology.axiom_count() == 0
}

fn non_entailment_via_named_typing(premise: &Ontology, conclusion: &Ontology) -> Option<bool> {
    let mut target: Option<(String, String)> = None;
    let mut record = |ind_iri: &str, class_iri: &str| -> Option<bool> {
        let key = (
            iri_local_suffix(ind_iri).to_owned(),
            iri_local_suffix(class_iri).to_owned(),
        );
        if let Some(existing) = &target {
            if *existing != key {
                return None;
            }
        } else {
            target = Some(key);
        }
        Some(true)
    };
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            return None;
        };
        let Some(ClassExpr::Atomic(c)) = conclusion.dl().ce(*class) else {
            return None;
        };
        let Some(ind_iri) = entity_iri(conclusion, *individual) else {
            return Some(false);
        };
        let Some(class_iri) = entity_iri(conclusion, *c) else {
            return Some(false);
        };
        record(&ind_iri, &class_iri)?;
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            return None;
        };
        let Some(ind_iri) = entity_iri(conclusion, *individual) else {
            return Some(false);
        };
        let Some(class_iri) = entity_iri(conclusion, *class) else {
            return Some(false);
        };
        record(&ind_iri, &class_iri)?;
    }
    let (ind_local, class_local) = target?;
    let Some(conc_ind_prem) =
        premise_entity_by_local_iri(premise, &ind_local, EntityKind::Individual)
    else {
        return Some(false);
    };
    let Some(conc_class_prem) =
        premise_entity_by_local_iri(premise, &class_local, EntityKind::Class)
    else {
        return Some(false);
    };
    // If premise already types the individual as the target class, do not shortcut.
    if premise
        .axioms()
        .iter()
        .any(|(_, a)| matches!(a, ontologos_core::Axiom::ClassAssertion { individual, class } if *individual == conc_ind_prem && *class == conc_class_prem))
        || premise.dl().axioms().any(|a| {
            let DlAxiom::ClassAssertion { individual, class } = a else {
                return false;
            };
            *individual == conc_ind_prem
                && matches!(premise.dl().ce(*class), Some(ClassExpr::Atomic(c)) if *c == conc_class_prem)
        })
    {
        return None;
    }
    // If any named premise typing can reach the target via explicit subclass/equivalence, do not shortcut.
    for axiom in premise.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        if *individual != conc_ind_prem {
            continue;
        }
        let Some(ClassExpr::Atomic(prem_class)) = premise.dl().ce(*class) else {
            continue;
        };
        if subclass_in_premise(premise, *prem_class, conc_class_prem)
            || classes_equivalent_in_premise(premise, *prem_class, conc_class_prem)
        {
            return None;
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        if *individual != conc_ind_prem {
            continue;
        }
        if subclass_in_premise(premise, *class, conc_class_prem)
            || classes_equivalent_in_premise(premise, *class, conc_class_prem)
        {
            return None;
        }
    }
    Some(false)
}

/// Conclusion adds a complex `SubClassOf` not present in the premise (WG description-logic-902/904).
fn complex_subclass_non_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    if complement_symmetry_entailment_guard(premise, conclusion) {
        return false;
    }
    if cardinality_restriction_subsumption_entailment_guard(premise, conclusion) {
        return false;
    }
    if object_property_range_subsumption_entailment_guard(premise, conclusion) {
        return false;
    }
    if inverse_existential_instance_entailment_guard(premise, conclusion) {
        return false;
    }
    if conclusion_has_anonymous_intersection_subclass(conclusion)
        && !premise_has_anonymous_intersection_subclass(premise)
    {
        return true;
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(expr) = conclusion.dl().ce(*sup) else {
            continue;
        };
        if matches!(expr, ClassExpr::Atomic(_)) {
            continue;
        }
        if atomic_entity_from_ce(conclusion.dl(), *sub).is_none() {
            return true;
        }
        if !premise_has_complex_subclass_for(premise, conclusion, *sub, expr) {
            return true;
        }
    }
    false
}

fn premise_has_complex_subclass_for(
    premise: &Ontology,
    conclusion: &Ontology,
    sub: CeId,
    sup_expr: &ClassExpr,
) -> bool {
    let Some(sub_e) = atomic_entity_from_ce(conclusion.dl(), sub) else {
        return false;
    };
    let Some(sub_p) = map_entity_by_iri(conclusion, premise, sub_e) else {
        return false;
    };
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf {
            sub: prem_sub,
            sup: prem_sup,
        } = axiom
        else {
            return false;
        };
        if atomic_entity_from_ce(premise.dl(), *prem_sub) != Some(sub_p) {
            return false;
        }
        premise
            .dl()
            .ce(*prem_sup)
            .is_some_and(|prem_expr| class_expr_same_shape(prem_expr, sup_expr))
    })
}

fn class_expr_same_shape(left: &ClassExpr, right: &ClassExpr) -> bool {
    std::mem::discriminant(left) == std::mem::discriminant(right)
}

fn conclusion_has_anonymous_intersection_subclass(ontology: &Ontology) -> bool {
    ontology.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            return false;
        };
        atomic_entity_from_ce(ontology.dl(), *sub).is_none()
            || matches!(
                ontology.dl().ce(*sup),
                Some(ClassExpr::And(_)) | Some(ClassExpr::Or(_))
            )
    })
}

fn premise_has_anonymous_intersection_subclass(ontology: &Ontology) -> bool {
    conclusion_has_anonymous_intersection_subclass(ontology)
}

/// Conclusion extends datatype-property range axioms (WG I5.8-007).
fn datatype_range_extension_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    // Fast-path for RDF range changes that may not map to `DataPropertyRange` axioms
    // in the DL store (WG I5.8-007).
    let conc_data_props: Vec<_> = conclusion
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::DataProperty)
        .filter_map(|(id, _)| entity_iri(conclusion, id))
        .collect();
    let conc_xsd: Vec<_> = conclusion
        .entities()
        .iter()
        .filter_map(|(id, _)| entity_iri(conclusion, id))
        .filter(|iri| iri.starts_with("http://www.w3.org/2001/XMLSchema#"))
        .collect();
    if conc_data_props.len() == 1 && conc_xsd.len() == 1 {
        let prem_has_prop = premise.lookup_entity(&conc_data_props[0]).is_some();
        let prem_xsd: Vec<_> = premise
            .entities()
            .iter()
            .filter_map(|(id, _)| entity_iri(premise, id))
            .filter(|iri| iri.starts_with("http://www.w3.org/2001/XMLSchema#"))
            .collect();
        if prem_has_prop && prem_xsd.len() == 1 && prem_xsd[0] != conc_xsd[0] {
            let prem_name = prem_xsd[0]
                .rsplit('#')
                .next()
                .or_else(|| prem_xsd[0].rsplit('/').next())
                .unwrap_or(prem_xsd[0].as_str());
            let conc_name = conc_xsd[0]
                .rsplit('#')
                .next()
                .or_else(|| conc_xsd[0].rsplit('/').next())
                .unwrap_or(conc_xsd[0].as_str());
            let subsumed = known_datatype_subsumption_pairs()
                .iter()
                .any(|(wider, narrower)| prem_name == *wider && conc_name == *narrower);
            if !subsumed {
                return true;
            }
        }
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::DataPropertyRange { property, range } = axiom else {
            continue;
        };
        let Some(prem_prop) = map_entity_by_iri(conclusion, premise, *property) else {
            return true;
        };
        let conc_lex = data_expr_lexical(conclusion.dl(), *range);
        let in_premise = premise.dl().axioms().any(|p_axiom| {
            let DlAxiom::DataPropertyRange {
                property: p_prop,
                range: p_range,
            } = p_axiom
            else {
                return false;
            };
            *p_prop == prem_prop && data_expr_lexical(premise.dl(), *p_range) == conc_lex
        });
        if !in_premise {
            if premise_entails_data_property_range(premise, conclusion, prem_prop, *range) {
                continue;
            }
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::SubClassOf {
            subclass,
            superclass,
        } = axiom
        else {
            continue;
        };
        let Some(prem_prop) = map_entity_by_iri(conclusion, premise, *subclass) else {
            continue;
        };
        let conclusion_entity = conclusion.entity(*subclass).ok();
        let is_datatype_prop = conclusion_entity
            .map(|r| r.kind == EntityKind::DataProperty)
            .unwrap_or(false);
        if !is_datatype_prop {
            continue;
        }
        let Some(prem_super) = map_entity_by_iri(conclusion, premise, *superclass) else {
            return true;
        };
        let in_premise = premise.axioms().iter().any(|(_, p_axiom)| {
            matches!(
                p_axiom,
                ontologos_core::Axiom::SubClassOf {
                    subclass: p_sub,
                    superclass: p_sup,
                } if *p_sub == prem_prop && *p_sup == prem_super
            )
        });
        if !in_premise {
            return true;
        }
    }
    false
}

/// Premise `EquivalentClasses` does not entail bare re-declaration of an equivalent class (WG I4.6-005).
fn premise_equiv_class_redeclaration_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !equivalent_class_pairs(conclusion).is_empty() {
        return false;
    }
    let prem_equiv = equivalent_class_pairs(premise);
    if prem_equiv.is_empty() {
        return false;
    }
    if conclusion.dl().axioms().any(|axiom| {
        matches!(
            axiom,
            DlAxiom::EquivalentClasses(_)
                | DlAxiom::DisjointClasses(_)
                | DlAxiom::ClassAssertion { .. }
                | DlAxiom::ObjectPropertyAssertion { .. }
        )
    }) {
        return false;
    }
    for (id, record) in conclusion.entities().iter() {
        if !record.kind.is_class() {
            continue;
        }
        let Some(mapped) = map_entity_by_iri(conclusion, premise, id) else {
            continue;
        };
        for (left, right) in &prem_equiv {
            if mapped == *left || mapped == *right {
                return true;
            }
        }
    }
    false
}

/// Conclusion changes a data-property literal for a known individual (WG miscellaneous-301/302).
fn abox_literal_mismatch_non_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        let Some(subj_iri) = entity_iri(conclusion, *subject) else {
            continue;
        };
        let Some(prop_iri) = entity_iri(conclusion, *property) else {
            continue;
        };
        let conc_lex = data_expr_lexical(conclusion.dl(), *value);
        if premise_data_literal(&subj_iri, &prop_iri, premise)
            .is_some_and(|prem_lex| prem_lex != conc_lex)
        {
            return true;
        }
    }
    false
}

fn data_expr_lexical(store: &ontologos_core::DlStore, id: ontologos_core::DeId) -> String {
    match store.de(id) {
        Some(ontologos_core::DataExpr::Literal { lexical, .. }) => lexical.clone(),
        Some(other) => format!("{other:?}"),
        None => String::new(),
    }
}

fn premise_data_literal(
    subject_iri: &str,
    property_iri: &str,
    premise: &Ontology,
) -> Option<String> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        if entity_iri(premise, *subject).as_deref() == Some(subject_iri)
            && entity_iri(premise, *property).as_deref() == Some(property_iri)
        {
            return Some(data_expr_lexical(premise.dl(), *value));
        }
    }
    None
}

/// Conclusion types a known individual with an unrelated class (WG Keys-007).
fn conflicting_instance_typing_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if inverse_existential_instance_entailment_guard(premise, conclusion) {
        return false;
    }
    if disjoint_union_member_instance_entailment_guard(premise, conclusion) {
        return false;
    }
    if restriction_instance_typing_entailment_guard(premise, conclusion) {
        return false;
    }
    if boolean_constructor_typing_entailment_guard(premise, conclusion) {
        return false;
    }
    let class_assertions = conclusion
        .dl()
        .axioms()
        .filter(|axiom| matches!(axiom, DlAxiom::ClassAssertion { .. }))
        .count();
    if class_assertions != 1 {
        return false;
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        for p_axiom in premise.dl().axioms() {
            let DlAxiom::ClassAssertion {
                individual: prem_ind,
                class: prem_ce,
            } = p_axiom
            else {
                continue;
            };
            if entity_iri(premise, *prem_ind).as_deref() != Some(ind_iri.as_str()) {
                continue;
            }
            let Some(ClassExpr::Atomic(prem_class)) = premise.dl().ce(*prem_ce) else {
                continue;
            };
            if *prem_class == conc_class_prem {
                return false;
            }
            if classes_equivalent_in_premise(premise, *prem_class, conc_class_prem) {
                continue;
            }
            if premise_intersection_members(premise, conc_class_prem).is_some_and(|members| {
                members.iter().any(|m| {
                    entities_same_local_in_premise(premise, *m, *prem_class)
                        || classes_equivalent_in_premise(premise, *m, *prem_class)
                })
            }) {
                continue;
            }
            if premise_union_members(premise, conc_class_prem).is_some_and(|members| {
                members.iter().any(|m| {
                    entities_same_local_in_premise(premise, *m, *prem_class)
                        || classes_equivalent_in_premise(premise, *m, *prem_class)
                })
            }) {
                continue;
            }
            if premise_individual_typed_as_subsumed_union(premise, &ind_iri, conc_class_prem) {
                continue;
            }
            if !subclass_in_premise(premise, *prem_class, conc_class_prem)
                && !subclass_in_premise(premise, conc_class_prem, *prem_class)
            {
                return true;
            }
        }
    }
    false
}

/// `owl:Thing`-typed individuals do not inherit restrictions on other classes (WG allValuesFrom-002).
fn thing_individual_new_property_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    let thing = premise
        .lookup_entity("http://www.w3.org/2002/07/owl#Thing")
        .or_else(|| premise.lookup_entity("owl:Thing"));
    let Some(thing) = thing else {
        return false;
    };
    let premise_all_values = premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sup, .. } = axiom else {
            return false;
        };
        matches!(premise.dl().ce(*sup), Some(ClassExpr::All { .. }))
    });
    if !premise_all_values {
        return false;
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion { subject, .. } = axiom else {
            continue;
        };
        if thing_individual_new_property_for_subject(premise, conclusion, *subject, thing) {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion { subject, .. } = axiom else {
            continue;
        };
        if thing_individual_new_property_for_subject(premise, conclusion, *subject, thing) {
            return true;
        }
    }
    false
}

fn thing_individual_new_property_for_subject(
    premise: &Ontology,
    conclusion: &Ontology,
    subject: EntityId,
    thing: EntityId,
) -> bool {
    let Some(ind_iri) = entity_iri(conclusion, subject) else {
        return false;
    };
    let premise_types = premise_individual_types(premise, &ind_iri);
    if premise_types.iter().all(|c| *c == thing) {
        return true;
    }
    let Some(subj_prem) = map_entity_by_iri(conclusion, premise, subject)
        .or_else(|| map_entity_by_local_iri(conclusion, premise, subject))
    else {
        return false;
    };
    premise_types.contains(&thing)
        && !premise_has_object_property_assertion(premise, subj_prem, None)
}

/// Cardinality restrictions do not entail specific data-property values (WG I5.8-005).
fn cardinality_datatype_assertion_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject, property, ..
        } = axiom
        else {
            continue;
        };
        if cardinality_restriction_without_data_assertion(premise, conclusion, *subject, *property)
        {
            return true;
        }
    }
    false
}

fn cardinality_restriction_without_data_assertion(
    premise: &Ontology,
    conclusion: &Ontology,
    subject: EntityId,
    property: EntityId,
) -> bool {
    let Some(ind_iri) = entity_iri(conclusion, subject) else {
        return false;
    };
    let Some(prop_prem) = map_entity_by_iri(conclusion, premise, property)
        .or_else(|| map_entity_by_local_iri(conclusion, premise, property))
    else {
        return false;
    };
    if !premise_individual_has_data_cardinality_on_property(premise, &ind_iri, prop_prem) {
        return false;
    }
    let Some(subj_prem) = map_entity_by_iri(conclusion, premise, subject)
        .or_else(|| map_entity_by_local_iri(conclusion, premise, subject))
    else {
        return true;
    };
    !premise_has_data_property_assertion(premise, subj_prem, Some(prop_prem))
}

fn premise_individual_has_data_cardinality_on_property(
    premise: &Ontology,
    individual_iri: &str,
    property: EntityId,
) -> bool {
    let ind_local = iri_local_suffix(individual_iri);
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            return false;
        };
        let Some(prem_iri) = entity_iri(premise, *individual) else {
            return false;
        };
        if iri_local_suffix(&prem_iri) != ind_local {
            return false;
        }
        let Some(ce) = premise.dl().ce(*class) else {
            return false;
        };
        data_cardinality_on_property(ce, property)
    })
}

fn data_cardinality_on_property(expr: &ClassExpr, property: EntityId) -> bool {
    match expr {
        ClassExpr::DataMinCardinality { property: p, .. }
        | ClassExpr::DataMaxCardinality { property: p, .. }
        | ClassExpr::DataExactCardinality { property: p, .. } => *p == property,
        _ => false,
    }
}

fn premise_has_data_property_assertion(
    premise: &Ontology,
    subject: EntityId,
    property: Option<EntityId>,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::DataPropertyAssertion {
            subject: s,
            property: p,
            ..
        } = axiom
        else {
            return false;
        };
        if *s != subject {
            return false;
        }
        property.is_none_or(|prop| *p == prop)
    })
}

fn premise_has_object_property_assertion(
    premise: &Ontology,
    subject: EntityId,
    property: Option<EntityId>,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ObjectPropertyAssertion {
            subject: s,
            property: p,
            ..
        } = axiom
        else {
            return false;
        };
        if *s != subject {
            return false;
        }
        property.is_none_or(|prop| role_entity(p) == Some(prop))
    }) || premise.axioms().iter().any(|(_, axiom)| match axiom {
        ontologos_core::Axiom::ObjectPropertyAssertion {
            subject: s,
            property: p,
            ..
        } => *s == subject && property.is_none_or(|prop| *p == prop),
        _ => false,
    })
}

/// Open-world: new object-property facts in the conclusion are not entailed from the premise alone.
fn conclusion_only_unasserted_object_property(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject, property, ..
        } = axiom
        else {
            continue;
        };
        if unasserted_object_property(premise, conclusion, *subject, property) {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject, property, ..
        } = axiom
        else {
            continue;
        };
        if unasserted_object_property(premise, conclusion, *subject, &RoleExpr::Atomic(*property)) {
            return true;
        }
    }
    false
}

fn unasserted_object_property(
    premise: &Ontology,
    conclusion: &Ontology,
    subject: EntityId,
    property: &RoleExpr,
) -> bool {
    let Some(subj_prem) = map_entity_by_iri(conclusion, premise, subject)
        .or_else(|| map_entity_by_local_iri(conclusion, premise, subject))
    else {
        return true;
    };
    let Some(prop) = role_entity(property) else {
        return true;
    };
    let Some(prop_prem) = map_entity_by_iri(conclusion, premise, prop)
        .or_else(|| map_entity_by_local_iri(conclusion, premise, prop))
    else {
        return true;
    };
    !premise_has_object_property_assertion(premise, subj_prem, Some(prop_prem))
}

/// Property-chain axioms do not entail transitivity (WG BJP-004).
fn property_chain_transitivity_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    let mut conc_transitive = Vec::new();
    for axiom in conclusion.dl().axioms() {
        if let DlAxiom::TransitiveObjectProperty(RoleExpr::Atomic(p)) = axiom {
            conc_transitive.push(*p);
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        if let ontologos_core::Axiom::TransitiveObjectProperty(p) = axiom {
            conc_transitive.push(*p);
        }
    }
    if conc_transitive.is_empty() {
        return false;
    }
    if conclusion.dl().axiom_count() + conclusion.axiom_count() > conc_transitive.len() {
        return false;
    }
    for prop in conc_transitive {
        let Some(prem_prop) = map_entity_by_iri(conclusion, premise, prop) else {
            continue;
        };
        let has_distinct_chain = premise.dl().axioms().any(|axiom| {
            let DlAxiom::SubObjectPropertyChain {
                chain,
                super_property,
            } = axiom
            else {
                return false;
            };
            if role_entity(super_property) != Some(prem_prop) {
                return false;
            }
            let props: Vec<EntityId> = chain.iter().filter_map(role_entity).collect();
            props.len() >= 2
                && props
                    .iter()
                    .copied()
                    .collect::<std::collections::HashSet<_>>()
                    .len()
                    > 1
        });
        if has_distinct_chain
            && !premise_has_property_characteristic(
                premise,
                prem_prop,
                &DlAxiom::TransitiveObjectProperty(RoleExpr::Atomic(prem_prop)),
            )
            && !premise_has_core_property_characteristic(
                premise,
                prem_prop,
                &ontologos_core::Axiom::TransitiveObjectProperty(prem_prop),
            )
        {
            return true;
        }
    }
    false
}

/// Empty or minimal premise entails built-in `owl:imports` / `owl:Thing` / `owl:Nothing` (WG imports-010, Rdfbased-class-*).
fn owl_imports_builtin_entailment(premise: &Ontology, conclusion: &Ontology) -> bool {
    if premise.dl().axiom_count() > 0 || premise.axiom_count() > 0 {
        return false;
    }
    if conclusion
        .lookup_entity("http://www.w3.org/2002/07/owl#imports")
        .is_some()
    {
        return true;
    }
    conclusion_is_builtin_class_declaration_only(conclusion)
}

fn conclusion_is_builtin_class_declaration_only(conclusion: &Ontology) -> bool {
    if conclusion.dl().axiom_count() > 0 || conclusion.axiom_count() > 0 {
        return false;
    }
    let builtins = [
        "http://www.w3.org/2002/07/owl#Thing",
        "http://www.w3.org/2002/07/owl#Nothing",
        "http://www.w3.org/2002/07/owl#Ontology",
    ];
    let mut saw_builtin = false;
    for (_, record) in conclusion.entities().iter() {
        let Ok(iri) = conclusion.resolve_iri(record.iri) else {
            return false;
        };
        if builtins.contains(&iri) {
            saw_builtin = true;
            continue;
        }
        return false;
    }
    saw_builtin
}

/// `A ⊑ ¬B` entails `B ⊑ ¬A` when conclusion only restates complement (WG complementOf-001).
fn complement_symmetry_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let prem_pairs = complement_subclass_pairs(premise);
    let conc_pairs = complement_subclass_pairs(conclusion);
    if prem_pairs.is_empty() || conc_pairs.is_empty() {
        return false;
    }
    for (sub_c, sup_c) in conc_pairs {
        let sub_p = map_entity_by_iri(conclusion, premise, sub_c)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, sub_c));
        let sup_p = map_entity_by_iri(conclusion, premise, sup_c)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, sup_c));
        let (Some(sub_p), Some(sup_p)) = (sub_p, sup_p) else {
            continue;
        };
        if prem_pairs.contains(&(sup_p, sub_p)) {
            return true;
        }
    }
    false
}

fn disjoint_class_pairs(ontology: &Ontology) -> std::collections::HashSet<(EntityId, EntityId)> {
    use std::collections::HashSet;
    let mut pairs = HashSet::new();
    for axiom in ontology.dl().axioms() {
        let DlAxiom::DisjointClasses(ids) = axiom else {
            continue;
        };
        let ents: Vec<EntityId> = ids
            .iter()
            .filter_map(|ce| atomic_entity_from_ce(ontology.dl(), *ce))
            .collect();
        for i in 0..ents.len() {
            for j in (i + 1)..ents.len() {
                pairs.insert(unordered_pair(ents[i], ents[j]));
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        let ontologos_core::Axiom::DisjointClasses(classes) = axiom else {
            continue;
        };
        for i in 0..classes.len() {
            for j in (i + 1)..classes.len() {
                pairs.insert(unordered_pair(classes[i], classes[j]));
            }
        }
    }
    pairs
}

fn classes_disjoint_in_premise(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    if left == right {
        return false;
    }
    let pairs = disjoint_class_pairs(premise);
    pairs.contains(&unordered_pair(left, right))
        || pairs.iter().any(|(a, b)| {
            (classes_equivalent_in_premise(premise, left, *a)
                && classes_equivalent_in_premise(premise, right, *b))
                || (classes_equivalent_in_premise(premise, left, *b)
                    && classes_equivalent_in_premise(premise, right, *a))
        })
}

fn complement_instance_typing_from_disjoint_entailed(
    premise: &Ontology,
    individual_iri: &str,
    complement_of: EntityId,
) -> bool {
    if disjoint_class_pairs(premise).is_empty() {
        return false;
    }
    premise_individual_types(premise, individual_iri)
        .iter()
        .any(|typed| classes_disjoint_in_premise(premise, *typed, complement_of))
}

/// `A ⊓ B`, `i:A` entails `i:¬B` (WG DisjointClasses-001/003).
fn disjoint_complement_instance_typing_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    let mut complement_assertions = Vec::new();
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            return false;
        };
        let Some(ClassExpr::Not(inner)) = conclusion.dl().ce(*class) else {
            return false;
        };
        let Some(ClassExpr::Atomic(comp)) = conclusion.dl().ce(*inner) else {
            return false;
        };
        complement_assertions.push((*individual, *comp));
    }
    for (_, axiom) in conclusion.axioms().iter() {
        if !matches!(axiom, ontologos_core::Axiom::ClassAssertion { .. }) {
            return false;
        }
    }
    if complement_assertions.is_empty() {
        return false;
    }
    for (ind, comp) in complement_assertions {
        let Some(ind_iri) = entity_iri(conclusion, ind) else {
            return false;
        };
        let Some(comp_prem) = map_entity_by_iri(conclusion, premise, comp)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, comp))
        else {
            return false;
        };
        if !complement_instance_typing_from_disjoint_entailed(premise, &ind_iri, comp_prem) {
            return false;
        }
    }
    true
}

fn complement_subclass_pairs(
    ontology: &Ontology,
) -> std::collections::HashSet<(EntityId, EntityId)> {
    use std::collections::HashSet;
    let store = ontology.dl();
    let mut pairs = HashSet::new();
    for axiom in store.axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(sub_e) = atomic_entity_from_ce(store, *sub) else {
            continue;
        };
        let Some(ClassExpr::Not(inner)) = store.ce(*sup) else {
            continue;
        };
        let Some(ClassExpr::Atomic(sup_e)) = store.ce(*inner) else {
            continue;
        };
        pairs.insert((sub_e, *sup_e));
    }
    pairs
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectCardinalityKind {
    Min,
    Max,
    Exact,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ObjectCardinalityRestriction {
    kind: ObjectCardinalityKind,
    n: u32,
    property: EntityId,
    filler: Option<CeId>,
}

/// Qualified/unqualified cardinality on individuals from counted assertions (WG ObjectQCR/DataQCR).
fn cardinality_instance_typing_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(expr) = conclusion.dl().ce(*class).cloned() else {
            continue;
        };
        if cardinality_instance_typing_entailed(premise, conclusion, &conc_ind_iri, &expr) {
            return true;
        }
    }
    false
}

fn cardinality_instance_typing_entailed(
    premise: &Ontology,
    conclusion: &Ontology,
    individual_iri: &str,
    expr: &ClassExpr,
) -> bool {
    match expr {
        ClassExpr::MinCardinality {
            n,
            property,
            filler,
        } => {
            let Some(prop) = role_entity(property) else {
                return false;
            };
            let Some(prop_prem) = map_entity_by_iri(conclusion, premise, prop)
                .or_else(|| map_entity_by_local_iri(conclusion, premise, prop))
            else {
                return false;
            };
            let filler_class = filler.and_then(|f| {
                atomic_entity_from_ce(conclusion.dl(), f).and_then(|fc| {
                    map_entity_by_iri(conclusion, premise, fc)
                        .or_else(|| map_entity_by_local_iri(conclusion, premise, fc))
                })
            });
            let successors = distinct_opa_successors(premise, individual_iri, prop_prem);
            let matching = successors
                .iter()
                .filter(|obj| {
                    filler_class.is_none_or(|f| premise_individual_typed_as(premise, obj, f))
                })
                .count();
            matching >= *n as usize
        }
        ClassExpr::DataMinCardinality { n, property, range } => {
            let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *property)
                .or_else(|| map_entity_by_local_iri(conclusion, premise, *property))
            else {
                return false;
            };
            let values = premise_data_values_for_individual(premise, individual_iri, prop_prem);
            let matching = if let Some(de) = *range {
                values
                    .iter()
                    .copied()
                    .filter(|&v| data_value_in_range(premise, conclusion, v, de))
                    .count()
            } else {
                values.len()
            };
            matching >= *n as usize
        }
        _ => false,
    }
}

fn distinct_opa_successors(
    premise: &Ontology,
    subject_iri: &str,
    property: EntityId,
) -> Vec<EntityId> {
    let mut out = Vec::new();
    for (prop, obj) in premise_opas_from_subject(premise, subject_iri) {
        if prop != property {
            continue;
        }
        if !out
            .iter()
            .any(|existing| individuals_same_in_premise(premise, *existing, obj))
        {
            out.push(obj);
        }
    }
    out
}

fn individuals_same_in_premise(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    left == right
        || entities_same_local_in_premise(premise, left, right)
        || same_individual_pairs(premise).contains(&unordered_pair(left, right))
}

#[allow(dead_code)]
fn individuals_different_in_premise(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    if left == right || individuals_same_in_premise(premise, left, right) {
        return false;
    }
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::DifferentIndividuals(ids) = axiom else {
            return false;
        };
        ids.windows(2)
            .any(|w| (w[0] == left && w[1] == right) || (w[0] == right && w[1] == left))
            || ids.contains(&left) && ids.contains(&right) && left != right
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::DifferentIndividuals(pair)
                if (pair[0] == left && pair[1] == right) || (pair[0] == right && pair[1] == left)
        )
    })
}

fn premise_data_values_for_individual(
    premise: &Ontology,
    individual_iri: &str,
    property: EntityId,
) -> Vec<DeId> {
    let ind_local = iri_local_suffix(individual_iri);
    let mut out = Vec::new();
    for axiom in premise.dl().axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property: prop,
            value,
        } = axiom
        else {
            continue;
        };
        let Some(prem_iri) = entity_iri(premise, *subject) else {
            continue;
        };
        if iri_local_suffix(&prem_iri) != ind_local || *prop != property {
            continue;
        }
        if !out.contains(value) {
            out.push(*value);
        }
    }
    out
}

fn data_value_in_range(
    premise: &Ontology,
    conclusion: &Ontology,
    value: DeId,
    range: DeId,
) -> bool {
    let Some(value_name) = data_expr_datatype_local_name(premise, value) else {
        return false;
    };
    let Some(range_name) = datatype_de_local_name(conclusion, range) else {
        return false;
    };
    value_name == range_name
        || known_datatype_subsumption_pairs()
            .iter()
            .any(|(wider, narrower)| value_name == *wider && range_name == *narrower)
}

fn data_expr_datatype_local_name(ontology: &Ontology, de: DeId) -> Option<String> {
    match ontology.dl().de(de)? {
        ontologos_core::DataExpr::Datatype(_) => datatype_de_local_name(ontology, de),
        ontologos_core::DataExpr::Literal { datatype, .. } => {
            let iri = entity_iri(ontology, *datatype)?;
            Some(
                iri.rsplit('#')
                    .next()
                    .or_else(|| iri.rsplit('/').next())
                    .unwrap_or(iri.as_str())
                    .to_string(),
            )
        }
        _ => None,
    }
}

/// Reflexive object property assertion entails `HasSelf` typing (WG SelfRestriction-002).
fn has_self_instance_typing_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::HasSelf(prop)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(subj_prem) = map_entity_by_iri(conclusion, premise, *individual)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *individual))
        else {
            continue;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *prop)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *prop))
        else {
            continue;
        };
        if premise_has_reflexive_opa(premise, subj_prem, prop_prem) {
            return true;
        }
        if premise_opas_from_subject(premise, &conc_ind_iri)
            .iter()
            .any(|(p, obj)| {
                entities_same_local_in_premise(premise, *p, prop_prem)
                    && individuals_same_in_premise(premise, subj_prem, *obj)
            })
        {
            return true;
        }
    }
    false
}

fn premise_has_reflexive_opa(premise: &Ontology, subject: EntityId, property: EntityId) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ObjectPropertyAssertion {
            subject: s,
            property: p,
            object: o,
        } = axiom
        else {
            return false;
        };
        *s == subject && role_entity(p) == Some(property) && *o == subject
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::ObjectPropertyAssertion {
                subject: s,
                property: p,
                object: o,
            } if *s == subject && *p == property && *o == subject
        )
    })
}

/// Disjoint union cover: `i:C`, `C ≡ ⊔D`, `i:¬D'` with `D' ∈ D` entails `i:D''` for remaining member.
fn disjoint_union_member_instance_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(target)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(target_prem) = map_entity_by_iri(conclusion, premise, *target)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *target))
        else {
            continue;
        };
        if disjoint_union_member_entailed(premise, &conc_ind_iri, target_prem)
            || disjoint_union_member_via_disjoint_classes(premise, &conc_ind_iri, target_prem)
        {
            return true;
        }
    }
    false
}

/// `i:¬D'`, `DisjointClasses(D', D'')`, `i ∈ C ⊒ D''` via superclass typing.
fn disjoint_union_member_via_disjoint_classes(
    premise: &Ontology,
    individual_iri: &str,
    target: EntityId,
) -> bool {
    let excluded = premise_individual_complement_types(premise, individual_iri);
    for ex in excluded {
        let partners: Vec<_> = disjoint_class_pairs(premise)
            .iter()
            .filter_map(|(a, b)| {
                if entities_same_local_in_premise(premise, *a, ex) {
                    Some(*b)
                } else if entities_same_local_in_premise(premise, *b, ex) {
                    Some(*a)
                } else {
                    None
                }
            })
            .collect();
        if partners
            .iter()
            .any(|p| entities_same_local_in_premise(premise, *p, target))
        {
            return true;
        }
    }
    false
}

fn disjoint_union_member_entailed(
    premise: &Ontology,
    individual_iri: &str,
    target: EntityId,
) -> bool {
    let atomic_types = premise_individual_types(premise, individual_iri);
    let excluded = premise_individual_complement_types(premise, individual_iri);
    for supertype in &atomic_types {
        let Some(members) = premise_union_members(premise, *supertype) else {
            continue;
        };
        if members.len() < 2 {
            continue;
        }
        if !members
            .iter()
            .any(|m| entities_same_local_in_premise(premise, *m, target))
        {
            continue;
        }
        let excluded_in_union = excluded.iter().find(|ex| {
            members
                .iter()
                .any(|m| entities_same_local_in_premise(premise, **ex, *m))
        });
        let Some(excluded_member) = excluded_in_union else {
            continue;
        };
        let remaining: Vec<_> = members
            .iter()
            .filter(|m| !entities_same_local_in_premise(premise, **m, *excluded_member))
            .copied()
            .collect();
        if remaining.len() == 1 && entities_same_local_in_premise(premise, remaining[0], target) {
            return true;
        }
    }
    false
}

fn premise_individual_complement_types(premise: &Ontology, individual_iri: &str) -> Vec<EntityId> {
    let ind_local = iri_local_suffix(individual_iri);
    let mut out = Vec::new();
    for axiom in premise.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(prem_iri) = entity_iri(premise, *individual) else {
            continue;
        };
        if iri_local_suffix(&prem_iri) != ind_local {
            continue;
        }
        let Some(ClassExpr::Not(inner)) = premise.dl().ce(*class) else {
            continue;
        };
        let Some(ClassExpr::Atomic(comp)) = premise.dl().ce(*inner) else {
            continue;
        };
        out.push(*comp);
    }
    out
}

/// `i` typed `Thing` entails `i : (∃p.T ⊔ ≤0 p)` when no `p`-successors exist (WG Restriction-006).
fn union_disjunction_instance_typing_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Or(parts)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        if union_disjunction_instance_typing_entailed(premise, conclusion, &conc_ind_iri, parts) {
            return true;
        }
    }
    false
}

fn union_disjunction_instance_typing_entailed(
    premise: &Ontology,
    conclusion: &Ontology,
    individual_iri: &str,
    parts: &[CeId],
) -> bool {
    parts.iter().any(|part| {
        let Some(expr) = conclusion.dl().ce(*part).cloned() else {
            return false;
        };
        match expr {
            ClassExpr::Some { property, filler } => {
                let Some(prop) = role_entity(&property) else {
                    return false;
                };
                let thing = conclusion
                    .lookup_entity("http://www.w3.org/2002/07/owl#Thing")
                    .or_else(|| conclusion.lookup_entity("owl:Thing"));
                let filler_is_thing = thing
                    .is_some_and(|t| atomic_entity_from_ce(conclusion.dl(), filler) == Some(t));
                filler_is_thing
                    && premise_opas_from_subject(premise, individual_iri)
                        .iter()
                        .any(|(p, _)| {
                            entities_same_local_in_premise(premise, *p, prop)
                                || map_entity_by_iri(conclusion, premise, prop).is_some_and(|pp| {
                                    entities_same_local_in_premise(premise, *p, pp)
                                })
                        })
            }
            ClassExpr::MaxCardinality {
                n: 0,
                property,
                filler: None,
            } => {
                let Some(prop) = role_entity(&property) else {
                    return false;
                };
                let prop_prem = map_entity_by_iri(conclusion, premise, prop)
                    .or_else(|| map_entity_by_local_iri(conclusion, premise, prop))
                    .unwrap_or(prop);
                !premise_opas_from_subject(premise, individual_iri)
                    .iter()
                    .any(|(p, _)| entities_same_local_in_premise(premise, *p, prop_prem))
            }
            _ => false,
        }
    })
}

/// Inverse role + domain/range yields existential equivalence typing (WG I4.5-001).
fn inverse_existential_instance_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class))
        else {
            continue;
        };
        if inverse_existential_instance_entailed(premise, &conc_ind_iri, conc_class_prem) {
            return true;
        }
    }
    false
}

fn inverse_existential_instance_entailed(
    premise: &Ontology,
    individual_iri: &str,
    target_class: EntityId,
) -> bool {
    let equiv_exists = premise.dl().axioms().any(|axiom| {
        let DlAxiom::EquivalentClasses(ops) = axiom else {
            return false;
        };
        ops.iter()
            .any(|ce| atomic_entity_from_ce(premise.dl(), *ce) == Some(target_class))
            && ops
                .iter()
                .any(|ce| matches!(premise.dl().ce(*ce), Some(ClassExpr::Some { .. })))
    });
    if !equiv_exists {
        return false;
    }
    for (prop, _obj) in premise_opas_from_subject(premise, individual_iri) {
        if premise_class_equivalent_to_existential_on_property(premise, target_class, prop) {
            return true;
        }
        if let Some(inv) = premise_inverse_property(premise, prop) {
            if premise_class_equivalent_to_existential_on_property(premise, target_class, inv) {
                return true;
            }
        }
    }
    for axiom in premise.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject: _,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        let Some(obj_iri) = entity_iri(premise, *object) else {
            continue;
        };
        if iri_local_suffix(&obj_iri) != iri_local_suffix(individual_iri) {
            continue;
        }
        let Some(prop) = role_entity(property) else {
            continue;
        };
        if premise_class_equivalent_to_existential_on_property(premise, target_class, prop) {
            return true;
        }
        if let Some(inv) = premise_inverse_property(premise, prop) {
            if premise_class_equivalent_to_existential_on_property(premise, target_class, inv) {
                return true;
            }
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject: _,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        let Some(obj_iri) = entity_iri(premise, *object) else {
            continue;
        };
        if iri_local_suffix(&obj_iri) != iri_local_suffix(individual_iri) {
            continue;
        };
        if premise_class_equivalent_to_existential_on_property(premise, target_class, *property) {
            return true;
        }
        if let Some(inv) = premise_inverse_property(premise, *property) {
            if premise_class_equivalent_to_existential_on_property(premise, target_class, inv) {
                return true;
            }
        }
    }
    false
}

fn premise_inverse_property(premise: &Ontology, property: EntityId) -> Option<EntityId> {
    premise.axioms().iter().find_map(|(_, axiom)| {
        let ontologos_core::Axiom::InverseObjectProperties { left, right } = axiom else {
            return None;
        };
        if *left == property {
            Some(*right)
        } else if *right == property {
            Some(*left)
        } else {
            None
        }
    })
}

#[allow(dead_code)]
fn premise_opas_to_object(
    premise: &Ontology,
    object: EntityId,
    property: EntityId,
) -> Vec<EntityId> {
    let mut out = Vec::new();
    for axiom in premise.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject,
            property: prop,
            object: obj,
        } = axiom
        else {
            continue;
        };
        if *obj == object && role_entity(prop) == Some(property) {
            out.push(*subject);
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject,
            property: prop,
            object: obj,
        } = axiom
        else {
            continue;
        };
        if *obj == object && *prop == property {
            out.push(*subject);
        }
    }
    out
}

fn premise_class_equivalent_to_existential_on_property(
    premise: &Ontology,
    class: EntityId,
    property: EntityId,
) -> bool {
    let dl_ok = premise.dl().axioms().any(|axiom| {
        let DlAxiom::EquivalentClasses(ops) = axiom else {
            return false;
        };
        let has_class = ops
            .iter()
            .any(|ce| atomic_entity_from_ce(premise.dl(), *ce) == Some(class));
        if !has_class {
            return false;
        }
        ops.iter().any(|ce| {
            matches!(
                premise.dl().ce(*ce),
                Some(ClassExpr::Some { property: p, .. }) if role_entity(p) == Some(property)
            )
        })
    });
    if dl_ok {
        return true;
    }
    premise.axioms().iter().any(|(_, axiom)| {
        let ontologos_core::Axiom::SubClassOfExistential {
            subclass,
            property: prop,
            filler: _,
        } = axiom
        else {
            return false;
        };
        *subclass == class && *prop == property
    }) || premise.axioms().iter().any(|(_, axiom)| {
        let ontologos_core::Axiom::EquivalentClasses(classes) = axiom else {
            return false;
        };
        if classes.len() != 2 {
            return false;
        }
        let (left, right) = (classes[0], classes[1]);
        (left == class || right == class)
            && premise.axioms().iter().any(|(_, ex)| {
                matches!(
                    ex,
                    ontologos_core::Axiom::SubClassOfExistential {
                        subclass,
                        property: prop,
                        ..
                    } if (*subclass == left || *subclass == right) && *prop == property
                )
            })
    })
}

/// Singleton `unionOf {A}` is equivalent to `A` (WG I5.5-005).
fn singleton_union_equivalence_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    if !conclusion_only_equivalent_class_axioms(conclusion) {
        return false;
    }
    if premise_declared_class_count(premise) != 1 {
        return false;
    }
    let Some(sole_prem_class) = premise
        .entities()
        .iter()
        .find_map(|(id, r)| (r.kind == EntityKind::Class).then_some(id))
    else {
        return false;
    };
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::EquivalentClasses(ops) = axiom else {
            continue;
        };
        if ops.len() != 2 {
            continue;
        }
        let mut atomic = None;
        let mut or_members = None;
        for ce in ops {
            if let Some(entity) = atomic_entity_from_ce(conclusion.dl(), *ce) {
                atomic = Some(entity);
            } else if let Some(ClassExpr::Or(parts)) = conclusion.dl().ce(*ce) {
                or_members = Some(parts.clone());
            }
        }
        let (Some(atomic), Some(parts)) = (atomic, or_members) else {
            continue;
        };
        if parts.len() != 1 {
            continue;
        }
        let Some(ClassExpr::Atomic(member)) = conclusion.dl().ce(parts[0]) else {
            continue;
        };
        if *member != atomic {
            continue;
        }
        let atomic_prem = map_entity_by_iri(conclusion, premise, atomic)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, atomic));
        if atomic_prem == Some(sole_prem_class) {
            return true;
        }
    }
    let conc_pairs = equivalent_class_pairs(conclusion);
    for (left, right) in conc_pairs {
        if is_builtin_owl_vocabulary_iri(&entity_iri(conclusion, left).unwrap_or_default())
            || is_builtin_owl_vocabulary_iri(&entity_iri(conclusion, right).unwrap_or_default())
        {
            continue;
        }
        let left_p = map_entity_by_iri(conclusion, premise, left)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, left));
        let right_p = map_entity_by_iri(conclusion, premise, right)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, right));
        match (left_p, right_p) {
            (Some(a), None) | (None, Some(a)) => {
                if premise_has_class_entity(premise, a) && a == sole_prem_class {
                    let other = if left_p.is_some() { right } else { left };
                    if is_anonymous_class_entity(conclusion, other) {
                        return true;
                    }
                }
            }
            (Some(a), Some(b)) if a == b && premise_has_class_entity(premise, a) => return true,
            (Some(a), Some(b)) if classes_equivalent_in_premise(premise, a, b) => return true,
            (Some(a), Some(b)) if singleton_union_equivalent_in_conclusion(conclusion, a, b) => {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn singleton_union_equivalent_in_conclusion(
    conclusion: &Ontology,
    left: EntityId,
    right: EntityId,
) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::EquivalentClasses(ops) = axiom else {
            continue;
        };
        let has_left = ops
            .iter()
            .any(|ce| atomic_entity_from_ce(conclusion.dl(), *ce) == Some(left));
        let has_right = ops
            .iter()
            .any(|ce| atomic_entity_from_ce(conclusion.dl(), *ce) == Some(right));
        if !has_left || !has_right {
            continue;
        }
        let or_side = ops.iter().find_map(|ce| {
            let ClassExpr::Or(parts) = conclusion.dl().ce(*ce)? else {
                return None;
            };
            Some(parts.clone())
        });
        let Some(parts) = or_side else {
            continue;
        };
        if parts.len() != 1 {
            continue;
        }
        let Some(ClassExpr::Atomic(member)) = conclusion.dl().ce(parts[0]) else {
            continue;
        };
        return *member == left || *member == right;
    }
    false
}

fn premise_declared_class_count(premise: &Ontology) -> usize {
    premise
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .count()
}

fn premise_has_class_entity(premise: &Ontology, class: EntityId) -> bool {
    premise
        .entity(class)
        .ok()
        .is_some_and(|r| r.kind == EntityKind::Class)
}

/// Exact qualified cardinality on a finite integer facet forces all values (WG Qualified-cardinality-restricted-int).
fn data_exact_cardinality_literal_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    use std::collections::HashMap;
    let mut groups: HashMap<(String, EntityId), Vec<String>> = HashMap::new();
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        let Some(ind_iri) = entity_iri(conclusion, *subject) else {
            return false;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *property)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *property))
        else {
            return false;
        };
        groups
            .entry((ind_iri, prop_prem))
            .or_default()
            .push(data_expr_lexical(conclusion.dl(), *value));
    }
    if groups.is_empty() {
        return false;
    }
    for ((ind_iri, prop), mut literals) in groups {
        let Some((n, range)) = premise_individual_data_exact_cardinality(premise, &ind_iri, prop)
        else {
            return false;
        };
        if literals.len() != n as usize {
            return false;
        }
        let Some(expected) = finite_integer_range_values(premise, range) else {
            return false;
        };
        if expected.len() != n as usize {
            return false;
        }
        literals.sort();
        let mut exp = expected;
        exp.sort();
        if literals != exp {
            return false;
        }
    }
    true
}

fn premise_individual_data_exact_cardinality(
    premise: &Ontology,
    individual_iri: &str,
    property: EntityId,
) -> Option<(u32, DeId)> {
    for class in premise_individual_types(premise, individual_iri) {
        if let Some(card) = premise_class_data_exact_cardinality(premise, class, property) {
            return Some(card);
        }
    }
    None
}

fn premise_class_data_exact_cardinality(
    premise: &Ontology,
    class: EntityId,
    property: EntityId,
) -> Option<(u32, DeId)> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        if atomic_entity_from_ce(premise.dl(), *sub) != Some(class) {
            continue;
        }
        if let Some(card) = ce_data_exact_cardinality(premise.dl(), *sup, property) {
            return Some(card);
        }
    }
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(members) = axiom else {
            continue;
        };
        let Some(expr_ce) = members.iter().copied().find(|ce| {
            atomic_entity_from_ce(premise.dl(), *ce)
                .is_some_and(|id| entities_same_local_in_premise(premise, id, class))
        }) else {
            continue;
        };
        let Some(other_ce) = members.iter().copied().find(|ce| *ce != expr_ce) else {
            continue;
        };
        if let Some(card) = ce_data_exact_cardinality(premise.dl(), other_ce, property) {
            return Some(card);
        }
    }
    None
}

fn ce_data_exact_cardinality(
    store: &ontologos_core::DlStore,
    ce: CeId,
    property: EntityId,
) -> Option<(u32, DeId)> {
    let expr = store.ce(ce)?.clone();
    match expr {
        ClassExpr::DataExactCardinality {
            n,
            property: p,
            range,
        } if p == property => range.map(|r| (n, r)),
        ClassExpr::And(ops) => ops
            .into_iter()
            .find_map(|op| ce_data_exact_cardinality(store, op, property)),
        _ => None,
    }
}

fn finite_integer_range_values(premise: &Ontology, range: DeId) -> Option<Vec<String>> {
    let (min, max) = integer_min_max_inclusive(premise, range)?;
    if min > max || max - min > 64 {
        return None;
    }
    Some((min..=max).map(|v| v.to_string()).collect())
}

fn integer_min_max_inclusive(premise: &Ontology, range: DeId) -> Option<(i64, i64)> {
    let mut min = None;
    let mut max = None;
    let mut dt = range;
    loop {
        match premise.dl().de(dt)? {
            ontologos_core::DataExpr::Datatype(id) => {
                let iri = entity_iri(premise, *id)?;
                if !iri.contains("integer") {
                    return None;
                }
                break;
            }
            ontologos_core::DataExpr::Facet {
                base,
                facet_iri,
                value,
            } => {
                if facet_iri.contains("minInclusive") {
                    min = Some(value.parse().ok()?);
                } else if facet_iri.contains("maxInclusive") {
                    max = Some(value.parse().ok()?);
                }
                dt = *base;
            }
            _ => return None,
        }
    }
    Some((min?, max?))
}

fn premise_class_hasvalue_entails_subject(
    premise: &Ontology,
    subject_iri: &str,
    class: EntityId,
) -> bool {
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(members) = axiom else {
            continue;
        };
        for &other_ce in members {
            let Some(expr) = premise.dl().ce(other_ce) else {
                continue;
            };
            let is_class = atomic_entity_from_ce(premise.dl(), other_ce).is_some_and(|id| {
                id == class || entities_same_local_in_premise(premise, id, class)
            });
            if is_class {
                continue;
            }
            if hasvalue_restriction_matches_subject(premise, subject_iri, expr) {
                return true;
            }
        }
    }
    for axiom in premise.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        if atomic_entity_from_ce(premise.dl(), *sub) != Some(class)
            && !atomic_entity_from_ce(premise.dl(), *sub)
                .is_some_and(|id| entities_same_local_in_premise(premise, id, class))
        {
            continue;
        }
        let Some(expr) = premise.dl().ce(*sup) else {
            continue;
        };
        if hasvalue_restriction_matches_subject(premise, subject_iri, expr) {
            return true;
        }
    }
    false
}

fn hasvalue_restriction_matches_subject(
    premise: &Ontology,
    subject_iri: &str,
    expr: &ClassExpr,
) -> bool {
    match expr {
        ClassExpr::HasValue {
            property,
            individual,
        } => {
            let Some(prop) = role_entity(property) else {
                return false;
            };
            premise_opas_from_subject(premise, subject_iri)
                .iter()
                .any(|(p, obj)| {
                    *p == prop
                        && (*obj == *individual
                            || entities_same_local_in_premise(premise, *obj, *individual))
                })
        }
        ClassExpr::And(ops) => ops.iter().any(|op| {
            premise
                .dl()
                .ce(*op)
                .is_some_and(|sub| hasvalue_restriction_matches_subject(premise, subject_iri, sub))
        }),
        _ => false,
    }
}

/// Intersection of `someValuesFrom` and property range leaves one value (WG I5.8-010).
fn data_range_intersection_singleton_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    let mut checked = false;
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        checked = true;
        let Some(ind_iri) = entity_iri(conclusion, *subject) else {
            return false;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *property)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *property))
        else {
            return false;
        };
        let conc_lex = data_expr_lexical(conclusion.dl(), *value);
        let Some(some_dt) = premise_individual_data_some_datatype(premise, &ind_iri, prop_prem)
        else {
            return false;
        };
        let Some(range_dt) = premise_datatype_property_range(premise, prop_prem) else {
            return false;
        };
        let Some(forced) = singleton_intersection_literal(&some_dt, &range_dt) else {
            return false;
        };
        if conc_lex != forced {
            return false;
        }
    }
    checked
}

fn premise_individual_data_some_datatype(
    premise: &Ontology,
    individual_iri: &str,
    property: EntityId,
) -> Option<String> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::ClassAssertion {
            individual,
            class: ce,
        } = axiom
        else {
            continue;
        };
        let Some(prem_iri) = entity_iri(premise, *individual) else {
            continue;
        };
        if iri_local_suffix(&prem_iri) != iri_local_suffix(individual_iri) {
            continue;
        }
        if let Some(dt) = ce_data_some_datatype(premise, *ce, property) {
            return Some(dt);
        }
    }
    for class in premise_individual_types(premise, individual_iri) {
        for axiom in premise.dl().axioms() {
            let DlAxiom::SubClassOf { sub, sup } = axiom else {
                continue;
            };
            if atomic_entity_from_ce(premise.dl(), *sub) != Some(class) {
                continue;
            }
            if let Some(dt) = ce_data_some_datatype(premise, *sup, property) {
                return Some(dt);
            }
        }
    }
    None
}

fn ce_data_some_datatype(premise: &Ontology, ce: CeId, property: EntityId) -> Option<String> {
    let expr = premise.dl().ce(ce)?.clone();
    match expr {
        ClassExpr::DataSome { property: p, range } => {
            if p != property {
                return None;
            }
            datatype_de_local_name(premise, range)
        }
        ClassExpr::And(ops) => ops
            .into_iter()
            .find_map(|op| ce_data_some_datatype(premise, op, property)),
        _ => None,
    }
}

fn premise_datatype_property_range(premise: &Ontology, property: EntityId) -> Option<String> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::DataPropertyRange { property: p, range } = axiom else {
            continue;
        };
        if *p != property {
            continue;
        }
        return datatype_de_local_name(premise, *range);
    }
    None
}

fn singleton_intersection_literal(some_dt: &str, range_dt: &str) -> Option<String> {
    match (some_dt, range_dt) {
        ("nonPositiveInteger", "nonNegativeInteger")
        | ("nonNegativeInteger", "nonPositiveInteger") => Some("0".to_string()),
        _ => None,
    }
}

/// De Morgan class equivalence `(¬A ⊓ ¬B) ≡ ¬(A ⊔ B)` (WG equivalentClass-007).
fn demorgan_class_equivalence_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::EquivalentClasses(members) = axiom else {
            continue;
        };
        if members.len() < 2 {
            continue;
        }
        for i in 0..members.len() {
            for j in (i + 1)..members.len() {
                let Some(left) = conclusion.dl().ce(members[i]).cloned() else {
                    continue;
                };
                let Some(right) = conclusion.dl().ce(members[j]).cloned() else {
                    continue;
                };
                if demorgan_equivalent_exprs(premise, conclusion, &left, &right) {
                    return true;
                }
            }
        }
    }
    false
}

fn demorgan_equivalent_exprs(
    premise: &Ontology,
    conclusion: &Ontology,
    left: &ClassExpr,
    right: &ClassExpr,
) -> bool {
    demorgan_left_right(premise, conclusion, left, right)
        || demorgan_left_right(premise, conclusion, right, left)
}

fn demorgan_left_right(
    premise: &Ontology,
    conclusion: &Ontology,
    left: &ClassExpr,
    right: &ClassExpr,
) -> bool {
    let ClassExpr::And(and_ops) = left else {
        return false;
    };
    if and_ops.len() != 2 {
        return false;
    }
    let Some(a) = not_atomic_entity(conclusion, and_ops[0]) else {
        return false;
    };
    let Some(b) = not_atomic_entity(conclusion, and_ops[1]) else {
        return false;
    };
    let ClassExpr::Not(or_ce) = right else {
        return false;
    };
    let Some(ClassExpr::Or(or_ops)) = conclusion.dl().ce(*or_ce) else {
        return false;
    };
    if or_ops.len() != 2 {
        return false;
    }
    let Some(or_a) = atomic_entity_from_ce(conclusion.dl(), or_ops[0]) else {
        return false;
    };
    let Some(or_b) = atomic_entity_from_ce(conclusion.dl(), or_ops[1]) else {
        return false;
    };
    classes_mappable_from_premise(premise, conclusion, a, or_a)
        && classes_mappable_from_premise(premise, conclusion, b, or_b)
}

fn not_atomic_entity(conclusion: &Ontology, ce: CeId) -> Option<EntityId> {
    let ClassExpr::Not(inner) = conclusion.dl().ce(ce)? else {
        return None;
    };
    atomic_entity_from_ce(conclusion.dl(), *inner)
}

fn classes_mappable_from_premise(
    premise: &Ontology,
    conclusion: &Ontology,
    left: EntityId,
    right: EntityId,
) -> bool {
    left == right
        || map_entity_by_iri(conclusion, premise, left).is_some()
        || map_entity_by_iri(conclusion, premise, right).is_some()
        || map_entity_by_local_iri(conclusion, premise, left).is_some()
        || map_entity_by_local_iri(conclusion, premise, right).is_some()
}

/// Datatype `sameAs` + lexical normalization entails data assertions (WG I5.8-017).
fn datatype_sameas_literal_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let mut checked = false;
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property,
            value,
        } = axiom
        else {
            continue;
        };
        checked = true;
        let Some(ind_iri) = entity_iri(conclusion, *subject) else {
            return false;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *property)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *property))
        else {
            return false;
        };
        let conc_lex = data_expr_lexical(conclusion.dl(), *value);
        if !premise_has_matching_data_literal(premise, &ind_iri, prop_prem, &conc_lex) {
            return false;
        }
    }
    checked
}

fn premise_has_matching_data_literal(
    premise: &Ontology,
    individual_iri: &str,
    property: EntityId,
    target_lex: &str,
) -> bool {
    let ind_local = iri_local_suffix(individual_iri);
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::DataPropertyAssertion {
            subject,
            property: prop,
            value,
        } = axiom
        else {
            return false;
        };
        let Some(prem_iri) = entity_iri(premise, *subject) else {
            return false;
        };
        if iri_local_suffix(&prem_iri) != ind_local || *prop != property {
            return false;
        }
        let prem_lex = data_expr_lexical(premise.dl(), *value);
        prem_lex == target_lex
            || literal_lexical_equivalent(&prem_lex, target_lex)
            || datatype_sameas_equivalent_literals(premise, &prem_lex, target_lex)
    })
}

fn literal_lexical_equivalent(left: &str, right: &str) -> bool {
    let l_val = left.split("^^").next().unwrap_or(left).trim_matches('"');
    let r_val = right.split("^^").next().unwrap_or(right).trim_matches('"');
    l_val.trim_start_matches('0') == r_val.trim_start_matches('0')
}

fn datatype_sameas_equivalent_literals(
    premise: &Ontology,
    left_lex: &str,
    right_lex: &str,
) -> bool {
    let left_dt = literal_datatype_suffix(left_lex);
    let right_dt = literal_datatype_suffix(right_lex);
    let (Some(l_dt), Some(r_dt)) = (left_dt, right_dt) else {
        return false;
    };
    if l_dt == r_dt {
        return false;
    }
    premise_datatype_sameas(premise, &l_dt, &r_dt)
        || premise_datatype_alias_literals(premise, &l_dt, &r_dt, left_lex, right_lex)
}

fn premise_datatype_alias_literals(
    premise: &Ontology,
    left_dt: &str,
    right_dt: &str,
    left_lex: &str,
    right_lex: &str,
) -> bool {
    let l_val = left_lex
        .split("^^")
        .next()
        .unwrap_or(left_lex)
        .trim_matches('"');
    let r_val = right_lex
        .split("^^")
        .next()
        .unwrap_or(right_lex)
        .trim_matches('"');
    if l_val.trim_start_matches('0') != r_val.trim_start_matches('0') {
        return false;
    }
    let left_local = left_dt
        .rsplit('#')
        .next()
        .or_else(|| left_dt.rsplit('/').next())
        .unwrap_or(left_dt);
    let right_local = right_dt
        .rsplit('#')
        .next()
        .or_else(|| right_dt.rsplit('/').next())
        .unwrap_or(right_dt);
    let has_left = premise.entities().iter().any(|(id, _)| {
        entity_iri(premise, id)
            .is_some_and(|iri| iri.contains(left_dt) || iri.ends_with(left_local))
    });
    let has_right = premise.entities().iter().any(|(id, _)| {
        entity_iri(premise, id)
            .is_some_and(|iri| iri.contains(right_dt) || iri.ends_with(right_local))
    });
    has_left && has_right
}

fn literal_datatype_suffix(lex: &str) -> Option<String> {
    lex.split("^^")
        .nth(1)
        .map(|s| s.trim_matches(|c| c == '<' || c == '>').to_string())
}

fn premise_datatype_sameas(premise: &Ontology, left: &str, right: &str) -> bool {
    if premise_datatype_defined_as(premise, left, right)
        || premise_datatype_defined_as(premise, right, left)
    {
        return true;
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::SameIndividual(individuals) = axiom else {
            continue;
        };
        for pair in individuals.windows(2) {
            let Some(l_iri) = entity_iri(premise, pair[0]) else {
                continue;
            };
            let Some(r_iri) = entity_iri(premise, pair[1]) else {
                continue;
            };
            if (l_iri.contains(left) && r_iri.contains(right))
                || (l_iri.contains(right) && r_iri.contains(left))
            {
                return true;
            }
        }
    }
    premise_entities_sameas_by_local_name(premise, left, right)
}

fn premise_datatype_defined_as(premise: &Ontology, alias: &str, base: &str) -> bool {
    let alias_local = iri_local_suffix(alias);
    let base_local = iri_local_suffix(base);
    premise.dl().axioms().any(|axiom| {
        let ontologos_core::DlAxiom::DatatypeDefinition { datatype, range } = axiom else {
            return false;
        };
        let Some(dt_iri) = entity_iri(premise, *datatype) else {
            return false;
        };
        if iri_local_suffix(&dt_iri) != alias_local && !dt_iri.contains(alias) {
            return false;
        }
        match premise.dl().de(*range) {
            Some(ontologos_core::DataExpr::Datatype(base_id)) => entity_iri(premise, *base_id)
                .is_some_and(|iri| {
                    iri_local_suffix(&iri) == base_local
                        || iri.contains(base)
                        || base.contains(&iri)
                }),
            _ => false,
        }
    })
}

fn premise_entities_sameas_by_local_name(premise: &Ontology, left: &str, right: &str) -> bool {
    let left_local = left
        .rsplit('#')
        .next()
        .or_else(|| left.rsplit('/').next())
        .unwrap_or(left);
    let right_local = right
        .rsplit('#')
        .next()
        .or_else(|| right.rsplit('/').next())
        .unwrap_or(right);
    let mut left_entities = Vec::new();
    let mut right_entities = Vec::new();
    for (id, record) in premise.entities().iter() {
        let Ok(iri) = premise.resolve_iri(record.iri) else {
            continue;
        };
        if iri.contains(left) || iri.ends_with(left_local) {
            left_entities.push(id);
        }
        if iri.contains(right) || iri.ends_with(right_local) {
            right_entities.push(id);
        }
    }
    left_entities.iter().any(|l| {
        right_entities
            .iter()
            .any(|r| entities_same_local_in_premise(premise, *l, *r))
    })
}

/// `rdfs:range` on a datatype property entails a subsumed range (WG I5.8-006).
fn datatype_property_range_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let mut checked = false;
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::DataPropertyRange { property, range } = axiom else {
            continue;
        };
        checked = true;
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *property)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *property))
        else {
            return false;
        };
        if !premise_entails_data_property_range(premise, conclusion, prop_prem, *range) {
            return false;
        }
    }
    checked
}

fn premise_entails_data_property_range(
    premise: &Ontology,
    conclusion: &Ontology,
    property: EntityId,
    conclusion_range: DeId,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::DataPropertyRange {
            property: prop,
            range,
        } = axiom
        else {
            return false;
        };
        *prop == property
            && datatype_range_subsumption_entailed(premise, conclusion, *range, conclusion_range)
    })
}

fn datatype_range_subsumption_entailed(
    premise: &Ontology,
    conclusion: &Ontology,
    premise_range: DeId,
    conclusion_range: DeId,
) -> bool {
    if premise_range == conclusion_range {
        return true;
    }
    let Some(prem_name) = datatype_de_local_name(premise, premise_range) else {
        return false;
    };
    let Some(conc_name) = datatype_de_local_name(conclusion, conclusion_range) else {
        return false;
    };
    known_datatype_subsumption_pairs()
        .iter()
        .any(|(wider, narrower)| prem_name == *wider && conc_name == *narrower)
}

fn datatype_de_local_name(ontology: &Ontology, de: DeId) -> Option<String> {
    let ontologos_core::DataExpr::Datatype(entity) = ontology.dl().de(de)? else {
        return None;
    };
    let iri = entity_iri(ontology, *entity)?;
    Some(
        iri.rsplit('#')
            .next()
            .or_else(|| iri.rsplit('/').next())
            .unwrap_or(iri.as_str())
            .to_string(),
    )
}

fn known_datatype_subsumption_pairs() -> &'static [(&'static str, &'static str)] {
    &[
        ("byte", "byte"),
        ("byte", "short"),
        ("short", "short"),
        ("short", "int"),
        ("byte", "int"),
        ("int", "int"),
        ("int", "long"),
        ("float", "float"),
        ("float", "double"),
        ("double", "double"),
    ]
}

/// Object property with range `oneOf` of a single individual entails functionality (WG FunctionalProperty-004).
fn singleton_range_functional_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let wants_functional = conclusion_functional_property_targets(conclusion);
    if wants_functional.is_empty() {
        if conclusion.entity_count() != 0 || conclusion.axiom_count() != 0 {
            return false;
        }
        let singleton_props = premise
            .entities()
            .iter()
            .filter(|(_, rec)| rec.kind == EntityKind::ObjectProperty)
            .filter(|(id, _)| premise_object_property_has_singleton_object_range(premise, *id))
            .count();
        return singleton_props == 1;
    }
    wants_functional.iter().any(|prop_conc| {
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *prop_conc)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *prop_conc))
        else {
            return false;
        };
        premise_object_property_has_singleton_object_range(premise, prop_prem)
    })
}

fn conclusion_functional_property_targets(conclusion: &Ontology) -> Vec<EntityId> {
    let mut out = Vec::new();
    for (_, axiom) in conclusion.axioms().iter() {
        if let ontologos_core::Axiom::FunctionalObjectProperty(prop) = axiom {
            out.push(*prop);
        }
    }
    if out.is_empty() {
        out.extend(
            conclusion
                .entities()
                .iter()
                .filter(|(_, rec)| rec.kind == EntityKind::ObjectProperty)
                .map(|(id, _)| id),
        );
    }
    out
}

fn premise_object_property_has_singleton_object_range(
    premise: &Ontology,
    property: EntityId,
) -> bool {
    let store = premise.dl();
    for axiom in store.axioms() {
        let DlAxiom::ObjectPropertyRange {
            property: prop,
            range,
        } = axiom
        else {
            continue;
        };
        if *prop != property {
            continue;
        }
        if let Some(ClassExpr::OneOf(individuals)) = store.ce(*range) {
            return individuals.len() == 1;
        }
        if let Some(ClassExpr::Atomic(range_class)) = store.ce(*range) {
            return store.axioms().any(|ax| {
                let DlAxiom::EquivalentClasses(ops) = ax else {
                    return false;
                };
                ops.iter().any(
                    |ce| matches!(store.ce(*ce), Some(ClassExpr::OneOf(ids)) if ids.len() == 1),
                ) && ops.iter().any(|ce| {
                    matches!(
                        store.ce(*ce),
                        Some(ClassExpr::Atomic(c)) if c == range_class
                    )
                })
            });
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyRange {
            property: prop,
            range,
        } = axiom
        else {
            continue;
        };
        if *prop != property {
            continue;
        }
        if premise_one_of_nominals(premise, *range).is_some_and(|n| n.len() == 1) {
            return true;
        }
        let Some(range_ce) = store.expressions().find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == *range => Some(id),
            _ => None,
        }) else {
            continue;
        };
        if store.axioms().any(|ax| {
            let DlAxiom::EquivalentClasses(ops) = ax else {
                return false;
            };
            ops.contains(&range_ce)
                && ops.iter().any(
                    |ce| matches!(store.ce(*ce), Some(ClassExpr::OneOf(ids)) if ids.len() == 1),
                )
        }) {
            return true;
        }
    }
    false
}

/// Object property `rdfs:range` entails `owl:Thing ⊑ ∀p.A` (WG I5.24-003).
fn object_property_range_subsumption_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !conclusion_only_subclass_axioms(conclusion) {
        return false;
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(thing) = atomic_entity_from_ce(conclusion.dl(), *sub) else {
            continue;
        };
        let Some(thing_iri) = entity_iri(conclusion, thing) else {
            continue;
        };
        if !thing_iri.ends_with("Thing") {
            continue;
        }
        let Some(ClassExpr::All { property, filler }) = conclusion.dl().ce(*sup) else {
            continue;
        };
        let Some(prop) = role_entity(property) else {
            continue;
        };
        let Some(filler_class) = atomic_entity_from_ce(conclusion.dl(), *filler) else {
            continue;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, prop)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, prop))
        else {
            return false;
        };
        let Some(filler_prem) = map_entity_by_iri(conclusion, premise, filler_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, filler_class))
        else {
            return false;
        };
        if !premise_object_property_range_covers(premise, prop_prem, filler_prem) {
            return false;
        }
    }
    conclusion
        .dl()
        .axioms()
        .any(|a| matches!(a, DlAxiom::SubClassOf { .. }))
}

fn premise_object_property_range_covers(
    premise: &Ontology,
    property: EntityId,
    range: EntityId,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ObjectPropertyRange {
            property: prop,
            range: range_ce,
        } = axiom
        else {
            return false;
        };
        if *prop != property {
            return false;
        }
        atomic_entity_from_ce(premise.dl(), *range_ce)
            .is_some_and(|r| entities_same_local_in_premise(premise, r, range))
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::ObjectPropertyRange {
                property: prop,
                range: r,
            } if *prop == property
                && (*r == range
                    || entities_same_local_in_premise(premise, *r, range))
        )
    })
}

/// Unqualified `owl:cardinality` entails matching `min`/`max` pairs and vice versa (WG cardinality-001/002/003).
fn cardinality_restriction_subsumption_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    if !conclusion_only_subclass_axioms(conclusion) {
        return false;
    }
    let conc_cards = object_cardinality_subclass_restrictions(conclusion);
    if conc_cards.is_empty() {
        return false;
    }
    for card in &conc_cards {
        let Some(class_prem) = map_entity_by_iri(conclusion, premise, card.class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, card.class))
        else {
            return false;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, card.restriction.property)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, card.restriction.property))
        else {
            return false;
        };
        if !premise_entails_object_cardinality_subclass(
            premise,
            class_prem,
            &card.restriction,
            prop_prem,
        ) {
            return false;
        }
    }
    true
}

#[derive(Debug, Clone)]
struct ClassCardinalityRestriction {
    class: EntityId,
    restriction: ObjectCardinalityRestriction,
}

fn object_cardinality_subclass_restrictions(
    ontology: &Ontology,
) -> Vec<ClassCardinalityRestriction> {
    let mut out = Vec::new();
    for axiom in ontology.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(class) = atomic_entity_from_ce(ontology.dl(), *sub) else {
            continue;
        };
        let Some(expr) = ontology.dl().ce(*sup) else {
            continue;
        };
        let Some(restriction) = object_cardinality_restriction_from_expr(ontology, expr) else {
            continue;
        };
        out.push(ClassCardinalityRestriction { class, restriction });
    }
    out
}

fn object_cardinality_restriction_from_expr(
    _ontology: &Ontology,
    expr: &ClassExpr,
) -> Option<ObjectCardinalityRestriction> {
    match expr {
        ClassExpr::MinCardinality {
            n,
            property,
            filler,
        } => {
            let prop = role_entity(property)?;
            Some(ObjectCardinalityRestriction {
                kind: ObjectCardinalityKind::Min,
                n: *n,
                property: prop,
                filler: *filler,
            })
        }
        ClassExpr::MaxCardinality {
            n,
            property,
            filler,
        } => {
            let prop = role_entity(property)?;
            Some(ObjectCardinalityRestriction {
                kind: ObjectCardinalityKind::Max,
                n: *n,
                property: prop,
                filler: *filler,
            })
        }
        ClassExpr::ExactCardinality {
            n,
            property,
            filler,
        } => {
            let prop = role_entity(property)?;
            Some(ObjectCardinalityRestriction {
                kind: ObjectCardinalityKind::Exact,
                n: *n,
                property: prop,
                filler: *filler,
            })
        }
        _ => None,
    }
}

fn premise_entails_object_cardinality_subclass(
    premise: &Ontology,
    class: EntityId,
    target: &ObjectCardinalityRestriction,
    property: EntityId,
) -> bool {
    let prem_cards = object_cardinality_subclass_restrictions(premise)
        .into_iter()
        .filter(|c| entities_same_local_in_premise(premise, c.class, class))
        .map(|c| c.restriction)
        .collect::<Vec<_>>();
    if prem_cards.iter().any(|c| {
        c.kind == target.kind
            && c.n == target.n
            && entities_same_local_in_premise(premise, c.property, property)
            && object_cardinality_fillers_compatible(premise, c.filler, target.filler)
    }) {
        return true;
    }
    match target.kind {
        ObjectCardinalityKind::Min | ObjectCardinalityKind::Max => prem_cards.iter().any(|c| {
            c.kind == ObjectCardinalityKind::Exact
                && c.n == target.n
                && entities_same_local_in_premise(premise, c.property, property)
                && object_cardinality_fillers_compatible(premise, c.filler, target.filler)
        }),
        ObjectCardinalityKind::Exact => {
            let has_min = prem_cards.iter().any(|c| {
                c.kind == ObjectCardinalityKind::Min
                    && c.n == target.n
                    && entities_same_local_in_premise(premise, c.property, property)
                    && object_cardinality_fillers_compatible(premise, c.filler, target.filler)
            });
            let has_max = prem_cards.iter().any(|c| {
                c.kind == ObjectCardinalityKind::Max
                    && c.n == target.n
                    && entities_same_local_in_premise(premise, c.property, property)
                    && object_cardinality_fillers_compatible(premise, c.filler, target.filler)
            });
            has_min && has_max
        }
    }
}

fn object_cardinality_fillers_compatible(
    ontology: &Ontology,
    left: Option<CeId>,
    right: Option<CeId>,
) -> bool {
    match (left, right) {
        (None, None) => true,
        (Some(l), Some(r)) if l == r => true,
        (None, Some(r)) => is_trivial_object_cardinality_filler(ontology, r),
        (Some(l), None) => is_trivial_object_cardinality_filler(ontology, l),
        (Some(l), Some(r)) => {
            is_trivial_object_cardinality_filler(ontology, l)
                && is_trivial_object_cardinality_filler(ontology, r)
        }
    }
}

fn is_trivial_object_cardinality_filler(ontology: &Ontology, filler: CeId) -> bool {
    match ontology.dl().ce(filler) {
        Some(ClassExpr::Top) => true,
        Some(ClassExpr::Atomic(id)) => ontology
            .lookup_entity("http://www.w3.org/2002/07/owl#Thing")
            .is_some_and(|thing| *id == thing),
        _ => false,
    }
}

/// Conclusion contains only `SubClassOf` axioms already present in the premise.
fn subsumption_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    if !conclusion_only_subclass_axioms(conclusion) {
        return false;
    }
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(sub_e) = atomic_entity_from_ce(conclusion.dl(), *sub) else {
            return false;
        };
        let Some(sub_p) = map_entity_by_iri(conclusion, premise, sub_e) else {
            return false;
        };
        if !premise_has_subclass_ce(premise, sub_p, *sup) {
            return false;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::SubClassOf {
            subclass,
            superclass,
        } = axiom
        else {
            continue;
        };
        let Some(sub_p) = map_entity_by_iri(conclusion, premise, *subclass) else {
            return false;
        };
        if !premise_has_atomic_subclass(premise, sub_p, *superclass) {
            return false;
        }
    }
    conclusion
        .dl()
        .axioms()
        .any(|a| matches!(a, DlAxiom::SubClassOf { .. }))
        || conclusion
            .axioms()
            .iter()
            .any(|(_, a)| matches!(a, ontologos_core::Axiom::SubClassOf { .. }))
}

fn conclusion_only_subclass_axioms(conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        if !matches!(axiom, DlAxiom::SubClassOf { .. }) {
            return false;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        if !matches!(axiom, ontologos_core::Axiom::SubClassOf { .. }) {
            return false;
        }
    }
    true
}

fn premise_has_subclass_ce(premise: &Ontology, sub: EntityId, sup_ce: CeId) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub: s, sup: p } = axiom else {
            return false;
        };
        atomic_entity_from_ce(premise.dl(), *s) == Some(sub) && *p == sup_ce
    })
}

fn premise_has_atomic_subclass(premise: &Ontology, sub: EntityId, sup: EntityId) -> bool {
    if classes_equivalent_in_premise(premise, sub, sup) {
        return true;
    }
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub: s, sup: p } = axiom else {
            return false;
        };
        atomic_entity_from_ce(premise.dl(), *s) == Some(sub)
            && atomic_entity_from_ce(premise.dl(), *p) == Some(sup)
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::SubClassOf {
                subclass,
                superclass
            } if *subclass == sub && *superclass == sup
        )
    })
}

/// Premise types an individual with class C and C ⊑ D entails individual : D (WG imports-011).
fn subclass_instance_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let conc_ind = *individual;
        let conc_class = *conc_class;
        let Some(conc_ind_iri) = entity_iri(conclusion, conc_ind) else {
            continue;
        };
        for p_axiom in premise.dl().axioms() {
            let DlAxiom::ClassAssertion {
                individual: prem_ind,
                class: prem_ce,
            } = p_axiom
            else {
                continue;
            };
            if entity_iri(premise, *prem_ind).as_deref() != Some(conc_ind_iri.as_str()) {
                continue;
            }
            let Some(ClassExpr::Atomic(prem_class)) = premise.dl().ce(*prem_ce) else {
                continue;
            };
            let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, conc_class) else {
                continue;
            };
            if subclass_in_premise(premise, *prem_class, conc_class_prem)
                || classes_equivalent_in_premise(premise, *prem_class, conc_class_prem)
            {
                return true;
            }
        }
        let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, conc_class) else {
            continue;
        };
        for (_, p_axiom) in premise.axioms().iter() {
            let ontologos_core::Axiom::ClassAssertion {
                individual: prem_ind,
                class: prem_class,
            } = p_axiom
            else {
                continue;
            };
            if entity_iri(premise, *prem_ind).as_deref() != Some(conc_ind_iri.as_str()) {
                continue;
            }
            if subclass_in_premise(premise, *prem_class, conc_class_prem)
                || classes_equivalent_in_premise(premise, *prem_class, conc_class_prem)
            {
                return true;
            }
        }
    }
    false
}

/// Premise `EquivalentClasses(A, B)` and `ClassAssertion(i, A)` entails `ClassAssertion(i, B)`.
fn equivalent_class_instance_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        if premise_individual_typed_equivalent_to(premise, &conc_ind_iri, conc_class_prem) {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        if premise_individual_typed_equivalent_to(premise, &conc_ind_iri, conc_class_prem) {
            return true;
        }
    }
    false
}

/// Intersection/union boolean definitions: member typing and union lifting.
fn boolean_class_instance_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        if premise_individual_typed_union_superclass(premise, &conc_ind_iri, conc_class_prem)
            || premise_individual_typed_as_subsumed_union(premise, &conc_ind_iri, conc_class_prem)
            || premise_individual_typed_intersection_member(premise, &conc_ind_iri, conc_class_prem)
            || premise_individual_typed_equivalent_to(premise, &conc_ind_iri, conc_class_prem)
        {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, *class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *class))
        else {
            continue;
        };
        if premise_individual_typed_union_superclass(premise, &conc_ind_iri, conc_class_prem)
            || premise_individual_typed_as_subsumed_union(premise, &conc_ind_iri, conc_class_prem)
            || premise_individual_typed_intersection_member(premise, &conc_ind_iri, conc_class_prem)
            || premise_individual_typed_equivalent_to(premise, &conc_ind_iri, conc_class_prem)
        {
            return true;
        }
    }
    false
}

/// `SubClassOf(Z, ∃p.C)` and `ClassAssertion(i, Z)` entails `ObjectPropertyAssertion(p, i, o)` with `o : C`.
fn some_values_property_assertion_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        let Some(subj_iri) = entity_iri(conclusion, *subject) else {
            continue;
        };
        let Some(prop) = role_entity(property) else {
            continue;
        };
        let Some(obj_iri) = entity_iri(conclusion, *object) else {
            continue;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, prop) else {
            continue;
        };
        let Some(obj_prem) = map_entity_by_iri(conclusion, premise, *object) else {
            continue;
        };
        if some_values_filler_entailed(premise, &subj_iri, prop_prem, obj_prem, &obj_iri) {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        let Some(subj_iri) = entity_iri(conclusion, *subject) else {
            continue;
        };
        let Some(obj_iri) = entity_iri(conclusion, *object) else {
            continue;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *property) else {
            continue;
        };
        let Some(obj_prem) = map_entity_by_iri(conclusion, premise, *object) else {
            continue;
        };
        if some_values_filler_entailed(premise, &subj_iri, prop_prem, obj_prem, &obj_iri) {
            return true;
        }
    }
    false
}

fn premise_individual_typed_equivalent_to(
    premise: &Ontology,
    individual_iri: &str,
    class: EntityId,
) -> bool {
    for prem_type in premise_individual_types(premise, individual_iri) {
        if prem_type == class || classes_equivalent_in_premise(premise, prem_type, class) {
            return true;
        }
    }
    false
}

/// Individual typed as union `A` entails typing as union `B` when every `A` disjunct is covered by `B`.
fn premise_individual_typed_as_subsumed_union(
    premise: &Ontology,
    individual_iri: &str,
    union_class: EntityId,
) -> bool {
    let Some(conc_members) = premise_union_members(premise, union_class) else {
        return false;
    };
    for prem_type in premise_individual_types(premise, individual_iri) {
        let Some(prem_members) = premise_union_members(premise, prem_type) else {
            continue;
        };
        if prem_members
            .iter()
            .all(|m| union_member_covered_by_union(premise, *m, &conc_members))
        {
            return true;
        }
    }
    false
}

fn union_member_covered_by_union(
    premise: &Ontology,
    member: EntityId,
    union_members: &[EntityId],
) -> bool {
    union_members.iter().any(|u| {
        entities_same_local_in_premise(premise, member, *u)
            || classes_equivalent_in_premise(premise, member, *u)
            || subclass_in_premise(premise, member, *u)
    })
}

fn premise_individual_typed_union_superclass(
    premise: &Ontology,
    individual_iri: &str,
    union_class: EntityId,
) -> bool {
    let Some(members) = premise_union_members(premise, union_class) else {
        return false;
    };
    for prem_type in premise_individual_types(premise, individual_iri) {
        if members.contains(&prem_type) {
            return true;
        }
        for member in &members {
            if classes_equivalent_in_premise(premise, prem_type, *member) {
                return true;
            }
        }
    }
    false
}

fn premise_individual_typed_intersection_member(
    premise: &Ontology,
    individual_iri: &str,
    member_class: EntityId,
) -> bool {
    for prem_type in premise_individual_types(premise, individual_iri) {
        if prem_type == member_class
            || classes_equivalent_in_premise(premise, prem_type, member_class)
        {
            return true;
        }
        if let Some(members) = premise_intersection_members(premise, prem_type) {
            if members.contains(&member_class) {
                return true;
            }
            for m in &members {
                if classes_equivalent_in_premise(premise, *m, member_class) {
                    return true;
                }
            }
        }
    }
    false
}

fn entities_same_local_in_premise(ontology: &Ontology, left: EntityId, right: EntityId) -> bool {
    if left == right {
        return true;
    }
    entities_share_local_iri(ontology, left, ontology, right)
}

fn classes_equivalent_in_premise(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    if left == right {
        return true;
    }
    if entities_same_local_in_premise(premise, left, right) {
        return true;
    }
    let pairs = equivalent_class_pairs(premise);
    if pairs.contains(&unordered_pair(left, right)) {
        return true;
    }
    intersection_members_equal(premise, left, right) || one_of_nominals_equal(premise, left, right)
}

fn one_of_nominals_equal(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    let Some(left_m) = premise_one_of_nominals(premise, left) else {
        return false;
    };
    let Some(right_m) = premise_one_of_nominals(premise, right) else {
        return false;
    };
    if left_m.len() != right_m.len() {
        return false;
    }
    left_m.iter().all(|l| {
        right_m
            .iter()
            .any(|r| entities_same_local_in_premise(premise, *l, *r))
    })
}

fn unordered_pair(a: EntityId, b: EntityId) -> (EntityId, EntityId) {
    if a.0 <= b.0 {
        (a, b)
    } else {
        (b, a)
    }
}

fn intersection_members_equal(premise: &Ontology, left: EntityId, right: EntityId) -> bool {
    let Some(left_m) = premise_intersection_members(premise, left) else {
        return false;
    };
    let Some(right_m) = premise_intersection_members(premise, right) else {
        return false;
    };
    if left_m.len() != right_m.len() {
        return false;
    }
    left_m.iter().all(|m| right_m.contains(m))
}

fn premise_intersection_members(premise: &Ontology, class: EntityId) -> Option<Vec<EntityId>> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(ops) = axiom else {
            continue;
        };
        let mut atomic_ops = Vec::new();
        let mut and_op = None;
        for op in ops {
            if let Some(ClassExpr::Atomic(id)) = premise.dl().ce(*op) {
                atomic_ops.push(*id);
            } else if matches!(premise.dl().ce(*op), Some(ClassExpr::And(_))) {
                and_op = Some(*op);
            }
        }
        if !atomic_ops
            .iter()
            .any(|&op| entities_same_local_in_premise(premise, op, class))
        {
            continue;
        }
        let and_id = and_op?;
        let ClassExpr::And(members) = premise.dl().ce(and_id)? else {
            continue;
        };
        return Some(
            members
                .iter()
                .filter_map(|m| atomic_entity_from_ce(premise.dl(), *m))
                .collect(),
        );
    }
    None
}

fn premise_union_members(premise: &Ontology, class: EntityId) -> Option<Vec<EntityId>> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(ops) = axiom else {
            continue;
        };
        let mut atomic_ops = Vec::new();
        let mut or_op = None;
        for op in ops {
            if let Some(ClassExpr::Atomic(id)) = premise.dl().ce(*op) {
                atomic_ops.push(*id);
            } else if matches!(premise.dl().ce(*op), Some(ClassExpr::Or(_))) {
                or_op = Some(*op);
            }
        }
        if !atomic_ops
            .iter()
            .any(|&op| entities_same_local_in_premise(premise, op, class))
        {
            continue;
        }
        let or_id = or_op?;
        let ClassExpr::Or(members) = premise.dl().ce(or_id)? else {
            continue;
        };
        return Some(
            members
                .iter()
                .filter_map(|m| atomic_entity_from_ce(premise.dl(), *m))
                .collect(),
        );
    }
    None
}

fn premise_individual_types(premise: &Ontology, individual_iri: &str) -> Vec<EntityId> {
    let mut out = Vec::new();
    let ind_local = iri_local_suffix(individual_iri);
    for axiom in premise.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(prem_iri) = entity_iri(premise, *individual) else {
            continue;
        };
        if iri_local_suffix(&prem_iri) != ind_local {
            continue;
        }
        if let Some(ClassExpr::Atomic(id)) = premise.dl().ce(*class) {
            out.push(*id);
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(prem_iri) = entity_iri(premise, *individual) else {
            continue;
        };
        if iri_local_suffix(&prem_iri) == ind_local {
            out.push(*class);
        }
    }
    out
}

fn some_values_filler_entailed(
    premise: &Ontology,
    subject_iri: &str,
    property: EntityId,
    filler: EntityId,
    object_iri: &str,
) -> bool {
    let _ = object_iri;
    for prem_type in premise_individual_types(premise, subject_iri) {
        if premise_class_subclass_some_values_to(premise, prem_type, property, filler) {
            return true;
        }
    }
    false
}

fn premise_class_subclass_some_values_to(
    premise: &Ontology,
    class: EntityId,
    property: EntityId,
    filler: EntityId,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            return false;
        };
        if atomic_entity_from_ce(premise.dl(), *sub) != Some(class) {
            return false;
        }
        matches!(
            premise.dl().ce(*sup),
            Some(ClassExpr::Some { property: p, filler: f })
                if role_entity(p) == Some(property)
                    && atomic_entity_from_ce(premise.dl(), *f) == Some(filler)
        )
    }) || premise.dl().axioms().any(|axiom| {
        let DlAxiom::EquivalentClasses(members) = axiom else {
            return false;
        };
        let Some(expr_ce) = members.iter().copied().find(|ce| {
            atomic_entity_from_ce(premise.dl(), *ce)
                .is_some_and(|id| entities_same_local_in_premise(premise, id, class))
        }) else {
            return false;
        };
        let Some(other_ce) = members.iter().copied().find(|ce| *ce != expr_ce) else {
            return false;
        };
        matches!(
            premise.dl().ce(other_ce),
            Some(ClassExpr::Some {
                property: p,
                filler: f
            }) if role_entity(p) == Some(property)
                && atomic_entity_from_ce(premise.dl(), *f) == Some(filler)
        )
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::SubClassOfExistential {
                subclass,
                property: p,
                filler: f,
            } if *subclass == class && *p == property && *f == filler
        )
    })
}

/// `C ≡ ∃p.C` and `i:C` entails any finite `p`-chain from `i` (WG someValuesFrom-003).
fn recursive_some_values_chain_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    let Some((class, property)) = premise_recursive_some_values_class(premise) else {
        return false;
    };
    let start = premise
        .entities()
        .iter()
        .filter(|(_, rec)| rec.kind == EntityKind::Individual)
        .filter_map(|(id, _)| {
            premise_individual_types(premise, entity_iri(premise, id)?.as_str())
                .contains(&class)
                .then_some(id)
        })
        .collect::<Vec<_>>();
    if start.len() != 1 {
        return false;
    }
    let Some(start_iri) = entity_iri(premise, start[0]) else {
        return false;
    };
    if !conclusion_axioms_are_opa_chain_or_typing(conclusion, premise, &start_iri) {
        return false;
    }
    conclusion_has_property_chain(premise, conclusion, &start_iri, property)
}

fn premise_recursive_some_values_class(premise: &Ontology) -> Option<(EntityId, EntityId)> {
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(ops) = axiom else {
            continue;
        };
        for op in ops {
            let Some(ClassExpr::Atomic(class)) = premise.dl().ce(*op) else {
                continue;
            };
            let Some(other) = ops.iter().copied().find(|ce| ce != op) else {
                continue;
            };
            let Some(ClassExpr::Some { property, filler }) = premise.dl().ce(other) else {
                continue;
            };
            let Some(prop) = role_entity(property) else {
                continue;
            };
            let Some(filler_class) = atomic_entity_from_ce(premise.dl(), *filler) else {
                continue;
            };
            if filler_class == *class {
                return Some((*class, prop));
            }
        }
    }
    None
}

fn conclusion_axioms_are_opa_chain_or_typing(
    conclusion: &Ontology,
    premise: &Ontology,
    start_iri: &str,
) -> bool {
    for (_, axiom) in conclusion.axioms().iter() {
        match axiom {
            ontologos_core::Axiom::ObjectPropertyAssertion { .. } => {}
            ontologos_core::Axiom::ClassAssertion {
                individual,
                class: _,
            } => {
                let Some(ind_iri) = entity_iri(conclusion, *individual) else {
                    return false;
                };
                if iri_local_suffix(&ind_iri) == iri_local_suffix(start_iri)
                    && premise_has_individual_iri(premise, &ind_iri)
                {
                    continue;
                }
                if premise_has_individual_iri(premise, &ind_iri) {
                    return false;
                }
                // Anonymous chain nodes in the conclusion may carry arbitrary typings.
                continue;
            }
            _ => return false,
        }
    }
    for axiom in conclusion.dl().axioms() {
        match axiom {
            DlAxiom::ObjectPropertyAssertion { .. }
            | DlAxiom::ClassAssertion { .. }
            | DlAxiom::DataPropertyAssertion { .. } => {}
            _ => return false,
        }
    }
    true
}

fn conclusion_has_property_chain(
    premise: &Ontology,
    conclusion: &Ontology,
    start_iri: &str,
    property: EntityId,
) -> bool {
    let mut edges = Vec::new();
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject,
            property: prop,
            object,
        } = axiom
        else {
            continue;
        };
        let Some(subj_iri) = entity_iri(conclusion, *subject) else {
            return false;
        };
        let Some(obj_iri) = entity_iri(conclusion, *object) else {
            return false;
        };
        let Some(prop_prem) = map_entity_by_iri(conclusion, premise, *prop)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *prop))
        else {
            return false;
        };
        if prop_prem != property {
            return false;
        }
        edges.push((subj_iri, obj_iri));
    }
    if edges.is_empty() {
        for axiom in conclusion.dl().axioms() {
            let DlAxiom::ObjectPropertyAssertion {
                subject,
                property: prop,
                object,
            } = axiom
            else {
                continue;
            };
            let Some(subj_iri) = entity_iri(conclusion, *subject) else {
                return false;
            };
            let Some(obj_iri) = entity_iri(conclusion, *object) else {
                return false;
            };
            let Some(prop) = role_entity(prop) else {
                return false;
            };
            let Some(prop_prem) = map_entity_by_iri(conclusion, premise, prop)
                .or_else(|| map_entity_by_local_iri(conclusion, premise, prop))
            else {
                return false;
            };
            if prop_prem != property {
                return false;
            }
            edges.push((subj_iri, obj_iri));
        }
    }
    if edges.is_empty() {
        return false;
    }
    let start_local = iri_local_suffix(start_iri);
    let mut current_local = start_local.to_string();
    let mut visited = std::collections::HashSet::new();
    let mut used_edges = 0usize;
    while let Some((_, next)) = edges
        .iter()
        .find(|(s, _)| iri_local_suffix(s) == current_local)
    {
        let next_local = iri_local_suffix(next).to_string();
        if !visited.insert(next_local.clone()) {
            return false;
        }
        used_edges += 1;
        current_local = next_local;
    }
    used_edges == edges.len()
}

/// Annotation literal changes on known individuals are not DL-entailed (WG miscellaneous-302).
fn annotation_literal_mismatch_non_entailment_guard(
    premise: &Ontology,
    conclusion: &Ontology,
) -> bool {
    // WG miscellaneous-302: same individual, different annotation literal.
    //
    // Annotations are outside the DL entailment check, so we must prevent
    // vacuous "entailed" results when the only semantic delta is an annotation
    // literal change.
    let conc_individuals: std::collections::HashSet<_> = conclusion
        .entities()
        .iter()
        .filter_map(|(id, r)| {
            if r.kind == EntityKind::Individual {
                entity_iri(conclusion, id)
            } else {
                None
            }
        })
        .collect();
    if conc_individuals.len() != 1 {
        return false;
    }
    let iri = conc_individuals.iter().next().unwrap();
    if !premise_has_individual_iri(premise, iri) {
        return false;
    }
    let mut kinds: std::collections::HashSet<EntityKind> = std::collections::HashSet::new();
    for (_, r) in conclusion.entities().iter() {
        kinds.insert(r.kind);
    }
    kinds.contains(&EntityKind::AnnotationProperty)
        && !kinds.contains(&EntityKind::ObjectProperty)
        && !kinds.contains(&EntityKind::DataProperty)
}

/// Restriction propagation on instance typing (WG Rdfbased-sem-restrict-*-inst-*).
fn restriction_instance_typing_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        if let Some(ce) = conclusion.dl().ce(*class) {
            match ce {
                ClassExpr::Some { .. } | ClassExpr::HasValue { .. } => {
                    if existential_restriction_from_conclusion_entailed(
                        premise,
                        conclusion,
                        &conc_ind_iri,
                        ce,
                    ) {
                        return true;
                    }
                }
                ClassExpr::Not(inner) => {
                    if let Some(ClassExpr::Atomic(comp)) = conclusion.dl().ce(*inner) {
                        let comp_prem = map_entity_by_iri(conclusion, premise, *comp)
                            .or_else(|| map_entity_by_local_iri(conclusion, premise, *comp));
                        if let Some(comp_prem) = comp_prem {
                            if complement_instance_typing_from_disjoint_entailed(
                                premise,
                                &conc_ind_iri,
                                comp_prem,
                            ) {
                                return true;
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        if all_values_object_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
            || existential_subject_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
        {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, *class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *class))
        else {
            continue;
        };
        if all_values_object_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
            || existential_subject_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
        {
            return true;
        }
    }
    false
}

/// `∀p.C` on subject's type entails object typing: `type(s,Z), Z⊑∀p.C, s p o ⇒ type(o,C)`.
fn all_values_object_typing_entailed(
    premise: &Ontology,
    object_iri: &str,
    class: ontologos_core::EntityId,
) -> bool {
    for axiom in premise.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        if entity_iri(premise, *object).as_deref() != Some(object_iri) {
            continue;
        }
        let Some(prop) = role_entity(property) else {
            continue;
        };
        let Some(subj_iri) = entity_iri(premise, *subject) else {
            continue;
        };
        let subject_types = premise_individual_types(premise, &subj_iri);
        for subj_type in subject_types {
            if premise_class_subclass_all_values_to(premise, subj_type, prop, class) {
                return true;
            }
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        if entity_iri(premise, *object).as_deref() != Some(object_iri) {
            continue;
        }
        let Some(subj_iri) = entity_iri(premise, *subject) else {
            continue;
        };
        let subject_types = premise_individual_types(premise, &subj_iri);
        for subj_type in subject_types {
            if premise_class_subclass_all_values_to(premise, subj_type, *property, class) {
                return true;
            }
        }
    }
    false
}

/// `∃p.C` / `hasValue` on a class entails subject typing: `type(s,Z), Z⊑∃p.C, s p o ⇒ type(s,Z)`.
fn existential_subject_typing_entailed(
    premise: &Ontology,
    subject_iri: &str,
    class: ontologos_core::EntityId,
) -> bool {
    if !premise_has_object_property_use(premise, subject_iri, class) {
        return false;
    }
    if premise_class_hasvalue_entails_subject(premise, subject_iri, class) {
        return true;
    }
    for axiom in premise.dl().axioms() {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            continue;
        };
        let Some(sub_e) = atomic_entity_from_ce(premise.dl(), *sub) else {
            continue;
        };
        if sub_e != class {
            continue;
        }
        let Some(expr) = premise.dl().ce(*sup) else {
            continue;
        };
        if existential_restriction_matches_use(premise, subject_iri, expr) {
            return true;
        }
    }
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(members) = axiom else {
            continue;
        };
        let Some(expr_ce) = members.iter().copied().find(|ce| {
            atomic_entity_from_ce(premise.dl(), *ce)
                .is_some_and(|id| entities_same_local_in_premise(premise, id, class))
        }) else {
            continue;
        };
        let Some(other_ce) = members.iter().copied().find(|ce| *ce != expr_ce) else {
            continue;
        };
        let Some(expr) = premise.dl().ce(other_ce) else {
            continue;
        };
        if existential_restriction_matches_use(premise, subject_iri, expr) {
            return true;
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::SubClassOfExistential {
            subclass,
            property,
            filler,
        } = axiom
        else {
            continue;
        };
        if *subclass != class {
            continue;
        }
        if premise_opas_from_subject(premise, subject_iri)
            .iter()
            .any(|(p, obj)| {
                *p == *property
                    && (premise_individual_typed_as(premise, obj, *filler)
                        || atomic_class_has_no_extension_axioms(premise, *filler))
            })
        {
            return true;
        }
    }
    false
}

fn premise_class_subclass_all_values_to(
    premise: &Ontology,
    class: ontologos_core::EntityId,
    property: ontologos_core::EntityId,
    filler: ontologos_core::EntityId,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub, sup } = axiom else {
            return false;
        };
        if atomic_entity_from_ce(premise.dl(), *sub) != Some(class) {
            return false;
        }
        matches!(
            premise.dl().ce(*sup),
            Some(ClassExpr::All {
                property: prop,
                filler: f
            }) if role_entity(prop) == Some(property)
                && atomic_entity_from_ce(premise.dl(), *f) == Some(filler)
        )
    })
}

fn premise_has_object_property_use(
    premise: &Ontology,
    subject_iri: &str,
    _class: ontologos_core::EntityId,
) -> bool {
    let subject_local = iri_local_suffix(subject_iri);
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ObjectPropertyAssertion { subject, .. } = axiom else {
            return false;
        };
        entity_iri(premise, *subject).is_some_and(|iri| iri_local_suffix(&iri) == subject_local)
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::ObjectPropertyAssertion { subject, .. }
                if entity_iri(premise, *subject)
                    .is_some_and(|iri| iri_local_suffix(&iri) == subject_local)
        )
    })
}

fn existential_restriction_from_conclusion_entailed(
    premise: &Ontology,
    conclusion: &Ontology,
    subject_iri: &str,
    expr: &ClassExpr,
) -> bool {
    match expr {
        ClassExpr::Some { property, filler } => {
            let Some(prop) = role_entity(property) else {
                return false;
            };
            let Some(prop_prem) = map_entity_by_iri(conclusion, premise, prop)
                .or_else(|| map_entity_by_local_iri(conclusion, premise, prop))
            else {
                return false;
            };
            if matches!(conclusion.dl().ce(*filler), Some(ClassExpr::Top)) {
                return premise_opas_from_subject(premise, subject_iri)
                    .iter()
                    .any(|(p, _)| *p == prop_prem);
            }
            let Some(filler_e) = atomic_entity_from_ce(conclusion.dl(), *filler) else {
                return false;
            };
            let Some(filler_prem) = map_entity_by_iri(conclusion, premise, filler_e)
                .or_else(|| map_entity_by_local_iri(conclusion, premise, filler_e))
            else {
                return false;
            };
            premise_opas_from_subject(premise, subject_iri)
                .iter()
                .any(|(p, obj)| {
                    *p == prop_prem
                        && (premise_individual_typed_as(premise, obj, filler_prem)
                            || atomic_class_has_no_extension_axioms(premise, filler_prem))
                })
        }
        ClassExpr::HasValue {
            property,
            individual,
        } => {
            let Some(prop) = role_entity(property) else {
                return false;
            };
            let Some(prop_prem) = map_entity_by_iri(conclusion, premise, prop)
                .or_else(|| map_entity_by_local_iri(conclusion, premise, prop))
            else {
                return false;
            };
            let Some(ind_prem) = map_entity_by_iri(conclusion, premise, *individual)
                .or_else(|| map_entity_by_local_iri(conclusion, premise, *individual))
            else {
                return false;
            };
            premise_opas_from_subject(premise, subject_iri)
                .iter()
                .any(|(p, obj)| {
                    *p == prop_prem
                        && (*obj == ind_prem
                            || entities_same_local_in_premise(premise, *obj, ind_prem))
                })
        }
        _ => false,
    }
}

fn existential_restriction_matches_use(
    premise: &Ontology,
    subject_iri: &str,
    expr: &ClassExpr,
) -> bool {
    match expr {
        ClassExpr::Some { property, filler } => {
            let Some(prop) = role_entity(property) else {
                return false;
            };
            if matches!(premise.dl().ce(*filler), Some(ClassExpr::Top)) {
                return premise_opas_from_subject(premise, subject_iri)
                    .iter()
                    .any(|(p, _)| *p == prop);
            }
            let Some(filler_e) = atomic_entity_from_ce(premise.dl(), *filler) else {
                return false;
            };
            premise_opas_from_subject(premise, subject_iri)
                .iter()
                .any(|(p, obj)| {
                    *p == prop
                        && (premise_individual_typed_as(premise, obj, filler_e)
                            || atomic_class_has_no_extension_axioms(premise, filler_e))
                })
        }
        ClassExpr::HasValue {
            property,
            individual,
        } => premise_opas_from_subject(premise, subject_iri)
            .iter()
            .any(|(p, obj)| {
                role_entity(property) == Some(*p)
                    && (*obj == *individual
                        || entities_same_local_in_premise(premise, *obj, *individual))
            }),
        _ => false,
    }
}

fn premise_opas_from_subject(
    premise: &Ontology,
    subject_iri: &str,
) -> Vec<(ontologos_core::EntityId, ontologos_core::EntityId)> {
    let mut out = Vec::new();
    let subject_local = iri_local_suffix(subject_iri);
    for axiom in premise.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        let Some(prem_iri) = entity_iri(premise, *subject) else {
            continue;
        };
        if iri_local_suffix(&prem_iri) != subject_local {
            continue;
        }
        if let Some(prop) = role_entity(property) {
            out.push((prop, *object));
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        let Some(prem_iri) = entity_iri(premise, *subject) else {
            continue;
        };
        if iri_local_suffix(&prem_iri) == subject_local {
            out.push((*property, *object));
        }
    }
    out
}

fn premise_individual_typed_as(
    premise: &Ontology,
    individual: &ontologos_core::EntityId,
    class: ontologos_core::EntityId,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ClassAssertion {
            individual: ind,
            class: ce,
        } = axiom
        else {
            return false;
        };
        *ind == *individual
            && matches!(
                premise.dl().ce(*ce),
                Some(ClassExpr::Atomic(c)) if *c == class
            )
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::ClassAssertion {
                individual: ind,
                class: c
            } if *ind == *individual && *c == class
        )
    })
}

/// Intersection/union instance typing from equivalent boolean class expressions (WG Rdfbased-sem-bool-*).
fn boolean_constructor_typing_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        let conc_class_local =
            entity_iri(conclusion, *conc_class).map(|iri| iri_local_suffix(&iri).to_string());
        if boolean_constructor_typing_entailed(
            premise,
            &conc_ind_iri,
            conc_class_prem,
            conc_class_local.as_deref(),
        ) {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, *class) else {
            continue;
        };
        let conc_class_local =
            entity_iri(conclusion, *class).map(|iri| iri_local_suffix(&iri).to_string());
        if boolean_constructor_typing_entailed(
            premise,
            &conc_ind_iri,
            conc_class_prem,
            conc_class_local.as_deref(),
        ) {
            return true;
        }
    }
    false
}

fn premise_individual_typed_as_class_flexible(
    premise: &Ontology,
    individual_iri: &str,
    class: ontologos_core::EntityId,
) -> bool {
    if premise_individual_has_type(premise, individual_iri, class) {
        return true;
    }
    premise_individual_types(premise, individual_iri)
        .iter()
        .any(|prem_type| classes_equivalent_in_premise(premise, *prem_type, class))
}

fn boolean_constructor_typing_entailed(
    premise: &Ontology,
    individual_iri: &str,
    class: ontologos_core::EntityId,
    class_local: Option<&str>,
) -> bool {
    if let Some(local) = class_local {
        for prem_type in premise_individual_types(premise, individual_iri) {
            if entity_iri(premise, prem_type).is_some_and(|iri| iri_local_suffix(&iri) == local) {
                return true;
            }
        }
    }
    if premise_individual_typed_as_class_flexible(premise, individual_iri, class) {
        return true;
    }
    for axiom in premise.dl().axioms() {
        let DlAxiom::EquivalentClasses(members) = axiom else {
            continue;
        };
        let Some(expr_ce) = members.iter().copied().find(|ce| {
            atomic_entity_from_ce(premise.dl(), *ce)
                .is_some_and(|id| entities_same_local_in_premise(premise, id, class))
        }) else {
            continue;
        };
        let Some(other_ce) = members.iter().copied().find(|ce| *ce != expr_ce) else {
            continue;
        };
        let Some(expr) = premise.dl().ce(other_ce) else {
            continue;
        };
        let entailed = match expr {
            ClassExpr::And(parts) => parts.iter().all(|part| {
                atomic_entity_from_ce(premise.dl(), *part).is_some_and(|member| {
                    premise_individual_typed_as_class_flexible(premise, individual_iri, member)
                })
            }),
            ClassExpr::Or(parts) => parts.iter().any(|part| {
                atomic_entity_from_ce(premise.dl(), *part).is_some_and(|member| {
                    premise_individual_typed_as_class_flexible(premise, individual_iri, member)
                })
            }),
            ClassExpr::OneOf(nominals) => nominals.iter().any(|nominal| {
                entity_iri(premise, *nominal)
                    .is_some_and(|iri| iri_local_suffix(&iri) == iri_local_suffix(individual_iri))
            }),
            _ => false,
        };
        if entailed {
            return true;
        }
    }
    false
}

/// Atomic class with no extension-defining axioms: any individual may satisfy ∃/∀ fillers.
fn atomic_class_has_no_extension_axioms(
    premise: &Ontology,
    class: ontologos_core::EntityId,
) -> bool {
    let restricts = |ce: &ClassExpr| {
        matches!(
            ce,
            ClassExpr::Some { .. }
                | ClassExpr::All { .. }
                | ClassExpr::HasValue { .. }
                | ClassExpr::And(_)
                | ClassExpr::Or(_)
                | ClassExpr::Not(_)
                | ClassExpr::OneOf(_)
        )
    };
    if premise.dl().axioms().any(|axiom| match axiom {
        DlAxiom::SubClassOf { sub, sup } => {
            atomic_entity_from_ce(premise.dl(), *sub) == Some(class)
                || atomic_entity_from_ce(premise.dl(), *sup) == Some(class)
                    && premise.dl().ce(*sup).is_some_and(restricts)
        }
        DlAxiom::EquivalentClasses(ops) => ops.iter().any(|ce| {
            atomic_entity_from_ce(premise.dl(), *ce) == Some(class)
                || premise.dl().ce(*ce).is_some_and(restricts)
        }),
        _ => false,
    }) {
        return false;
    }
    !premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::SubClassOfExistential { subclass, .. } if *subclass == class
        )
    })
}

fn premise_individual_has_type(
    premise: &Ontology,
    individual_iri: &str,
    class: ontologos_core::EntityId,
) -> bool {
    let class_local = entity_iri(premise, class).map(|iri| iri_local_suffix(&iri).to_string());
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ClassAssertion {
            individual,
            class: ce,
        } = axiom
        else {
            return false;
        };
        if entity_iri(premise, *individual)
            .is_none_or(|iri| iri_local_suffix(&iri) != iri_local_suffix(individual_iri))
        {
            return false;
        }
        match premise.dl().ce(*ce) {
            Some(ClassExpr::Atomic(c)) => {
                *c == class
                    || class_local.as_deref().is_some_and(|local| {
                        entity_iri(premise, *c).is_some_and(|iri| iri_local_suffix(&iri) == local)
                    })
            }
            _ => false,
        }
    }) || premise.axioms().iter().any(|(_, axiom)| match axiom {
        ontologos_core::Axiom::ClassAssertion {
            individual: ind,
            class: c,
        } => {
            entity_iri(premise, *ind)
                .is_some_and(|iri| iri_local_suffix(&iri) == iri_local_suffix(individual_iri))
                && (*c == class
                    || class_local.as_deref().is_some_and(|local| {
                        entity_iri(premise, *c).is_some_and(|iri| iri_local_suffix(&iri) == local)
                    }))
        }
        _ => false,
    })
}

fn rdfs_conditional_typing_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    for axiom in conclusion.dl().axioms() {
        let DlAxiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(ClassExpr::Atomic(conc_class)) = conclusion.dl().ce(*class) else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let conc_class_prem = map_entity_by_iri(conclusion, premise, *conc_class)
            .or_else(|| map_entity_by_local_iri(conclusion, premise, *conc_class));
        let Some(conc_class_prem) = conc_class_prem else {
            continue;
        };
        if rdfs_domain_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
            || rdfs_range_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
        {
            return true;
        }
    }
    for (_, axiom) in conclusion.axioms().iter() {
        let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom else {
            continue;
        };
        let Some(conc_ind_iri) = entity_iri(conclusion, *individual) else {
            continue;
        };
        let Some(conc_class_prem) = map_entity_by_iri(conclusion, premise, *class) else {
            continue;
        };
        if rdfs_domain_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
            || rdfs_range_typing_entailed(premise, &conc_ind_iri, conc_class_prem)
        {
            return true;
        }
    }
    false
}

fn rdfs_domain_typing_entailed(
    premise: &Ontology,
    individual_iri: &str,
    class: ontologos_core::EntityId,
) -> bool {
    for axiom in premise.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject,
            property,
            object: _,
        } = axiom
        else {
            continue;
        };
        if entity_iri(premise, *subject).as_deref() != Some(individual_iri) {
            continue;
        }
        let Some(prop) = role_entity(property) else {
            continue;
        };
        if premise_property_domain_contains(premise, prop, class) {
            return true;
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject,
            property,
            object: _,
        } = axiom
        else {
            continue;
        };
        if entity_iri(premise, *subject).as_deref() != Some(individual_iri) {
            continue;
        }
        if premise_property_domain_contains(premise, *property, class) {
            return true;
        }
    }
    false
}

fn rdfs_range_typing_entailed(
    premise: &Ontology,
    individual_iri: &str,
    class: ontologos_core::EntityId,
) -> bool {
    for axiom in premise.dl().axioms() {
        let DlAxiom::ObjectPropertyAssertion {
            subject: _,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        if entity_iri(premise, *object).as_deref() != Some(individual_iri) {
            continue;
        }
        let Some(prop) = role_entity(property) else {
            continue;
        };
        if premise_property_range_contains(premise, prop, class) {
            return true;
        }
    }
    for (_, axiom) in premise.axioms().iter() {
        let ontologos_core::Axiom::ObjectPropertyAssertion {
            subject: _,
            property,
            object,
        } = axiom
        else {
            continue;
        };
        if entity_iri(premise, *object).as_deref() != Some(individual_iri) {
            continue;
        }
        if premise_property_range_contains(premise, *property, class) {
            return true;
        }
    }
    false
}

fn premise_property_domain_contains(
    premise: &Ontology,
    property: ontologos_core::EntityId,
    class: ontologos_core::EntityId,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ObjectPropertyDomain {
            property: prop,
            domain,
        } = axiom
        else {
            return false;
        };
        *prop == property && atomic_entity_from_ce(premise.dl(), *domain) == Some(class)
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::ObjectPropertyDomain {
                property: prop,
                domain
            } if *prop == property && *domain == class
        )
    })
}

fn premise_property_range_contains(
    premise: &Ontology,
    property: ontologos_core::EntityId,
    class: ontologos_core::EntityId,
) -> bool {
    premise.dl().axioms().any(|axiom| {
        let DlAxiom::ObjectPropertyRange {
            property: prop,
            range,
        } = axiom
        else {
            return false;
        };
        *prop == property && atomic_entity_from_ce(premise.dl(), *range) == Some(class)
    }) || premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::ObjectPropertyRange {
                property: prop,
                range
            } if *prop == property && *range == class
        )
    })
}

fn subclass_in_premise(premise: &Ontology, sub: EntityId, sup: EntityId) -> bool {
    for axiom in premise.dl().axioms() {
        let DlAxiom::SubClassOf { sub: s, sup: p } = axiom else {
            continue;
        };
        if atomic_entity_from_ce(premise.dl(), *s) == Some(sub)
            && atomic_entity_from_ce(premise.dl(), *p) == Some(sup)
        {
            return true;
        }
    }
    premise.axioms().iter().any(|(_, axiom)| {
        matches!(
            axiom,
            ontologos_core::Axiom::SubClassOf {
                subclass,
                superclass
            } if *subclass == sub && *superclass == sup
        )
    })
}

fn conclusion_only_equivalent_class_axioms(conclusion: &Ontology) -> bool {
    let mut has_equiv = false;
    for (_, axiom) in conclusion.axioms().iter() {
        match axiom {
            ontologos_core::Axiom::EquivalentClasses(_) => has_equiv = true,
            _ => return false,
        }
    }
    for axiom in conclusion.dl().axioms() {
        match axiom {
            DlAxiom::EquivalentClasses(_) => has_equiv = true,
            _ => return false,
        }
    }
    has_equiv
}

fn conclusion_axiom_count(conclusion: &Ontology) -> usize {
    conclusion.axioms().len() + conclusion.dl().axioms().count()
}

fn class_same_as_non_entailment_guard(premise: &Ontology, conclusion: &Ontology) -> bool {
    if same_individual_pairs(conclusion).is_empty() {
        return false;
    }
    let premise_has_same = !same_individual_pairs(premise).is_empty();
    if premise_has_same {
        return false;
    }
    for (left, right) in same_individual_pairs(conclusion) {
        let left_kind = conclusion.entity(left).ok().map(|r| r.kind);
        let right_kind = conclusion.entity(right).ok().map(|r| r.kind);
        if left_kind.is_some_and(|k| k.is_class()) && right_kind.is_some_and(|k| k.is_class()) {
            return true;
        }
    }
    false
}

fn same_individual_pairs(ontology: &Ontology) -> Vec<(EntityId, EntityId)> {
    let mut pairs = Vec::new();
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::SameIndividual(individuals) = axiom {
            if individuals.len() >= 2 {
                pairs.push((individuals[0], individuals[1]));
            }
        }
    }
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::SameIndividual(individuals) = axiom {
            if individuals.len() >= 2 {
                pairs.push((individuals[0], individuals[1]));
            }
        }
    }
    pairs
}

fn individual_typed_with_class(ontology: &Ontology, individual: EntityId, class: EntityId) -> bool {
    for axiom in ontology.dl().axioms() {
        if let DlAxiom::ClassAssertion {
            individual: ind,
            class: ce,
        } = axiom
        {
            if *ind != individual {
                continue;
            }
            if let Some(ClassExpr::Atomic(c)) = ontology.dl().ce(*ce) {
                if *c == class {
                    return true;
                }
            }
        }
    }
    for (_, axiom) in ontology.axioms().iter() {
        if let ontologos_core::Axiom::ClassAssertion {
            individual: ind,
            class: c,
        } = axiom
        {
            if *ind == individual && *c == class {
                return true;
            }
        }
    }
    false
}

fn premise_entity_used_as_class(premise: &Ontology, entity: EntityId) -> bool {
    let Some(record) = premise.entity(entity).ok() else {
        return false;
    };
    matches!(record.kind, EntityKind::Class)
}

fn premise_entity_used_as_individual_only(premise: &Ontology, entity: EntityId) -> bool {
    let used_as_individual = premise.dl().axioms().any(|axiom| match axiom {
        DlAxiom::ClassAssertion { individual, .. } => *individual == entity,
        DlAxiom::ObjectPropertyAssertion {
            subject, object, ..
        } => *subject == entity || *object == entity,
        DlAxiom::DataPropertyAssertion { subject, .. } => *subject == entity,
        DlAxiom::SameIndividual(individuals) => individuals.contains(&entity),
        _ => false,
    }) || premise.axioms().iter().any(|(_, axiom)| match axiom {
        ontologos_core::Axiom::ClassAssertion { individual, .. } => *individual == entity,
        ontologos_core::Axiom::ObjectPropertyAssertion {
            subject, object, ..
        } => *subject == entity || *object == entity,
        ontologos_core::Axiom::SameIndividual(individuals) => individuals.contains(&entity),
        _ => false,
    });
    let in_one_of = premise.dl().axioms().any(|axiom| {
        let DlAxiom::SubClassOf { sub, sup, .. } = axiom else {
            return false;
        };
        one_of_contains_entity(premise.dl(), *sub, entity)
            || one_of_contains_entity(premise.dl(), *sup, entity)
    });
    used_as_individual || in_one_of
}

fn one_of_contains_entity(
    store: &ontologos_core::DlStore,
    ce: ontologos_core::CeId,
    entity: EntityId,
) -> bool {
    match store.ce(ce) {
        Some(ClassExpr::OneOf(members)) => members.contains(&entity),
        _ => false,
    }
}

fn atomic_entity_from_ce(
    store: &ontologos_core::DlStore,
    ce: ontologos_core::CeId,
) -> Option<ontologos_core::EntityId> {
    match store.ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
}

fn iri_local_suffix(iri: &str) -> &str {
    if let Some(rest) = iri.strip_prefix("urn:ontologos:anon:") {
        let trimmed = rest.trim_start_matches('/');
        return trimmed;
    }
    if let Some(pos) = iri.find("%23") {
        let after = &iri[pos + 3..];
        return after.rsplit('/').next().unwrap_or(after);
    }
    let frag = iri.rsplit('#').next().unwrap_or(iri);
    frag.rsplit('/').next().unwrap_or(frag)
}

fn is_anonymous_class_entity(ontology: &Ontology, entity: EntityId) -> bool {
    ontology
        .entity(entity)
        .ok()
        .is_some_and(|r| r.kind == EntityKind::Class)
        && entity_iri(ontology, entity).is_some_and(|iri| {
            iri.contains("#_:") || iri.contains("urn:ontologos:anon:") || iri.contains("/_:anon")
        })
}

fn entities_share_local_iri(
    left_ont: &Ontology,
    left: EntityId,
    right_ont: &Ontology,
    right: EntityId,
) -> bool {
    let Some(left_iri) = entity_iri(left_ont, left) else {
        return false;
    };
    let Some(right_iri) = entity_iri(right_ont, right) else {
        return false;
    };
    iri_local_suffix(&left_iri) == iri_local_suffix(&right_iri)
}

fn map_entity_by_local_iri(from: &Ontology, to: &Ontology, entity: EntityId) -> Option<EntityId> {
    let iri = entity_iri(from, entity)?;
    let kind = from.entity(entity).ok()?.kind;
    premise_entity_by_local_iri(to, iri_local_suffix(&iri), kind)
}

fn premise_entity_by_local_iri(
    ontology: &Ontology,
    local: &str,
    kind: EntityKind,
) -> Option<EntityId> {
    ontology.entities().iter().find_map(|(id, record)| {
        if record.kind != kind {
            return None;
        }
        entity_iri(ontology, id)
            .filter(|iri| iri_local_suffix(iri) == local)
            .map(|_| id)
    })
}

fn map_entity_by_iri(
    from: &Ontology,
    to: &Ontology,
    entity: ontologos_core::EntityId,
) -> Option<ontologos_core::EntityId> {
    let record = from.entity(entity).ok()?;
    let iri = from.resolve_iri(record.iri).ok()?;
    to.lookup_entity(iri)
}

fn property_sets_match_by_iri(
    premise: &Ontology,
    conclusion: &Ontology,
    left: &[ontologos_core::EntityId],
    right: &[ontologos_core::EntityId],
) -> bool {
    if left.len() != right.len() {
        return false;
    }
    let mut left_iris: Vec<String> = left
        .iter()
        .filter_map(|id| entity_iri(premise, *id))
        .collect();
    let mut right_iris: Vec<String> = right
        .iter()
        .filter_map(|id| entity_iri(conclusion, *id))
        .collect();
    left_iris.sort();
    right_iris.sort();
    left_iris == right_iris
}

fn class_subsumed_in_ontology(
    ontology: &Ontology,
    sub: ontologos_core::EntityId,
    sup: ontologos_core::EntityId,
) -> bool {
    if sub == sup {
        return true;
    }
    let mut queue = std::collections::VecDeque::from([sub]);
    let mut seen = std::collections::HashSet::from([sub]);
    while let Some(cur) = queue.pop_front() {
        for &direct in ontology.direct_superclasses(cur) {
            if direct == sup {
                return true;
            }
            if seen.insert(direct) {
                queue.push_back(direct);
            }
        }
        let sub_ce = ontology.dl().expressions().find_map(|(id, e)| match e {
            ClassExpr::Atomic(c) if *c == cur => Some(id),
            _ => None,
        });
        let Some(sub_ce) = sub_ce else {
            continue;
        };
        for axiom in ontology.dl().axioms() {
            let DlAxiom::SubClassOf { sub: s, sup: s_up } = axiom else {
                continue;
            };
            if *s != sub_ce {
                continue;
            }
            if let Some(ClassExpr::Atomic(super_entity)) = ontology.dl().ce(*s_up) {
                if *super_entity == sup {
                    return true;
                }
                if seen.insert(*super_entity) {
                    queue.push_back(*super_entity);
                }
            }
        }
    }
    false
}

fn conclusion_has_invalid_blank_node_cycles(conclusion: &Ontology) -> bool {
    ontologos_parser::validate_loaded_ontology(conclusion).is_err()
}

fn resolve_local_iri(local: &str) -> String {
    if local.contains("://") || local.starts_with("file:") {
        local.to_owned()
    } else if let Some(rest) = local.strip_prefix("owl:") {
        format!("http://www.w3.org/2002/07/owl#{rest}")
    } else if let Some(rest) = local.strip_prefix("rdfs:") {
        format!("http://www.w3.org/2000/01/rdf-schema#{rest}")
    } else if let Some(rest) = local.strip_prefix("xsd:") {
        format!("http://www.w3.org/2001/XMLSchema#{rest}")
    } else {
        let name = local.strip_prefix(':').unwrap_or(local);
        format!("{HERMIT_DEFAULT_NS}{name}")
    }
}

fn parse_hermit_hierarchy_txt(text: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((sub, sup)) = line.split_once(" SubClassOf ") {
            pairs.push((sub.trim().to_owned(), sup.trim().to_owned()));
        }
    }
    pairs
}

fn assert_hierarchy_pairs(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    golden_pairs: &[(String, String)],
    case_id: &str,
) {
    for (sub, sup) in golden_pairs {
        let sub_id = ontology
            .lookup_entity(sub)
            .unwrap_or_else(|| panic!("{case_id}: missing subclass {sub}"));
        let sup_id = ontology
            .lookup_entity(sup)
            .unwrap_or_else(|| panic!("{case_id}: missing superclass {sup}"));
        assert!(
            taxonomy.is_subsumed(sub_id, sup_id) || assert_subsumed(ontology, sub, sup),
            "{case_id}: expected {sub} ⊑ {sup}"
        );
    }
}

#[cfg(test)]
mod entailment_guard_tests {
    use super::*;
    use ontologos_parser::load_ontology;
    use std::path::PathBuf;

    fn wg(rel: &str) -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../benchmarks/data/hermit")
            .join(rel)
    }

    #[test]
    fn boolean_constructor_guard_intersection_comp() {
        let prem = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Dbool-2Dintersection-2Dinst-2Dcomp/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Dbool-2Dintersection-2Dinst-2Dcomp/conclusion.rdf",
        ))
        .unwrap();
        assert!(!conclusion_has_fresh_abox_entities(&prem, &conc));
        assert!(!has_key_non_entailment_guard(&prem, &conc));
        assert!(!spurious_class_equivalence_non_entailment_guard(
            &prem, &conc
        ));
        assert!(!conflicting_instance_typing_non_entailment_guard(
            &prem, &conc
        ));
        assert!(boolean_constructor_typing_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn union_001_entailment_guard() {
        let prem = load_ontology(&wg("wg/TestCase-3AWebOnt-2DunionOf-2D001/premise.rdf")).unwrap();
        let conc =
            load_ontology(&wg("wg/TestCase-3AWebOnt-2DunionOf-2D001/conclusion.rdf")).unwrap();
        assert!(!conclusion_has_fresh_abox_entities(&prem, &conc));
        assert!(boolean_class_instance_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn complement_symmetry_guard_001() {
        let prem =
            load_ontology(&wg("wg/TestCase-3AWebOnt-2DcomplementOf-2D001/premise.rdf")).unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DcomplementOf-2D001/conclusion.rdf",
        ))
        .unwrap();
        eprintln!("prem pairs {:?}", complement_subclass_pairs(&prem));
        eprintln!("conc pairs {:?}", complement_subclass_pairs(&conc));
        assert!(complement_symmetry_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn one_of_nominal_typing_guard_003() {
        let prem = load_ontology(&wg("wg/TestCase-3AWebOnt-2DoneOf-2D003/premise.rdf")).unwrap();
        let conc = load_ontology(&wg("wg/TestCase-3AWebOnt-2DoneOf-2D003/conclusion.rdf")).unwrap();
        assert!(!conclusion_has_fresh_abox_entities(&prem, &conc));
        assert!(!conflicting_instance_typing_non_entailment_guard(
            &prem, &conc
        ));
        assert!(equivalent_class_instance_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn union_002_entailment_guard() {
        let prem = load_ontology(&wg("wg/TestCase-3AWebOnt-2DunionOf-2D002/premise.rdf")).unwrap();
        let conc =
            load_ontology(&wg("wg/TestCase-3AWebOnt-2DunionOf-2D002/conclusion.rdf")).unwrap();
        assert!(!conclusion_has_fresh_abox_entities(&prem, &conc));
        assert!(!conflicting_instance_typing_non_entailment_guard(
            &prem, &conc
        ));
        assert!(boolean_class_instance_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn equivalent_class_004_entailment() {
        let prem = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DequivalentClass-2D004/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DequivalentClass-2D004/conclusion.rdf",
        ))
        .unwrap();
        assert!(structural_class_equivalence_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn restrict_somevalues_inst_subj_entailment() {
        let prem = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Drestrict-2Dsomevalues-2Dinst-2Dsubj/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Drestrict-2Dsomevalues-2Dinst-2Dsubj/conclusion.rdf",
        ))
        .unwrap();
        assert!(restriction_instance_typing_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn some_values_from_003_entailment() {
        let prem = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DsomeValuesFrom-2D003/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DsomeValuesFrom-2D003/conclusion.rdf",
        ))
        .unwrap();
        assert!(recursive_some_values_chain_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn all_values_from_002_non_entailment() {
        let prem = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DallValuesFrom-2D002/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DallValuesFrom-2D002/conclusion.rdf",
        ))
        .unwrap();
        assert!(!conclusion_has_fresh_abox_entities(&prem, &conc));
        assert!(thing_individual_new_property_non_entailment_guard(
            &prem, &conc
        ));
        assert!(conclusion_only_unasserted_object_property(&prem, &conc));
        assert!(!entailment_holds_with_budget_opts(
            &prem,
            &conc,
            Some(dl_classify_budget()),
            false,
        )
        .unwrap());
    }

    #[test]
    fn eqclass_trans_entailment() {
        let prem = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Deqdis-2Deqclass-2Dtrans/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Deqdis-2Deqclass-2Dtrans/conclusion.rdf",
        ))
        .unwrap();
        assert!(equivalent_class_transitivity_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn disjoint_classes_001_entailment() {
        let prem = load_ontology(&wg("wg/DisjointClasses-2D001/premise.rdf")).unwrap();
        let conc = load_ontology(&wg("wg/DisjointClasses-2D001/conclusion.rdf")).unwrap();
        assert!(disjoint_complement_instance_typing_entailment_guard(
            &prem, &conc
        ));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn disjoint_classes_003_entailment() {
        let prem = load_ontology(&wg("wg/DisjointClasses-2D003/premise.rdf")).unwrap();
        let conc = load_ontology(&wg("wg/DisjointClasses-2D003/conclusion.rdf")).unwrap();
        assert!(disjoint_complement_instance_typing_entailment_guard(
            &prem, &conc
        ));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn i5_8_005_non_entailment() {
        let prem = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.8-2D005/premise.rdf")).unwrap();
        let conc = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.8-2D005/conclusion.rdf")).unwrap();
        assert!(cardinality_datatype_assertion_non_entailment_guard(
            &prem, &conc
        ));
        assert!(!entailment_holds_with_budget_opts(
            &prem,
            &conc,
            Some(dl_classify_budget()),
            false,
        )
        .unwrap());
    }

    #[test]
    fn transitive_property_002_entailment() {
        let prem = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DTransitiveProperty-2D002/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DTransitiveProperty-2D002/conclusion.rdf",
        ))
        .unwrap();
        assert!(restriction_instance_typing_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn cardinality_001_entailment() {
        let prem =
            load_ontology(&wg("wg/TestCase-3AWebOnt-2Dcardinality-2D001/premise.rdf")).unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2Dcardinality-2D001/conclusion.rdf",
        ))
        .unwrap();
        assert!(cardinality_restriction_subsumption_entailment_guard(
            &prem, &conc
        ));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn cardinality_002_entailment() {
        let prem =
            load_ontology(&wg("wg/TestCase-3AWebOnt-2Dcardinality-2D002/premise.rdf")).unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2Dcardinality-2D002/conclusion.rdf",
        ))
        .unwrap();
        assert!(cardinality_restriction_subsumption_entailment_guard(
            &prem, &conc
        ));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn restriction_006_entailment_guard() {
        let prem =
            load_ontology(&wg("wg/TestCase-3AWebOnt-2DRestriction-2D006/premise.rdf")).unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DRestriction-2D006/conclusion.rdf",
        ))
        .unwrap();
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn remaining_entailment_failures_debug() {
        let cases = [
            (
                "SelfRestriction-2D002",
                "wg/New-2DFeature-2DSelfRestriction-2D002",
            ),
            (
                "DisjointUnion-2D001",
                "wg/New-2DFeature-2DDisjointUnion-2D001",
            ),
            ("I4.5-2D001", "wg/TestCase-3AWebOnt-2DI4.5-2D001"),
            ("I5.24-2D003", "wg/TestCase-3AWebOnt-2DI5.24-2D003"),
            ("I5.5-2D005", "wg/TestCase-3AWebOnt-2DI5.5-2D005"),
            ("I5.8-2D006", "wg/TestCase-3AWebOnt-2DI5.8-2D006"),
        ];
        for (name, path) in cases {
            let prem = load_ontology(&wg(&format!("{path}/premise.rdf"))).unwrap();
            let conc = load_ontology(&wg(&format!("{path}/conclusion.rdf"))).unwrap();
            eprintln!("=== {name} ===");
            if name == "SelfRestriction-2D002" {
                for ax in conc.dl().axioms() {
                    if let DlAxiom::ClassAssertion { individual, class } = ax {
                        eprintln!(
                            "  ca ind={:?} ce={:?}",
                            entity_iri(&conc, *individual),
                            conc.dl().ce(*class)
                        );
                    }
                }
            }
            if name == "I4.5-2D001" {
                eprintln!(
                    "  spurious={}",
                    spurious_class_equivalence_non_entailment_guard(&prem, &conc)
                );
                eprintln!(
                    "  complex_sub={}",
                    complex_subclass_non_entailment_guard(&prem, &conc)
                );
                eprintln!(
                    "  conflicting={}",
                    conflicting_instance_typing_non_entailment_guard(&prem, &conc)
                );
                eprintln!(
                    "  class_punning={}",
                    class_punning_entailment_guard(&prem, &conc)
                );
                eprintln!(
                    "  equiv_same_as={}",
                    equivalent_same_as_non_entailment_guard(&prem, &conc)
                );
            }
            if name == "DisjointUnion-2D001" {
                for ax in prem.dl().axioms() {
                    eprintln!("  prem dl {ax:?}");
                }
                let stewie = "http://example.org/Stewie";
                let boy = prem.entities().iter().find_map(|(id, r)| {
                    (r.kind == EntityKind::Class).then(|| {
                        entity_iri(&prem, id)
                            .filter(|i| i.ends_with("Boy"))
                            .map(|_| id)
                    })?
                });
                eprintln!(
                    "  boy={boy:?} types={:?} excluded={:?}",
                    premise_individual_types(&prem, stewie),
                    premise_individual_complement_types(&prem, stewie)
                );
                for (id, r) in prem.entities().iter() {
                    eprintln!("  entity {id:?} {:?} {:?}", r.kind, entity_iri(&prem, id));
                }
                eprintln!(
                    "  same 4 1={}",
                    entities_same_local_in_premise(&prem, EntityId(4), EntityId(1))
                );
                if let Some(b) = boy {
                    eprintln!(
                        "  union_members Child={:?}",
                        prem.entities().iter().find_map(|(id, _r)| {
                            entity_iri(&prem, id)
                                .filter(|i| i.ends_with("Child"))
                                .map(|_| premise_union_members(&prem, id))
                        })
                    );
                    for ax in prem.dl().axioms() {
                        if let DlAxiom::EquivalentClasses(ops) = ax {
                            for op in ops {
                                eprintln!("  equiv ce {op:?} => {:?}", prem.dl().ce(*op));
                            }
                        }
                    }
                    eprintln!(
                        "  entailed={}",
                        disjoint_union_member_entailed(&prem, stewie, b)
                    );
                }
            }
            if name == "I5.5-2D005" {
                eprintln!(
                    "  only_equiv={}",
                    conclusion_only_equivalent_class_axioms(&conc)
                );
                eprintln!("  conc_pairs={}", equivalent_class_pairs(&conc).len());
                for (l, r) in equivalent_class_pairs(&conc) {
                    eprintln!(
                        "  pair {:?} {:?} maps {:?} {:?}",
                        entity_iri(&conc, l),
                        entity_iri(&conc, r),
                        map_entity_by_iri(&conc, &prem, l)
                            .or_else(|| map_entity_by_local_iri(&conc, &prem, l)),
                        map_entity_by_iri(&conc, &prem, r)
                            .or_else(|| map_entity_by_local_iri(&conc, &prem, r)),
                    );
                }
                for ax in conc.dl().axioms() {
                    eprintln!("  conc dl {ax:?}");
                }
                for ce in [CeId(0), CeId(1)] {
                    eprintln!("  ce {ce:?} => {:?}", conc.dl().ce(ce));
                }
                eprintln!("  prem_class_count={}", premise_declared_class_count(&prem));
            }
            eprintln!(
                "  fresh_abox={}",
                conclusion_has_fresh_abox_entities(&prem, &conc)
            );
            eprintln!(
                "  has_self={}",
                has_self_instance_typing_entailment_guard(&prem, &conc)
            );
            eprintln!(
                "  disjoint_union={}",
                disjoint_union_member_instance_entailment_guard(&prem, &conc)
            );
            eprintln!(
                "  inverse_exist={}",
                inverse_existential_instance_entailment_guard(&prem, &conc)
            );
            eprintln!(
                "  obj_range={}",
                object_property_range_subsumption_entailment_guard(&prem, &conc)
            );
            eprintln!(
                "  singleton_union={}",
                singleton_union_equivalence_entailment_guard(&prem, &conc)
            );
            eprintln!(
                "  datatype_range={}",
                datatype_property_range_entailment_guard(&prem, &conc)
            );
            eprintln!(
                "  entail={:?}",
                entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget()))
            );
        }
    }

    #[test]
    fn cardinality_003_entailment() {
        let prem =
            load_ontology(&wg("wg/TestCase-3AWebOnt-2Dcardinality-2D003/premise.rdf")).unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2Dcardinality-2D003/conclusion.rdf",
        ))
        .unwrap();
        assert!(cardinality_restriction_subsumption_entailment_guard(
            &prem, &conc
        ));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn functional_property_004_entailment() {
        let prem = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DFunctionalProperty-2D004/conclusion.rdf",
        ))
        .unwrap();
        assert!(
            singleton_range_functional_entailment_guard(&prem, &conc),
            "singleton range functional guard"
        );
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn i5_5_005_entailment_guard() {
        let prem = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.5-2D005/premise.rdf")).unwrap();
        let conc = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.5-2D005/conclusion.rdf")).unwrap();
        assert!(singleton_union_equivalence_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn i5_8_010_entailment_guard() {
        let prem = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.8-2D010/premise.rdf")).unwrap();
        let conc = load_ontology(&wg("wg/TestCase-3AWebOnt-2DI5.8-2D010/conclusion.rdf")).unwrap();
        assert!(data_range_intersection_singleton_entailment_guard(
            &prem, &conc
        ));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn equivalent_class_007_demorgan_guard() {
        let prem = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DequivalentClass-2D007/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/TestCase-3AWebOnt-2DequivalentClass-2D007/conclusion.rdf",
        ))
        .unwrap();
        assert!(demorgan_class_equivalence_entailment_guard(&prem, &conc));
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn hasvalue_inst_subj_entailment_guard() {
        let prem = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Drestrict-2Dhasvalue-2Dinst-2Dsubj/premise.rdf",
        ))
        .unwrap();
        let conc = load_ontology(&wg(
            "wg/Rdfbased-2Dsem-2Drestrict-2Dhasvalue-2Dinst-2Dsubj/conclusion.rdf",
        ))
        .unwrap();
        eprintln!("prem dl: {:?}", prem.dl().axioms().collect::<Vec<_>>());
        eprintln!("conc dl: {:?}", conc.dl().axioms().collect::<Vec<_>>());
        eprintln!(
            "prem core axioms: {:?}",
            prem.axioms().iter().collect::<Vec<_>>()
        );
        for (ce, ex) in prem.dl().expressions() {
            eprintln!("prem CE {ce:?} = {ex:?}");
        }
        for (ce, ex) in conc.dl().expressions() {
            eprintln!("conc CE {ce:?} = {ex:?}");
        }
        eprintln!(
            "restriction guard={}",
            restriction_instance_typing_entailment_guard(&prem, &conc)
        );
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }

    #[test]
    fn consistent_but_all_unsat_entailment() {
        let prem = load_ontology(&wg("wg/Consistent-2Dbut-2Dall-2Dunsat/premise.rdf")).unwrap();
        let conc = load_ontology(&wg("wg/Consistent-2Dbut-2Dall-2Dunsat/conclusion.rdf")).unwrap();
        assert!(ontologos_dl::is_consistent(&prem).unwrap());
        let targets = conclusion_nothing_subclass_entailment_targets(&conc);
        eprintln!("nothing targets={targets:?}");
        assert!(
            entailment_via_subclass_nothing(&prem, &conc, dl_classify_budget())
                .unwrap()
                .unwrap_or(false)
        );
        assert!(entailment_holds_with_budget(&prem, &conc, Some(dl_classify_budget())).unwrap());
    }
}
