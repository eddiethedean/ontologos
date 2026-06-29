//! Hand-written ports for HermiT `OWLReasonerTest` query API checks.

use ontologos_alc::inverse_object_property_expressions;
use ontologos_core::RoleExpr;
use ontologos_parser::load_ontology;
use std::collections::HashSet;
use std::path::PathBuf;

const INVERSE_CYCLE_OFN: &str =
    "hermit_reasoner_owlreasonertest_testgetinverseobjectpropertyexpressions.ofn";
const NS: &str = "file:/c/test.owl#";

fn inverse_cycle_ontology() -> ontologos_core::Ontology {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/")
        .join(INVERSE_CYCLE_OFN);
    load_ontology(&path).expect("inverse cycle OFN")
}

/// `OWLReasonerTest.testgetInverseObjectPropertyExpressions`
#[test]
fn owlreasoner_inverse_object_property_expressions_cycle() {
    let ontology = inverse_cycle_ontology();
    let r = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
    let s = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
    let t = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}t")).expect("t"));
    let inv_r = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
    let inv_s = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
    let inv_t = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}t")).expect("t"));

    let r_inverses = inverse_object_property_expressions(&ontology, &r).expect("inverses of r");
    assert_eq!(
        r_inverses,
        HashSet::from([inv_r.clone(), s.clone(), inv_t.clone()])
    );

    let inv_r_inverses =
        inverse_object_property_expressions(&ontology, &inv_r).expect("inverses of inv(r)");
    assert_eq!(inv_r_inverses, HashSet::from([inv_s, r.clone(), t]));
}
