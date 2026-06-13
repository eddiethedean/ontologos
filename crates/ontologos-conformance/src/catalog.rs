//! Auto-generated HermiT test catalog runner.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use ontologos_core::Ontology;
use ontologos_parser::load_ontology;
use ontologos_rdfs::RdfsEngine;
use serde::Deserialize;

use crate::{assert_subsumed, classification_fixture_path, HERMIT_DEFAULT_NS};

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

#[must_use]
pub fn catalog_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/hermit/catalog/cases.json")
}

#[must_use]
pub fn wg_catalog_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/hermit/catalog/wg_cases.json")
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
        "ported" | "excluded" | "deferred" | "internal" | "planned" => {
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
        let taxonomy = ontologos_dl::classify(&ontology).expect("dl classify");
        check_subsumptions_dl(&ontology, &taxonomy, case);
        if let Some(expected) = case.consistent {
            let consistent = ontologos_dl::is_consistent(&ontology).expect("consistent");
            assert_eq!(consistent, expected, "{}: consistency", case.id);
        }
        return;
    }

    materialize_ontology(case, &mut ontology);
    check_subsumptions(&ontology, case);

    if let Some(expected) = case.consistent {
        let consistent = ontologos_dl::is_consistent(&ontology).unwrap_or(true);
        assert_eq!(consistent, expected, "{}: consistency", case.id);
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
    for sub in &case.subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
        let actual = assert_subsumed(ontology, &sub_iri, &sup_iri);
        assert_eq!(
            actual, sub.expected,
            "{}: expected {} ⊑ {} = {}",
            case.id, sub_iri, sup_iri, sub.expected
        );
    }
}

fn check_subsumptions_dl(
    ontology: &Ontology,
    taxonomy: &ontologos_core::Taxonomy,
    case: &HermitCase,
) {
    for sub in &case.subsumptions {
        let sub_iri = resolve_local_iri(&sub.sub);
        let sup_iri = resolve_local_iri(&sub.sup);
        let sub_id = ontology
            .lookup_entity(&sub_iri)
            .unwrap_or_else(|| panic!("{}: missing {sub_iri}", case.id));
        let sup_id = ontology
            .lookup_entity(&sup_iri)
            .unwrap_or_else(|| panic!("{}: missing {sup_iri}", case.id));
        let actual =
            taxonomy.is_subsumed(sub_id, sup_id) || assert_subsumed(ontology, &sub_iri, &sup_iri);
        assert_eq!(
            actual, sub.expected,
            "{}: expected {} ⊑ {} = {}",
            case.id, sub_iri, sup_iri, sub.expected
        );
    }
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
    let premise = case.premise_ofn.as_ref().expect("wg premise");
    let path = hermit_data_path(premise);
    let ontology = load_ontology(&path).expect("load wg premise");

    if let Some(expected) = case.expected_consistent {
        let actual = ontologos_dl::is_consistent(&ontology).expect("wg consistent");
        assert_eq!(actual, expected, "WG {} consistency", case.id);
        return;
    }

    if let (Some(conclusion_rel), Some(expected)) = (&case.conclusion_ofn, case.expected_entailment)
    {
        let conclusion_path = hermit_data_path(conclusion_rel);
        let conclusion = load_ontology(&conclusion_path).expect("wg conclusion");
        let entailed = wg_entailment_holds(&ontology, &conclusion);
        assert_eq!(
            entailed, expected,
            "WG {} entailment expected {}",
            case.id, expected
        );
    }
}

fn wg_entailment_holds(premise: &Ontology, conclusion: &Ontology) -> bool {
    let prem_tax = ontologos_dl::classify(premise).ok();
    let conc_tax = ontologos_dl::classify(conclusion).ok();
    match (prem_tax, conc_tax) {
        (Some(p), Some(c)) => {
            for &(sub, sup) in &c.subsumptions {
                if !p.is_subsumed(sub, sup) {
                    return false;
                }
            }
            true
        }
        _ => false,
    }
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
