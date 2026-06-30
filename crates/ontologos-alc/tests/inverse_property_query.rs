//! HermiT `OWLReasonerTest.testgetInverseObjectPropertyExpressions`.

use ontologos_alc::inverse_object_property_expressions;
use ontologos_core::RoleExpr;
use ontologos_parser::load_ontology;
use std::collections::HashSet;
use std::path::PathBuf;

const NS: &str = "file:/c/test.owl#";

fn load_inverse_cycle() -> (
    ontologos_core::Ontology,
    RoleExpr,
    RoleExpr,
    RoleExpr,
    RoleExpr,
    RoleExpr,
    RoleExpr,
) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms/hermit_reasoner_owlreasonertest_testgetinverseobjectpropertyexpressions.ofn");
    let ontology = load_ontology(&path).expect("load inverse OFN");
    let r = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
    let s = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
    let t = RoleExpr::Atomic(ontology.lookup_entity(&format!("{NS}t")).expect("t"));
    let inv_r = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}r")).expect("r"));
    let inv_s = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}s")).expect("s"));
    let inv_t = RoleExpr::Inverse(ontology.lookup_entity(&format!("{NS}t")).expect("t"));
    (ontology, r, s, t, inv_r, inv_s, inv_t)
}

#[test]
fn hermit_inverse_object_property_expressions_cycle() {
    let (ontology, r, inv_r, s, t, inv_t, inv_s) = {
        let (ontology, r, s, t, inv_r, inv_s, inv_t) = load_inverse_cycle();
        (ontology, r, inv_r, s, t, inv_t, inv_s)
    };

    let r_inverses = inverse_object_property_expressions(&ontology, &r).expect("inverses of r");
    let inv_r_inverses =
        inverse_object_property_expressions(&ontology, &inv_r).expect("inverses of inv(r)");

    assert_eq!(
        r_inverses,
        HashSet::from([inv_r.clone(), s.clone(), inv_t.clone()])
    );
    assert_eq!(inv_r_inverses, HashSet::from([inv_s, r.clone(), t]));
}
