//! Facade entailment query loop benchmarks.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ontologos_core::{EntityKind, Reasoner};
use ontologos_facade::{classify, is_subsumption_entailed};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data")
        .join(name)
}

fn entailment_loop_benches(c: &mut Criterion) {
    let path = data_path("family.owl");
    if !path.is_file() {
        return;
    }
    let ontology = load_ontology(&path).expect("load");
    let mut reasoner = Reasoner::builder().build(ontology).expect("build");
    classify(&mut reasoner).expect("classify");
    let iris = reasoner.ontology().iris();
    let classes: Vec<String> = reasoner
        .ontology()
        .entities()
        .iter()
        .filter(|(_, r)| r.kind == EntityKind::Class)
        .filter_map(|(_, r)| iris.resolve(r.iri).ok().map(str::to_owned))
        .take(32)
        .collect();
    c.bench_function("entailment_loop_1000", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let sub = &classes[i % classes.len()];
                let sup = &classes[(i + 1) % classes.len()];
                let _ = black_box(is_subsumption_entailed(&mut reasoner, sub, sup));
            }
        })
    });
}

criterion_group!(benches, entailment_loop_benches);
criterion_main!(benches);
