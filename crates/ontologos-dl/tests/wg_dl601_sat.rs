use ontologos_alc::{is_named_class_satisfiable_with_seed, DlOntology, TableauSeed};
use ontologos_parser::load_ontology;
use std::path::Path;

#[test]
fn dl601_class_sat() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(
        "../../benchmarks/data/hermit/wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D601/premise.rdf",
    );
    let ont = load_ontology(&path).unwrap();
    let dl = DlOntology::from_ontology(&ont).unwrap();
    let seed = TableauSeed::default();
    for name in [
        "Unsatisfiable",
        "Unsatisfiable.comp",
        "C.6.comp",
        "C.7.comp",
        "C.8.comp",
        "a",
        "b",
        "c",
    ] {
        let iri = if name.starts_with("C.") {
            format!("http://www.w3.org/2002/03owlt/description-logic/inconsistent601#{name}")
        } else {
            format!("http://oiled.man.example.net/test#{name}")
        };
        if let Some(e) = ont.lookup_entity(&iri) {
            let sat = is_named_class_satisfiable_with_seed(&dl, e, &seed).unwrap();
            eprintln!("{name} SAT={sat}");
        }
    }
}
