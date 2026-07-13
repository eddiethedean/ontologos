//! Triage harness for promoted WG cases that should be consistent but fail today.

use ontologos_alc::{DlOntology, TableauSeed, is_named_class_satisfiable_with_seed};
use ontologos_dl::{classify, is_consistent, is_datatype_consistent};
use ontologos_parser::load_ontology;
use std::path::Path;

fn wg_premise(case: &str) -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg")
        .join(case)
        .join("premise.rdf")
}

fn triage(case: &str) -> bool {
    let path = wg_premise(case);
    let ont = load_ontology(&path).expect("load");
    let store = ont.dl();
    eprintln!("\n========== {case} ==========");
    eprintln!("dl axioms={}", store.axiom_count());
    for ax in store.axioms() {
        eprintln!("  dl: {ax:?}");
    }
    for (_, ax) in ont.axioms().iter() {
        eprintln!("  core: {ax:?}");
    }

    let dt_ok = is_datatype_consistent(&ont);
    eprintln!("datatype_consistent={dt_ok}");
    if !dt_ok {
        return false;
    }

    let has_ca = store
        .axioms()
        .any(|ax| matches!(ax, ontologos_core::DlAxiom::ClassAssertion { .. }));
    let has_comp = ont.entities().iter().any(|(_, record)| {
        record.kind == ontologos_core::EntityKind::Class
            && ont
                .resolve_iri(record.iri)
                .ok()
                .is_some_and(|iri| iri.contains(".comp"))
    });
    if has_ca
        && has_comp
        && let Ok(tax) = classify(&ont)
    {
        let comp_unsat: Vec<_> = tax
            .unsatisfiable
            .iter()
            .filter(|e| {
                ont.entity(**e)
                    .ok()
                    .and_then(|r| ont.resolve_iri(r.iri).ok())
                    .is_some_and(|iri| iri.contains(".comp"))
            })
            .collect();
        eprintln!("flower path: comp_unsat={}", comp_unsat.len());
    }

    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let seed = TableauSeed::default();
    let unsat_iri = "http://oiled.man.example.net/test#Unsatisfiable";
    if let Some(unsat) = ont.lookup_entity(unsat_iri) {
        let class_sat = is_named_class_satisfiable_with_seed(&dl, unsat, &seed).expect("class sat");
        eprintln!("Unsatisfiable class SAT={class_sat}");
    }

    let consistent = is_consistent(&ont).expect("check");
    eprintln!("consistent={consistent}");
    consistent
}

#[test]
#[ignore = "manual debug — dump WG expression mapping"]
fn debug_expressions() {
    for case in [
        "TestCase-3AWebOnt-2Ddescription-2Dlogic-2D017",
        "TestCase-3AWebOnt-2DRestriction-2D001",
        "TestCase-3AWebOnt-2DmaxCardinality-2D001",
        "TestCase-3AWebOnt-2DI4.5-2D002",
    ] {
        let path = wg_premise(case);
        let ont = load_ontology(&path).expect("load");
        eprintln!("\n=== {case} expressions ===");
        for (id, ce) in ont.dl().expressions() {
            eprintln!("  CeId({}): {ce:?}", id.0);
        }
    }
}

const EXPECTED_INCONSISTENT: &[&str] = &[
    "TestCase-3AWebOnt-2Ddescription-2Dlogic-2D017",
    "TestCase-3AWebOnt-2Ddescription-2Dlogic-2D033",
    "TestCase-3AWebOnt-2Ddescription-2Dlogic-2D633",
    "TestCase-3AWebOnt-2DmaxCardinality-2D001",
    "TestCase-3AWebOnt-2DRestriction-2D001",
    "TestCase-3AWebOnt-2DRestriction-2D002",
    "TestCase-3AWebOnt-2DI4.5-2D002",
];

#[test]
fn wg_promoted_inconsistency_cases() {
    let mut failures = Vec::new();
    for case in EXPECTED_INCONSISTENT {
        if triage(case) {
            failures.push(*case);
        }
    }
    assert!(failures.is_empty(), "expected inconsistent: {failures:?}");
}
