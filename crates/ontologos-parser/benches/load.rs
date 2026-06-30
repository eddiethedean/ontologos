//! Parser load wall-time benchmarks.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data")
        .join(name)
}

fn load_benches(c: &mut Criterion) {
    for (label, file) in [
        ("load_family", "family.owl"),
        ("load_pizza", "pizza.owl"),
        ("load_go_subset", "go-subset.owl"),
    ] {
        let path = data_path(file);
        if !path.is_file() {
            continue;
        }
        c.bench_function(label, |b| {
            b.iter(|| load_ontology(black_box(&path)).expect("load"))
        });
    }
}

criterion_group!(benches, load_benches);
criterion_main!(benches);
