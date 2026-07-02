use ontologos_core::Axiom;
use std::path::Path;

use ontologos_parser::load_ontology;
use ontologos_rl::rdfs::RdfsEngine;

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn family_corpus() -> std::path::PathBuf {
    let path = repo_root().join("benchmarks/data/family.owl");
    assert!(
        path.exists(),
        "missing family corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );
    path
}

fn has_property_domain(ontology: &ontologos_core::Ontology, property: &str, domain: &str) -> bool {
    let property = ontology.lookup_entity(property).expect("property");
    let domain = ontology.lookup_entity(domain).expect("domain");
    ontology.axioms().iter().any(|(_, ax)| {
        matches!(
            ax,
            Axiom::ObjectPropertyDomain {
                property: p,
                domain: d
            } if *p == property && *d == domain
        )
    })
}

#[test]
fn family_corpus_inherits_domain_on_subproperty() {
    let mut ontology = load_ontology(&family_corpus()).expect("load family");
    RdfsEngine::new()
        .materialize(&mut ontology)
        .expect("materialize family");

    let ns = "http://a.com/ontology#";
    assert!(
        has_property_domain(&ontology, &format!("{ns}hasFather"), &format!("{ns}Person")),
        "RDFS should inherit hasParent domain onto hasFather"
    );
}

#[test]
fn pizza_corpus_materializes_with_rdfs_inferences() {
    let path = repo_root().join("benchmarks/data/pizza.owl");
    assert!(
        path.exists(),
        "missing pizza corpus at {} (run ./benchmarks/scripts/download.sh)",
        path.display()
    );

    let mut ontology = load_ontology(&path).expect("load pizza");
    let initial = ontology.axiom_count();
    let report = RdfsEngine::new()
        .materialize(&mut ontology)
        .expect("materialize pizza");

    assert!(report.final_axiom_count >= initial);
    assert!(
        report.inferred_total() > 0,
        "pizza corpus must produce RDFS inferences"
    );
}
