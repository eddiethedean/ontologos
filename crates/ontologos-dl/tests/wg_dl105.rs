use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::Path;

#[test]
fn wg_dl105_is_inconsistent() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D105/premise.rdf",
    );
    let ont = load_ontology(&path).expect("load");
    let dl = ontologos_alc::DlOntology::from_ontology(&ont).expect("dl");
    let seed = ontologos_alc::TableauSeed::default();
    if let Some(id) = ont.lookup_entity("http://oiled.man.example.net/test#Unsatisfiable") {
        let sat = ontologos_alc::is_named_class_satisfiable_with_seed(&dl, id, &seed).unwrap();
        eprintln!("Unsatisfiable sat? {sat}");
    }
    assert!(!is_consistent(&ont).expect("check"));
}
