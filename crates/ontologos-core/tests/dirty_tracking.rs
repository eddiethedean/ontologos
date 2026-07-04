use ontologos_core::{Axiom, EntityKind, Ontology};

#[test]
fn add_axiom_marks_dirty_and_bumps_revision() {
    let mut ontology = Ontology::new();
    assert_eq!(ontology.revision().counter(), 0);
    assert!(!ontology.dirty().is_dirty());

    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    let id = ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .unwrap();

    assert_eq!(ontology.revision().counter(), 1);
    assert!(ontology.dirty().is_dirty());
    assert_eq!(ontology.dirty().added(), &[id]);
    ontology.clear_dirty();
    assert!(!ontology.dirty().is_dirty());
}

#[test]
fn remove_axiom_marks_dirty_and_rebuilds_index() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    let id = ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .unwrap();
    ontology.clear_dirty();

    ontology.remove_axiom(id).unwrap();
    assert_eq!(ontology.axiom_count(), 0);
    assert!(ontology.dirty().has_removals());
    assert_eq!(ontology.index().direct_superclasses(a).len(), 0);
    assert_eq!(ontology.revision().counter(), 2);
}

#[test]
fn dirty_signatures_union_entities() {
    let mut ontology = Ontology::new();
    let a = ontology
        .entity_id("http://ex.org/A", EntityKind::Class)
        .unwrap();
    let b = ontology
        .entity_id("http://ex.org/B", EntityKind::Class)
        .unwrap();
    ontology
        .add_axiom(Axiom::SubClassOf {
            subclass: a,
            superclass: b,
        })
        .unwrap();
    let sig = ontology.dirty_signatures();
    assert_eq!(sig.len(), 2);
}
