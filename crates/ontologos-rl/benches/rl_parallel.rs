use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ontologos_core::{Axiom, EntityKind, Ontology};
use ontologos_rl::RlEngine;

fn build_large_ontology(assertions: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let person = ontology
        .entity_id("http://example.org/Person", EntityKind::Class)
        .expect("Person");
    let knows = ontology
        .entity_id("http://example.org/knows", EntityKind::ObjectProperty)
        .expect("knows");
    ontology
        .add_axiom(Axiom::ObjectPropertyDomain {
            property: knows,
            domain: person,
        })
        .expect("domain");
    ontology
        .add_axiom(Axiom::ObjectPropertyRange {
            property: knows,
            range: person,
        })
        .expect("range");

    for i in 0..assertions {
        let iri = format!("http://example.org/i{i}");
        let next = format!("http://example.org/i{}", (i + 1) % assertions);
        let a = ontology
            .entity_id(&iri, EntityKind::Individual)
            .expect("individual");
        let b = ontology
            .entity_id(&next, EntityKind::Individual)
            .expect("next");
        ontology
            .add_axiom(Axiom::ObjectPropertyAssertion {
                subject: a,
                property: knows,
                object: b,
            })
            .expect("assertion");
    }
    ontology
}

fn bench_rl_parallel(c: &mut Criterion) {
    let ontology = build_large_ontology(10_000);
    let mut group = c.benchmark_group("rl_parallel_10k");

    group.bench_function("parallelism_1", |b| {
        b.iter(|| {
            let mut copy = black_box(ontology.clone());
            RlEngine::new(1).saturate(&mut copy).expect("saturate");
        });
    });

    group.bench_function("parallelism_4", |b| {
        b.iter(|| {
            let mut copy = black_box(ontology.clone());
            RlEngine::try_new(4)
                .expect("engine")
                .saturate(&mut copy)
                .expect("saturate");
        });
    });

    group.finish();
}

criterion_group!(benches, bench_rl_parallel);
criterion_main!(benches);
