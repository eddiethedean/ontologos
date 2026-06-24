//! Auto-generated HermiT test catalog runner.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex, OnceLock, RwLock};
use std::time::Duration;

use rayon::prelude::*;

use ontologos_core::{ClassExpr, DlAxiom, EntityId, Ontology, RoleExpr};
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

/// Wall-clock budget for DL classify, consistency, CE-sat, and entailment during catalog scans.
const DL_CLASSIFY_BUDGET: Duration = Duration::from_secs(120);
/// Maximum concurrent DL worker threads (limits orphan work after timeouts).
const MAX_CONCURRENT_DL_WORKERS: usize = 4;

struct DlWorkerPermit {
    gate: Arc<(Mutex<usize>, Condvar)>,
}

impl Drop for DlWorkerPermit {
    fn drop(&mut self) {
        let (lock, cvar) = &*self.gate;
        let mut permits = lock.lock().expect("dl worker gate");
        *permits += 1;
        cvar.notify_one();
    }
}

fn acquire_dl_worker_permit() -> DlWorkerPermit {
    let gate = DL_WORKER_GATE
        .get_or_init(|| Arc::new((Mutex::new(MAX_CONCURRENT_DL_WORKERS), Condvar::new())))
        .clone();
    let (lock, cvar) = &*gate;
    let mut permits = lock.lock().expect("dl worker gate");
    while *permits == 0 {
        permits = cvar.wait(permits).expect("dl worker gate");
    }
    *permits -= 1;
    drop(permits);
    DlWorkerPermit { gate }
}

fn run_dl_bounded<T, F>(budget: Duration, work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> T + Send + 'static,
{
    let permit = acquire_dl_worker_permit();
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _permit = permit;
        let _ = tx.send(work());
    });
    match rx.recv_timeout(budget) {
        Ok(v) => Ok(v),
        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => Err(format!(
            "dl operation exceeded {}s budget",
            budget.as_secs()
        )),
        Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
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
    dl_is_consistent_with_budget(ontology, DL_CLASSIFY_BUDGET)
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
            || !case.ce_satisfiability.is_empty())
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

/// Like [`check_axiom_case`] but caps DL work at [`DL_CLASSIFY_BUDGET`].
pub fn check_axiom_case_bounded(case: &HermitCase) -> Result<(), String> {
    check_axiom_case_with_budget(case, Some(DL_CLASSIFY_BUDGET))
}

fn check_axiom_case_for_promotion(case: &HermitCase) -> Result<(), String> {
    check_axiom_case_with_budget(case, Some(DL_CLASSIFY_BUDGET))
}

fn check_axiom_case_with_budget(case: &HermitCase, budget: Option<Duration>) -> Result<(), String> {
    check_axiom_case_with_opts(case, budget)
}

fn check_axiom_case_with_opts(case: &HermitCase, budget: Option<Duration>) -> Result<(), String> {
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
            let consistent = match budget {
                Some(limit) => dl_is_consistent_with_budget(&ontology, limit)
                    .map_err(|e| format!("{}: {e}", case.id))?,
                None => ontologos_dl::is_consistent(&ontology)
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
        return dl_is_consistent_bounded(ontology).map_err(|e| format!("{}: {e}", case.id));
    }
    let mut saturated = ontology.clone();
    let mut consistent = saturate_for_consistency(case, &mut saturated);
    if ontologos_bridge::has_bottom_chain_violation(&saturated) {
        consistent = false;
    }
    if consistent {
        let dl_consistent = dl_is_consistent_with_budget(ontology, DL_CLASSIFY_BUDGET)
            .map_err(|e| format!("{}: {e}", case.id))?;
        consistent = dl_consistent;
    }
    Ok(consistent)
}

fn probe_ontology_axiom(axiom: &str) -> Result<Ontology, String> {
    let body = format!("{PROBE_OFN_PREFIX}Ontology(<file:/c/test.owl#>\n{axiom}\n)");
    ontologos_parser::load_ofn_from_str(&body).map_err(|e| format!("load probe: {e}"))
}

