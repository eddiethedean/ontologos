//! JSON snapshot round-trip regressions (C-01, H-06, M-10).

use ontologos_core::{ClassExpr, DlAxiom, Ontology, SwrlAtom, SwrlIArg, SwrlRule};

#[test]
fn json_v4_round_trips_dl_and_swrl() {
    let mut ontology = Ontology::builder()
        .class("http://example.org/A")
        .expect("class")
        .class("http://example.org/B")
        .expect("class")
        .build()
        .expect("build");
    let a = ontology.lookup_entity("http://example.org/A").expect("A");
    let b = ontology.lookup_entity("http://example.org/B").expect("B");
    let ce_a = ontology.dl_mut().intern_ce(ClassExpr::Atomic(a));
    let ce_b = ontology.dl_mut().intern_ce(ClassExpr::Atomic(b));
    ontology.dl_mut().push_axiom(DlAxiom::SubClassOf {
        sub: ce_a,
        sup: ce_b,
    });
    ontology
        .push_swrl_rule(SwrlRule {
            body: vec![SwrlAtom::Class {
                class: a,
                arg: SwrlIArg::Individual(a),
            }],
            head: vec![SwrlAtom::Class {
                class: b,
                arg: SwrlIArg::Individual(a),
            }],
        })
        .expect("swrl");

    let json = ontology.to_json().expect("to_json");
    assert!(json.contains("\"format_version\": 4"));
    let restored = Ontology::from_json(&json).expect("from_json");
    assert_eq!(restored.dl().axiom_count(), 1);
    assert_eq!(restored.swrl_rules().len(), 1);
    assert!(!restored.dirty().is_dirty());
}

#[test]
fn json_v4_round_trips_parse_meta() {
    let mut ontology = Ontology::builder()
        .class("http://example.org/A")
        .expect("class")
        .build()
        .expect("build");
    ontology.set_parse_meta(ontologos_core::ParseMeta {
        warnings: vec!["skipped test axiom".into()],
        mapped_axiom_count: 1,
        skipped_axiom_count: 1,
        logical_axiom_count: 2,
        ..Default::default()
    });

    let json = ontology.to_json().expect("to_json");
    assert!(json.contains("\"parse_meta\""));
    let restored = Ontology::from_json(&json).expect("from_json");
    let meta = restored.parse_meta().expect("parse_meta");
    assert_eq!(meta.skipped_axiom_count, 1);
    assert_eq!(meta.warnings.len(), 1);
}

#[test]
fn json_export_omits_inferred_axioms() {
    let mut ontology = Ontology::builder()
        .class("http://example.org/A")
        .expect("class")
        .class("http://example.org/B")
        .expect("class")
        .class("http://example.org/C")
        .expect("class")
        .subclass_of("http://example.org/A", "http://example.org/B")
        .expect("sub")
        .build()
        .expect("build");
    let c = ontology.lookup_entity("http://example.org/C").expect("C");
    let b = ontology.lookup_entity("http://example.org/B").expect("B");
    ontology
        .add_inferred_axiom(ontologos_core::Axiom::SubClassOf {
            subclass: c,
            superclass: b,
        })
        .expect("inferred");
    let json = ontology.to_json().expect("to_json");
    let restored = Ontology::from_json(&json).expect("from_json");
    assert_eq!(restored.axiom_count(), 1);
}
