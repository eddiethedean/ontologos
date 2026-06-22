use ontologos_alc::{is_ce_satisfiable_with_seed, is_named_class_satisfiable_with_seed, DlOntology, TableauSeed};
use ontologos_dl::{classify, is_datatype_consistent};
use ontologos_parser::load_ontology;
use std::path::Path;

#[test]
fn dl601_sat_matrix() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf");
    let ont = load_ontology(&path).unwrap();
    let dl = DlOntology::from_ontology(&ont).unwrap();
    let seed = TableauSeed::default();
    eprintln!("datatype_consistent={}", is_datatype_consistent(&ont));
    for name in [
        "Unsatisfiable",
        "Unsatisfiable.comp",
        "C.7.comp",
        "C.8.comp",
        "C.7",
        "C.8",
        "C.6",
        "a",
        "b",
        "c",
    ] {
        let iri = format!("http://oiled.man.example.net/test#{name}");
        if let Some(e) = ont.lookup_entity(&iri) {
            let sat = is_named_class_satisfiable_with_seed(&dl, e, &seed).unwrap();
            eprintln!("{name} SAT={sat}");
        }
    }
    let tax = classify(&ont).unwrap();
    for u in &tax.unsatisfiable {
        eprintln!(
            "classify unsat: {}",
            ont.resolve_iri(ont.entity(*u).unwrap().iri).unwrap()
        );
    }
    let store = ont.dl();
    for (id, ce) in store.expressions() {
        if format!("{ce:?}").contains("And") {
            let sat = is_ce_satisfiable_with_seed(&dl, id, &seed).unwrap();
            eprintln!("CeId({}) {ce:?} SAT={sat}", id.0);
        }
    }
}
