//! Targeted checks for the remaining Phase 4 WG cases (not yet promoted).

use ontologos_dl::{is_consistent, is_datatype_consistent};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg_premise(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

fn check_consistency(rel: &str, expected: bool) -> Result<(), String> {
    let ont = load_ontology(&wg_premise(rel)).map_err(|e| format!("load: {e}"))?;
    let actual = is_consistent(&ont).map_err(|e| format!("check: {e}"))?;
    if actual == expected {
        Ok(())
    } else {
        Err(format!(
            "expected consistent={expected}, got {actual} (datatype={})",
            is_datatype_consistent(&ont)
        ))
    }
}

#[test]
#[ignore = "manual debug — dump misc-203 axiom mapping"]
fn diagnose_misc203_axioms() {
    let rel = "wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    eprintln!("axioms={}", ont.dl().axiom_count());
    eprintln!("datatype_consistent={}", is_datatype_consistent(&ont));
    eprintln!("consistent={:?}", is_consistent(&ont));
    for ax in ont.dl().axioms() {
        eprintln!("  {ax:?}");
    }
    let store = ont.dl();
    for de in 0..store.de_count() {
        let id = ontologos_core::DeId(de as u32);
        if let Some(expr) = store.de(id) {
            eprintln!("  de {id:?}: {expr:?}");
        }
    }
}

#[test]
#[ignore = "manual debug — dump dl-650 unsatisfiable mapping"]
fn diagnose_dl650_unsatisfiable() {
    use ontologos_alc::{
        DlOntology, TableauSeed, is_ce_intersection_satisfiable_with_seed,
        is_ce_satisfiable_with_seed,
    };

    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D650/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let store = ont.dl();
    let e = ont
        .lookup_entity("http://oiled.man.example.net/test#e")
        .expect("e");
    let e_ce = store
        .expressions()
        .find_map(|(id, expr)| match expr {
            ontologos_core::ClassExpr::Atomic(c) if *c == e => Some(id),
            _ => None,
        })
        .expect("e ce");
    let ecomp = ont
        .lookup_entity("http://oiled.man.example.net/test#e.comp")
        .expect("e.comp");
    let ecomp_ce = store
        .expressions()
        .find_map(|(id, expr)| match expr {
            ontologos_core::ClassExpr::Atomic(c) if *c == ecomp => Some(id),
            _ => None,
        })
        .expect("ecomp ce");
    let seed = TableauSeed::default();
    eprintln!(
        "e+e.comp intersection={:?}",
        is_ce_intersection_satisfiable_with_seed(&dl, e_ce, ecomp_ce, &seed)
    );
    let unsat = ont
        .lookup_entity("http://oiled.man.example.net/test#Unsatisfiable")
        .expect("unsat");
    let unsat_ce = store
        .expressions()
        .find_map(|(id, expr)| match expr {
            ontologos_core::ClassExpr::Atomic(c) if *c == unsat => Some(id),
            _ => None,
        })
        .expect("unsat ce");
    let equiv_and = store
        .axioms()
        .find_map(|ax| match ax {
            ontologos_core::DlAxiom::EquivalentClasses(ids) if ids.contains(&unsat_ce) => {
                ids.iter().copied().find(|&id| id != unsat_ce)
            }
            _ => None,
        })
        .expect("equiv");
    eprintln!(
        "unsat equiv sat={:?}",
        is_ce_satisfiable_with_seed(&dl, equiv_and, &seed)
    );
    eprintln!("kb consistent={:?}", is_consistent(&ont));
}

#[test]
fn phase4_remaining_consistency_cases_fast() {
    let cases = [
        ("One_equals_two", "wg/One_equals_two/premise.rdf", false),
        (
            "dl-650",
            "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D650/premise.rdf",
            false,
        ),
        (
            "dl-910",
            "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D910/premise.rdf",
            false,
        ),
        (
            "misc-203",
            "wg/TestCase-3AWebOnt-2Dmiscellaneous-2D203/premise.rdf",
            false,
        ),
        (
            "misc-204",
            "wg/TestCase-3AWebOnt-2Dmiscellaneous-2D204/premise.rdf",
            false,
        ),
    ];
    let mut failures = Vec::new();
    for (name, rel, expected) in cases {
        if let Err(e) = check_consistency(rel, expected) {
            failures.push(format!("{name}: {e}"));
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

#[test]
fn phase4_misc_wine_consistency_cases() {
    let cases = [
        (
            "misc-001",
            "wg/TestCase-3AWebOnt-2Dmiscellaneous-2D001/premise.rdf",
            true,
        ),
        (
            "misc-002",
            "wg/TestCase-3AWebOnt-2Dmiscellaneous-2D002/premise.rdf",
            true,
        ),
    ];
    for (name, rel, expected) in cases {
        check_consistency(rel, expected).unwrap_or_else(|e| panic!("{name}: {e}"));
    }
}
