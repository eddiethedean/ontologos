//! HermiT RIARegularityTest OFN fixtures (excluded catalog cases 4/7/8/9).

use ontologos_dl::is_property_hierarchy_regular;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn ax(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

#[test]
fn hermit_ria4_irregular_chain_cycle() {
    let ont = load_ontology(&ax(
        "hermit_reasoner_riaregularitytest_testriaregularity4.ofn",
    ))
    .expect("load");
    assert!(
        !is_property_hierarchy_regular(&ont).unwrap(),
        "RIA4 should be irregular"
    );
}

#[test]
fn hermit_ria7_irregular_inverse_chain() {
    let ont = load_ontology(&ax(
        "hermit_reasoner_riaregularitytest_testriaregularity7.ofn",
    ))
    .expect("load");
    assert!(
        !is_property_hierarchy_regular(&ont).unwrap(),
        "RIA7 should be irregular"
    );
}

#[test]
fn hermit_ria8_regular_equivalent_cycle() {
    let ont = load_ontology(&ax(
        "hermit_reasoner_riaregularitytest_testriaregularity8.ofn",
    ))
    .expect("load");
    assert!(
        is_property_hierarchy_regular(&ont).unwrap(),
        "RIA8 should be regular"
    );
}

#[test]
fn hermit_ria9_role_simplicity() {
    let ont = load_ontology(&ax(
        "hermit_reasoner_riaregularitytest_testriaregularity9.ofn",
    ))
    .expect("load");
    assert!(
        !is_property_hierarchy_regular(&ont).unwrap(),
        "RIA9 should be irregular"
    );
}
