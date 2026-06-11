use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ontologos_core::{Axiom, EntityKind, Ontology};

fn build_medium_ontology(axiom_count: usize) -> Ontology {
    let mut ontology = Ontology::builder()
        .class("http://example.org/Root")
        .expect("root")
        .build()
        .expect("build");

    for i in 0..axiom_count {
        let child = format!("http://example.org/Class{i}");
        let parent = if i == 0 {
            "http://example.org/Root".to_string()
        } else {
            format!("http://example.org/Class{}", i - 1)
        };
        ontology
            .entity_id(&child, EntityKind::Class)
            .expect("entity");
        ontology
            .entity_id(&parent, EntityKind::Class)
            .expect("entity");
        let sub = ontology.lookup_entity(&child).expect("sub");
        let sup = ontology.lookup_entity(&parent).expect("sup");
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: sub,
                superclass: sup,
            })
            .expect("axiom");
    }
    ontology
}

fn serialize_benchmark(c: &mut Criterion) {
    let ontology = build_medium_ontology(10_000);

    c.bench_function("serialize_10k_axioms", |b| {
        b.iter(|| black_box(ontology.to_json().expect("json")));
    });

    let json = ontology.to_json().expect("json");
    c.bench_function("deserialize_10k_axioms", |b| {
        b.iter(|| black_box(Ontology::from_json(&json).expect("parse")));
    });
}

criterion_group!(benches, serialize_benchmark);
criterion_main!(benches);
