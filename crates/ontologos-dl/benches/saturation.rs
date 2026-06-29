//! EL saturation wall-time benchmarks (Tier D perf tracking).

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ontologos_el::ElClassifier;
use ontologos_parser::load_ontology;
use std::path::PathBuf;

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data")
        .join(name)
}

fn bench_saturation(c: &mut Criterion, label: &str, file: &str) {
    let path = data_path(file);
    if !path.is_file() {
        return;
    }
    let ontology = load_ontology(&path).expect("load corpus");
    c.bench_function(label, |b| {
        b.iter(|| {
            ElClassifier::new()
                .classify(black_box(&ontology))
                .expect("classify")
        })
    });
}

fn saturation_benches(c: &mut Criterion) {
    bench_saturation(c, "family_el_saturation", "family.owl");
}

criterion_group!(benches, saturation_benches);
criterion_main!(benches);
