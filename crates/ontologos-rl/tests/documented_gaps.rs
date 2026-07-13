//! Tests aligned with documented reasonable adapter gaps (see reasonable-limits.md).
//!
//! These encode upstream limitations so CI does not silently regress when a gap
//! is closed (test starts failing → promote to behavioral success test).

use ontologos_core::Ontology;
use ontologos_rl::rdfs::RdfsEngine;

const NS: &str = "http://example.org/gap/";

#[test]
#[ignore = "reasonable gap: mutual subPropertyOf from equivalentProperty (reasonable-limits.md)"]
fn equivalent_property_does_not_expand_subproperties_yet() {
    let mut ontology = Ontology::builder()
        .object_property(&format!("{NS}p"))
        .expect("p")
        .object_property(&format!("{NS}q"))
        .expect("q")
        .equivalent_object_properties(&[&format!("{NS}p"), &format!("{NS}q")])
        .expect("equiv")
        .build()
        .expect("build");
    RdfsEngine::new()
        .materialize(&mut ontology)
        .expect("materialize");
    let p = ontology.lookup_entity(&format!("{NS}p")).expect("p");
    let q = ontology.lookup_entity(&format!("{NS}q")).expect("q");
    assert!(
        ontology.direct_superproperties(p).contains(&q),
        "when implemented: p ⊑ q from equivalentProperty"
    );
    assert!(
        ontology.direct_superproperties(q).contains(&p),
        "when implemented: q ⊑ p from equivalentProperty"
    );
}
