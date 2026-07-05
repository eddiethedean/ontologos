use ontologos_core::{ClassExpr, DlAxiom, EntityKind, Ontology, OwlConstruct, Profile};
use ontologos_profile::{
    OwlProfile, classify_hybrid, detect_profile, resolve_route, subontology_with_axioms,
};
use ontologos_core::{DetectedProfileKind, EngineKind};

fn ontology_with_dl_union() -> Ontology {
    let mut ontology = Ontology::builder()
        .class("http://ex.org/A")
        .unwrap()
        .class("http://ex.org/B")
        .unwrap()
        .class("http://ex.org/C")
        .unwrap()
        .build()
        .unwrap();
    let a = ontology.lookup_entity("http://ex.org/A").unwrap();
    let b = ontology.lookup_entity("http://ex.org/B").unwrap();
    let c = ontology.lookup_entity("http://ex.org/C").unwrap();
    let dl = ontology.dl_mut();
    let b_ce = dl.intern_ce(ClassExpr::Atomic(b));
    let c_ce = dl.intern_ce(ClassExpr::Atomic(c));
    let union = dl.intern_ce(ClassExpr::Or(vec![b_ce, c_ce]));
    let a_ce = dl.intern_ce(ClassExpr::Atomic(a));
    dl.push_axiom(DlAxiom::SubClassOf {
        sub: a_ce,
        sup: union,
    });
    ontology
}

#[test]
fn detect_profile_sees_dl_store_union() {
    let ontology = ontology_with_dl_union();
    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Dl));
}

#[test]
fn dl_store_union_survives_core_axiom_mutation() {
    let mut ontology = ontology_with_dl_union();
    let a = ontology.lookup_entity("http://ex.org/A").unwrap();
    let b = ontology.lookup_entity("http://ex.org/B").unwrap();
    ontology
        .add_axiom(ontologos_core::Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .expect("axiom");

    let report = detect_profile(&ontology).expect("detect");
    assert_eq!(report.detected, Some(OwlProfile::Dl));
}

#[test]
fn classify_hybrid_includes_dl_store_only_ontology() {
    let ontology = ontology_with_dl_union();
    let report = classify_hybrid(&ontology).expect("hybrid");
    assert_eq!(report.modules.len(), 1);
    assert_eq!(report.modules[0].profile, OwlProfile::Dl);
    assert!(report.modules[0].include_dl_store);
}

#[test]
fn resolve_auto_routes_dl_store_ontology_to_dl_engine() {
    let ontology = ontology_with_dl_union();
    let route = resolve_route(Profile::Auto, &ontology).expect("route");
    assert_eq!(route.kind, EngineKind::Dl);
    assert_eq!(route.detected, Some(DetectedProfileKind::Dl));
}

#[test]
fn subontology_with_axioms_preserves_dl_store() {
    let ontology = ontology_with_dl_union();
    let report = classify_hybrid(&ontology).expect("hybrid");
    let module = &report.modules[0];
    let sub = subontology_with_axioms(&ontology, &module.axiom_ids, module.include_dl_store)
        .expect("sub");
    assert_eq!(sub.dl().axiom_count(), ontology.dl().axiom_count());
}

#[test]
fn scan_constructs_includes_dl_store_after_parse_meta_invalidation() {
    use ontologos_core::{Axiom, ParseMeta};
    use ontologos_profile::scanner::scan_constructs;

    let mut ontology = ontology_with_dl_union();
    let mut meta = ParseMeta::default();
    meta.profile_constructs.insert(OwlConstruct::ObjectUnionOf);
    ontology.set_parse_meta(meta);

    let a = ontology
        .entity_id("http://ex.org/X", EntityKind::Class)
        .expect("class");
    let y = ontology
        .entity_id("http://ex.org/Y", EntityKind::Class)
        .expect("class");
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: y,
        })
        .expect("axiom");

    let constructs = scan_constructs(&ontology);
    assert!(constructs.contains(&OwlConstruct::ObjectUnionOf));
    assert!(constructs.contains(&OwlConstruct::SubClassOfNamed));
}
