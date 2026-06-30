//! DL classification wall-time benchmarks (Tier D perf tracking).

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ontologos_dl::classify;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data")
        .join(name)
}

fn bench_corpus(c: &mut Criterion, label: &str, file: &str) {
    let path = data_path(file);
    if !path.is_file() {
        return;
    }
    let ontology = load_ontology(&path).expect("load corpus");
    c.bench_function(label, |b| {
        b.iter(|| classify(black_box(&ontology)).expect("classify"))
    });
}

fn classify_benches(c: &mut Criterion) {
    bench_corpus(c, "family_dl", "family.owl");
    bench_corpus(c, "pizza_dl", "pizza.owl");
}

criterion_group!(benches, classify_benches);
criterion_main!(benches);
