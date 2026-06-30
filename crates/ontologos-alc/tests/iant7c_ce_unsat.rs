//! IanT7c CE unsatisfiability — isolated from other CE probe tests to avoid parallel temp-file races.

mod support;

use support::ian_ce_probe::ce_sat;

const IANT7C_CE: &str = "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))) ObjectSomeValuesFrom(ObjectInverseOf(:f) :p1))";

#[test]
fn iant7c_ce_is_unsatisfiable() {
    assert!(
        !ce_sat("hermit_reasoner_reasonertest_testiant7c.ofn", IANT7C_CE),
        "IanT7c CE should be unsatisfiable"
    );
}
