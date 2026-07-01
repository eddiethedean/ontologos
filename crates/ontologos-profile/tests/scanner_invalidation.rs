use ontologos_core::{Axiom, EntityKind, Ontology, OwlConstruct, ParseMeta};
use ontologos_profile::scanner::scan_constructs;

#[test]
fn mutation_invalidates_cached_profile_constructs() {
    let mut meta = ParseMeta::default();
    meta.profile_constructs.insert(OwlConstruct::ObjectUnionOf);
    let mut ontology = Ontology::new();
    ontology.set_parse_meta(meta);

    let before = scan_constructs(&ontology);
    assert!(before.contains(&OwlConstruct::ObjectUnionOf));

    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .expect("class");
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .expect("class");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .expect("axiom");

    let after = scan_constructs(&ontology);
    assert!(!after.contains(&OwlConstruct::ObjectUnionOf));
    assert!(after.contains(&OwlConstruct::SubClassOfNamed));
}
