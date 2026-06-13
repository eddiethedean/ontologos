//! EL incremental vs full classify performance gate (local/CI optional).

use std::time::Instant;

use ontologos_core::{Axiom, EntityKind, Ontology, Profile, Reasoner, ReasonerConfig};

fn build_chain(n: usize) -> Ontology {
    let mut ontology = Ontology::new();
    let mut ids = Vec::new();
    for i in 0..=n {
        ids.push(
            ontology
                .entity_id(&format!("http://ex.org/C{i}"), EntityKind::Class)
                .unwrap(),
        );
    }
    for i in 0..n {
        ontology
            .add_axiom(Axiom::SubClassOf {
                subclass: ids[i],
                superclass: ids[i + 1],
            })
            .unwrap();
    }
    ontology
}

#[test]
#[ignore = "performance gate: run via benchmarks/scripts/bench-el-incremental.sh"]
fn incremental_el_at_least_5x_faster_on_ten_axiom_delta() {
    const ITERS: usize = 20;
    const DELTA_PER_ITER: usize = 10;

    let base = build_chain(50);
    let mut reasoner = Reasoner::builder()
        .profile(Profile::El)
        .config(ReasonerConfig {
            incremental: true,
            ..ReasonerConfig::default()
        })
        .build(base)
        .unwrap();

    ontologos_el::classify_reasoner(&mut reasoner).unwrap();

    let full_start = Instant::now();
    for iter in 0..ITERS {
        let mut ont = reasoner.ontology().clone();
        for i in 0..DELTA_PER_ITER {
            let x = ont
                .entity_id(&format!("http://ex.org/X{iter}_{i}"), EntityKind::Class)
                .unwrap();
            let target = ont
                .entity_id(&format!("http://ex.org/C{i}"), EntityKind::Class)
                .unwrap();
            ont.add_axiom(Axiom::SubClassOf {
                subclass: x,
                superclass: target,
            })
            .unwrap();
        }
        let _ = ontologos_el::ElClassifier::new().classify(&ont).unwrap();
    }
    let full_elapsed = full_start.elapsed();

    let incr_start = Instant::now();
    for iter in 0..ITERS {
        for i in 0..DELTA_PER_ITER {
            let ont = reasoner.ontology_mut();
            let x = ont
                .entity_id(&format!("http://ex.org/Y{iter}_{i}"), EntityKind::Class)
                .unwrap();
            let target = ont
                .entity_id(&format!("http://ex.org/C{i}"), EntityKind::Class)
                .unwrap();
            ont.add_axiom(Axiom::SubClassOf {
                subclass: x,
                superclass: target,
            })
            .unwrap();
        }
        ontologos_el::classify_reasoner(&mut reasoner).unwrap();
    }
    let incr_elapsed = incr_start.elapsed();

    let ratio = full_elapsed.as_secs_f64() / incr_elapsed.as_secs_f64().max(1e-9);
    eprintln!("full={full_elapsed:?} incremental={incr_elapsed:?} ratio={ratio:.1}x");
    assert!(
        ratio >= 5.0,
        "expected incremental >=5x faster, got {ratio:.1}x"
    );
}
