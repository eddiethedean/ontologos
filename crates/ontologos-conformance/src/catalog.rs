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

/// Load catalog from disk (not cached) for promotion tooling.
pub fn read_catalog_file() -> Vec<HermitCase> {
    let path = catalog_path();
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("missing HermiT catalog at {}: {e}", path.display()));
    serde_json::from_str(&text).expect("parse cases.json")
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
            || !case.datalog_queries.is_empty())
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

    if case.load_error_expected {
        return if load_ontology(&path).is_err() {
            Ok(())
        } else {
            Err(format!("{}: expected ontology load to fail", case.id))
        };
    }

    let mut ontology = load_ontology(&path).map_err(|e| format!("{}: load: {e}", case.id))?;

    if let Some(inc_rel) = &case.incremental_ofn {
        let inc_path = hermit_data_path(inc_rel);
        if !inc_path.is_file() {
            return Err(format!(
                "{}: missing incremental fixture {}",
                case.id,
                inc_path.display()
            ));
        }
        let inc =
            load_ontology(&inc_path).map_err(|e| format!("{}: load incremental: {e}", case.id))?;
        merge_ontology_axioms(&mut ontology, &inc);
    }

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
        let entailed = entailment_holds(&ontology, &conclusion);
        if entailed != expected {
            return Err(format!(
                "{}: entailment expected {expected}, got {entailed}",
                case.id
            ));
        }
        return Ok(());
    }

    if case.engine == "dl" || case.engine == "swrl" || case.engine == "alc" {
        let taxonomy =
            ontologos_dl::classify(&ontology).map_err(|e| format!("{}: dl: {e}", case.id))?;

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
    check_data_property_subsumptions_result(&ontology, case)?;
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

fn merge_ontology_axioms(target: &mut Ontology, source: &Ontology) {
    for (_, axiom) in source.axioms().iter() {
        let _ = target.add_axiom(axiom.clone());
    }
    for axiom in source.dl().axioms() {
        target.dl_mut().push_axiom(axiom.clone());
    }
}

fn check_class_satisfiability_result(
    taxonomy: &ontologos_core::Taxonomy,
    ontology: &Ontology,
    case: &HermitCase,
) -> Result<(), String> {
    for exp in &case.class_satisfiability {
        let iri = resolve_local_iri(&exp.class);
        let class_id = ontology
            .lookup_entity(&iri)
            .ok_or_else(|| format!("{}: missing class {iri}", case.id))?;
        let unsat = taxonomy.unsatisfiable.contains(&class_id);
        let satisfiable = !unsat;
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
        let ind_id = ontology
            .lookup_entity(&ind_iri)
            .ok_or_else(|| format!("{}: missing individual {ind_iri}", case.id))?;
        let class_id = ontology
            .lookup_entity(&class_iri)
            .ok_or_else(|| format!("{}: missing class {class_iri}", case.id))?;
        let actual = individual_has_type(ontology, taxonomy, ind_id, class_id, exp.direct);
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
) -> bool {
    let asserted = ontology.classes_of(individual);
    if direct {
        return asserted.contains(&class);
    }
    asserted
        .iter()
        .any(|&t| t == class || taxonomy.is_subsumed(t, class))
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

fn check_individual_instances_result(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) -> Result<(), String> {
    for exp in &case.individual_instances {
        let class_iri = resolve_local_iri(&exp.class);
        let class_id = ontology
            .lookup_entity(&class_iri)
            .ok_or_else(|| format!("{}: missing class {class_iri}", case.id))?;
        let mut actual: std::collections::HashSet<String> = std::collections::HashSet::new();
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
                if individual_has_type(ontology, taxonomy, ind, class_id, false) {
                    if let Some(local) = entity_local_name(ontology, ind) {
                        actual.insert(format!(":{local}"));
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
            let mut actual: std::collections::HashSet<String> = std::collections::HashSet::new();
            for &ind in ontology.individuals_of(class_id) {
                if let Some(local) = entity_local_name(ontology, ind) {
                    actual.insert(format!(":{local}"));
                }
            }
            for (ind, record) in ontology.entities().iter() {
                if record.kind != ontologos_core::EntityKind::Individual {
                    continue;
                }
                if individual_has_type(ontology, taxonomy, ind, class_id, false) {
                    if let Some(local) = entity_local_name(ontology, ind) {
                        actual.insert(format!(":{local}"));
                    }
                }
            }
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
        let sub_id = ontology
            .lookup_entity(&sub_iri)
            .ok_or_else(|| format!("{}: missing data property {sub_iri}", case.id))?;
        let sup_id = ontology
            .lookup_entity(&sup_iri)
            .ok_or_else(|| format!("{}: missing data property {sup_iri}", case.id))?;
        let actual = ontology.direct_subproperties(sup_id).contains(&sub_id);
        if actual != sub.expected {
            return Err(format!(
                "{}: expected data property {} ⊑ {} = {}",
                case.id, sub_iri, sup_iri, sub.expected
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
    let rel = case
        .axiom_ofn
        .as_ref()
        .expect("axiom case missing axiom_ofn path");
    let path = hermit_data_path(rel);
    assert!(path.is_file(), "missing axiom fixture {}", path.display());

    let mut ontology = load_ontology(&path).expect("load axiom ofn");

    if case.engine == "swrl" {
        ontologos_swrl::apply_swrl_rules(&mut ontology).expect("swrl rules");
    }

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
    check_axiom_case(case).expect("swrl case");
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
        let conclusion = load_ontology(&conclusion_path)
            .map_err(|e| format!("{}: load conclusion: {e}", case.id))?;
        let entailed = entailment_holds(&ontology, &conclusion);
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
    match check_axiom_case(case) {
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
        .iter()
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

fn entailment_holds(premise: &Ontology, conclusion: &Ontology) -> bool {
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
