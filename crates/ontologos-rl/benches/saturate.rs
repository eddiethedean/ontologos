//! RL saturation wall-time benchmarks.

use criterion::{Criterion, black_box, criterion_group, criterion_main};
use ontologos_parser::load_ontology;
use ontologos_rl::RlEngine;
use std::path::PathBuf;

fn data_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data")
        .join(name)
}

fn saturate_benches(c: &mut Criterion) {
    let path = data_path("family.owl");
    if !path.is_file() {
        return;
    }
    let ontology = load_ontology(&path).expect("load");
    c.bench_function("family_rl_saturate", |b| {
        b.iter(|| {
            let mut copy = black_box(ontology.clone());
            RlEngine::new(1).saturate(&mut copy).expect("saturate")
        })
    });
}

criterion_group!(benches, saturate_benches);
criterion_main!(benches);
