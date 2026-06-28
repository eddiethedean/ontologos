//! v1.6 ABox exit — Family corpus individuals and sameAs closure.

use ontologos_abox::{is_abox_consistent, materialize_abox, same_as_closure};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

#[test]
fn family_individuals_typed_after_materialize() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../benchmarks/data/family.owl");
    let mut ont = load_ontology(&path).expect("family");
    let report = materialize_abox(&mut ont).expect("materialize");
    assert!(
        report.rl_inferences > 0 || !report.same_as_clusters.is_empty(),
        "family ABox materialize should infer typings or sameAs clusters"
    );
    assert!(is_abox_consistent(&ont).expect("consistent"));
}

#[test]
fn same_as_chain_merges_transitively() {
    let ont = ontologos_core::Ontology::builder()
        .individual("http://ex/a")
        .expect("a")
        .individual("http://ex/b")
        .expect("b")
        .individual("http://ex/c")
        .expect("c")
        .same_individual(&["http://ex/a", "http://ex/b"])
        .expect("ab")
        .same_individual(&["http://ex/b", "http://ex/c"])
        .expect("bc")
        .build()
        .expect("build");
    let closure = same_as_closure(&ont);
    let a = ont.lookup_entity("http://ex/a").unwrap();
    let c = ont.lookup_entity("http://ex/c").unwrap();
    assert_eq!(closure.representative(a), closure.representative(c));
}
