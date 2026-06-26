//! CE satisfiability for dl-650 Unsatisfiable intersection pattern.

use ontologos_alc::{DlOntology, TableauSeed, is_ce_satisfiable_with_seed};
use ontologos_core::DlAxiom;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn wg_premise(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit")
        .join(rel)
}

#[test]
fn dl650_unsatisfiable_equiv_is_unsat() {
    let rel = "wg/TestCase-3AWebOnt-2Ddescription-2Dlogic-2D650/premise.rdf";
    let ont = load_ontology(&wg_premise(rel)).expect("load");
    let dl = DlOntology::from_ontology(&ont).expect("dl");
    let store = ont.dl();
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
            DlAxiom::EquivalentClasses(ids) if ids.contains(&unsat_ce) => {
                ids.iter().copied().find(|&id| id != unsat_ce)
            }
            _ => None,
        })
        .expect("equiv");
    let sat = is_ce_satisfiable_with_seed(&dl, equiv_and, &TableauSeed::default()).expect("sat");
    assert!(!sat, "Unsatisfiable equiv And should be unsatisfiable");
}
