//! Triage excluded Ian/ComplexConcept CE cases against tableau probes.

use ontologos_alc::{DlOntology, TableauSeed};
use ontologos_core::DlAxiom;
use ontologos_parser::load_ontology;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static TRIAGE_SEQ: AtomicU64 = AtomicU64::new(0);

fn ax(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../benchmarks/data/hermit/axioms")
        .join(name)
}

fn ce_sat(ofn: &str, ce_ofn: &str) -> bool {
    let assertion = format!("ClassAssertion({} :__probe__)", ce_ofn);
    let body = format!(
        "Prefix(:=<file:/c/test.owl#>)\nPrefix(a:=<file:/c/test.owl#>)\nPrefix(rdfs:=<http://www.w3.org/2000/01/rdf-schema#>)\nPrefix(owl:=<http://www.w3.org/2002/07/owl#>)\nPrefix(xsd:=<http://www.w3.org/2001/XMLSchema#>)\nOntology(<file:/c/test.owl#>\n{assertion}\n)"
    );
    let temp = std::env::temp_dir().join(format!(
        "ian-ce-triage-{}-{}-{}.ofn",
        std::process::id(),
        TRIAGE_SEQ.fetch_add(1, Ordering::Relaxed),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    std::fs::write(&temp, &body).unwrap();
    let probe = load_ontology(&temp).unwrap();
    let base = load_ontology(&ax(ofn)).unwrap();
    let mut merged = base.clone();
    for (_, record) in probe.entities().iter() {
        let Ok(iri) = probe.resolve_iri(record.iri) else {
            continue;
        };
        if merged.lookup_entity(iri).is_none() {
            let _ = merged.entity_id(iri, record.kind);
        }
    }
    let entity_map: std::collections::HashMap<_, _> = probe
        .entities()
        .iter()
        .filter_map(|(id, record)| {
            let iri = probe.resolve_iri(record.iri).ok()?;
            Some((id, merged.lookup_entity(iri)?))
        })
        .collect();
    merged
        .dl_mut()
        .import_axioms_from(probe.dl(), |id| entity_map.get(&id).copied().unwrap());
    let dl = DlOntology::from_ontology(&merged).unwrap();
    let ce_id = merged
        .dl()
        .axioms()
        .filter_map(|axiom| {
            let DlAxiom::ClassAssertion { class, .. } = axiom else {
                return None;
            };
            Some(*class)
        })
        .last()
        .unwrap();
    ontologos_alc::is_ce_satisfiable_with_seed(&dl, ce_id, &TableauSeed::default()).unwrap()
}

fn check(ofn: &str, ce: &str, expected: bool) {
    let actual = ce_sat(ofn, ce);
    assert_eq!(
        actual, expected,
        "ofn={ofn} expected sat={expected} got {actual}"
    );
}

#[test]
fn iant6_unsat_regression() {
    let ce = "ObjectIntersectionOf(ObjectComplementOf(:c) ObjectSomeValuesFrom(ObjectInverseOf(:f) :d) ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectSomeValuesFrom(ObjectInverseOf(:f) :d)))";
    check(
        "hermit_reasoner_reasonertest_testiant6.ofn",
        ce,
        false,
    );
}

#[test]
fn iant7c_unsat_regression() {
    let ce = "ObjectIntersectionOf(:p1 ObjectSomeValuesFrom(:r ObjectSomeValuesFrom(:r ObjectIntersectionOf(:p1 ObjectAllValuesFrom(ObjectInverseOf(:r) ObjectComplementOf(:p1))))) ObjectSomeValuesFrom(ObjectInverseOf(:f) :p1))";
    check(
        "hermit_reasoner_reasonertest_testiant7c.ofn",
        ce,
        false,
    );
}

#[test]
fn ianbug1b_unsat() {
    let ce = "ObjectIntersectionOf(ObjectComplementOf(:c) :a ObjectComplementOf(:b) :d)";
    check(
        "hermit_reasoner_reasonertest_testianbug1b.ofn",
        ce,
        false,
    );
}

// IanT9 promoted to conformance; kept here as regression guard.
#[test]
fn iant9_unsat_regression() {
    let ce = "ObjectIntersectionOf(:Infinite-Tree-Root ObjectAllValuesFrom(:descendant ObjectSomeValuesFrom(ObjectInverseOf(:successor) :root)))";
    check(
        "hermit_reasoner_reasonertest_testiant9.ofn",
        ce,
        false,
    );
}

#[test]
fn iant11_unsat_regression() {
    let ce = "ObjectIntersectionOf(ObjectComplementOf(:p) ObjectSomeValuesFrom(:f ObjectIntersectionOf(ObjectAllValuesFrom(ObjectInverseOf(:s) :p) ObjectAllValuesFrom(ObjectInverseOf(:f) ObjectSomeValuesFrom(:s :p)))) ObjectSomeValuesFrom(:f1 ObjectIntersectionOf(ObjectAllValuesFrom(ObjectInverseOf(:s) :p) ObjectAllValuesFrom(ObjectInverseOf(:f1) ObjectSomeValuesFrom(:s :p)))))";
    check(
        "hermit_reasoner_reasonertest_testiant11.ofn",
        ce,
        false,
    );
}

#[test]
fn iant13_unsat_regression() {
    let ce = "ObjectIntersectionOf(:a2 ObjectSomeValuesFrom(:s ObjectAllValuesFrom(ObjectInverseOf(:s) ObjectAllValuesFrom(:r :c))))";
    check(
        "hermit_reasoner_reasonertest_testiant13.ofn",
        ce,
        false,
    );
}

#[test]
#[ignore = "catalog/HermiT mismatch: union CE is SAT under extracted DisjointClasses OFN"]
fn ianfact1_unsat() {
    let ce = "ObjectUnionOf(ObjectIntersectionOf(:a :b) ObjectIntersectionOf(:a ObjectComplementOf(:b)) ObjectIntersectionOf(ObjectComplementOf(:a) :b))";
    check(
        "hermit_reasoner_reasonertest_testianfact1.ofn",
        ce,
        false,
    );
}
