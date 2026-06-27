//! Phase 5 HermiT datatype tranche regressions (assertDRSatisfiable families).

use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn hermit_axiom(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_{name}.ofn"
    ))
}

fn assert_consistent_ofn(name: &str, expected: bool) {
    let path = hermit_axiom(name);
    let ont = load_ontology(&path).expect("load");
    let actual = is_consistent(&ont).expect("check");
    assert_eq!(
        actual,
        expected,
        "{}: expected {expected}, got {actual}",
        path.display()
    );
}

mod numerics {
    use super::*;

    #[test]
    fn integer_range1_inconsistent() {
        assert_consistent_ofn("numericstest_testintegerrange1", false);
    }

    #[test]
    fn integer_range2_4_consistent() {
        assert_consistent_ofn("numericstest_testintegerrange2_4", true);
    }

    #[test]
    fn real_not_decimal_consistent() {
        assert_consistent_ofn("numericstest_testrealnotdecimal", true);
    }

    #[test]
    fn invalid_min_max_inconsistent() {
        assert_consistent_ofn("numericstest_testinvalidminmax", false);
    }

    #[test]
    fn enum_int_2_inconsistent() {
        assert_consistent_ofn("numericstest_testenumint_2", false);
    }

    #[test]
    fn large_range1_3_inconsistent() {
        assert_consistent_ofn("numericstest_testlargerange1_3", false);
    }
}

mod float_double {
    use super::*;

    #[test]
    fn float_zero_range_consistent() {
        assert_consistent_ofn("floatdoubletest_testfloatzerorange_1", true);
    }
}

mod any_uri {
    use super::*;

    #[test]
    fn length_1_consistent() {
        assert_consistent_ofn("anyuritest_testlength_1", true);
    }
}
