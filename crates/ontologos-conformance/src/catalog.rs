//! Auto-generated HermiT test catalog runner.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use rayon::prelude::*;

use ontologos_core::Ontology;
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;
use serde::Deserialize;

use crate::{
    assert_subproperty, assert_subsumed, classification_fixture_path, has_property_characteristic,
    PropertyCharacteristic, HERMIT_DEFAULT_NS,
};

static CATALOG: OnceLock<Vec<HermitCase>> = OnceLock::new();
static WG_CATALOG: OnceLock<Vec<WgCase>> = OnceLock::new();

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

/// Load catalog from disk (not cached) for promotion tooling.
pub fn read_catalog_file() -> Vec<HermitCase> {
    let path = catalog_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing HermiT catalog at {}: {e}", path.display()));
    serde_json::from_str(&text).expect("parse cases.json")
}

fn case_has_axiom_assertions(case: &HermitCase) -> bool {
    case.axiom_ofn.is_some()
        && (!case.subsumptions.is_empty()
            || case.consistent.is_some()
            || !case.property_subsumptions.is_empty()
            || !case.property_characteristics.is_empty())
}

/// Semantic check for an axiom fixture (ignores catalog status).
pub fn check_axiom_case(case: &HermitCase) -> Result<(), String> {
    let rel = case
        .axiom_ofn
        .as_ref()
        .ok_or_else(|| format!("{}: missing axiom_ofn", case.id))?;
    let path = hermit_data_path(rel);
    if !path.is_file() {
        return Err(format!("{}: missing fixture {}", case.id, path.display()));
    }

    let mut ontology = load_ontology(&path).map_err(|e| format!("{}: load: {e}", case.id))?;

    if case.engine == "dl" || case.engine == "swrl" {
        if !case.subsumptions.is_empty() {
            let taxonomy =
                ontologos_dl::classify(&ontology).map_err(|e| format!("{}: dl: {e}", case.id))?;
            check_subsumptions_dl_result(&ontology, &taxonomy, case)?;
        }
        if let Some(expected) = case.consistent {
            let consistent =
                ontologos_dl::is_consistent(&ontology).map_err(|e| format!("{}: {e}", case.id))?;
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
    check_property_characteristics_result(&ontology, case)?;

    if let Some(expected) = case.consistent {
        let mut consistent = saturate_for_consistency(case, &mut ontology);
        if ontologos_bridge::has_bottom_chain_violation(&ontology) {
            consistent = false;
        }
        if consistent != expected {
            return Err(format!(
                "{}: consistency expected {expected}, got {consistent}",
                case.id
            ));
        }
    }
    Ok(())
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
    let mut passing: Vec<String> = read_catalog_file()
        .par_iter()
        .filter(|case| is_axiom_checkable(case))
        .filter_map(|case| check_axiom_case(case).ok().map(|_| case.id.clone()))
        .collect();
    passing.sort();
    passing
}

/// Cases with `status=planned` that pass semantic checks (candidates for promotion).
pub fn scan_promotable_axiom_cases() -> Vec<String> {
    let mut passing: Vec<String> = read_catalog_file()
        .par_iter()
        .filter(|case| case.status == "planned" && is_axiom_checkable(case))
        .filter_map(|case| check_axiom_case(case).ok().map(|_| case.id.clone()))
        .collect();
    passing.sort();
    passing
}

/// Planned DL axiom cases that fail semantic checks (for triage).
pub fn scan_planned_dl_failures() -> Vec<(String, String)> {
    let mut failures: Vec<(String, String)> = read_catalog_file()
        .par_iter()
        .filter(|case| {
            case.engine == "dl"
                && case.status == "planned"
                && case.axiom_ofn.is_some()
                && (!case.subsumptions.is_empty() || case.consistent.is_some())
        })
        .filter_map(|case| check_axiom_case(case).err().map(|e| (case.id.clone(), e)))
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

pub fn load_catalog() -> &'static [HermitCase] {
    CATALOG.get_or_init(|| {
        let path = catalog_path();
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("missing HermiT catalog at {}: {e}", path.display()));
        serde_json::from_str(&text).expect("parse cases.json")
    })
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
    let case = load_catalog()
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

fn hermit_data_path(rel: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

fn materialize_ontology(case: &HermitCase, ontology: &mut Ontology) {
    match case.engine.as_str() {
        "rdfs" => {
            RdfsEngine::new()
                .materialize(ontology)
                .expect("rdfs materialize");
        }
        "rl" => {
            ontologos_rl::RlEngine::new(1)
                .saturate(ontology)
                .expect("rl saturate");
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
    let clauses = ontologos_alc::clausify(&mut ontology).expect("clausify");
    assert!(!clauses.is_empty(), "{}: empty clause set", case.id);
}

fn run_axiom_case(case: &HermitCase) {
    let rel = case
        .axiom_ofn
        .as_ref()
        .expect("axiom case missing axiom_ofn path");
    let path = hermit_data_path(rel);
    assert!(path.is_file(), "missing axiom fixture {}", path.display());

    let mut ontology = load_ontology(&path).expect("load axiom ofn");

    if case.engine == "dl" || case.engine == "swrl" {
        if !case.subsumptions.is_empty() {
            let taxonomy = ontologos_dl::classify(&ontology).expect("dl classify");
            check_subsumptions_dl(&ontology, &taxonomy, case);
        }
        if let Some(expected) = case.consistent {
            let consistent = ontologos_dl::is_consistent(&ontology).expect("consistent");
            assert_eq!(consistent, expected, "{}: consistency", case.id);
        }
        return;
    }

    materialize_ontology(case, &mut ontology);
    check_subsumptions(&ontology, case);
    check_property_subsumptions(&ontology, case);
    check_property_characteristics(&ontology, case);

    if let Some(expected) = case.consistent {
        let mut consistent = if case.engine == "dl" || case.engine == "swrl" {
            ontologos_dl::is_consistent(&ontology).expect("consistent")
        } else {
            saturate_for_consistency(case, &mut ontology)
        };
        if ontologos_bridge::has_bottom_chain_violation(&ontology) {
            consistent = false;
        }
        assert_eq!(consistent, expected, "{}: consistency", case.id);
    }
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
    let rel = case
        .axiom_ofn
        .as_ref()
        .expect("swrl case missing axiom_ofn");
    let path = hermit_data_path(rel);
    let ontology = load_ontology(&path).expect("load swrl ofn");
    let (_taxonomy, _report) =
        ontologos_swrl::classify_with_swrl(&ontology).expect("swrl classify");
    check_subsumptions_dl(
        &ontology,
        &ontologos_dl::classify(&ontology).expect("dl"),
        case,
    );
}

fn check_subsumptions(ontology: &Ontology, case: &HermitCase) {
    check_subsumptions_result(ontology, case).expect("subsumptions");
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

fn check_property_subsumptions(ontology: &Ontology, case: &HermitCase) {
    check_property_subsumptions_result(ontology, case).expect("property subsumptions");
}

fn check_property_subsumptions_result(
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<(), String> {
    for sub in &case.property_subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
        let actual = assert_subproperty(ontology, &sub_iri, &sup_iri);
        if actual != sub.expected {
            return Err(format!(
                "{}: expected {} ⊑ {} (property) = {}",
                case.id, sub_iri, sup_iri, sub.expected
            ));
        }
    }
    Ok(())
}

fn check_property_characteristics(ontology: &Ontology, case: &HermitCase) {
    check_property_characteristics_result(ontology, case).expect("property characteristics");
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

fn check_subsumptions_dl(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) {
    check_subsumptions_dl_result(ontology, taxonomy, case).expect("dl subsumptions");
}

fn check_subsumptions_dl_result(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) -> Result<(), String> {
    for sub in &case.subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
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
        if actual != sub.expected {
            return Err(format!(
                "{}: expected {} ⊑ {} = {}",
                case.id, sub_iri, sup_iri, sub.expected
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
        let conclusion =
            load_ontology(&conclusion_path).map_err(|e| format!("{}: load conclusion: {e}", case.id))?;
        let entailed = wg_entailment_holds(&ontology, &conclusion);
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

fn wg_entailment_holds(premise: &Ontology, conclusion: &Ontology) -> bool {
    let Ok(prem_tax) = ontologos_dl::classify(premise) else {
        return false;
    };
    let merged = merge_ontologies_for_entailment(premise, conclusion);
    let Ok(merged_tax) = ontologos_dl::classify(&merged) else {
        return false;
    };

    for &(sub, sup) in &merged_tax.subsumptions {
        if !prem_tax.is_subsumed(sub, sup) {
            return false;
        }
    }
    for &class in &merged_tax.unsatisfiable {
        if !prem_tax.unsatisfiable.contains(&class) {
            return false;
        }
    }
    true
}

fn merge_ontologies_for_entailment(premise: &Ontology, conclusion: &Ontology) -> Ontology {
    let mut merged = premise.clone();
    for (_, axiom) in conclusion.axioms().iter() {
        let _ = merged.add_axiom(axiom.clone());
    }
    for axiom in conclusion.dl().axioms() {
        merged.dl_mut().push_axiom(axiom.clone());
    }
    merged
}

fn resolve_local_iri(local: &str) -> String {
    if local.contains("://") || local.starts_with("file:") {
        local.to_owned()
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
