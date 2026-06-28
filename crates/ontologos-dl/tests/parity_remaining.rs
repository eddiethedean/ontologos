use ontologos_dl::is_consistent;
use ontologos_parser::load_ontology;

fn ofn(name: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../benchmarks/data/hermit/axioms/hermit_reasoner_{name}.ofn"
    ))
}

fn assert_consistent(name: &str, expected: bool) {
    let path = ofn(name);
    let ont = load_ontology(&path).unwrap_or_else(|e| panic!("load {}: {e}", path.display()));
    let got = is_consistent(&ont).expect("check");
    assert_eq!(got, expected, "{name}");
}

#[test]
fn finite2_2_inconsistent() {
    assert_consistent("datetimetest_testfinite2_2", false);
}

#[test]
fn mizedtzs_2_inconsistent() {
    assert_consistent("datetimetest_testmizedtzs_2", false);
}

#[test]
fn float_zero_range_2_consistent() {
    assert_consistent("floatdoubletest_testfloatzerorange_2", true);
}

#[test]
fn integer_range2_2_inconsistent() {
    assert_consistent("numericstest_testintegerrange2_2", false);
}

#[test]
fn decimal_not_integer_2_inconsistent() {
    assert_consistent("numericstest_testdecimalnotinteger_2", false);
}
