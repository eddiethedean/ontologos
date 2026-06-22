use ontologos_dl::{classify, is_consistent};
use ontologos_parser::load_ontology;
use std::path::Path;

fn analyze(case: &str) {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg")
        .join(case)
        .join("premise.rdf");
    let ont = load_ontology(&path).expect("load");
    eprintln!("\n========== {case} ==========");
    if let Ok(t) = classify(&ont) {
        eprintln!("unsatisfiable classes:");
        for e in &t.unsatisfiable {
            eprintln!(
                "  {}",
                ont.resolve_iri(ont.entity(*e).unwrap().iri).unwrap()
            );
        }
    }
    for ax in ont.dl().axioms() {
        eprintln!("  {ax:?}");
    }
    eprintln!("consistent={}", is_consistent(&ont).unwrap());
    let store = ont.dl();
    for (id, ce) in store.expressions() {
        if format!("{ce:?}").contains("Unsatisfiable") || format!("{ce:?}").contains("And") {
            eprintln!("  CeId({}): {ce:?}", id.0);
        }
    }
    if case.contains("601") {
        for ax in store.axioms() {
            if let ontologos_core::DlAxiom::ClassAssertion { individual, class } = ax {
                eprintln!(
                    "  CA {:?} -> {:?}",
                    ont.resolve_iri(ont.entity(*individual).unwrap().iri)
                        .unwrap(),
                    store.ce(*class)
                );
            }
        }
    }
}

#[test]
fn analyze_remaining() {
    analyze("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601");
    analyze("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D026");
    analyze("TestCase-3AWebOnt-2Ddescription-2Dlogic-2D626");
}
