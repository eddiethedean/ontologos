//! Hybrid module routing tests.

use ontologos_core::{EntityId, Ontology, Taxonomy};
use ontologos_profile::{classify_hybrid, engine_for_profile, merge_taxonomies, OwlProfile};

#[test]
fn hybrid_report_single_module() {
    let ontology = Ontology::default();
    let report = classify_hybrid(&ontology).expect("hybrid");
    assert_eq!(report.modules.len(), 1);
}

#[test]
fn engine_for_el_profile() {
    assert_eq!(engine_for_profile(OwlProfile::El), "el");
    assert_eq!(engine_for_profile(OwlProfile::Dl), "dl");
}

#[test]
fn merge_taxonomies_preserves_equivalences() {
    let a = EntityId(1);
    let b = EntityId(2);
    let c = EntityId(3);
    let left = Taxonomy {
        subsumptions: vec![(a, b), (b, a)],
        equivalences: vec![vec![a, b]],
        unsatisfiable: vec![],
    };
    let right = Taxonomy {
        subsumptions: vec![(b, c)],
        equivalences: vec![vec![b, c]],
        unsatisfiable: vec![],
    };
    let merged = merge_taxonomies(vec![left, right]);
    assert_eq!(merged.equivalences.len(), 2);
    assert!(merged.is_subsumed(a, c));
}