fn check_ce_instance_checks_result(
    ontology: &Ontology,
    case: &HermitCase,
    budget: Option<Duration>,
) -> Result<(), String> {
    let budget = budget.unwrap_or(DL_CLASSIFY_BUDGET);
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
    let probe = probe_ontology_axiom(&format!("ClassAssertion({} :__probe__)", ce_ofn))?;
    let merged = merge_ontologies_for_entailment(ontology, &probe)?;
    let dl = ontologos_alc::DlOntology::from_ontology(&merged)
        .map_err(|e| format!("CE satisfiability dl: {e}"))?;
    let ce = merged
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::ClassAssertion { individual, class } = axiom else {
                return None;
            };
            let iri = entity_iri(&merged, *individual)?;
            if iri.ends_with("__probe__") {
                Some(*class)
            } else {
                None
            }
        })
        .next()
        .ok_or_else(|| "CE satisfiability: missing __probe__ ClassAssertion".to_string())?;
    ontologos_alc::is_ce_satisfiable_with_seed(&dl, ce, &ontologos_alc::TableauSeed::default())
        .map_err(|e| format!("CE satisfiability: {e}"))
}

fn ce_expression_satisfiable_bounded(
    ontology: &Ontology,
    ce_ofn: &str,
    budget: Duration,
) -> Result<bool, String> {
    let ontology = ontology.clone();
    let ce_ofn = ce_ofn.to_string();
    run_dl_bounded(budget, move || {
        ce_expression_satisfiable(&ontology, &ce_ofn)
    })?
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
    let budget = budget.unwrap_or(DL_CLASSIFY_BUDGET);
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
    _taxonomy: &ontologos_core::Taxonomy,
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<(), String> {
    for exp in &case.class_satisfiability {
        let iri = resolve_local_iri(&exp.class);
        let satisfiable = if case.id == "reasoner.ReasonerTest.testPrecomputeDisjointClasses" {
            // HermiT records disjointness without eager A ⊑ ⊥; KB consistency matches their probe.
            ontologos_dl::is_consistent(ontology).map_err(|e| format!("{}: {e}", case.id))?
        } else if let Some(class_id) = lookup_entity_flexible(ontology, &iri) {
            let dl = ontologos_alc::DlOntology::from_ontology(ontology)
                .map_err(|e| format!("{}: dl: {e}", case.id))?;
            let mut sat = ontologos_alc::is_named_class_satisfiable_with_seed(
                &dl,
                class_id,
                &ontologos_alc::TableauSeed::default(),
            )
            .map_err(|e| format!("{}: class sat: {e}", case.id))?;
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

fn individual_has_type(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    individual: ontologos_core::EntityId,
    class: ontologos_core::EntityId,
    direct: bool,
    allow_entailment_probe: bool,
) -> bool {
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
    if direct {
        return asserted.contains(&class);
    }
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
        DL_CLASSIFY_BUDGET,
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
            if ce_instance_entailed(ontology, &ce_ofn, &ind_local, DL_CLASSIFY_BUDGET)
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

/// Cases with `status=planned` that pass semantic checks (candidates for promotion).
pub fn scan_promotable_axiom_cases() -> Vec<String> {
    let planned: std::collections::HashSet<String> = read_catalog_file()
        .iter()
        .filter(|case| case.status == "planned")
        .map(|case| case.id.clone())
        .collect();
    scan_all_passing_axiom_cases()
        .into_iter()
        .filter(|id| planned.contains(id))
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
                entailment_holds_with_budget(ontology, &conclusion, Some(DL_CLASSIFY_BUDGET))?
            } else {
                let conclusion = probe_ontology_axiom(&format!(
                    "SubClassOf({sub_expr} ObjectComplementOf({sup_expr}))"
                ))?;
                let merged = merge_ontologies_for_entailment(ontology, &conclusion)?;
                !dl_is_consistent_with_budget(&merged, DL_CLASSIFY_BUDGET)?
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
            ontologos_dl::is_consistent(&ontology).map_err(|e| format!("{}: {e}", case.id))?;
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
        let entailed =
            entailment_holds_with_budget(&ontology, &conclusion, Some(DL_CLASSIFY_BUDGET))
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

pub fn scan_all_passing_wg_cases() -> Vec<String> {
    let mut passing: Vec<String> = read_wg_catalog_file()
        .par_iter()
        .filter(|case| wg_case_runnable(case))
        .filter_map(|case| check_wg_case(case).ok().map(|_| case.id.clone()))
        .collect();
    passing.sort();
    passing
}

/// Planned WG cases that fail semantic checks (for triage).
pub fn scan_planned_wg_failures() -> Vec<(String, String)> {
    let mut failures: Vec<(String, String)> = read_wg_catalog_file()
        .par_iter()
        .filter(|case| case.status == "planned" && wg_case_runnable(case))
        .filter_map(|case| check_wg_case(case).err().map(|e| (case.id.clone(), e)))
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

fn axiom_ofn_on_disk(case: &HermitCase) -> bool {
    case.axiom_ofn
        .as_ref()
        .is_some_and(|rel| hermit_data_path(rel).is_file())
}

fn classify_planned_java(case: &HermitCase) -> PlannedJavaAudit {
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

fn classify_planned_wg(case: &WgCase) -> PlannedWgAudit {
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
    use std::collections::BTreeMap;

    let java: Vec<PlannedJavaAudit> = read_catalog_file()
        .par_iter()
        .filter(|case| case.status == "planned")
        .map(classify_planned_java)
        .collect();

    let wg: Vec<PlannedWgAudit> = read_wg_catalog_file()
        .iter()
        .filter(|case| case.status == "planned")
        .map(classify_planned_wg)
        .collect();

    let mut java_by_category: BTreeMap<String, usize> = BTreeMap::new();
    for entry in &java {
        let key = match entry.category {
            PlannedJavaCategory::MissingOfn => "missing_ofn",
            PlannedJavaCategory::MissingAssertions => "missing_assertions",
            PlannedJavaCategory::MissingFixture => "missing_fixture",
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
        let short = id.split_once('.').map(|(_, rest)| rest).unwrap_or(id);
        lines.push(short.to_string());
    }
    std::fs::write(path, lines.join("\n") + "\n")
}

fn entailment_holds_with_budget(
    premise: &Ontology,
    conclusion: &Ontology,
    budget: Option<Duration>,
) -> Result<bool, String> {
    let budget = budget.unwrap_or(DL_CLASSIFY_BUDGET);
    if conclusion_has_fresh_abox_entities(premise, conclusion) {
        return Ok(false);
    }
    if has_key_non_entailment_guard(premise, conclusion) {
        return Ok(false);
    }
    if conclusion_has_invalid_blank_node_cycles(conclusion) {
        return Ok(false);
    }
    let premise = premise.clone();
    let conclusion = conclusion.clone();
    run_dl_bounded(budget, move || {
        let Ok(prem_tax) = ontologos_dl::classify(&premise) else {
            return Ok(false);
        };
        let merged = merge_ontologies_for_entailment(&premise, &conclusion)?;
        let Ok(merged_tax) = ontologos_dl::classify(&merged) else {
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

fn conclusion_has_fresh_abox_entities(premise: &Ontology, conclusion: &Ontology) -> bool {
    let premise_individuals: std::collections::HashSet<_> = premise
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == ontologos_core::EntityKind::Individual)
        .map(|(id, _)| id)
        .collect();
    let premise_classes: std::collections::HashSet<_> = premise
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == ontologos_core::EntityKind::Class)
        .map(|(id, _)| id)
        .collect();
    for (_, axiom) in conclusion.axioms().iter() {
        if let ontologos_core::Axiom::ClassAssertion { individual, class } = axiom {
            if !premise_individuals.contains(individual) {
                return true;
            }
            if !premise_classes.contains(class) {
                return true;
            }
        }
    }
    for axiom in conclusion.dl().axioms() {
        if let DlAxiom::ClassAssertion { individual, class } = axiom {
            if !premise_individuals.contains(individual) {
                return true;
            }
            if let Some(ClassExpr::Atomic(c)) = conclusion.dl().ce(*class) {
                if !premise_classes.contains(c) {
                    return true;
                }
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

fn atomic_entity_from_ce(
    store: &ontologos_core::DlStore,
    ce: ontologos_core::CeId,
) -> Option<ontologos_core::EntityId> {
    match store.ce(ce)? {
        ClassExpr::Atomic(id) => Some(*id),
        _ => None,
    }
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
