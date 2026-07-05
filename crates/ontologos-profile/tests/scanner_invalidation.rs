use ontologos_core::{Axiom, ClassExpr, DlAxiom, EntityKind, Ontology, OwlConstruct, ParseMeta};
use ontologos_profile::scanner::scan_constructs;

#[test]
fn mutation_preserves_dl_store_profile_constructs() {
    let mut meta = ParseMeta::default();
    meta.profile_constructs.insert(OwlConstruct::ObjectUnionOf);
    let mut ontology = Ontology::new();
    ontology.set_parse_meta(meta);

    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .expect("class");
    let c = ontology
        .entity_id("http://ex.org/C", EntityKind::Class)
        .expect("class");
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .expect("class");
    let dl = ontology.dl_mut();
    let b_ce = dl.intern_ce(ClassExpr::Atomic(b));
    let c_ce = dl.intern_ce(ClassExpr::Atomic(c));
    let union = dl.intern_ce(ClassExpr::Or(vec![b_ce, c_ce]));
    let a_ce = dl.intern_ce(ClassExpr::Atomic(a));
    dl.push_axiom(DlAxiom::SubClassOf {
        sub: a_ce,
        sup: union,
    });

    let before = scan_constructs(&ontology);
    assert!(before.contains(&OwlConstruct::ObjectUnionOf));

    let a = ontology.lookup_entity("http://ex.org/A").expect("class");
    let b = ontology.lookup_entity("http://ex.org/B").expect("class");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .expect("axiom");

    let after = scan_constructs(&ontology);
    assert!(after.contains(&OwlConstruct::ObjectUnionOf));
    assert!(after.contains(&OwlConstruct::SubClassOfNamed));
}
