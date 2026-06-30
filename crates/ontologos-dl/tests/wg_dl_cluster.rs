use ontologos_alc::{DlOntology, TableauSeed, is_named_class_satisfiable_with_seed};
use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::Path;

fn check(case: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg")
        .join(case)
        .join("premise.rdf");
    let ont = load_ontology(&path).expect("load");
    let store = ont.dl();
    eprintln!("\n=== {case} dl={} ===", store.axiom_count());
    for ax in store.axioms() {
        eprintln!("  dl: {ax:?}");
    }
    for (_, ax) in ont.axioms().iter() {
        eprintln!("  core: {ax:?}");
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
    assert!(!consistent, "{case} should be inconsistent");
}

#[test]
fn wg_dl035() {
    check("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D035");
}

#[test]
fn wg_dl026() {
    check("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D026");
}

#[test]
fn wg_dl601() {
    check("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601");
}

#[test]
fn wg_dl626() {
    check("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D626");
}
